# Reviewed wgpu browser patch

2026-11-20. Upstream [wgpu22.1.0](https://github.com/gfx-rs/wgpu/tree/5c5c8b1d4d2d965fbd10b290ee26f4e7eb158d7c/wgpu), MIT OR Apache-2.0.
Published crates.io archive155280B, SHA-256
`e1d1c4ba43f80542cf63a0a6ed3134629ae73e8ab51e4b765a67f3aa062eb433`. VCS`5c5c8b1d4d2d965fbd10b290ee26f4e7eb158d7c`,
path`wgpu`. All144 published files are retained; only the files
in the table below differ. Original full notices remain byte-exact beside this file.

Actual Chrome157 rejects the removed `maxInterStageShaderComponents` descriptor
key. The WebGPU backend detects the available limit name. Legacy browsers retain
the old component field; modern browsers use `maxInterStageShaderVariables`.
Advertise a conservative old component capacity equal to the modern variable count,
since an individual scalar may occupy one variable. Never multiply by four or
claim these two limits are generally interchangeable. Reject invalid numeric
capacity by advertising zero. The browser UI separately clamps its required limit
to the advertised bound. All other descriptor fields and native backend sources
are unchanged. This does not add support for arbitrary shader interfaces.

| Changed file | Published bytes / SHA-256 | Patched bytes / SHA-256 |
|---|---|---|
| src/backend/webgpu.rs | 144070 / `a71e9ca5172dbad6858583113c96d65efdc0864bdbbc92d45f941071f86426e9` | 145256 / `41483d7cf8923b26c67ad76b0694ef41ff2c279417a789d6aeb67632345081b0` |

Byte-exact upstream notices:

- [LICENSE.MIT](LICENSE.MIT), SHA-256`c7fea58d1cfe49634cd92e54fc10a9d871f4b275321a4cd8c09e449122caaeb4`.
- [LICENSE.APACHE](LICENSE.APACHE), SHA-256`a6cba85bc92e0cff7a450b1d873c0eaa2e9fc96bf472df0247a26bec77bf3ff9`.

[Research](../../docs/research/web-workspace.md) records the official sources,
and [receiving evidence](../../docs/verification-web-workspace.md) separates real
Chrome WebGPU execution, native checks and the strict target-specific runtime audit
from unverified browsers and complete M5. Keep this a local reviewed fork; no
upstream acceptance or broad browser compatibility is claimed. Reinspect actual
consumers and rerun the workflow/license gates when upgrading.
