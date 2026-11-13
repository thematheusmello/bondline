# Clean-room source register

Reviewed 2026-11-13. A source is permission to learn documented behavior, not to copy
proprietary implementation, help text or assets. Record more specific sections as features land.

| ID | Public source | Decision / limitations |
|---|---|---|
| CDX-INTRO | https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm | Binary header, little-endian records, object/property lengths and terminators. No ChemDraw executable inspected. |
| CDXML-INTRO | https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDXML.htm | XML object/attribute mapping, flexible numeric spelling, opaque preservation. |
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
| REA-AUDIT | https://github.com/morluto/rea/blob/main/docs/release-exposure.md | External inspection of Bondline artifacts only; not proof of security or feature parity. |
| CORPUS-RDKIT | https://github.com/rdkit/rdkit/tree/c686c2c945df370c2f95ebe20aae2872a36f7c17 | BSD-licensed independently authored interchange fixtures; exclude proprietary template collections; record per-file provenance/hashes when added. |

Engineering documents and third-party MIT/Apache/BSD libraries may be inspected or reused
with their notices. This does not relax the prohibition on proprietary product code/assets.
