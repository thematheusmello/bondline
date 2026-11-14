# Complete SMILES source-file sessions

2026-11-14. Source base df34c66; real codec ca11b2f and IO28c2238 received into the
isolated app checkout before any GREEN compile. Root owns Git. No backend stub, second
registry, dependency/lock/asset/golden change, GUI/input/activation or Actions execution.

## Shared contract

Two closed commands add only path plus optional discard_unsaved=false (Open) or
overwrite=false (Save). The registry has24 names; all22 prior names/parameter assertions
remain. Both commands write session/disk state and are read_only:false. They select the
whole accepted source file, never a record or a drawing conversion.

StoredContent::SmilesFile owns one immutable SmilesFileDocument. No redundant
original/current pair is introduced without a file edit API. Current/saved/captured
plans retain the whole content. Accounting adds the outer snapshot once and delegates
source/index deduplication to the codec, excluding inline storage already counted.
Metadata remains part of equality. There is no mutable graph or first/empty Drawing.

Open uses the common out-of-lock FilePlan and applies only under existing dirty,
revision/generation/cancellation/close guards. Success starts a clean generation and
clears history. Save captures the complete content and existing shared write lease,
uses the real exact-byte publisher, and retains ownership/reservation through Completed
finish/drop. An old-generation publication returns its real historical receipt without
associating or marking a newer Open saved. No separate worker/ACK/close policy is added.

Automatic state has document:null, no SDF collection and compact
smiles_file:{format:"smiles",byte_length,record_count}. It has no recordsVec, notation or
opaque data. Source.modified stays false because no source-file edit command exists.
Drawing, SDF and cross-format Save operations return unsupported_content; no dummy or
first-record conversion grants source authority. The old explicit conversion/export
commands retain their distinct source, format, byte-limit and active-session rules.

## Independent expectations and actual workflows

An original mixed LF/CRLF/final-EOF source contains three meaningful records, ignored
blank/prefix lines and opaque non-UTF-8 data. Literal complete file equality, clean
source/file state, no dummy document and no automatic JSON/Debug disclosure are checked
through Open/Save/reopen. Zero-record files retain every ignored byte. Malformed,
unsupported and non-ASCII later notation must preserve the complete prior drawing or
SMILES session. Dirty refusal, stale prepared Open, committed-close refusal, old-generation
save receipt and the shared save lease through finish are exercised with actual files.
Wrong-kind drawing/capture/SDF/export/other-format Save requests cannot create output.
A private accounting test independently sums inline/source/index storage across aliases,
clones and independent complete owners; different metadata remains different content.

Actual CLI, stdio MCP and authenticated loopback execute Open→Save→reopen under Unicode
paths and misleading suffixes. Every record/opaque byte stays exact, summary/source/file
metadata stays truthful, and notation/data stays absent from responses. The socket also
checks failed later-record retention. These adapters call the same registry and real IO.
Independent raw spans and unchanged per-record RDKit consumer expectations are established
by [codec](verification-smiles-file.md) and [actual native I/O](verification-smiles-file-io.md)
tests, rather than using a molecular supplier's lossy name/line behavior as a raw oracle.

## Actual RED/GREEN

- First command runtime RED37457 exits101 against df34c66: the one initial Open/Save test
  fails with UnknownCommand("document.open_smiles"), before Root command implementation.
  Log target/smiles-files-app-red.log in the app checkout.
- Broader Root94529 runtime RED against the untouched public df34c66 source executes all
  seven public tests:0passed/7failed. Five UnknownCommand failures and two missing-name
  schema/wrong-kind assertions establish the absent commands. The isolated draft already
  existed then; this is a baseline reproduction, not another pre-implementation claim.
  Temporary exact test copy was removed only after hash verification; its source is
  retained in the app checkout. Root target/smiles-file-app-all-red.{log,json} records it.
- Draft formatting preceded the actual IO backend compile. Focused37014 exits0:
  seven public command tests pass. No implementation/expectation correction was needed
  after that first actual backend compile. target/smiles-file-app-focused.log.
- Full affected-package6062 exits0:242passed/0failed/0ignored across32 result lines,
  including11 new command/accounting/CLI/stdio/socket tests. Log
  target/smiles-file-app-adapters.log,29,874B/SHA256
  1cb024a3c2d628c37903e404c6b382f57ddc6b768775885247557d94b00f82b3.
- All-target warnings-denied clippy65571 for commands/mcp/cli and whole-workspace fmt
  exit0. Logs target/smiles-file-app-clippy.log and -fmt.log. All own jobs are terminal.

Commands used the own absolute app target, incremental0, dev/test debug0, Rust1.94.1
and the existing pinned interpreter through BONDLINE_PYTHON. Source-only peer review
found no confirmed defect; Root report target/ui-smiles-command-draft-review.md is6738B/
SHA256 e858a5a412b0beb1f7657dce7f6dff024623df736efe9607f636b9ebb34fc173.
Review is distinct from runtime evidence. The retained-byte test is an added regression,
not a separately observed test-first failure. No old source authority, parser guard,
consumer, writer/layout or editing/history expectation was weakened.

## UI and limits

[UI evidence](verification-smiles-file-ui.md) separately records the unchanged five-file
draft's98-test GREEN, real background file worker, chooser/retry/known-Save/clean-close
drain and synthetic egui privacy/accessibility/gesture tests. Its initial RED was a
missing-backend compilation failure; no UI assertion ran then. Actual native execution
is unverified while the user is using the computer. Windows/Linux and hosted execution
are unverified; Actions stays disabled. No chemical preview/editing, record selection,
metadata editor, broader notation/chemistry or completed M0/M1 claim follows.
