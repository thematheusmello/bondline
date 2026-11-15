# Bondline documentation

Start with the [project README](../README.md) and [current status](../STATUS.md).
The app is in early development; documented plans are not implemented features.

| Task | Read |
|---|---|
| Build and run tests | [Development](development.md) |
| Contribute a change | [Contribution guide](../CONTRIBUTING.md), [agent instructions](../AGENTS.md) |
| Understand the engine | [Architecture](architecture.md), [model](model.md) |
| Automate the app | [Command and control protocol](control-protocol.md) |
| Understand format support | [Capability matrix](file-formats.md), [native file I/O](native-file-io.md) |
| Use the delivered file workflow | [Native file UI and exact verification](native-file-ui.md) |
| Start a fresh drawing safely | [New document, native Save/Discard/Cancel and race evidence](verification-new-document.md) |
| Convert a supported SMILES file | [Native conversion contract and exact verification](verification-native-smiles-conversion.md) |
| Check asynchronous file browsing | [Stable controls and visible errors](verification-file-picker-stability.md) |
| Inspect SMILES notation records | [Original inspector and bounded native file workflow](verification-smiles-record-integration.md) |
| Remove a complete SDF record | [Source preservation, confirmation and native evidence](verification-sdf-record-removal.md) |
| Inspect ordered SDF field occurrences | [Read-only inspector, TDD and bounded native checks](verification-sdf-inspector.md) |
| Edit an existing SDF field value | [Transactional editor and native evidence](verification-sdf-editor-integration.md), [App cases](verification-sdf-editor.md), [worker gates](verification-sdf-editor-lifecycle.md) |
| Navigate a read-only SDF chemical preview | [Integrated native scope](verification-sdf-view-integration.md), [command checks](verification-sdf-view-command.md), [App checks](verification-sdf-view-ui.md), [physical raster](verification-view-raster.md) |
| Inspect rendering limits | [Rendering](rendering.md) |
| Check centered Double/Triple geometry | [Renderer evidence](verification-multiple-render.md) |
| Check ordered bond commands and transient tools | [Authoring contract](verification-multiple-authoring.md) |
| Check multiple-bond file/image/fuzz acceptance | [Independent checks](verification-multiple-commands.md) |
| Check actual application-frame tool workflows | [UI evidence](verification-multiple-ui.md) |
| Inspect the refreshed visual direction | [Pinned references](research/visual-direction.md), [actual workspace checks](verification-workspace-presentation.md), [offscreen capture utility](verification-visual-capture.md) |
| Find the next feature | [Roadmap](../ROADMAP.md), [parity checklist](parity-checklist.md) |
| Review the latest installed reference UI | [User-authorized live File/Edit/View/New observations and gaps](research/live-ui-review-2026-11-15.md) |
| Check evidence | [Verification](verification.md), [CI scope](ci.md), [corpus](corpus/README.md) |
| Review provenance and licenses | [Source register](clean-room/sources.md), [open questions](clean-room/open-questions.md), [dependency policy](dependency-policy.md), [attribution](../ATTRIBUTION.md) |
| Report a problem | [Support](../SUPPORT.md), [security reporting](../SECURITY.md) |
| Reproduce README imagery | [Media sources](media/SOURCES.md) |

The core works offline. Python tools in the development instructions are independent test
consumers; they are not part of the desktop application's runtime.
