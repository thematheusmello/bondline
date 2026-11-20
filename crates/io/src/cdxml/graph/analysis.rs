//! A complete chemical graph can be readable without any usable drawing coordinates.
use super::ElementalGraph;
use bondline_model::{BondOrder, Fragment, Radical};
use std::collections::BTreeMap;

impl ElementalGraph<'_> {
    pub(crate) fn has_carbon_connectivity(&self) -> bool {
        let features = self.source.features();
        if (features.doctype && !self.source.has_bare_cdxml_doctype())
            || features.namespace_declarations
            || features.processing_instructions
            || features.cdata
            || self.fragments.is_empty()
        {
            return false;
        }
        for node in self.source.xml().descendants() {
            if node.is_text()
                && node
                    .text()
                    .is_some_and(|s| !s.trim_matches([' ', '\r', '\n', '\t']).is_empty())
            {
                return false;
            }
            if !node.is_element() {
                continue;
            }
            let parent = node.parent_element().map(|p| p.tag_name().name());
            let allowed: &[&str] = match (parent, node.tag_name().name()) {
                (None, "CDXML") => &["CreationProgram"],
                (Some("CDXML"), "page") | (Some("page"), "fragment") => &["id"],
                (Some("CDXML"), "fonttable" | "colortable") => &[],
                (Some("fonttable"), "font") => &["id", "name", "charset"],
                (Some("colortable"), "color") => &["r", "g", "b"],
                (Some("fragment"), "n") => &[
                    "id",
                    "NodeType",
                    "Element",
                    "NumHydrogens",
                    "p",
                    "Charge",
                    "Radical",
                    "Isotope",
                ],
                (Some("fragment"), "b")
                    if matches!(node.attribute("Order"), Some("1" | "2" | "3")) =>
                {
                    &["id", "B", "E", "Order"]
                }
                _ => return false,
            };
            if node.tag_name().namespace().is_some()
                || !node
                    .attributes()
                    .all(|a| a.namespace().is_none() && allowed.contains(&a.name()))
            {
                return false;
            }
        }
        self.source.resources().is_ok() && self.fragments.iter().all(complete_carbon_valences)
    }
}

fn complete_carbon_valences(fragment: &Fragment) -> bool {
    let mut occupied = BTreeMap::new();
    for atom in fragment.atoms() {
        if atom.element.get() != 6
            || atom.charge != 0
            || atom.radical != Radical::None
            || atom.isotope.is_some()
        {
            return false;
        }
        occupied.insert(atom.id, 0u16);
    }
    for bond in fragment.bonds() {
        let weight = match bond.order {
            BondOrder::Single => 1,
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            _ => return false,
        };
        for id in [bond.begin, bond.end] {
            let Some(sum) = occupied.get_mut(&id) else {
                return false;
            };
            *sum += weight;
            if *sum > 4 {
                return false;
            }
        }
    }
    !occupied.is_empty()
        && fragment.atoms().iter().all(|a| {
            a.explicit_hydrogens
                .is_none_or(|h| occupied.get(&a.id).is_some_and(|sum| h == 4 - sum))
        })
}
