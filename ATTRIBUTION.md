# Attribution

Original Bondline code, documentation, procedural app icon and documentation banners are
Copyright © 2026 Bondline contributors, licensed MIT OR Apache-2.0. Root license files apply.

| Component | Source and permission | Retained record |
|---|---|---|
| UI font | Google Roboto, Apache-2.0 | [Font notice](assets/fonts/LICENSE-Roboto.txt), exact revision/hash in [asset manifest](assets/manifest.json) |
| Native framework source forks | eframe, winit and AccessKit; their upstream MIT/Apache/BSD notices remain adjacent | [Dependency policy](docs/dependency-policy.md), vendor BONDLINE-PATCH.md files |
| Compatibility adapters | Original Bondline implementations; package names retained for Cargo patch routing | [Agent instructions](AGENTS.md), [dependency policy](docs/dependency-policy.md) |
| Interchange test corpus | Reviewed RDKit fixtures, BSD-3-Clause; downloaded files are not committed | [Corpus manifest](docs/corpus/manifest.json), [BSD notice](docs/corpus/rdkit-license.txt) |
| Documentation visuals | Original banners and actual native Bondline captures | [Media sources, build identity and hashes](docs/media/SOURCES.md) |

Cargo.lock pins the dependency graph. `cargo xtask licenses` checks the declared native
runtime closure and two bundled runtime assets; it is not a completed release source/notice
audit. In particular, the missing upstream windows-permissions notice remains recorded in
[native-file-io.md](docs/native-file-io.md). Documentation media are not runtime app assets.

PhotoCraft's public README informed the presentation structure. Its screenshots, logos,
artwork, prose and community branding were not copied. Engineering references and their
precise adoption limits are in [reference-projects.md](docs/research/reference-projects.md).
