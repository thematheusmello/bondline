# Original CDX caption creation witnesses

The files and generator are MIT OR Apache-2.0. The baseline is the
[original existing-caption fixture](../cdx-caption-edit/SOURCES.md)
`single_before.cdx`, 156 bytes, SHA-256
`05b1a1c45dbd11611223a5907c17986eea5f675008c71e6b2116c508bc5699c5`.
It contains Page 90, original Text 73 and an explicit UTF-8 Roboto
table. The source contract records published caption and ObjectTag
fields and the original Bondline ID policy.
[Contract](../../../../../docs/research/cdx-caption-creation.md).

Python builds one child with raw ID 91, baseline (144.25, 216.75) pt,
depth 1, left alignment and 12 pt black Roboto Regular. Its body is
`Phase β & q`. The unreferenced ObjectTag attaches directly to that
Text. The reader independently checks matched owners, payloads and
framing. Deleting the single inserted interval restores the complete
input, including its original extended property lengths and resources.

| File | Identity expectation | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| first.cdx | Existing ID 1, appended ID 2 | 268 | 676d779de38f350c5c86e63ab0305e8b95863ce487e34bdcc6199863f5a6727f |
| after_undo.cdx | Existing ID 1, appended ID 3 | 268 | 3e1e81733a583f172c823c21aeef6dd4cb120978ac40c64dc5d54a6a8188c777 |
| implicit_persistence.cdx | ID 3 with documented implicit persistence | 263 | 610d15954c140e85b0d5e2c490a259d89fde406aa3be2984f84a81064da59e53 |
| duplicate_live_id.cdx | Refuse a second ID 1 | 268 | 93feb867c1ecfbccd7a8321f0bb6e60368b40553ede3fffa0de65f89c4ce039e |
| unknown_version.cdx | Retain opaque version 2 metadata | 268 | f6c6629740f44145e0612883753cb08655f31aa5a5a863e5ff0d086405860857 |
| nonpersistent.cdx | Retain a tag with explicit false persistence as opaque | 268 | 503a526050ab54dae8c2d6fd3b1666f8a8f5daab78bb4fffbb812f1992c7fa7f |

The generator confirms structural and byte properties. Bondline
admission, rendering and runtime acceptance require receiving tests.

```sh
python3 -B crates/io/tests/fixtures/cdx-caption-create/generate.py
```
