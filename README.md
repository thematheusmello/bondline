<p align="center">
  <img src="docs/media/banner.svg" alt="Bondline — an offline chemical drawing editor built in Rust" width="100%">
</p>

<h1 align="center">Bondline</h1>
<p align="center">Free, open-source tools for drawing chemical structures and reactions.</p>

<p align="center">
  <a href="docs/ci.md"><img src="https://img.shields.io/badge/Actions-disabled%20during%20development-6b7280" alt="GitHub Actions disabled during development"></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-167b72" alt="License: MIT OR Apache-2.0"></a>
  <a href="rust-toolchain.toml"><img src="https://img.shields.io/badge/Rust-1.94.1-b35936" alt="Rust 1.94.1"></a>
  <a href="STATUS.md"><img src="https://img.shields.io/badge/status-early%20development-d89b32" alt="Early development"></a>
</p>

<p align="center">
  <a href="#what-works-today">Features</a> ·
  <a href="#get-started">Get started</a> ·
  <a href="docs/README.md">Documentation</a> ·
  <a href="ROADMAP.md">Roadmap</a> ·
  <a href="CONTRIBUTING.md">Contributing</a> ·
  <a href="#license-and-credits">License</a>
</p>

Bondline is a native Rust desktop application with an offline command engine, editable
chemical documents and a thin egui/wgpu interface. No account or hosted service is needed
for ordinary drawing and saving.

**Early development:** Single-bond carbon chains have native macOS verification.
The new Double/Triple tools pass headless drawing, preview, history and file checks;
their native-window drawing acceptance is pending. Rings, general labels and chemical analysis are still being built.
M1 file interoperability is the active implementation milestone; the complete M1–M5 target
is in [ROADMAP.md](ROADMAP.md). Full M0 is not yet complete.

<p align="center">
  <img src="docs/media/workspace-current.jpg" alt="The actual refreshed Bondline macOS window with a left bond-tool rail, Single/Double/Triple structures on white paper, and a right document inspector" width="100%">
  <br>
  <sub>Actual macOS / Metal window: original light workspace, three bond tools and a live document inspector.<br>The structures were authored through the shared registry and reopened as CDXML; <a href="docs/media/SOURCES.md">capture identity and limits</a>.</sub>
</p>

## What works today

| Capability | Delivered scope |
|---|---|
| Drawing and history | Drag a C–C bond and extend a supported terminal carbon with an explicit H policy. Native C3/C4 authoring, Undo/Redo and editable CDXML reopening are verified on macOS; general chemistry remains missing. |
| Double and Triple bonds | Select a tool, preview/draw a centered bond, extend a supported carbon terminal, Undo/Redo and save/reopen. Actual application-frame and independent file/image tests pass; new native drawing acceptance and context-sensitive styles remain unverified/missing. |
| Workspace | Light header, compact two-column bond rail, white paper, document/source/count inspector and contextual guidance. Real macOS rendering, Double selection and file-picker inspection pass; complete native drawing acceptance remains pending. |
| Pages | A4 and Letter presets, rotation and validated physical dimensions. |
| CDXML files | Native Open, Save and Save As; editable reopening of proven Bondline-native files; unknown imported content remains backed by its original bytes. |
| MOL V2000 files | Source-preserving native Open/Save/Save As, retained headers/frame, explicit format choice and guarded close. Verified on macOS; one-fragment/layout/metadata limits apply. |
| MOL V3000 files | Native Open/Save/Save As and guarded close verified on macOS; retained source/headers, full-precision output and explicit versions. One-fragment/layout/metadata limits apply. |
| CDX files | Guarded native Open/Save/Save As verified on macOS, retaining exact accepted source. Limited projection; editing and faithful preview remain unavailable. |
| SDF files | Whole mixed-version collection Open/Save/Save As/reopen verified on macOS. CLI/MCP inspect ordered fields and edit existing values with Undo/Redo. Restricted neutral Single/Double/Triple capture and asynchronous read-only preview pass headless checks. Actual native preview acceptance and field-edit UI remain pending. |
| File protection | Explicit overwrite, unsaved-change decisions, guarded retries and close-Save receipts. |
| Rendering | Shared CPU/GPU neutral-element scenes, centered multiple bonds, literal H, source-preserving derived MOL/SDF display H, component-aware hover and PNG captures. The centered style is original; new native label/bond workflows and reference style parity remain unverified. |
| Automation | 26 typed commands in one registry, used by the UI, CLI, stdio MCP and opt-in authenticated localhost control. |

Native drawing/file workflows have been inspected on macOS. Earlier hosted CI tested all
three platforms and window launches. GitHub Actions is disabled during development to
avoid hosted-run costs; local checks continue. New hosted checks and interactive
Windows/Linux workflows remain unverified. [Verification evidence](docs/verification.md)
distinguishes those scopes.

<table>
  <tr>
    <td width="50%" valign="top">
      <h3>Keep the document</h3>
      Supported CDXML opens and saves with source data retained. Unsupported imported edits
      fail explicitly. Atomic publication, overwrite confirmation and a saved baseline protect
      the delivered file workflow.
      <p><a href="docs/file-formats.md">Format capabilities and limits →</a></p>
    </td>
    <td width="50%" valign="top">
      <h3>Inspect every command</h3>
      The native menus and shortcuts use typed engine bindings. CLI and MCP discover the same
      command schemas; localhost control can address the actual desktop session.
      <p><a href="docs/control-protocol.md">Automation contract →</a></p>
    </td>
  </tr>
</table>

<p align="center">
  <img src="docs/media/format-chooser.jpg" alt="The actual Bondline Open structure dialog listing CDXML, CDX, MOL V2000/V3000, SDF and SMILES formats" width="100%">
  <br>
  <sub>Actual native format chooser. Import/edit/preview limits differ by format; see the <a href="docs/file-formats.md">capability matrix</a>.</sub>
</p>

## Get started

Source builds are available now. Signed installers and browser delivery are planned in M5;
there is no published installer release yet.

```sh
git clone https://github.com/thematheusmello/bondline.git
cd bondline
cargo run --locked -p bondline-ui --bin bondline
```

Rust **1.94.1** is pinned by the repository. Linux prerequisites and the independent test
environment are documented in [development.md](docs/development.md). Python is used for
independent developer test oracles, not as an application runtime.

Use File → Open/Save/Save As for supported CDXML, CDX, MOL or SDF documents. Cmd/Ctrl+O, S, Shift+S and W are wired;
actual Mac shortcut and accessibility-field evidence is in [native-file-ui.md](docs/native-file-ui.md).

```sh
# Discover the engine and the current control bindings.
cargo run --locked -p bondline-cli -- commands
cargo run --locked -p bondline-cli -- run --cmd '{"command":"ui.controls","params":{}}'

# Create a Double bond through the shared command; omitted order defaults to Single.
cargo run --locked -p bondline-cli -- run --cmd '{"command":"bond.create","params":{"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200,"order":"double"}}'

# Convert a supported 1–12 carbon path into a new CDXML file, preserving input.smi.
cargo run --locked -p bondline-cli -- run --cmd '{"command":"file.convert_smiles_to_cdxml","params":{"expected_revision":0,"input_path":"input.smi","output_path":"new.cdxml","bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,"page_width_pt":612,"page_height_pt":792}}'

# Create and export one explicitly identified fragment in the same headless session.
cargo run --locked -p bondline-cli -- run --cmd '{"command":"bond.create","params":{"page_id":1,"begin_x_pt":100,"begin_y_pt":200,"end_x_pt":140,"end_y_pt":200}}' --cmd '{"command":"file.export_smiles_fragment","params":{"expected_revision":1,"page_id":1,"fragment_id":1,"output_path":"ethane.smi"}}'

# Run a process-local stdio MCP session.
cargo run --locked -p bondline-cli -- mcp
```

Headless sessions do not pretend to open native dialogs. `ui.file` requires an attached
native host; explicit file commands remain available headlessly. To control the desktop
session, start it with `--control-port 0` and follow the [authentication protocol](docs/control-protocol.md).
Its startup token is a credential; keep it out of reports and screenshots.

## Files and the road ahead

| Area | Current state | Next work |
|---|---|---|
| CDXML | Source preservation, checked attribute changes, native elemental encoding and the supported app workflow | Broader source-preserving edits and mappings |
| CDX | Guarded shared/native Mac Open/Save, exact source retention, limited elemental metadata view and standalone syntax/property/new-tree APIs, checked independently | Faithful preview, broader metadata/chemistry, safe edits/new encoding and foreign/general corpus |
| MOL / SDF | MOL V2000/V3000 and mixed-version SDF native Mac files, with independent byte/record/data checks and restricted all-record headless SDF capture | Actual native preview acceptance, data-edit UI, broader metadata/layout support and interactive Windows/Linux |
| RXN | Source-preserving legacy container library with ordered roles and raw V2000/V3000 molecules; independent consumers | Reaction model, application/native files, V3000 reaction wrapper and broader extensions |
| SMILES | Complete accepted multi-record source Open/Save and explicit bounded record inspection through shared commands/CLI/MCP/loopback; exact opaque data/endings, explicit-H fragment export and restricted carbon-path→CDXML conversion | Chemical preview/editing of opened files, native acceptance of the wired file chooser, native export/conversion controls, general layout/valence, aromaticity/stereo and canonicalization |
| InChI | Planned | Reference-tool comparison and import/export |
| Publishing | Page PNG capture exists | SVG/PDF/PNG publishing, printing and clipboard workflows |
| Web / installers | Planned M5 | Approved browser dependency graph, packaging and signing |

The compatibility target includes CDXML and CDX files used by ChemDraw Professional.
Bondline is independent and is not affiliated with, endorsed by or sponsored by ChemDraw
or Revvity Signals. Reference names identify compatibility; no reference icons, fonts,
templates or application assets are bundled.

## Engineering and verification

The engine is independent of the GUI. Cargo enforces the crate layers; standalone format
crates preserve unknown data. Complete document snapshots support history, and immutable
background plans keep I/O and rendering outside the engine lock.

After setting up the pinned [external test consumers](docs/development.md):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask ci
```

GitHub Actions is disabled during development to control costs. Run the local checks
above; hosted checks will be restored when the project is complete. See [CI policy](docs/ci.md).

The aggregate also checks layers, declared native dependency licenses, bundled assets and
the pinned corpus. Current corpus coverage is limited: a passing gate is not full format or
chemistry parity. Parser fuzzing has its own [bounded workflow](fuzz/README.md).

| Read | Purpose |
|---|---|
| [STATUS.md](STATUS.md) | Exact integrated checkpoint, evidence and next executable tasks |
| [ROADMAP.md](ROADMAP.md) | Complete M0–M5 target |
| [Architecture](docs/architecture.md) | Crate responsibilities, commands and data ownership |
| [Parity checklist](docs/parity-checklist.md) | Sourced feature inventory with explicit missing/wired/verified states |
| [Source register](docs/clean-room/sources.md) | Public specifications and provenance decisions |
| [CI evidence](docs/ci.md) | Actual platform results and their limits |

## Contribute

<a href="CONTRIBUTING.md"><img src="docs/media/contribute.svg" alt="Build chemistry tools with us: contribute code, chemistry, tests or documentation" width="100%"></a>

Chemists, Rust developers, documentation writers and test authors are welcome. Pick a
concrete gap, cite its public source and bring an independent test. Small contributions
are useful: a clear reproduction, an original licensed fixture or a corrected workflow
record can move the project forward.

Start with [CONTRIBUTING.md](CONTRIBUTING.md). Use the [issue forms](https://github.com/thematheusmello/bondline/issues/new/choose)
for bugs and feature requests, [SUPPORT.md](SUPPORT.md) for questions and
[SECURITY.md](SECURITY.md) for private vulnerability reporting. Parallel agent contributors
follow the same [AGENTS.md](AGENTS.md), file ownership and verification rules.

## License and credits

Original Bondline code and documentation are dual-licensed under **[MIT](LICENSE-MIT) OR
[Apache-2.0](LICENSE-APACHE)**, at your option. Copyright © 2026 Bondline contributors.

Third-party software and fonts retain their notices. See [NOTICE](NOTICE),
[ATTRIBUTION.md](ATTRIBUTION.md) and the [dependency policy](docs/dependency-policy.md).
The banners are original project artwork; the screenshots show Bondline itself.
Their [sources and hashes](docs/media/SOURCES.md) are recorded separately from the approved
render-test golden. No paid service or copied reference artwork is required.
