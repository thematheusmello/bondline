// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{Action, ActionHandler, ActionRequest, NodeId, TreeUpdate};
use accesskit_consumer::Tree;
use objc2::rc::{Id, WeakId};
use objc2_app_kit::*;
use objc2_foundation::MainThreadMarker;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::node::PlatformNode;

fn update_string_value_setters(setters: &mut HashSet<NodeId>, update: &TreeUpdate) {
    for (id, node) in &update.nodes {
        if node.supports_action(Action::SetValue) {
            setters.insert(*id);
        } else {
            setters.remove(id);
        }
    }
}

fn prune_string_value_setters(setters: &mut HashSet<NodeId>, tree: &Tree) {
    setters.retain(|id| tree.state().has_node(*id));
}

pub(crate) trait ActionHandlerNoMut {
    fn do_action(&self, request: ActionRequest);
}

pub(crate) struct ActionHandlerWrapper<H: ActionHandler>(RefCell<H>);

impl<H: 'static + ActionHandler> ActionHandlerWrapper<H> {
    pub(crate) fn new(inner: H) -> Self {
        Self(RefCell::new(inner))
    }
}

impl<H: ActionHandler> ActionHandlerNoMut for ActionHandlerWrapper<H> {
    fn do_action(&self, request: ActionRequest) {
        self.0.borrow_mut().do_action(request)
    }
}

pub(crate) struct Context {
    pub(crate) view: WeakId<NSView>,
    pub(crate) tree: RefCell<Tree>,
    pub(crate) action_handler: Rc<dyn ActionHandlerNoMut>,
    platform_nodes: RefCell<HashMap<NodeId, Id<PlatformNode>>>,
    string_value_setters: RefCell<HashSet<NodeId>>,
    pub(crate) mtm: MainThreadMarker,
}

impl Context {
    pub(crate) fn new(
        view: WeakId<NSView>,
        initial_state: TreeUpdate,
        is_view_focused: bool,
        action_handler: Rc<dyn ActionHandlerNoMut>,
        mtm: MainThreadMarker,
    ) -> Rc<Self> {
        let mut string_value_setters = HashSet::new();
        update_string_value_setters(&mut string_value_setters, &initial_state);
        Rc::new(Self {
            view,
            tree: RefCell::new(Tree::new(initial_state, is_view_focused)),
            action_handler,
            platform_nodes: RefCell::new(HashMap::new()),
            string_value_setters: RefCell::new(string_value_setters),
            mtm,
        })
    }

    pub(crate) fn get_or_create_platform_node(self: &Rc<Self>, id: NodeId) -> Id<PlatformNode> {
        let mut platform_nodes = self.platform_nodes.borrow_mut();
        if let Some(result) = platform_nodes.get(&id) {
            return result.clone();
        }

        let result = PlatformNode::new(Rc::downgrade(self), id);
        platform_nodes.insert(id, result.clone());
        result
    }

    pub(crate) fn remove_platform_node(&self, id: NodeId) -> Option<Id<PlatformNode>> {
        let mut platform_nodes = self.platform_nodes.borrow_mut();
        platform_nodes.remove(&id)
    }

    pub(crate) fn do_action(&self, request: ActionRequest) {
        self.action_handler.do_action(request);
    }

    pub(crate) fn update_string_value_setters(&self, update: &TreeUpdate) {
        update_string_value_setters(&mut self.string_value_setters.borrow_mut(), update);
    }

    pub(crate) fn prune_string_value_setters(&self, tree: &Tree) {
        prune_string_value_setters(&mut self.string_value_setters.borrow_mut(), tree);
    }

    pub(crate) fn supports_string_value(&self, id: NodeId) -> bool {
        self.string_value_setters.borrow().contains(&id)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let platform_nodes = self.platform_nodes.borrow();
        for platform_node in platform_nodes.values() {
            unsafe {
                NSAccessibilityPostNotification(
                    platform_node,
                    NSAccessibilityUIElementDestroyedNotification,
                )
            };
        }
    }
}

#[cfg(test)]
mod bondline_action_cache_tests {
    use super::*;
    use accesskit::{NodeBuilder, Role, Tree as TreeData};

    #[test]
    fn flag_removal_partial_updates_and_deleted_nodes_track_the_current_tree() {
        let mut root = NodeBuilder::new(Role::Window);
        root.set_children(vec![NodeId(2)]);
        let mut input = NodeBuilder::new(Role::TextInput);
        input.set_value("file.cdxml");
        input.add_action(Action::SetValue);
        let initial = TreeUpdate {
            nodes: vec![
                (NodeId(1), root.build()),
                (NodeId(2), input.clone().build()),
            ],
            tree: Some(TreeData::new(NodeId(1))),
            focus: NodeId(2),
        };
        let mut setters = HashSet::new();
        update_string_value_setters(&mut setters, &initial);
        let mut tree = Tree::new(initial, true);
        assert_eq!(setters, HashSet::from([NodeId(2)]));

        input.remove_action(Action::SetValue);
        let removal = TreeUpdate {
            nodes: vec![(NodeId(2), input.clone().build())],
            tree: None,
            focus: NodeId(2),
        };
        update_string_value_setters(&mut setters, &removal);
        tree.update(removal);
        prune_string_value_setters(&mut setters, &tree);
        assert!(
            setters.is_empty(),
            "removing the action must revoke dispatch"
        );

        input.add_action(Action::SetValue);
        let restored = TreeUpdate {
            nodes: vec![(NodeId(2), input.build())],
            tree: None,
            focus: NodeId(2),
        };
        update_string_value_setters(&mut setters, &restored);
        tree.update(restored);
        let partial = TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: NodeId(1),
        };
        update_string_value_setters(&mut setters, &partial);
        tree.update(partial);
        prune_string_value_setters(&mut setters, &tree);
        assert_eq!(
            setters,
            HashSet::from([NodeId(2)]),
            "unchanged nodes retain their supported actions"
        );

        let removed = TreeUpdate {
            nodes: vec![(NodeId(1), NodeBuilder::new(Role::Window).build())],
            tree: None,
            focus: NodeId(1),
        };
        update_string_value_setters(&mut setters, &removed);
        tree.update(removed);
        prune_string_value_setters(&mut setters, &tree);
        assert!(!tree.state().has_node(NodeId(2)));
        assert!(
            setters.is_empty(),
            "deleted nodes must not leave retained action IDs"
        );
    }
}
