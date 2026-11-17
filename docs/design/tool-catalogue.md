# Available and planned tools

The native workspace keeps its fourteen delivered tool bindings at the top of the original
64-point rail. A scrollable section shows the planned tools in grey. **All tools…** opens
a searchable, grouped catalogue; search accepts names, group names and source IDs.

There are **140 display entries covering 124 documented tool roles**: TD-001–058,
TQ-001–034 and TB-001–032 in the [parity checklist](../parity-checklist.md). The count
expands the explicitly documented ring sizes, two chairs, arc angles and bracket forms.
Fourteen entries reference delivered bindings; **126 remain disabled**. This is the
documented inventory, not a claim to know every modern dynamic palette variant.

Groups cover selection, bonds, chains, rings, reactions, shapes, brackets, curves,
labels/data, chemical symbols, orbitals, queries, biology and HELM. Icons are original
procedural drawings. Biology replaces the reference feature's proprietary category name.
Unnamed arrow/attachment variants and dynamic template/monomer libraries remain open
families; no proprietary artwork or library is copied.

Planned entries have no command binding, are unselected, and expose **Not implemented**
through tooltips and accessibility descriptions. Delivered tools unavailable for the
current document instead say **Unavailable for this document**. Nothing in the catalogue
grants editing authority. Existing file, atom-label and SDF foreground forms take input
priority, including Escape. Opening/searching/closing this catalogue changes no document,
history, persistent selection, file state or effective tool. Choosing an available tool
uses its existing registered binding.

`Engine.controls()` continues to list delivered controls only. The roadmap catalogue is
UI presentation data in `crates/ui/src/tool_catalogue_data.rs`, with no fake commands.
The UI resolves real bindings from current controls and replays them through the existing
ordered path. Search is bounded to 128 characters over a fixed-size inventory.

To enable a future entry, first implement and independently verify its real shared
command and complete workflow, add its typed native control, then set the catalogue's
binding ID to that delivered control. Update exact inventory tests and scoped parity
evidence in the same commit. A grey icon alone never changes a feature's missing status.

The [design contract](branding-and-tools.md) and [receiving evidence](../verification-branding-and-tools.md)
record the original request, public-source boundaries and actual validation.
The later [chain receiving](../verification-native-chains.md) adds only Acyclic and
Snaking bindings after independent App/adapter/file and bounded native Mac checks.
Both chairs remain planned. Native Control-drag/DPI and full platform/reference parity
are separate from the checked chain workflows.
