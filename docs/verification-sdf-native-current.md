# Native SDF file check and preview limit

2026-11-15, actual macOS Bondline PID41233, after the user requested the desktop preview.
This is a finite native chooser/summary/persistence observation, **not successful native
chemical-preview acceptance**. The original saved drawing was restored and left open.

The existing native binary is21,188,872 bytes/SHA-256
`af8a72ed079d15c32b4c1d68c2f7cf891ba5ab56f34029ae8505dbb903259d93`,
from the ordered-File build recorded in verification-ordered-file-actions.md. It predates
the new SDF field-work bridge; no field inspector/editor was present. No proprietary
program or asset was inspected and no native input was delegated.

## Observations and actual output

An exact copy of the existing original740-byte
`crates/io/tests/fixtures/sdf-preview/valence-absent-mixed.sdf` was placed under ignored
`target/sdf-native-current/Bondline ordinary β.sdf`. SHA-256:
`e5cbdc14329c59bad55148c5de37058de90f651af10966c30cf8b89b3b5efd39`.

The actual Command+O chooser opened that Unicode file via its row. The workspace showed
the complete2-record/740-byte summary and read-only role. Expanding Records showed
record1 V3000/410 bytes/1 field and record2 V2000/330 bytes/1 field. Drawing tool controls
were absent; no native SDF authoring or hit test is claimed. Native Save As, using clipboard-preserving Unicode paste, wrote
`Bondline native source λ.sdf`; its full740 bytes equal the input exactly. Selecting the
saved file's actual chooser row reopened the same collection without a dirty warning.

Both actual saved MOL spans were supplied on stdin to the unchanged pinned
`crates/io/tests/mol_oracle.py`, using RDKit2026.09.2. Literal expectations pass:
`[0,371)` CCO/C2H6O/H[3,2,1] and `[410,713)` CN/CH5N/H[3,2]. Whole-byte equality,
including original field text and record framing, is the preservation evidence; an
external molecular property dictionary is not a raw SDF preservation oracle.

Command+O with clipboard-preserving paste then reopened the prior952-byte original
CDXML drawing. The actual screenshot again showed Single/Double/Triple at retained150%.
Read-only loopback inspection confirmed revision6, clean file state and view epoch1.
No loopback token is included in public records. No source fixture was overwritten.

## Limitations and setup corrections

The default window visibly refused chemical preview with the existing8..256 em guard.
The returned window image was1160×832; effective native DPI/client origin were not
instrumented and must not be inferred from screenshot dimensions. Fullscreen shortcut
and a bottom-right resize drag produced no observed size change. Neither establishes
successful resizing, another DPI, actual labels or a two-record chemical image. The
[four measured headless cases](research/sdf-native-acceptance.md) retain their separate
DPI1 refusal/DPI2 success scope. No guard or fixture was weakened to manufacture a pass.

Initial `typeText` omitted β from the path; failed Open preserved the old drawing and
the chooser listed the correct filename. Choosing that actual row corrected input.
Later Unicode paste and actual-row selection succeeded. This is not a parser or Unicode
filesystem failure. The first external-oracle invocation mistakenly passed a path as
argv instead of stdin, so it rejected empty input; the corrected invocation used the
unchanged consumer and actual literal spans. Neither setup error is a product RED.

Ignored evidence: `target/sdf-native-current/input.json`, exact copied/saved files,
two actual MOL spans and `native-receipt.json`. Native observations are retained in the
computer-use trace; no new screenshot/golden asset or human approval is claimed.

Still unverified natively: successful chemical preview at an eligible view, DPI/resize
invalidation, Cancel/Retry timing, source/edit cache retirement, queued worker/close
barriers and field presentation. Existing independent actual-App scheduling/texture
checks remain decisive within their own scope. This check does not complete M0 or M1,
and Actions remains disabled.
