# Bondline macOS text-value bridge patch

Reviewed 2026-11-13. This is a narrow source fork of published `accesskit_macos 0.17.4`,
with its package name, version, manifest and public API retained. It is not original
Bondline code. No dependency upgrade is required.

## Origin and retained notices

The exact crates.io archive is
<https://static.crates.io/crates/accesskit_macos/accesskit_macos-0.17.4.crate>.
Its SHA-256, verified against Bondline's lockfile before extraction, is
`bfc6c1ecd82053d127961ad80a8beaa6004fb851a3a5b96506d7a6bd462403f6`.
The archive's `.cargo_vcs_info.json` records AccessKit commit
`c1bd2d610615a244fd9e8bba0a6c3aecbfe46821`, path `platforms/macos`:
<https://github.com/AccessKit/accesskit/tree/c1bd2d610615a244fd9e8bba0a6c3aecbfe46821/platforms/macos>.

The package declares `MIT OR Apache-2.0`. Source headers additionally identify
Chromium-derived BSD code. All source headers, archive metadata, README and changelog
remain. The published archive omits the license files referenced by those headers;
the following complete files were fetched unchanged from the same exact Git commit.
Their Git blob hashes were checked against the public repository contents response.

| File | Git blob | SHA-256 |
|---|---|---|
| LICENSE-APACHE | `1b5ec8b78e237b5c3b3d812a7c0a6589d0f7161d` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |
| LICENSE-MIT | `31aa79387f27e730e33d871925e152e35e428031` | `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| LICENSE.chromium | `a32e00ce6be3620adca152e53734f7de8fda38cf` | `845022e0c1db1abb41a6ba4cd3c4b674ec290f3359d9d3c78ae558d4c0ed9308` |
| AUTHORS | `432f1b7bb779a9082fd10a4ef1e4f29c44b512c3` | `3bab8c36f6a85657504aaefb37c7ff34e29b8c917290f3d28a1f0920c992e502` |

For example, the retained BSD notice is sourced from
<https://github.com/AccessKit/accesskit/blob/c1bd2d610615a244fd9e8bba0a6c3aecbfe46821/LICENSE.chromium>.
The manifest's dual-license declaration does not replace that derived-code notice.

## Change and safety boundary

Published `src/node.rs` implements `setAccessibilityValue:` as an empty function while
allowing that selector for editable text. Actual Bondline native file-chooser controls
advertised writable values but did not change after an accessibility SetValue request.
The native RED observations are recorded by the UI verification owner; an input-model
test alone does not establish native accessibility dispatch.

The replacement dispatches `ActionRequest { action: SetValue, target, data: Value(...) }`
through the existing resolved-node context/action handler. It requires an explicit
SetValue action, an enabled editable text-input node, a string-valued node, no numeric
value, and an included accessibility node. It ignores nil and non-NSString values.
The existing VoiceOver selector-allowance condition remains unchanged.

AccessKit consumer 0.24.3 keeps its generic action query private. `Context` therefore
retains only the IDs that advertise SetValue in the actual incoming TreeUpdates.
Adapter updates refresh changed IDs and prune removed IDs against the updated tree.
This avoids cloning or serializing the accessibility tree for every setter request.

New unsafe code stays in this existing Objective-C OS bridge:

- The selector's nullable NSObject argument is checked with `isKindOfClass: NSString`
  before a read-only cast. It remains borrowed only for the synchronous selector call.
  The nullable ABI matches the pinned AppKit protocol declaration.
- NSString's UTF-16 length is checked before allocating at most 32768 UTF-16 units.
  `getCharacters:range:` receives an initialized buffer and an in-range length.
  Rust's checked UTF-16 conversion rejects lone surrogates; resulting values over
  32768 UTF-8 bytes or containing NUL are ignored. No unchecked string conversion is used.
- A native test constructs one initialized lone-surrogate NSString through the
  documented UTF-16 initializer to verify rejection. It does not forge a main-thread marker.

Existing upstream OS FFI and unrelated TODOs are unchanged. This exception does not
permit unsafe code in Bondline's core crates or copying unrelated platform behavior.

## Verification and continuation

On macOS/arm64, Rust 1.94.1: three native Objective-C/AccessKit tests passed after
an intended RED failure for the missing bridge helper. They independently assert the
exact Unicode target/action/payload and rejection of unsupported actions, read-only or
disabled nodes, non-input roles, NSNumber, oversized UTF-16/UTF-8, NUL and malformed UTF-16.
The cache test verifies action withdrawal, unchanged nodes in partial updates, and actual
consumer-tree deletion without constructing or forging a main-thread marker.

The independent native UI verifier then exercised actual accessibility SetValue on the
Folder and File name controls, including Unicode `native-file-ui-AX-λ.cdxml`, navigated
with Go, and saved successfully with the filename updated and dirty state cleared.
The verified executable SHA-256 is
`6a4ff7786a5a74b51710c95e85618536b69a3acee7383a236e7857b89444f891`.
After the cache-helper refactor, the exact final binary
`f331610a4689105b11495016c18513e3a2a69c04921acfeb82c3e25e93961b1d` was independently
rechecked for actual Folder/Unicode-name SetValue, Save/Open, and dirty Quit Cancel/Save.
Its saved two-fragment file passed external XML/RDKit checks; the app was closed.
Conversation screenshots/accessibility state and the ignored native evidence record
describe those observations. This proves these two controls on this Mac build; general
VoiceOver workflows, other accessibility controls, and other platforms are unverified.

From the repository root, with this directory excluded from the workspace and the
root path patch enabled, these checks use the root's locked dependency graph:

```sh
CARGO_TARGET_DIR="$PWD/target/accesskit-macos-patch" cargo test -p accesskit_macos --lib --locked
CARGO_TARGET_DIR="$PWD/target/accesskit-macos-patch" cargo clippy -p accesskit_macos --all-targets --locked -- -D warnings
```

The root-lock native tests passed too. Initial RED/GREEN and clippy checks used the
standalone `--manifest-path vendor/accesskit_macos/Cargo.toml` invocation before the root
patch was enabled; its dependency versions matched the root graph. The generated standalone
Cargo.lock was removed. Generated locks and all build outputs must not be committed.
Evidence logs are disposable
`target/accesskit-macos-red.log`, `target/accesskit-macos-green.log` and
`target/accesskit-macos-clippy.log`, `target/accesskit-macos-root-lock-tests.log`, and
`target/accesskit-macos-root-lock-clippy.log`. The first clippy pass caught an MSRV 1.75-incompatible
pointer helper; the patch uses an equivalent older-compatible cast instead.
