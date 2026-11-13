# Pinned corpus metadata checkout

The first SDF corpus source504be5f passed Mac/Linux/sanitizer jobs in
[run43515232724](https://github.com/thematheusmello/bondline/actions/runs/43515232724),
but Windows job132195408100 failed the original-manifest identity assertion. Its physical
checkout had1067bytes/SHA cb9c0ddd07eadf12ee4c1723dbb4431746e6d656833d7f83490563b9e04fec3a;
the committed unchanged LF manifest is1041bytes/SHA
820739ca30312745b53df2839f0238b30d1470e73446430b4c954a142cb88ca0.

This was Git checkout conversion, not a change in license declarations or source pins.
Converting the original LF bytes to CRLF reproduces the exact hosted hash. An actual
disposable repository/clone with isolated configuration and core.autocrlf=true reproduces
that failure. Adding only `/docs/corpus/manifest.json -text` gives a fresh1041-byte checkout
with the exact original SHA and a passing identity check. Git's public unset-text rule
disables checkin/checkout conversion: https://git-scm.com/docs/gitattributes#_text .

The correction preserves original metadata/assertions. It does not normalize reader input,
rewrite the manifest or broaden the dependency/fixture license policy. The accompanying
CDX fixture byte rule likewise protects existing original binary vectors; no parser behavior
changes. Local evidence is under target/sdf-manifest-autocrlf-yoq1rmg2: red-check.log,
green-check.log, red-metadata.json and verification.json. No global Git configuration changed.

JSON semantics tolerate either line ending; the stronger immutable-source assertion is
intentional. The source file itself must retain the reviewed bytes across checkouts.
Corrected hosted execution is recorded after the new configuration is published. The
failed Windows run remains a failure; completed jobs are not silently relabelled or rerun.
