# Desktop capability-chart audit

Reviewed2026-11-13. This is reference inventory research, not implemented behavior or a
complete26.1 menu inventory. No reference application binary was acquired or run.

The [official chart](https://assets.revvitysignals.com/asset/0b9656e8-339d-47ab-9834-6c08beb6abee/Full-Dot-Chart-ChemDraw-Suite-of-Products.pdf)
returned862778bytes, SHA256
`3a90af112d7f25623decddb3d28bc68229ffedb12be51a5c92208e149d8100e3`.
It has13 pages. Metadata creation/modification is2026-05-18; the visible final footer says
May2026. All13 Poppler-rendered pages were visually inspected, including Professional
column markers, blank cells and platform labels. **This establishes no26.1 build.** Pages
below are one-based PDF pages; the printed page is one less. The matrix describes advertised
capabilities, not menu placement, numeric defaults, algorithms or reproduced workflows.

The [December2025 URL](https://assets.revvitysignals.com/m/ac400415cba91ceb/original/Full-Dot-Chart-ChemDraw-Suite-of-Products-December-2025.pdf)
returned404. No bytes/hash or December-to-May comparison is claimed. Raw PDF/renders and
acquisition/image hashes remain ignored under .cache/research/capability-charts/; no vendor
image, artwork, template or prose is bundled. The comparison checkpoint is the checklist
SHA256510a153e4dc26a1a33ccd905583906f542a56e0f9b98dcc285af25c5f35e4cb2.

The following refinements have a Professional marker. They extend existing anchors rather
than inflate the576-control count. They are all missing, with no implementation claim.

| ID | PDF page; existing anchor | Explicit requirement and unresolved evidence |
|---|---|---|
| CC-01 |3; ME-009/KM-012 | Pasting into occupied drawings should avoid overlap. Determine displacement and selection/history rules from behavior; preserve graph/style/opaque data. |
| CC-02 |5; DP-003/015 | A document style can suit a dark drawing background, distinct from the application's UI theme. Author original values and test persistence/export; no stylesheet copying. |
| CC-03 |5; MS-019–021 | Include atropisomer and axial/allene handedness descriptors, beyond R/S and enhanced groups. Establish standards-based inputs, transformations and independent chemistry vectors. |
| CC-04 |6; TQ-032–034 | An interpretation option can override a document-level chirality flag. Flag kinds/scopes/defaults remain unknown; preserve source flags. |
| CC-05 |5–6; MS-004/TD-020/TB-031 | Include hydrogen-bond relationships in3D cleanup and fragment grouping. Selection, analysis/export and connectivity effects are unspecified; no covalent merging is inferred. |
| CC-06 |8; DP-028/029/R26-007 | Explicit descriptor coverage includes tPSA/MR/LogP on Win/Mac and pKa/LogS/cLogP on Win. Methods and numeric oracles are absent. The reference's Win limitation need not constrain Bondline's independent cross-platform engine. |
| CC-07 |10; format overview/ME-016 | Record SKC/RDF read-write, CIF input, EPS/GIF/JPEG/BMP/TIFF output and SLN paste, with explicit RDF/RXN V2000/V3000 distinctions. Require public specs/independent consumers; extension recognition is insufficient. |
| CC-08 |6; TB-025/031/DP-008 | Distinguish uncertain residue tokens, multi-monomer selection and monomer search/replacement. Retain ambiguity and selection scope through persistence. Library versioning is already covered. |
| CC-09 |9; MS-017/MX-030 | Keep CAS-RN resolution and SciFinder connector roles visible alongside existing integrations. Public APIs/access limits remain research; ordinary offline creation must never require these services. |

PDF5/10 assigns additional3MF features to Signals; preserve the older advanced-export
backlog with tier/version qualifications. PDF6 distinguishes Professional local monomer
editing from Signals library sharing: offline libraries must not gain an account requirement.
PDF12–13 lists Win suite companions; their commands are not editor menu evidence. ChemScript
platform labels differ between PDF2 and13, so existing integration rows retain that question.
Web-only/admin/licensing/commerce items add no offline editor requirements. The chart's web
accessibility claims do not verify desktop accessibility.

Next work: carry these cases into M1/M3/M4/M5 tests when their components land. Public
standards, independent consumers and actual before/after native workflows remain necessary.
No complete inventory, exact screen parity or successful functionality follows from a marker.
The bounded acquisition/render/research jobs exited; no research browser or process remains.
