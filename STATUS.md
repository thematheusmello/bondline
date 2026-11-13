# Status

2026-11-13: goal active; initial repository documentation checkpoint.

Verified: empty starting directory, local Git identity, Rust 1.94.1, public CDX/CDXML
specifications, reference-project licensing/architecture inspection. No application code yet.

Desktop licensing work is active, not an accepted permanent limitation. PhotoCraft uses
pinned egui/winit forks and asset provenance but ships licenses outside Bondline's stricter
policy. Reuse the pin/fork mechanism, not those exceptions. VoiceStudio's AGPL code is excluded.

Next: write a failing runtime-license closure test, select minimal graphics features, replace
remaining non-allowlisted runtime dependencies with audited compatible implementations,
then compile and launch the actual native UI. Full M0 is not complete.

No running jobs at this checkpoint. Platform CI and application checks have not run.
