# Bondline documentation

Start with the [project README](../README.md) and [current status](../STATUS.md).
The app is in early development; documented plans are not implemented features.

Current selection/translation passes the1361/0/1 local gate and an inspected offscreen
actual-App frame. Native selection interaction remains unverified; see its receiving row.
The earlier physical-density checkpoint has five renderer/five actual-App passes, but full
local pipeline78964 failed767/1 in the unchanged MCP shutdown replacement-bind test.
Separate remaining430/0/1 checks are not an aggregate pass; native2×/DPI is unverified.
The earlier component1188/0/1 and CDX1172/0/1 gates remain historical checkpoints.

| Task | Read |
|---|---|
| Build and run tests | [Development](development.md) |
| Contribute a change | [Contribution guide](../CONTRIBUTING.md), [agent instructions](../AGENTS.md) |
| Understand the engine | [Architecture](architecture.md), [model](model.md) |
| Automate the app | [Command and control protocol](control-protocol.md) |
| Understand format support | [Capability matrix](file-formats.md), [native file I/O](native-file-io.md) |
| Use the delivered file workflow | [Native file UI and exact verification](native-file-ui.md) |
| Start a fresh drawing safely | [New document, native Save/Discard/Cancel and race evidence](verification-new-document.md) |
| Edit an elemental atom label | [Native target/fields/Apply/Cancel, history and independent saved-file evidence](verification-native-atom-labels.md), [literal authoring contract](research/atom-label-authoring-contract.md) |
| Select and move native atoms | [Shared commands, literal geometry/history/file consumers and App input evidence; native interaction pending](verification-selection-authoring.md), [locked public-source contract](research/selection-translation-plan.md) |
| Convert a supported SMILES file | [Fourth profile: historical1188/0/1 checkpoint and bounded Mac100% workflow; Fit/pan limits](verification-smiles-components-integration.md), [independent layout](verification-smiles-components-layout.md), [earlier ring aggregate and Mac evidence](verification-smiles-ring-integration.md), [acyclic/native evidence](verification-smiles-acyclic-integration.md), [original lifecycle](verification-native-smiles-conversion.md) |
| Check asynchronous file browsing | [Stable controls and visible errors](verification-file-picker-stability.md) |
| Inspect SMILES notation records | [Original inspector and bounded native file workflow](verification-smiles-record-integration.md) |
| Remove a complete SDF record | [Source preservation, confirmation and native evidence](verification-sdf-record-removal.md) |
| Inspect ordered SDF field occurrences | [Read-only inspector, TDD and bounded native checks](verification-sdf-inspector.md) |
| Edit an existing SDF field value | [Transactional editor and native evidence](verification-sdf-editor-integration.md), [App cases](verification-sdf-editor.md), [worker gates](verification-sdf-editor-lifecycle.md) |
| Navigate a read-only SDF chemical preview | [Integrated native scope](verification-sdf-view-integration.md), [command checks](verification-sdf-view-command.md), [App checks](verification-sdf-view-ui.md), [physical raster](verification-view-raster.md) |
| Inspect rendering limits | [Rendering](rendering.md) |
| Check Drawing physical density | [Renderer/App evidence, bounded Mac history/persistence and remaining Fit/DPI limits](verification-native-drawing-density.md) |
| Check centered Double/Triple geometry | [Renderer evidence](verification-multiple-render.md) |
| Check ring/literal-H attachment | [Fixed ring pixels/hits](verification-literal-ring-attachment.md), [compact fallback and source compatibility](verification-literal-compact-star-attachment.md), [passing rotated-star control](verification-literal-rotated-star-attachment.md) |
| Check ordered bond commands and transient tools | [Authoring contract](verification-multiple-authoring.md) |
| Check multiple-bond file/image/fuzz acceptance | [Independent checks](verification-multiple-commands.md) |
| Check actual application-frame tool workflows | [UI evidence](verification-multiple-ui.md) |
| Inspect the refreshed visual direction | [Pinned references](research/visual-direction.md), [actual workspace checks](verification-workspace-presentation.md), [offscreen capture utility](verification-visual-capture.md) |
| Find the next feature | [Roadmap](../ROADMAP.md), [parity checklist](parity-checklist.md) |
| Review proposed SMILES authoring | [Editable Paste Special/aromatic prerequisites and complete-source record editing plan; unimplemented](research/smiles-source-import-plan.md) |
| Check canonical CDX FileEnd support | [Independent/ASan, bounded Mac and full1172/0/1 gate evidence](verification-cdx-file-end-integration.md), [independent declarations](verification-cdx-file-end-tests.md), [public framing decision](research/cdx-file-end-plan.md). Earlier intermittent MCP failure remains unexplained; no general compatibility or corpus admission |
| Review historical installed reference evidence | [Earlier File/Edit/View/New observations and gaps](research/live-ui-review-2026-11-15.md); current work follows the public-source-only boundary in AGENTS.md |
| Check evidence | [Verification](verification.md), [CI scope](ci.md), [corpus](corpus/README.md) |
| Review provenance and licenses | [Source register](clean-room/sources.md), [open questions](clean-room/open-questions.md), [dependency policy](dependency-policy.md), [attribution](../ATTRIBUTION.md) |
| Report a problem | [Support](../SUPPORT.md), [security reporting](../SECURITY.md) |
| Reproduce README imagery | [Media sources](media/SOURCES.md) |

The core works offline. Python tools in the development instructions are independent test
consumers; they are not part of the desktop application's runtime.
