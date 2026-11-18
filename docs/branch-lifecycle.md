# Branch delivery and recovery

The user authorized landing completed branches and PRs into `main` on2026-11-18.
Main is the integration line throughout development; completion of M5 is not a condition
for merging a verified feature. GitHub Actions remains disabled under the cost policy.

## Delivery rule

1. Start a focused branch from current main and record one concrete task/owner in STATUS.
2. Keep source, independent tests and relevant evidence together in atomic commits.
   Push every commit immediately and verify the remote ref.
3. Complete the applicable local gates and review. Mark unsupported/unverified scope
   honestly. A planned API, disabled tool or compiler probe is not a finished workflow.
4. Open a PR into main and land the verified slice promptly. Preserve the atomic commits
   with a merge commit. Use the expected PR head SHA; resolve any changed base/conflict
   and validate the actual result. Do not bypass protection or turn on hosted Actions.
5. Retire merged remote branches after proving their tips are reachable from main.
   Keep active work on a small, named branch with an exact next task. Archive superseded
   worker history and preserve unverified working files before any cleanup.

The user’s authorization persists for ordinary verified feature integrations. Force-push,
history rewriting, release publication and enabling Actions require their own authority.
Do not merge an obsolete prototype over current code merely because Git calls it unique.

## Current CDX delivery

[PR 3](https://github.com/thematheusmello/bondline/pull/3) and
[PR 4](https://github.com/thematheusmello/bondline/pull/4) landed the resource and
raw string readers. Both source branch tips are reachable from main and their
remote refs have been retired. [PR 5](https://github.com/thematheusmello/bondline/pull/5)
tracks the caption preview. Its full local gate and actual macOS workflow are
recorded in [caption verification](verification-cdx-caption-preview.md).

## Main integration checkpoint

[PR1](https://github.com/thematheusmello/bondline/pull/1) merged199 atomic commits at
`96794f4573237aa019e6fb1c28aa65113547c38e`. Main’s tree exactly equals reviewed candidate
`e7dcd850cdaead345858fedd965e3b0f9275eae5` (`1229f81fcf45ea147c5cd486f5d4b732da8c2ab2`).
Code matches the clean local2882 gate on1606d62: workspace2187/0/1 plus vendor3/0 and all
formatting/lint/layer/license/asset/corpus checks. Later candidate changes are Markdown.
The uncommitted CDX resource tests/fixtures/decoder work are excluded from that landing.
No redundant build is claimed for an identical tree, and historical MCP failures stay open.

The [inventory](branch-inventory-2026-11-18.json) records all preserved refs:

- All42 former remote heads were contained in the reviewed candidate. After landing,
  41 merged feature/policy remote branches were retired; main was retained.
- 158 inactive local heads moved to `archive/2026-11-18/<original-name>`. Commit IDs,
  reflogs, worktree paths and working files are preserved. No source directory was removed.
- Active work is `feat/cdx-resources-integration` with core worker `feat/cdx-resources`.
  The planned standalone resource reader is still in progress, with no implementation
  or compiler/runtime verification claimed at the checkpoint.

An ancestry/patch audit found nine local feature patches already received through four
integration commits. One historical canvas checkpoint remains recovery material, not a
feature to merge blindly. This was a bounded provenance audit, not an exhaustive code review.

## Unverified worker recovery

The73-worktree audit found13 clean and60 dirty checkouts. Nine dirty checkouts already
matched the candidate. Thirty-three older file variants across28 Rust/Markdown paths
were not reachable from existing refs. Their exact bytes and ownership are preserved in
the remote tag [archive/legacy-worker-recovery-2026-11-18](https://github.com/thematheusmello/bondline/tree/archive/legacy-worker-recovery-2026-11-18),
commit `140754baddfe11f91c9bb4a6675b94f0924dfe8b`, with a manifest and independent hashes.
This tag is an unverified source snapshot, not an application release or an open feature PR.
The original files remain in their worktrees. Active resource WIP is excluded and preserved.

Fetch explicitly to inspect or recover it:

```sh
git fetch origin tag archive/legacy-worker-recovery-2026-11-18
git show archive/legacy-worker-recovery-2026-11-18:manifest.json
```

Recover a useful change onto a fresh branch from main, compare it with current behavior,
and add decisive tests before integration. In particular, archived MCP/test prototypes
must not weaken the protected listener policy or replace the unexplained-failure record.
Private startup logs, ignored targets, user apps/drawings and proprietary assets were
not inspected or copied into this archive. Recovery is tracked separately from active delivery.
