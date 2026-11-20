# Roadmap

The complete target is an offline desktop and WebAssembly chemical editor with familiar
documented workflows. The initial reference baseline is Professional 26.1; older public
guides are versioned evidence, not proof of complete modern parity.

The persistent goal includes **all M0–M5 open points** and every remaining in-scope row
in the parity checklist. It is complete only after actual install/create/open/edit/
analyse/cleanup/history/save/recovery/reopen/publish/clipboard journeys have independent
evidence on the supported desktop platforms and browser. A scaffold, command count,
grey tool, passing component or partial milestone cannot complete it. Continue from
STATUS rather than restarting or repeating historical next-task paragraphs. Actions
stays OFF during development.

The [CDX caption increment](docs/verification-cdx-caption-preview.md) opens one
source-qualified caption and preserves original files through display and Save.
Headless CLI, MCP, loopback, CPU/Metal and native macOS Open/view/Save/reopen
checks passed. Additional typography, editing and other platforms remain required.

[Ordered CDX captions](docs/verification-cdx-caption-order.md) add multiple
explicitly positioned captions on one page. Independent overlaps establish
paint order and macOS Save/reopen preserves the complete source. Tied depths
and unproved default ordering remain open.

[Explicit CDX style runs](docs/verification-cdx-caption-runs.md) extend that
preview to multiple resolved styles within a caption. Verified cases retain
their colours, sizes and run boundaries through exact native Save/reopen.
[Existing caption editing](docs/verification-cdx-caption-edits.md) updates
those runs through the command adapters and repaints the app. The
[Text tool](docs/verification-native-caption-editing.md) can edit one visible
existing run in a qualified CDX or CDXML caption. [Caption creation](docs/verification-native-caption-creation.md)
adds single-line text to blank paper with stable IDs, Undo/Redo and editable native
CDXML Save/reopen. The same workflow now appends to qualified caption-only CDX
pages and reopens with stable live IDs. An empty single Poster page with a qualifying
font and white paper preserves its physical size and accepts its first caption.
Qualified mixed CDX pages now display structures and frontmost captions together
and support editing existing caption runs. The Text tool respects paint order
when a caption overlaps a label. Blank paper on a qualified mixed or chemical-only
page accepts a new caption, with byte-exact Undo and editable Save/reopen.
[Chemical drawing caption creation](docs/verification-cdx-chemical-caption-creation.md).
Changing an existing supported atom label on the mixed drawing now preserves the
new annotation and prior captions. [Mixed label editing](docs/verification-cdx-mixed-label-edits.md).

Qualified binary skeletal carbon chains open on the declared paper and preserve
their original CDX through Save/reopen. Marquee now moves selected skeletal atoms
with Undo, Redo and source-preserving Save. Drawings with hidden carbons and
explicitly positioned heteroatom labels now support viewing and exact unchanged
Save/reopen. Moving or editing those labels and additional drawing styles
remain M1/M2 work. [Printed labels on skeletal paper](docs/verification-cdx-mixed-skeleton.md).
[Movement verification](docs/verification-cdx-skeletal-movement.md). [Skeletal drawing acceptance](docs/verification-cdx-skeletal.md).
General chemical edits on mixed pages, interleaved depths, missing font resources, multiline text,
inherited styles, ambiguous starts and general shaping remain open.
[CDX creation](docs/verification-cdx-caption-creation.md), [empty Poster verification](docs/verification-cdx-empty-caption.md), [mixed pages](docs/verification-cdx-mixed-caption.md), [mixed edits](docs/verification-cdx-mixed-caption-edits.md).

The [CDX Node label increment](docs/verification-cdx-node-labels.md)
preserves explicit child Text beside literal atom fields. Independent
examples and native macOS Save and reopen verify exact source retention.
Next establish sourced placement and label agreement before allowing
faithful preview and chemical editing for qualified CDX.

The [CDX label preview increment](docs/verification-cdx-atom-label-preview.md)
adds isolated and connected labelled structures whose source determines
placement, paper and plain bond styles. Mac viewing and unchanged Save
are verified. Atomic typed binary source replacement and an independently
tested [existing-label edit workflow](docs/verification-cdx-atom-label-edits.md)
now preserve Element, H, styled Text and box across change, history and
Save/reopen. The [implicit Element increment](docs/verification-cdx-implicit-element.md)
can keep carbon omitted or insert a changed element. CDX creation and
editing of other objects remain required.

The [CDXString increment](docs/verification-cdx-string.md) preserves literal style runs
and text bytes, with borrowed property boundaries and independent original fixtures.
The caption workflow uses this reader for its explicit validated run.

The M1 [CDX resource increment](docs/verification-cdx-resources.md) reads document
font and colour tables on demand. The independent fixtures and sanitizer cover empty,
malformed and maximum-sized tables, literal unsigned fields and exact source retention.
The full local gate passed. The raw string reader builds on this source coverage.

The current M1 [plain CDX bond receiving](docs/verification-cdx-plain-bonds.md) adds
read-only Single/Double/Triple and explicit directed position, preserving raw automatic/
manual provenance and every source byte. Independent16-source/149-refusal checks, actual
adapters/App/native Unicode files, fullIO541/0 and bounded ASan pass. No display/chemical-
authoring/loss authority, command or grey tool is added. Next source-lock checked standalone
CDX font/colour resources before interpreting styled Text or granting faithful preview.
Broader CDX/interchange and every remaining M0–M5 workflow stay required; STATUS records
the latest aggregate and exact continuation rather than these historical next-task notes.

The preceding M1 [literal CDX receiving](docs/verification-cdx-literal-atom-metadata.md)
adds read-only isotope/charge/radical/explicit-H projection with exact source retention.
Frozen independent facts, actual adapters/native Mac Unicode files and bounded ASan pass.
The final aggregate fails at the unchanged MCP listener-release case; all unexecuted
stages pass separately. No display/chemical-authoring/loss authority or tool is added.
Next source-lock plain CDX Single/Double/Triple order and explicit bond display/position
metadata, then freeze independent bitmask/direction/absence/owner vectors before code.
CDX fonts/Text/styles, faithful display/editing and every remaining M0–M5 workflow stay open.

The preceding M1 [scalar-default metadata receiving](docs/verification-label-default-metadata.md)
qualifies explicit-label files carrying checked unused LabelFont/Size/Face on supported
owners. Existing label/Selection/history/Unicode Save/reopen works through immutable
source spans, with bounded native Mac, exact CPU/Metal and independent source/font/
adapter/sanitizer evidence. Missing actual run fields remain refused: public omission
examples do not resolve general Label*/Caption*/stylesheet inheritance. Registry49 and
catalogue16/124 stay exact. Broader effective typography/rich labels remain required;
continue the independent CDX literal-atom metadata workstream rather than guessing
unproved inheritance. Full M0–M5 and all workflows below remain active/incomplete.

The preceding M1 [printed-label creation receiving](docs/verification-printed-label-creation.md)
adds explicit-style hidden-carbon child Text creation, retained flags, stable IDs and a
separate source capability that survives partial/all-labelled Save/reopen. Native Mac
creation/continued editing/history/movement/Unicode files and exact CPU/Metal captures
pass bounded independent checks. Old profiles/full/loss authority stay literal; new49
preserves all old48 metadata and catalogue16/124. The user now requires pushing each
atomic commit; Actions stays OFF. Effective foreign typography/default ownership, broader
resources/placement, CDX/general interchange and every remaining M0–M5 workflow stay open.

The preceding M1 [mixed skeletal-label receiving](docs/verification-mixed-skeletal-labels.md)
adds a separate foreign-source capability for explicit printed labels plus explicitly
hidden, known-H carbon. Current no-glyph proofs prevent borrowed fonts after point/neighbor
changes. Existing label/selection/history/Save workflows and bounded native Mac exact
Unicode files pass. Fresh local2091/0/1+vendor3/0 and10000-run ASan pass; prior listener
failures remain unexplained, not fixed. Old Boxed/Connected/full-authority boundaries,
registry48/catalogue16/124 and Actions OFF remain. Full foreign defaults/styles, hidden-
node label creation and all remaining M0–M5 workflows below are required.

The preceding M1 [dependent-label receiving](docs/verification-dependent-label-translation.md)
extends existing selection/translation to qualified Boxed and Connected sources. Current
Node points and raw label boxes move together, preserving exact source/history/Save owners
and refusing invalid whole-page candidates. Independent source/adapters/App and bounded Mac
drag/nudge/Unicode Save/reopen checks pass. The native all-atom CPU/Metal capture is exact
after an independently checked nine-pixel coverage correction. The unchanged MCP listener
aggregate failure remains open; other local components are verified separately.
Full M0–M5, human golden approval and other-platform acceptance remain open; Actions is OFF.

The preceding M1 [connected-label receiving](docs/verification-connected-labels.md) adds
explicit foreign bonded diagrams through a separate source capability, actual-outline
margin shortening and the existing synchronized label command/native form. Old Boxed
remains zero-bond, global full staysfalse and all48 metadata/catalogue16/124 remain exact.
Independent source/render/history/adapters and bounded Mac Unicode Save/reopen pass.
Aggregate MCP listener-release failure remains open; other components and sanitizer pass.
Continue literal dependent Node+box movement with independently declared source/geometry/
history proofs, broader foreign styles/fonts/text and the full remaining milestones.
Human golden approval and exact reference/platform behavior remain unverified.

The preceding M1 [box-positioned label receiving](docs/verification-boxed-atom-labels.md)
delivers a complete bounded imported-file workflow: authored display, synchronized
Element/H/body/bounds edits, history, Unicode Save/reopen and a reused native atom form.
Foreign sources require all atoms explicitly labelled and no bonds; unknown font/ink
defaults cannot borrow native conventions. Independent font/XML/image/adapter checks,
CPU/Metal equality and bounded actual macOS
acceptance pass. Registry48 preserves all old47 metadata; catalogue16/124 stays.
The latest aggregate MCP listener failure remains recorded, not repaired. A human-approved
label golden, broader placement/styles/defaults/dependent edits and general Text-tool
editing remain open. First source-lock the next connected foreign label/style workflow, including explicit
bond ink/gap/margin and effective-font ownership. Admit it only after independent source/
render/history/native proof; other writers keep explicit refusals. Continue full M1, then all remaining M2–M5 work below.

The following paragraphs record historical checkpoints. Their next-task/count statements
retain that scope; current executable continuation is in STATUS.

The earlier M1 [owned child-Text boundary](docs/verification-owned-atom-label-data.md)
preserves explicit Node label data in the model/native codec, accounts for shared history
and refuses unsupported edits, rendering and lossy conversion. Independent actual
CLI/stdio/socket unchanged Unicode Save/reopen passes. This does not finish the chemical
Text workflow: supported layout/semantic agreement, effective font/face/default data,
synchronized source-preserving edits and a native editor remain the next work. The full
M0–M5 target below and registry47/catalogue16/124 are unchanged.

The M1 [label-default lookup prerequisite](docs/verification-label-default-lookup.md)
now has owned per-property provenance, independent Expat/Decimal/source tests and bounded
ASan. Fresh local1919/0/1 passes; the earlier MCP failure remains unexplained. This adds
no drawing/foreign-authoring authority. Node child Text precedence, unresolved Style-run
fallback and conflicting public placement evidence must guide the next complete
model/source/render/edit/Save workflow; do not invent a universal baseline or style sheet.

Current M1 [plain-caption receiving](docs/verification-styled-captions.md) integrates
explicit styled runs, a separate source capability, actual CPU/Metal display and one
shared run-body edit/history/Save command. Bounded Mac Open, remote-edit repaint, keyboard
history and Unicode Save As/reopen pass. Registry47 preserves the old46 byte prefix;
catalogue16/124 stays. This is partial Text support: the native Text tool/editor, chemical
formatting, dependent labels, inherited fonts/styles, mixed foreign diagrams and CDX text
remain missing. The aggregate's old MCP bind failure remains unexplained; final receiving
records the separate local components. Full M0–M5 remains active/incomplete.

Earlier paragraphs below are historical checkpoints with their recorded evidence/scope.

Explicit native root relative spacing now follows the
[preferred-length contract](docs/research/document-bond-spacing.md) and
[receiving](docs/verification-document-bond-spacing.md): independent source deltas,
shared ink/hit/render certificates, one final settings transaction and a four-field form.
Missing spacing retains legacy geometry; Abs/local overrides/default inheritance remain
guarded. The full M0–M5 goal remains active. Native width/spacing interaction is unverified;
no planned tool is enabled. Continue broader M1 effective fonts/colours/Text and foreign
style ownership, then CDX semantic mapping and the remaining milestone workflows.

M1 now has [source-preserving text-only syntax edits](docs/verification-source-preserving-xml-text.md),
with independent bytes/Expat, old parser regressions and bounded ASan. All local components
pass; an initial aggregate MCP bind failure remains recorded/unexplained. Effective styles,
fonts and synchronized dependent labels still need real model/render/history/Save/native
integration before general foreign authoring. The full target below is unchanged.

M1 [logical font/colour resource decoding](docs/verification-cdxml-resources.md) now passes
independent format/Expat expectations, full local1811/0/1 and qualified bounded ASan.
It retains raw source and explicit missing references without granting foreign authoring.
Continue with checked source-preserving Text edits and complete effective-style/label
resolution before activating a broader imported-file profile. The later spacing increment
above supplies bounded explicit-root evidence, without inventing foreign defaults. All milestones and
the full end-to-end goal remain open.

The user explicitly recreated/resumed the full end-to-end M0–M5 goal. Explicit document
LineWidth now has [independent receiving](docs/verification-document-line-width.md):
typed/source-preserving data, actual width separate from em/spacing, complete-owner
display/previews/capture, atomic three-field settings and preserved pair behavior. Full
local1800/0/1 and bounded10000-run ASan pass; registry46 retains all45 prior schemas/order,
catalogue16/124 stays. Native access fails, so new native width interaction is unverified.
Next broaden M1 styles/font/text/default resolution with independent foreign fixtures,
then CDX semantic mapping/writing and the remaining file workflows. All M0–M5 requirements
below remain; no milestone or full source compatibility is complete. Actions stays OFF.

The [source-aware closed-v0 profile](docs/verification-source-preserving-cdxml-profile.md)
now preserves lexical/order/scoped-ID variants through all delivered typed edits/appends,
with complete syntax/semantic proof and current-prefix guards. Actual CLI/stdio/socket/
Unicode history/files, independent Expat/Decimal/RDKit, paired authoring/CPU captures,
two bounded10000-run ASan checks and full local1766/0/1 pass. Unknown semantics remain
guarded and whole-MOL stays canonical-only. Native access is unavailable, so the new
desktop path is wired/unverified; foreign styles/defaults/labels/objects and full M1
remain open. Next source-lock broader published style/default data and independent
foreign-source retention/render vectors; revisit native acceptance when access returns.
Full M0–M5 stays active/incomplete, Actions OFF. Earlier paragraphs are checkpoints.

The next M1 source-preserving workflow now has its [batch syntax prerequisite](docs/verification-source-preserving-xml-edits.md):
ordinary attribute changes/removals and self-contained child appends preserve unrelated
bytes and namespace meaning. Eighteen independent cases, actual nesting/namespace REDs,
bounded10000-run ASan and full local1735/0/1 pass. No new command/tool/source authority is
enabled. The original noncanonical-file edit still refuses; next complete source-aware
model mapping/writing and actual adapter/history/native-file acceptance without relaxing
unknown-data, preview or whole-MOL loss guards. This does not complete M0, M1 or M2–M5.
The next [source-aware IO contract](docs/research/source-preserving-cdxml-profile.md) is
locked with exact proof, typed drift/write and format-loss requirements; its headless
implementation/receiving is now above, while native interaction remains unverified.

The [native selection acceptance/fix](docs/verification-native-selection.md) continues
base0c9e6af with actual Mac click/Shift/rectangle/component, drag/nudge/history/modal and
Unicode Save/reopen evidence. Eleven actual native/fixture files pass independent XML/
Decimal/RDKit checks. Four new preimplementation regressions expose/fix hidden Atom/
settings modal authority and equal surface-restoration revival; ordinary no-loss controls
and all old cases stay. Full local1717/0/1 passes; registry45/control/catalogue outputs
stay unchanged. Native races/additive marquee/measured DPI/other platforms, Lasso and
broader transforms remain open. Next return to the explicit M1 priority: source-lock
safe noncanonical elemental CDXML mapping/writing, starting with equivalent whitespace/
ID/order variants and unknown-source retention. A real native noncanonical fixture still
refuses faithfully; do not relax the canonical byte-profile gate without complete
mapping, independent source/format/history tests and native evidence. Full M0–M5 remains
active and incomplete; Actions stays OFF. Earlier paragraphs retain checkpoint scope.

The [native sprout/preferences increment](docs/verification-native-ring-sprout.md)
continues the [atomic headless checkpoint](docs/verification-ring-sprout.md). Registry45
retains all44 earlier complete CLI records/schema/order; controls39/22/16/12 add only
File Preferences, while catalogue16/124 stays. All ten ring tools use the shared candidate
for atom-targeted preference OR Alt sprout. One session checkbox has ordered Defaults/OK/
Cancel and source/modal/retirement/ABA guards. Fresh full local1713/0/1 and bounded ASan
pass. An isolated actual Mac app checks Alt-sprout, preference-driven Chair2, history,
64pt setting, Unicode Save/reopen and Defaults-Cancel; independent actual-file readers
confirm geometry/H/C12H16/C8H16. Native all-tool/modifier/race/DPI/other-platform coverage,
restart/full preferences/styles/stereo and exact26.1 policies remain open. Next source-
lock and finish an unblocked labels/styles/selection acceptance slice, with independent
RED expectations and full registry/native/file evidence before enabling a grey tool.
Actions stays OFF; full M0–M5 stays active and incomplete. The paragraphs below retain
earlier checkpoints and their original scope.

The user-directed identity and planned-tool presentation exposes all124 documented
TD/TQ/TB roles as140 entries, with sixteen real bindings and124 disabled planned entries.
Original logo assets, native review and independent input/foreground/font regressions are
recorded in [receiving](docs/verification-branding-and-tools.md). A visible grey icon does
not implement its tool; the full M0–M5 scope and missing-feature states remain intact.
The current [chair increment](docs/verification-native-chairs.md) adds both original
constitutional depictions through the two existing ring commands, with full same-builder
previews and source/H/ID/history/physical guards. Independent core/modal/App/actual-wire/
Unicode-file checks and bounded native Mac horizontal/vertical, terminal,9/0, refusal,
history,64pt preference and persistence workflows pass. Full local1681/0/1 and bounded
ASan pass; four earlier inventory/lint gate stops remain explicit. Registry43 stays;
controls38/21/15/11, catalogue16/124. Native direct fusion/spiro, hardware/DPI, other
platforms, stereo/ring flips, Alt-sprout and wider labels/styles remain required.
This does not complete M0, M1 or M2–M5.

After that visual increment, the explicit headless chain candidate follows the
[locked contract](docs/research/chain-authoring-plan.md) and
[receiving evidence](docs/verification-chain-authoring.md). Continue its complete native
workflow is now integrated below; continue the remaining drawing, chemistry, publishing
and distribution scope.
The [native dependency contract](docs/research/native-chain-plan.md) now has a literal
turning-path/preview prerequisite with [independent evidence](docs/verification-chain-paths.md).
The [drawing-settings data prerequisite](docs/research/drawing-settings-data.md) now has
[independent persistence/loss evidence](docs/verification-drawing-settings-data.md).
Root BondLength/ChainAngle preserve explicit supported data and opaque source values;
new-chain application is now checked, while full style inheritance remains missing. File > Document
Settings… now has a [bounded two-preference form](docs/verification-native-drawing-settings.md)
with actual Mac set/clear/history/Save/reopen evidence. It does not complete the dialog;
new-chain application belongs to the separate workflow below.
The [atomic settings command/history/adapters](docs/verification-drawing-settings-command.md)
now pass independent required/null/source/lease/history and actual file/wire checks.
Ordered native modifier transport now has [SDK and actual Mac OS evidence](docs/verification-ordered-native-modifiers.md).
The [complete native chain interaction lock](docs/research/native-chain-interactions.md)
now has [independent App/adapter/file and bounded actual Mac evidence](docs/verification-native-chains.md).
Acyclic/Snaking alone become available: drag/count, Add Chain, repeat, supported-carbon
attachment, stored preferences, fixed switches, full previews and one-edit history.
Registry43 preserves all old42 actual records/schema/order; controls36/21/15/11,
catalogue14/126. First full local10212 passes1657/0/1; actual preimplementation and later
counter/repeat/refusal RED→GREENs remain separate from compile/lint/storage qualifications.
Native supported Control-drag/reversal/backtracking and measured2×/DPI remain unverified;
other platforms and exact26.1 defaults/hotkeys stay open. Frame-global Control is never
substituted for observed ordered events. Next continue missing drawing tools, including
sprout and broader labels/styles, with source locks and independent workflows before
binding grey entries. The earlier native-settings gate passes1629/0/1 after correcting
one newly added headless-control expectation; that first gate remains recorded as failed.
Earlier settings-command
MCP replacement-bind and UI readiness failures remain recorded; the latter has a
[test-only qualification](docs/verification-smiles-order-readiness.md). This new pass
does not establish a causal socket repair. Preserve all diagnostic boundaries.

The [carbon-label data prerequisite](docs/verification-carbon-label-data.md) now has a
[shared skeletal display/explicit-label follow-up](docs/verification-skeletal-carbon.md).
Known-H/C4 suppression preserves mandatory labels, hits and real-ink guards; existing
atom.set_label restores visibility through the same registry/form without changing old
None/true no-ops. Native OS acceptance/default styles/preferences remain outstanding.
The separate terminal-label prerequisite now has
[independent data/render/editor/file evidence](docs/verification-terminal-carbon-labels.md).
Actual degree selects the appropriate flag and isolated C/C2 labels remain mandatory.
The chain engine, native dependency and full interaction contracts are locked. Add Chain,
repeat, Snaking and preference application now have scoped evidence above; remaining
settings/styles and native modifier/DPI/platform acceptance remain required.
Named benzene/cyclopentadiene candidates now share ring.create with the unchanged sized
requests. Independent phase/H/source/history and actual adapter/file checks pass; see
[receiving](docs/verification-unsaturated-rings.md). Their two native controls now share
the same-builder preview and ordered Shift flow with independent actual-App/adapters/
file checks; [native receiving](docs/verification-native-unsaturated-rings.md) separates
this from pending OS/monitor/platform acceptance. Continue chair/sprout/chains/styles
and broader chemistry; full scope remains intact.

The aromatic-tool prerequisite now persists directed DoublePosition through the
model and native CDXML, with independent property/file/history tests and explicit loss/
render refusals. [Receiving evidence](docs/verification-double-position.md) separates
this from later depiction. Shortened inset strokes with actual bounds/gaps/hits now
have independent geometry/pixel/density/CPU-GPU and actual-file/history evidence in
[renderer receiving](docs/verification-inset-doubles.md). The explicit nonterminal
skeletal policy now retains literal H; weighted benzene/cyclopentadiene candidates now
preserve old shared edges and rank bounded phases against proven H budgets. The ranking
is original Bondline policy, not established reference precedence. Named native tools
are wired with actual-App evidence; real native acceptance remains. The full parity scope is intact.
The [inset rendering contract](docs/research/inset-double-rendering.md) retains the
preimplementation geometry, component-role joins and independent guard requirements.

The shared `ring.create` command now implements explicit3–8 carbon Single polygons,
including shared-atom and shared-Single-edge attachments. Independent geometry,
cyclohexane/decalin/spiro/terminal chemistry, source/history/ID guards, Unicode files,
CPU/GPU and actual CLI/MCP/socket checks pass. Its rounded-precision counterexamples
are fixed without a blanket minimum length. Registry38 now adds ui.ring_tool, one
effective tool mode, six original procedural controls and same-candidate preview plus
free/atom/bond gestures. Native Drawing controls29; retained/SDF/raw stay20/14/10.
Independent App/adapter/file checks are in [native receiving](docs/verification-native-rings.md);
actual native OS interaction remains unverified while the desktop is in use. Follow
with native selection/ring acceptance and aromatic/chair/sprout/styles and broader
chemistry. This is bounded integration, not complete native ring parity or M2.

Bounded native Mac atom labels now support fresh Alt-click target/fields/Apply/Cancel,
OH/NH2 editing, history, draft-close protection and Unicode editable CDXML Save/reopen.
Independent consumers verify the actual files; see
[atom-editor evidence](docs/verification-native-atom-labels.md). This original form is not
full inline Text/alias/charge/isotope/radical parity. The selection checkpoint's registry36 adds
persistent selection and literal translation; native Drawing controls23, retained-SMILES
Drawing20, SDF14 and raw SMILES10. Independent command/context/wire and actual-App input
checks now exercise selection/movement/history/Unicode persistence. Native selection
execution remains unverified; see [receiving](docs/verification-selection-authoring.md).
The atom checkpoint's full gate fails848/1 at the old MCP replacement bind; remaining480/0/1 and
xtask stages pass separately. The current selection gate passes1361/0/1 with no socket
policy change; that pass does not explain the historical failure. The [controlled inheritance experiment](docs/research/macos-listener-lifetime.md)
does not attribute that natural failure. Full M0–M5 remains incomplete. Continue source-
preserving M2 authoring and independent native checks; keep broader format/chemistry,
publishing and M5 workflows visible rather than treating this slice as completion.
The [locked selection/translation contract](docs/research/selection-translation-plan.md)
now has a source-qualified implementation, reviewed local gates and an inspected
offscreen App frame. Finish actual native acceptance, then continue ring/chain/style
and broader label workflows. Full
selection/transform/26.1 parity remains required beyond this bounded atom-set slice.
The [ring-authoring contract](docs/research/ring-authoring-plan.md) locks the delivered
command and independent cyclohexane/decalin/spiro persistence. Lock its remaining native
tool/gesture/default policies before App REDs; it does not resolve aromatic/chair/sprout
or style workflows.

Native source-backed SMILES records now have bounded Mac selected/all import, refusal/
retry, registered click/key order editing, history and Unicode whole-file Save/reopen/
reimport, with independent actual-file consumers. See
[native receiving](docs/verification-native-smiles-records.md) and the earlier
headless1270/0/1 record checkpoint. The later atom-label command gate passes1306/0/1;
the earlier intermittent MCP listener-release AddrInUse failure remains unexplained.
Fit78.79% still refuses the edited multiple-bond scene, Actual100% recovers, pan
attempts did not change the view, and native races/DPI/other platforms remain unverified.
Full M0–M5 remains open. Continue M2 atom/selection/ring/style authoring after preserving
these complete-source/worker boundaries; broader record and chemistry parity remains.

The earlier native editable SMILES checkpoint passes the full local gate1245/0/1, with bounded
Mac OS clipboard/refusal/retry/history/edit/Unicode Save/reopen and independent file checks.
The earlier density pipeline78964's MCP `AddrInUse` failure remains unexplained; this pass
is not a causal fix. Automatic clipboard capture, exact reference-dialog behavior and
native2×/DPI remain unverified. See
[paste evidence](docs/verification-native-smiles-paste.md) and
[density evidence](docs/verification-native-drawing-density.md).

## M0: foundation — in progress

- [x] Resolve declared runtime/asset license closure for native egui/wgpu on all three platforms.
      Actual Mac Metal drawing/readback verified; hosted Mac/Windows window smoke passes.
      Linux native launch passes after the runtime-package correction below.
- [x] Cargo workspace, pinned toolchain, lints, build-time layer and license gates.
- [ ] Pure document model, named typed JSON commands, transactions and undo/redo tests.
      Page controls/history are delivered; headless elemental graphs now have structural/
      identity/budget tests. Independent C-C creation and transactional drag/history now work;
      connected editing, broader models and a general transaction/job engine remain.
      Conditional close intent now joins the shared registry, with epoch-bound Save and
      native worker/reply shutdown barriers. Shared delivered File/Edit/Page bindings and guarded
      file-presentation requests now exist; broader dialog discovery remains incomplete;
      see docs/research/m0-command-ui-audit.md for the remaining distinction.
- [x] CLI, stdio MCP and opt-in loopback control using that registry.
- [ ] Native UI and CPU/GPU reference canvas with actual launch/render evidence.
      Native Mac drawing/history/recovery and one human-approved golden now pass.
      All three hosted window smokes pass; interactive Windows/Linux workflows and broader rendering
      remain unverified.
- [x] macOS/Windows/Linux CI, licensed corpus tooling, parser fuzz infrastructure.
      Corrected hosted run 43451706668 passes aggregate checks and actual window launches
      on all three platforms plus Linux ASan. Corpus now checks eight pinned files from
      five bundles: CDXML, MOLV2000/V3000, ordered SDF, one original CDX scene and three
      original SMILES notations. Foreign/general CDX, broader SMILES/InChI and chemistry
      corpus coverage remain missing.
- [ ] Complete public-document menu/tool/shortcut inventory; continuation instructions.
      582 stable-ID source-cited control rows now exist, including three provisional
      unversioned Help roles; modern menus, unnamed glyphs and remaining
      context menus still need evidence. Six versioned screen/flow groups and pinned public
      research now guide implementation; the download center requires login and observed
      Signals demo states are not desktop evidence. Do not infer completeness from row counts.

## M1: files — in progress

Fresh document sessions now have `document.new`, File → New Document and primaryN,
guarded dirty/draft decisions and Save-before-New through actual worker retirement.
Independent tests reproduce and close both old-surface pointer races. The local aggregate
passes1090/0/1; actual Mac SDF→New, Cancel/Discard/Save, Unicode output, editable reopening
and Undo pass. See docs/verification-new-document.md. Styles/templates and multiple
windows remain missing. This closes the source-file→fresh-drawing dead end, not M0/M1.
The later user-authorized live26.1.0.6801 pass actually observes New creating a second
styled window while retaining the first; see docs/research/live-ui-review-2026-11-15.md.
That independent-window/style workflow remains required for parity.

The original native restricted SMILES→CDXML workflow is now delivered through the same
registry and registered-worker lifecycle. Input/output browsing, explicit geometry,
existing-destination refusal, successful Unicode retry without replacing unsaved work,
guarded explicit Open, Double edit/Undo and separate Save As/reopen pass on macOS.
Independent consumers accept both actual433-byte outputs. The final local aggregate
passes1116/0/1; native race/close/cancellation and other platforms remain unverified.
See docs/verification-native-smiles-conversion.md. This is a1–12 neutral-carbon Single
path utility, not general SMILES import or Copy As/Paste Special parity.

The additive neutral-acyclic conversion profile now delivers connected trees up to64
H/C/N/O/F/Cl/Br/I atoms, branches and Single/Double/Triple edges under the existing fixed-H
policy and bounded geometry. Original default/profile/receipt contracts remain exact.
Eight IO, two renderer, five command, two actual-wire and three App checks add20 passes;
the full local gate passes1136/0/1. Actual Mac profile/refusal/Unicode retry, retained unsaved
work, guarded Open, branch Double edit/Undo/Redo and Unicode Save As/reopen pass. Both actual
766-byte files agree with independent raw/RDKit expectations. Source/H, stereo, geometry
and platform limits are in docs/verification-smiles-acyclic-integration.md.

The additive `neutral_simple_ring_v1` profile now implements one actual connected3–8
Single cycle plus supported attached trees, at most64 neutral H/C/N/O/F/Cl/Br/I atoms.
It retains fixed H/source/IDs, exact supplied length, minimum atom-center origin and full
geometry/ink guards without scaling. The ordered third native choice uses the same
create-only command/worker/lease and grants no active-session Save or source-file editing
authority. Focused9 IO/4 command/2 wire/3 App/4 renderer checks pass, as do the producer's
308 default IO tests. Bounded native Mac profile/refusal/Unicode retry, retained unsaved
work, explicit Open Cancel/Discard, external methyl-bond Double edit/Undo/Redo and separate
Unicode Save As/reopen pass. Independent readers accept both actual1254-byte outputs,
C7H14→C7H12; source and existing-output sentinel remain exact. Geometry entry/browsing,
native races, large/bulky cases and other platforms remain unverified. The final local aggregate passes1158/0/1, including unchanged legacy receipt checks
after the shared dialog sizing fix. See [the integration record](docs/verification-smiles-ring-integration.md),
[independent layout checks](docs/verification-smiles-ring-layout.md), and the separate
[ring attachment](docs/verification-literal-ring-attachment.md) and
[compact-H extension](docs/verification-literal-compact-star-attachment.md) evidence.

The additive `neutral_components_v1` / **Neutral components** fourth choice now derives
all supported actual components into one complete Fragment, up to 64 total atoms. It
normalizes once, retains global arrays/IDs/directed edges/fixed H and changes positions
only. Existing tree or 3–8 Single-cycle poses use source-order center-box shelves with
gap L, no rotation/rescaling, divided search quotas and one whole geometry/ink certificate.
One-component output remains byte-exact with the corresponding earlier profile.

Root90518 passes eight independent IO cases. Root5986 passes three command, one schema,
two actual-wire and three App cases, including the complete 9-atom/7-bond/3-component
C8H20O drawing, identical CPU/GPU capture, explicit Open, first-edge Double edit to C8H18O,
Undo/Redo and Unicode Save/reopen through independent consumers. The fourth profile's
component checkpoint's full local gate passed1188/0/1, including fmt, workspace/vendor warnings-denied clippy,
tests, layers, licenses and corpus. Bounded Mac legacy/existing-output refusals, Unicode
retry with dirty work retained, explicit Open Cancel/Discard, Double edit/Undo/Redo and
Unicode Save As/reopen now pass. Independent consumers verify both actual 1407-byte
9/7/3 outputs as C8H20O/C8H18O, with only the intended first-edge order/H change; input,
sentinel and prior ring remain exact. Authoring worked at100%; edited Fit78.79% refuses
the distinct-pixel bond guard and an initial top label was clipped. Full native overview/
pan and other platforms remain unverified. Earlier counts stay historical checkpoints;
no new parser-fuzz or dependency change.
See [component integration](docs/verification-smiles-components-integration.md) and
[independent layout evidence](docs/verification-smiles-components-layout.md).

Drawing's additive physical-display path now shares the existing pixel transform with
exact effective DPI/fractional device origin, fresh glyph generation and inverse paint/hit
mapping. Logical zoom, source authority and the old1×/capture routes remain unchanged.
Five renderer and five actual-App cases pass, including honest1× refusal,2× Metal strip/
gap checks, fractional phase and logical-readability limits. This is headless verification,
not a native2× measurement or a Fit fix. The latest Mac build still refuses the edited
component scene at Fit78.79%; hotspot2/Undo/Redo/Actual100% and exact Unicode Save/reopen
pass, while two clicks with Double selected produced no mutation of unknown cause. Full overview/
pan remains unverified. [Receiving evidence](docs/verification-native-drawing-density.md)
records the failed aggregate and separately passing remaining checks.

The external CDX framing counterexample now has an additive canonical 28-byte implicit/
explicit Document + FileEnd path: eight syntax and six IO/engine/wire cases pass, retaining
all legacy successes, source bytes and limited application authority. The new oracle's
18 controls and a 10,000-run ASan mutation smoke pass. Bounded Mac Open, Unicode Save As
and explicit reopen preserve all 258 original bytes, with faithful preview still refused.
The final local receiving gate81324 passes1172/0/1, including the original MCP test.
Earlier97729's754/1 replacement-bind AddrInUse remains unexplained; twenty passing
diagnostic contexts and a contradicted outbound-owner hypothesis establish no causal fix.
See [integration](docs/verification-cdx-file-end-integration.md) and
[independent declarations](docs/verification-cdx-file-end-tests.md). No corpus admission
or broad CDX compatibility follows. That checkpoint proposed source-qualified SMILES drawing
import: selected/all-record behavior, complete opaque source/baseline and ID mapping,
stale-job/history/unsaved decisions and explicit Save/loss authority. Broader packing,
larger/polycyclic/multiple-bond rings and chemistry remain; retain unsupported source
rather than silently projecting it. The new utility still does not grant an opened SMILES source
session molecular editing authority or complete clipboard parity.
SDF creation/reordering and wider format editing stay in the backlog. The earlier proposed
SDF reordering priority was deferred after the sourced File New audit, not implemented.
The [SMILES authoring/import study](docs/research/smiles-source-import-plan.md) recommends
editable Paste Special with real benzene support, then source-preserving record editing
and whole-file Save. Neutral fixed-H/Kekulé chemistry, bounded layout and the one-step
Drawing append now pass independent engine/CLI/stdio/socket and actual saved-file checks;
the explicit native Paste Special form now adds actual OS clipboard/refusal/retry,
one Undo/Redo, native bond edit and Unicode Save/reopen. The full gate passes1245/0/1;
see docs/verification-native-smiles-paste.md. The later bounded record engine now adds
selected/all retained Drawing import, synchronized order edits and whole-file Save through
all delivered headless adapters. The later [native record gate](docs/verification-native-smiles-records.md)
adds bounded Mac explicit import/order/history/whole-file Save/reopen. Wider navigation, automatic
clipboard capture and broader chemistry/import remain. Finish native record acceptance
from the same plan, then the missing M2 atom/selection/ring/style workflows below.

Whole-record SDF removal now has a source-preserving codec, the shared typed command,
CLI/stdio/authenticated-loopback routes and original Records confirmation UI. Independent
literal raw/per-MOL consumers and actual-App checks cover exact surviving bytes,
Undo/Redo, draft decisions and Unicode Save/reopen. A real pre-spawn file-context race
was reproduced, fixed and retested without changing its regression. Root's final local
gate passes1060/0/1; native evidence is scoped in
docs/verification-sdf-record-removal.md. Record creation/reordering, broader format
editing and complete M1 remain required. This does not complete M0 or any later milestone.

The original SDF occurrence inspector and existing-value editor are delivered. Nine
independent actual-App editor cases cover exact duplicate-field edits, history, real Save,
Apply→Save input ordering, navigation decisions, external requests/source replacement,
no-op CRLF retention and refusal without publication. Two finite registered-worker gates
protect real commit/cancellation/retirement. Native Mac multiline Apply, Unicode Save As,
Undo/Redo and reopen preserve the predeclared788-byte output and both molecular records.
The editor's994-test local gate passes. See docs/verification-sdf-editor-integration.md and the
earlier inspector evidence. Native draft/close races, record operations and full M1 remain
required next work; the later view acceptance is below.
The subsequent [SDF view integration](docs/verification-sdf-view-integration.md) now delivers
Actual/Zoom/pan through the shared camera and source-preserving current-view captures.
Seven command cases, four actual-App cases, two native-adapter gates and eight renderer
cases pass; the full local gate is1016/0/1. Native Mac F5/Actual and both position controls
visibly display the ordinary CCO/CN records without changing source/history. Native wheel,
physical-DPI measurement and the combined edit/preview/Save workflow remain unverified.
The [original proposal and receiving decision](docs/research/sdf-view-plan.md) retain why
success-only prepared bounds replaced the proposed error-side-effect bootstrap.

The subsequent Fit readout correction labels SDF Fit textually before/after extent
preparation and retains explicit percentage entry, without changing legacy images or
camera routing. Actual-App cases and native read-only AX/painting are recorded in
docs/verification-fit-readout.md. Native percentage typing remains separate; high-DPI
relative steps still use the declared logical Fit basis rather than a measured overview.

A restricted all-record SDF capture route now connects source-preserving mixed-version
projection, separate display-H provenance and faithful resolved-H labels through the
existing command/CLI/MCP interfaces. Ordinary VAL-absent CCO/CN, both source orientations,
full CPU/GPU image comparisons and exact Save pass headlessly. The native read-only workspace
preview now has registered background work, source/view/DPI/
close-epoch invalidation and independent actual-frame texture/paint/retirement tests. Actual
default Fit can still refuse its em guard; the bounded native Actual/pan recovery above
now works. Ordinary source-backed MOL
absent-H depiction now uses the same closed display facts in its actual Page/IDs/frame,
with complete deferred ownership, independent CPU/GPU/source checks and headless canvas
acceptance. Authored None and source bytes remain exact; source-free/CDXML stay literal.
See docs/verification-mol-display-tests.md and docs/verification-mol-display-ui.md.
Centered Double/Triple depiction now passes independent analytic pixel/geometry, actual
CPU/GPU and source-preserving MOL/SDF checks. It uses an original3w profile, without
mapping inherited styles or automatic side selection. Explicit positioned doubles now
have the separate inset evidence above. See
docs/verification-multiple-render.md and docs/verification-multiple-commands.md.
Broader formats, source metadata, SDF record operations and full M1 interoperability remain unfinished.
This is partial M1, with neutral Single/Double/Triple geometry and explicit whole-capture
refusals; general layout and reference packing
remain open. See docs/verification-sdf-capture-command.md and the linked independent evidence.

The complete SDF session/Open/Save slice now retains every mixed-version record and
ordered data byte through current/saved/history and all shared adapters. Native Mac mixed-
version Open/Save/Unicode Save As/reopen and failed-import retention now pass onbb6d5b9;
the690-byte and additional refusal checks are controller-only. Root's590 gate and matching
three-platform hosted checks pass. Bounded field inspection/existing-value commands now pass engine/CLI/MCP tests.
Broader native chemical preview acceptance, record operations and
broader SDF interoperability remain missing; see docs/verification-sdf-session-files.md and
docs/verification-native-sdf-and-extension.md. That earlier native build used a static summary;
the new read-only preview is wired/headlessly tested, with its native workflow unverified.
This is not M1 completion.

The V3000 application slice now retains versioned source/header/frame/history and exposes
both versions through the existing two shared MOL commands. Auto Save keeps its associated
version; Save As chooses V2000/V3000 explicitly; Open validates the actual family member.
Independent IO/engine/CLI/stdio/loopback and UI tests plus the corrected local372+3 gate pass.
Actual Mac author/save/reopen/refusal/close and CPU/GPU reply draining now pass at1c53312.
This feature's own hosted checks pass in43531603511 on all three platforms; see
docs/verification-v3000-application.md. Conservative conversion/layout/source-loss refusals
remain. Broader SDF/native editing workflows and mappings still follow; this is not M1 completion.

Current user-directed priority: finish M1, then continue M2–M5. MOLV2000 now has a source-
preserving codec, owned backing/history, atomic filesystem I/O and two shared commands;
engine/CLI/MCP tests use published BIOVIA2026 fields and external RDKit consumers. The native
format choice/Save/reopen/close workflow is verified on Mac; all three hosted aggregates and
native window smokes pass at c879f4c. One CTAB/fragment and layout/metadata guards
are partial interoperability, not completed M1. SDF V2000 now has a source-preserving
container library, ordered duplicate/empty/DT data, a wrapper writer and independent
raw-field/molecular consumers. A fetched original SDF corpus now checks two records and
every ordered field without a lossy property map. Broader SDF editing/display and wider mappings remain next work. Standalone V3000 parsing,
writing and the version-aware MOL application slice are now integrated; mixed-version SDF
container and reviewed legacy RXN container are integrated.
These libraries retain complete raw molecular/data/role payloads; SDF whole-session files
are now wired, while reaction models/commands/native workflows remain separate missing work.
The original bounded SMILES notation/source library now has literal graph/token/limit
tests and qualified pinned external-parser evidence. Literal aromatic atom/bond markers
are preserved without perception. Bare implicit H, bracket H0 and
physical H remain distinct. Named opt-in normalization materializes only a closed neutral
fixed-H subset; a separate Fragment exporter generates noncanonical notation. Both have
literal/external-consumer/boundary tests and combined ASAN evidence. General drawing
import/layout, aromaticity/stereo/perception and canonicalization remain missing. Complete
source-file ownership is a distinct path below. See docs/verification-smiles.md.
Full M0
inventory/dialog/addressability gaps remain tracked while this format work proceeds.

The restricted carbon-path layout now also has a create-only filesystem converter into
new editable native CDXML. Its 64 KiB input, source preservation, publication failure/race
guards and independently consumed actual output pass the local 670-test aggregate.
The shared conversion command now passes engine/CLI/stdio/authenticated-loopback workflows
through explicit CDXML Open/edit/history/Save/reopen, plus the local 686-test gate. It
preserves the active session and uses the same save lease through finish/drop. Native
conversion controls/flows and general cleanup remain missing.

The explicit-H writer now also creates real single-record SMILES files through the shared
atomic publisher, preserving its isotope/physical-H/branch/ring/disconnected/Single/Double/
Triple scope. Exact LF framing, actual-file consumers and publication guards pass the local
704-test gate. The shared explicit-selection command/adapters now pass the local715 gate,
including actual nonfirst native-source export, full graph/metadata/order comparisons and
semantic negative controls. It retains the complete owner/save lease through finish/drop,
reports omitted geometry/document context and preserves the active session. Native export
controls and broader chemistry remain missing.

Complete accepted SMILES files now retain all records, ignored lines, opaque data and
LF/CRLF/final-EOF bytes through source ownership, bounded native I/O and the shared
document.open_smiles/save_smiles commands. Any unsupported/malformed later notation
rejects the whole Open. Independent raw and unchanged per-record consumers verify the
codec/file adapters. Commands/CLI/stdio/authenticated-loopback retain complete snapshots,
save leases and generation/close rules; summary state has no dummy drawing or automatic
record data. A separate ASan target passes a bounded10,000-run smoke check. Native chooser
and static summary are wired/headlessly tested, with actual native acceptance pending
user availability. All-record chemical preview/editing, record operations, broader syntax
and native conversion/export remain missing. See [file/application evidence](docs/verification-smiles-file-app.md)
and [UI evidence](docs/verification-smiles-file-ui.md). M0/M1 remain incomplete and Actions remains disabled.

The next [original record-inspector plan](docs/research/smiles-native-workflow-plan.md)
now has a delivered typed native query bridge: three literal/background/guard/authority
tests and the full Root1024/0/1 local gate pass. See docs/verification-smiles-ui-work.md.
The actual UI component is now delivered with five independent App cases and two finite
worker/close gates. Root's full local gate passes1031/0/1. Native Mac AX/painting shows
ordinal/Inspect/navigation and no automatic notation disclosure, with an exact native-
process query/source check. Native input/clipboard/dialog acceptance, chemical preview
and complete M1 remain unverified/missing. See docs/verification-smiles-record-integration.md.


Standalone CDXML source preservation and checked existing-attribute replacement have
original-fixture/Expat and ASan smoke evidence plus one pinned unchanged RDKit corpus
document. A source-backed direct-fragment node geometry view now uses validated model
points; missing coordinates remain absent. Model atom/bond/fragment storage is validated.
Read-only elemental mapping now assigns fresh application IDs, resolves fragment-local
references and retains source backing; an external Expat/Decimal oracle checks the corpus.
Owned application snapshots/history retain that backing; unsynchronized edits cannot
export stale XML. New native elemental CDXML encoding now preserves modeled metadata,
connectivity and explicit Poster canvas extents; an external RDKit consumer opens the
actual single-bond command's output. Shared registry/native CDXML Open/Save/Save As and
Mac safe-close/reopen/edit workflows now have independent file-consumer evidence. Unknown
source passthrough/refusal, bounded atomic saves and background/stale-result guards pass.
The original single-document chooser is partial File coverage; broader imported editing remains.
Native independent C-C/three-H rendering is delivered, with explicit terminal labels.
Source/history now has shared-allocation accounting and oldest-first byte eviction.
A proven generated-file profile now reopens/edits/renders headlessly, retaining source and
exact history; extended/noncanonical files remain guarded. Next: general source-preserving
structural edits, broader native file parity and retention/recovery scenarios;
broader query/label/style mapping, application CDX mapping/editing, MOL/SDF V2000
and V3000, RXN, SMILES and explicit data-loss diagnostics. Every fixture needs a pinned
source, license and SHA-256. Never silently treat unsupported formats as passing.

The first standalone CDX slice now indexes bounded little-endian records, preserves unknown
source bytes and checks opaque property replacements. Original vectors face independent
Python struct and external RDKit consumers. Implicit/explicit roots have specific probe
evidence. A bounded new-tree syntax writer now independently reproduces an original chemical
scene and passes literal lengths/resource checks and external RDKit. General file compatibility
and editable encoding/faithful preview remain missing. Guarded shared/native Mac CDX
Open/Save preserves accepted source exactly; it does not regenerate changed projections.
One original explicit chemical scene is fetched from its real public commit,
SHA-verified and checked against independent struct/RDKit consumers. Foreign/general CDX
corpus and reference/chemical mapping remain M1 work.

## M2: drawing — in progress

The earlier ring command37 verifies explicit3–8/free/fused/spiro construction through the
engine/adapters/files/renderers; native tools now have their separately scoped evidence
above. Current carbon-chain controls, Add/repeat/preferences and bounded Mac evidence
are in [receiving](docs/verification-native-chains.md). Selection
and movement have14 actual-App passes plus the separately scoped bounded native Mac
acceptance. See [ring receiving](docs/verification-ring-authoring.md),
[native selection](docs/verification-native-selection.md) and
[selection receiving](docs/verification-selection-authoring.md) for exact scope.

The [elemental authoring contract](docs/research/atom-label-authoring-contract.md) now
has a bounded headless `atom.set_label` implementation. Nine core, four actual-wire and
one independent MOL-reference CPU/GPU case pass, including complete history and actual
saved-file consumers; see [receiving evidence](docs/verification-atom-label-authoring.md).
It changes only explicit element/literal-H fields in existing neutral atoms in source-free/
native CDXML. The original `ui.atom_label` presentation now adds fresh atom/label targets,
explicit fields, Apply/Cancel and stale/modal/Save/close guards. Actual Mac OH/NH2,
history, close protection and Unicode Save/reopen pass; see
[native evidence](docs/verification-native-atom-labels.md). No general valence inference.
Modern label-entry and repeat-label evidence stay separate; full inline labels, aliases,
charges/isotopes/radicals, full native selection/transforms/rings/chains and styles remain required.

The shared118-symbol catalog, positioned neutral-element labels and literal H have
independent font/CPU/GPU/file evidence. MOL/SDF display H stays separate from authored
None. Centered Single/Double/Triple scenes use logical bond IDs, actual component hits,
attachment/mask/visibility guards and the unchanged approved Single golden. Other metadata,
context-sensitive multiple placement and source-faithful styles remain missing.

`bond.create` accepts an explicit Single/Double/Triple order (defaultSingle) and authors
literal H=`4-order_weight`. `bond.extend` preserves identities/source/history while
extending a consistent neutral-carbon terminal within its explicit H budget; source-free
or byte-proven native CDXML only. Native Single C3/C4 authoring/history/CDXML reopening
has Mac evidence. The new multiple workflows are wired and pass independent actual-frame,
file/chemistry/image tests. Bounded native multiple creation now has Mac evidence at the
recorded readable scale; native multiple terminal extension remains unverified.

`ui.bond_tool` supplies three procedural accessible palette controls and a checked
transient epoch, outside authored history/dirty/source state. Ordered activations cancel
older gestures; held previews and hover use the same actual component geometry. Existing
Single/attachment regressions remain. See docs/verification-multiple-ui.md and
docs/verification-multiple-authoring.md. This does not finish M2 or establish26.1 styles.

The original refreshed workspace has a compact rail and contextual document inspector,
with source/count/file state and stable paper geometry. Native macOS visual review,
Single/Double/Triple selection, pointer authoring, existing-edge edits, history and Unicode
CDXML Save As now have finite actual-window evidence. The later scene is native-authored;
its420pt-square page was explicitly registry-prepared. Broader native attachment, source,
platform and view workflows remain unverified. See docs/verification-bond-order-native.md;
the earlier registry-authored captures retain their original scope.

Existing-bond editing now has a sourced public-guide contract and one typed
`bond.set_order` command with explicit order/revision/IDs and a required literal neutral-
carbon C4 H policy. Exact no-op/history/source/overflow/error guards, real CLI/stdio/socket,
independent Expat/Decimal/RDKit saved files and fixed-seed edit properties pass. The UI
wires absolute1/2/3, source-supported Single conversion clicks and Single-tool endpoint
Single→Double / Double→Triple; unsupported alignment/click cases report their gap.
Ordered commands reacquire full backing, graph and page geometry. See
verification-bond-order-{command,ui,native}.md under docs/ for exact evidence and limits.
Actual native Mac drawing/conversion/hotkeys/history/Unicode Save As pass for the recorded
readable-scale scene; no universal26.1 click matrix or native multiple extension is claimed.

Usable transient view navigation now has one typed command, exact legacy Fit, uniform
logical zoom, numeric entry, Actual/Fit/Magnify/Reduce, two-axis controls and wheel pan.
Independent renderer/command/adapters/actual-App tests and native Mac unchanged-A4
Single/Double/Triple creation and150% editing/persistence pass. The fresh-focus SDK
filter and rapid Undo→Save prepass bugs were independently reproduced and corrected.
The aggregate961-test gate passes; source/scale/label guards remain strict. No physical
DPI calibration, SDF navigation, saved view preferences or complete reference parity.
See docs/verification-view-{render,commands,ui,native}.md and
docs/verification-ordered-file-actions.md.

Next: outstanding native modifier/DPI/race/ring acceptance, broader preferences,
remaining chair stereo/platform workflows, snapping, transforms, broader labels and styles. Do not restart the
delivered chain prerequisites or treat their bounded Mac checks as full tool/platform parity.
M1 source-faithful interoperability and M0 addressability/inventory/platform evidence
remain open. Keep imported-source/mapping gaps visible; these bounded workflows do not
complete a milestone.

Bonds, rings, chains, labels/abbreviations, charges, isotopes, radicals, wedge/hash bonds,
selection, transforms, hotkeys, undo/redo and style sheets from public journal guidelines.

## M3: chemistry — in progress

Resolved composition now formats molecular formulas in strict Hill order.
Combined element totals and numbered isotope terms both preserve the net charge.
Twenty independently specified structures were checked with external RDKit.
Carbon-free formulas use alphabetical element order. The library reports
unresolved H explicitly. The native Analysis window, CLI and MCP show the formula of a supported drawing or the contribution of selected atoms. Qualified skeletal CDX and elemental V2000/V3000 MOL resolve supported omitted H counts from complete fragment connectivity before selection. [MOL scope and tests](docs/verification-mol-analysis.md). Broader source-specific H inference, mass, isotope distributions, elemental percentages and live analysis captions remain open. [Composition evidence](docs/verification-formula-composition.md), [formula conventions](docs/research/hill-formula.md) and [native and wire acceptance](docs/verification-analysis-window.md).

Valence, aromaticity, stereo/CIP, checking, cleanup, formula/mass, analysis, reactions,
curved arrows, brackets and chemically formatted captions. Compare against independently
run RDKit, reference InChI and OPSIN; document conventions and tolerances before testing.

## M4: publishing — planned

SVG/PDF/PNG, physical-scale output, documented clipboard/Office workflows, atomic saving,
recovery and independent export consumers. Editable embedding needs public evidence.

## M5: depth and distribution — planned

Independent name/structure conversion, original templates, biopolymers/HELM, WebAssembly,
packaging and signed installers. Keep documented advanced capabilities (including naming,
predictions, annotations and file types) visible in the parity backlog until supported.

No milestone claims full commercial parity. Account/cloud/licensing services are not required
for Bondline; unsupported reference integrations remain identified rather than disappearing.

The [May2026 capability audit](docs/research/capability-chart-audit.md) refines existing
milestones with explicit placement-aware paste, dark drawing styles, axial/M/P stereo,
chirality-flag interpretation, hydrogen-bond relationships, named descriptors, additional
format directions and monomer cases. All remain missing with source/platform qualifications.
Optional scientific-service connectors must not become offline creation dependencies.
The [CDX candidate review](docs/research/cdx-corpus-candidate.md) adds a concrete unsupported
framing/provenance question; it does not add a passing corpus file or justify relaxed parsing.

The [installed desktop map](docs/research/installed-feature-map.md) and
[implementation tickets](docs/research/installed-implementation-tickets.md) add exact-build
control/state/flow evidence without replacing the complete M0–M5 target. First-pass503 menu
entries and a successful original local-file flow do not complete inventory or parity.
Native CDX framing precedes safe CDX projection/application I/O; collection ownership and
record editing precede application SDF; general graph scenes/hit testing precede connected
gestures. Account/cloud routes stay optional to the offline authoring product.

The header-embedded CDX framing slice now preserves the22-byte prefix/root/separate-file-end
profile while keeping successful legacy parses and all existing strict negative tests.
An unchanged original native1196-byte save and original public-field vectors face independent
record/byte/consumer checks. Receiving root417+3 and normal-graph ASAN pass; the new source
needs hosted checks. This parser feature precedes model mapping and application CDX files.

The first CDX elemental view and owned complete-source snapshot now have literal/raw/
consumer/identity/bounds/error/history-storage evidence. Unchanged export is exact; edited/
cross-format/model-generated export is refused. Native metadata and application CDX I/O
remain next, alongside source-aware safe rendering. Receiving438+3 and normal adapter
ASAN pass; this new source requires hosted checks. Do not count it as completed M1.

The CDX filesystem library now reuses bounded reads and security-preserving atomic saves
for unchanged complete snapshots. Both original framing variants' actual output has strict
independent consumer checks; receiving443+3 and all aggregate gates pass. Registry/CLI/MCP/
native chooser, broader native metadata and editing remain separate next features. This
library-only slice needs its own hosted checks and does not complete application CDX or M1.


Guarded CDX application files now have exact-build Mac native chooser/Known Save/Unicode
Save As/reopen/retry/dirty-close refusal and Undo recovery, plus independent actual-output
consumers. Four original variants and one ignored own-drawing source retain every byte.
Source450962c passes its own three-platform aggregates/window smokes and six ASAN targets.
The16-command terminal-extension integration501a315 separately passes523/521/513 hosted
tests/windows and six ASAN targets. Actual native attachment acceptance remains next; SDF
collections remain M1 library-to-application work. No milestone is completed by these slices.


The SDF whole-source ownership prerequisite now passes9 new literal/consumer/identity/
resource/accounting cases, receiving549 aggregate checks and a normal10000-run ASAN smoke.
It retains all records/fields/versions in shared immutable snapshots; Engine/file/native
collection integration remains next. Field edits are being implemented as a separate
immutable candidate operation, not yet exposed as an application command. Installed SDF
observation confirms both drawings in one document; an initial record browser is unproven.


Existing SDF field-value replacement now has10 independent literal/no-op/invalid/boundary/
consumer cases, receiving559 aggregate checks and normal10000-run ASAN. It preserves every
other source byte and leaves original snapshots immutable. Whole-collection Engine/current/
saved/history/file commands and native data editing remain the next M1 integration; this
library is not advertised as an application editor or complete format parity.

## Explicit SMILES record addressability

The source-file slice now has smiles.record.inspect: one explicit revision/ordinal, exact
notation and global ranges/counts, with opaque data retained privately. It uses the same
historical deferred-result/cancellation/close machinery, no write lease or mutation, and
a128KiB serialized-object budget. Core and independent raw/graph/wire evidence are linked
from docs/verification-smiles-record.md; native selection/preview/editing remain missing.
The newer bounded source-backed record engine now delivers selected/all import, supported
order edits and complete-source Save/history headlessly. Finish its native worker/control
gate next. Wider CDX typed metadata/Text/Double/style mapping,
faithful rendering and source-backed editing remain required; these source/automation
foundations do not complete format or editor parity.
