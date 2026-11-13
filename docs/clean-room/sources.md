# Clean-room source register

Reviewed 2026-11-13. A source is permission to learn documented behavior, not to copy
proprietary implementation, help text or assets. Record more specific sections as features land.

| ID | Public source | Decision / limitations |
|---|---|---|
| CDX-INTRO | https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm | Binary header, little-endian records, object/property lengths and terminators. No ChemDraw executable inspected. |
| CDXML-INTRO | https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDXML.htm | XML object/attribute mapping, flexible numeric spelling, opaque preservation. |
| CDX-COORDS | https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/DataType/CDXCoordinates.htm | CDXML points (72 per inch), origin at top-left, positive right/down; binary coordinate order differs and is not inferred from XML. |
| PAGE-UNITS | https://www.w3.org/TR/css-values-4/#absolute-lengths | Physical point/inch/mm conversion, independent numeric test reference. |
| PAGE-SIZES | https://www.w3.org/TR/css-page-3/#page-size-prop | A4 210 by 297 mm. A4 as Bondline's initial page and the 100,000-point authoring bound are independent product choices, not reference parity claims. |
| MAC-HISTORY | https://support.apple.com/en-us/102650 | Documented Cmd+Z and Shift+Cmd+Z history shortcuts. Non-macOS currently uses Ctrl in the same bindings as a Bondline policy, pending reference-platform inventory. |
| CDX-ARCHIVE | https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/ | Archived 2019 documentation, recovered 2024; not proof of all modern properties. |
| CDX-OBJECTS | https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/AllCDXObjects.htm | Public document/page/fragment/node/bond/text and other object inventory. |
| PROFESSIONAL-26.1 | https://support.revvitysignals.com/hc/en-us/articles/54009539308052-Release-Announcement-ChemDraw-Desktop-26-1 | Public desktop release baseline; separate from cloud-only features. |
| GUIDE-25 | https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf | Publicly hosted vendor-authored 25 guide; inventory still to be extracted and reviewed; no help text/assets copied. |
| PHOTO-ARCH | https://github.com/storytold/photocraft/blob/main/docs/architecture.md | Engine/command separation, standalone formats, CPU/GPU comparison. |
| PHOTO-FORKS | https://github.com/storytold/photocraft/blob/main/Cargo.toml | Pinned egui/winit fork mechanism. Uses a broader license/asset set than Bondline. |
| PHOTO-ASSETS | https://github.com/storytold/photocraft/blob/main/ATTRIBUTION.md | Asset provenance and pinned, SHA-verified corpus practice. ISC/OFL assets are NOT adopted. |
| VOICE-REVIEW | https://github.com/debpalash/VoiceStudio/blob/main/docs/licensing/model-review-workflow.md | Evidence records distinguish declarations, exact bytes and review state. Conceptual reference only; AGPL code not copied. |
| RUST-GUIDE | https://github.com/apollographql/skills/blob/main/skills/rust-best-practices/SKILL.md | Ownership, typed errors, lints and focused tests; verify recommendations against actual APIs. |
| MCP-TOOLS | https://modelcontextprotocol.io/specification/2025-11-25/server/tools | Shared tool schemas, discovery, invocation and errors. |
| MCP-SDK | https://docs.rs/rmcp/3.9.0/rmcp/ | Official Rust SDK, server and stdio only; inspected ServerHandler, structured tool results, lifecycle and bounded AsyncRwTransport APIs. |
| TOKIO | https://docs.rs/tokio/1.54.1/tokio/runtime/struct.Builder.html | Native adapter runtime with I/O/time drivers; engine and model remain independent of the runtime. |
| TCP-LIFETIME | https://doc.rust-lang.org/std/net/struct.TcpListener.html | Bind only IPv4 localhost, obtain the actual port after bind(0), and close the listener with its owned lifetime. APIs checked on the pinned compiler. |
| TOKIO-TAKE | https://github.com/tokio-rs/tokio/blob/tokio-1.54.1/tokio/src/io/util/take.rs | Bounded AsyncBufRead before JSON parsing; join/cancellation and deadlines are adapter responsibilities. |
| JSON-GRAMMAR | https://www.rfc-editor.org/rfc/rfc8259 | JSON request grammar. LF framing, connection/deadline caps and reject/close policies are independent Bondline design decisions, not reference parity claims. |
| SCHEMARS | https://docs.rs/schemars/1.2.3/schemars/ | Generate schemas from the exact serializable parameter types; closed objects and validation bounds are tested. |
| PROPTEST | https://docs.rs/proptest/1.12.0/proptest/ | Fixed-seed generated edit sequences independently assert exact undo/redo invariants and expected page dimensions. Build/test dependency only. |
| REA-AUDIT | https://github.com/morluto/rea/blob/main/docs/release-exposure.md | External inspection of Bondline artifacts only; not proof of security or feature parity. |
| WGSL-LITERALS | https://www.w3.org/TR/2026/CRD-WGSL-20261106/ | Exact binary floating point literals; independently implemented conversion tested with IEEE-754 bit vectors. |
| DLOPEN2 | https://docs.rs/dlopen2/0.9.0/dlopen2/raw/struct.Library.html | MIT backend for OS library ownership and symbol resolution; implementation inspected from the published crate. |
| WIN-MODULE | https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandleexw | Existing modules acquire an owned reference without rerunning library initializers; release with FreeLibrary. |
| WIN-SYMBOL | https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getprocaddress | Resolve by validated symbol name while retaining that module reference. |
| WIN-CLIPBOARD | https://learn.microsoft.com/en-us/windows/win32/dataxchg/using-the-clipboard | Clipboard transaction/window ownership and global memory transfer; no example source copied. |
| WIN-CFHTML | https://learn.microsoft.com/en-us/windows/win32/dataxchg/html-clipboard-format | UTF-8 and byte-based fragment offsets with bounded validation. |
| WIN-FILES | https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-dragqueryfilew | Lossless UTF-16 file list query; bounded paths/count. |
| ROBOTO | https://github.com/googlefonts/roboto-2/tree/38062f4b4a0be4346d07a928408da21602545e9e | Independently sourced Apache-2.0 UI font; exact file hash/notice in assets/manifest.json. |
| PYTHON-IEEE | https://docs.python.org/3/library/stdtypes.html#float.hex | Independent IEEE reference conversion. Original seeded test vectors/generator retained under compat/hexf-parse/tests and scripts. |
| CARGO-GRAPH | https://doc.rust-lang.org/cargo/commands/cargo-metadata.html | Resolve opaque package IDs and dependency edges across targets/features for the layer gate. |
| CORPUS-RDKIT | https://github.com/rdkit/rdkit/tree/c686c2c945df370c2f95ebe20aae2872a36f7c17 | BSD-licensed independently authored interchange fixtures; exclude proprietary template collections; record per-file provenance/hashes when added. |

Engineering documents and third-party MIT/Apache/BSD libraries may be inspected or reused
with their notices. This does not relax the prohibition on proprietary product code/assets.
