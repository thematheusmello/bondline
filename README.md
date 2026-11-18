# Legacy worker recovery archive

This tag preserves 33 unverified Rust/Markdown file variants from older Bondline
worker checkouts. It is a recovery snapshot, not an application release or a feature
branch to merge. The verified application lives on main.

Original bytes, blob IDs, SHA256 hashes and ownership are in manifest.json. The
same files remain in their original local worktrees; no working file was removed.
Review and test a recovered change against current main before using it. Old MCP
listener/prototype changes must not bypass current tests or source-authority rules.

Fetch explicitly with `git fetch origin tag archive/legacy-worker-recovery-2026-11-18`.
Original Bondline sources retain their repository MIT OR Apache-2.0 terms and any
existing file notices. No proprietary assets, fonts or binaries are in this snapshot.
