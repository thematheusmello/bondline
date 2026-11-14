# Combined SMILES normalization/export fuzz checks

Root adds invariant coverage to the existingsmiles_parse target, not a new parser,
chemical engine or application flow. Original production/IO tests/oracles/dependencies/
locks/fixtures/golden are unchanged. Fixed public-spec/external-consumer vectors in
verification-smiles-write.md and verification-smiles-normalize.md remain independent
interoperability evidence; these generated invariants supplement them.

## Assertions and independent review

Successful normalization is checked directly against source IR: dense local identities,
elements, original positive isotopes, neutral charge, exact bracket H, independently
recomputed finite implicit H, and every source endpoint/order. Positions remain absent;
source bytes and parser tokens stay exact. Unsupported chemistry/zero-isotope data may
refuse explicitly; successful valid normalization cannot become an invalid model silently.

A cloned test graph uses unique positive isotope markers derived from dense IDs. Export
and reparse must retain every marked atom/element/literal H and every undirected edge/order,
regardless of DFS output order; repeated output must be identical. Physical existence of
markers is not claimed. The writer may refuse documented resource budgets. This does not
assert arbitrary-mutation chemistry using our own parser as an independent consumer.

Read-only review first identified a real test gap: source-to-normalized topology was
not compared, so same-count rewiring could escape the export-to-normalized check. Root
added the direct IR comparisons above before the final run. The initial weaker-harness
10000-run pass remains in ignoredsmiles-export-fuzz-asan-initial.log; it is not the final
source's evidence and was not a production failure or manufactured RED. Original TDD REDs
for both library features remain in their specific records.

Final reviewtarget/smiles-export-fuzz-review.md7711B/SHA
5ea4f10976ee5689f8b0c8947644e485aa6f6f107e0ad5aba1608d84ae632956 retains the finding and
resolution. No further confirmed defect; no reviewer execution. Its mechanical reversal
recovers all original151-line parser assertions exactly5795B/SHA
1a490077b248f4cba02cba5642ab2cfe4a3a88aef23985ab4478bebc05348eb5.
Final harness11921B/SHA38fbd779a1e1f687dc48ab4010c76cc559380774249d521991a17ad878038382.

## Actual final execution

The normal isolated fuzz manifest/unchanged lock use cargo-fuzz0.13.2, libFuzzer0.4.13,
nightly-2026-04-01 and AddressSanitizer. Own absolute Root/fuzz/target; incremental0,
release-debug0; no minimal alternate graph, new installation or shared worktree target.
Full-graph stable all-target clippy-Dwarnings exits0, final log163B/SHA
3973c86b8d96f227ff007dd9273131b6f00fe30b82183837f265db0c19504979.
Workspace and isolated fuzz rustfmt checks pass.

Fresh ignored.cache/fuzz/smiles-export-final-20261114 contains five original seeds:
three unchanged committed licensed17/21/21B files plus original4097B carbon-chain and20B
multiring vectors,4176B total. The seed recordtarget/smiles-export-fuzz-seeds.json hashes
every initial byte; mutations never touch committed fixtures. CI still starts with its
three original SMILES seeds, separate from this local five-seed set.

Final session45229 exits0:10000 iterations, seed703592681, max_len65536, timeout5,
coverage1867/features6734,246retained inputs/reported309KiB, reportedRSS474MiB,6seconds
after compilation. Largest starting/exercised input4097B;65536 is a ceiling, not exercised
coverage. Large/deep/dense/boundary unit tests remain separate. Logtarget/smiles-export-fuzz-asan-final.log
48350B/SHA816a138fb8a77dcc261bb99765a39d961e2a8b438b5114285934e566234a35ee.
This bounded smoke is not exhaustive conformance, chemistry verification or a security audit.
No application import, backing/Save authority, canonicalization, layout/native flow or M1
completion follows. Exact-source hosted execution remains separate and initially pending.


## Hosted receiving-source acceptance, 2026-11-14

Exact sourcee6471961bc88eeb97a0bb7a8fe115c1bed2392e7 passes
[run43583494325](https://github.com/thematheusmello/bondline/actions/runs/43583494325),
terminal success10:15:28UTC. Root inspected all four latest-attempt job outcomes and
successful logs: Mac132413004015:634 (631workspace+3bridge), Linux132413003718:632,
Windows132413003985:624, plus ASAN132413003970:seven10000-run parser targets. The normal
hosted SMILES target includes the final source-to-normalized/unique-marker export checks;
its three starting seeds are distinct from the five-seed local run. Both locks remain exact.
All three actual window launches report Metal/Apple Paravirtual device, Vulkan/llvmpipe
(LLVM20.1.2) CPU and Dx12/Microsoft Basic Render Driver CPU. These are real window/API
smokes, not interactive Windows/Linux chemistry or hardware-performance measurements.
Main advanced to this exact source only after terminal GREEN. No new native20-command
field editor, application SMILES flow, golden approval or complete M0/M1 claim follows.
