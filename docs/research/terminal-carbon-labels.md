# Terminal-carbon labels: locked integration

2026-11-16, after native-ring checkpoint672744f. This is a data/display prerequisite
for chain authoring, not chain-tool, style-preference or full M0–M5 completion. Lock
before new declarations/product changes. The open user bundle/draft stays untouched.

## Public basis and limits

[SDK TableOfProperties](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/TableOfProperties.htm)
defines ShowTerminalCarbonLabels0x0442 for carbon with zero/one incident bond, separately
from0x0443 and literal NumHydrogens. Root verifies the primary table online. The
[official DTD](https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd) permits node/root
yes/no with default `no`. Web access rejects its application/xml-dtd content type; the
unchanged28821-byte primary cache SHA2565311978e514ffe154108540c3634c314dc66031a4f3c877a681fc6dca8e128c2
supplies lines79/249. SDK cache96359B/SHA25695d46f9a2329a145bd9da2057b9ad51c39eede119aa92318b2edaaecfc88538d.

[IUPAC2008 GR1.4/2.1.1–2.1.2](https://iupac.qmul.ac.uk/drawing/drawing.html#GR211)
supports skeletal termini while requiring labels for unbonded carbon and isolated
two-carbon/one-bond structures, including multiple-bond orders. Root reads these primary
rules online. They are depiction recommendations, not vendor26.1 execution evidence.
Root/node inheritance, child Text and vendor defaults remain unresolved; no extra source
authority or implicit inheritance follows. The policies below are original Bondline choices.

## Data, source and loss boundaries

Add Atom.show_terminal_carbon_labels:Option<bool>, independent of the existing
nonterminal flag and H. None/default/null serialize omitted and preserve the exact old
JSON/native-v0 profile; Some(false/true) serialize literal JSON booleans. Existing closed
Atom validation remains. Positively upgrade only the old negative that used this now-
recognized field: keep the same input and check exact new value/unchanged old fields.
Other aliases/invalid booleans/identity/graph errors remain rejected.

Map only the exact node XML attribute and yes/no values; missing stays None, other values
are InvalidProperty. Write a present terminal attribute before the existing nonterminal
attribute. Standalone CDXML/CDX parsers and source passthrough remain unchanged. Node
flags do not model root inheritance or child Text, and cannot bypass byte-proven native
completeness. Keep foreign/opaque original bytes and refuse unsafe edits/export.

Both model-to-MOL writers refuse any specified terminal value, including dormant/false,
with a terminal-specific metadata-loss error. Retained-SMILES replacement/equality and
MOL regeneration checks retain this field just like the existing nonterminal datum.
No writer silently deletes it. No new default/tool setting, command name, dependency,
asset, stylesheet or general format capability is introduced.

## One shared depiction decision

Build actual incidence once per fragment. For carbon degree0/1, consult only the terminal
flag; for degree>=2, consult only the existing nonterminal flag. The other datum is dormant.
None follows the exact old path. Noncarbon flags stay dormant with existing metadata
guards. Degree counts actual Bond objects, including physical H, not weighted order.

An active true prints the full known literal/resolved-H label, without a new C4 restriction.
An active false requires known H and checked incident Single/Double/Triple weight+H=4;
coverage alone is not proof. Unknown/invalid counts or unsupported orders refuse, never
infer H or erase it. The existing nonterminal collinear/two-Double mandatory rules stay exact.

After that proof, degree0 always retains the full label. A degree1 carbon also retains it
when its sole neighbor is carbon and has degree1: this proves the actual isolated C2
component without a Fragment-size heuristic, irrespective of directed order, storage,
unrelated components or Single/Double/Triple weight. Other eligible degree1 carbons hide.
This component interpretation and tri-state legacy profile are explicit original policy.

Hidden glyphs and masks disappear together; stable atom IDs/anchors/hits and complete
literal chemistry remain. Actual stroke/gap/paper/pixel/resource/cancellation guards
stay strict; entirely glyphless scenes report no label em. No nominal-glyph fallback,
blank isolated carbon, weak mask, source override or second renderer path is allowed.

## Existing explicit-label workflow

atom.set_label restores each target flag that is exactly false to true, together with
the supplied element/H. None/true and unrelated fields stay exact. Same element/H with
one or both false flags is one real commit; repeated true/None is the existing validated
no-op. Restore both false flags independently, even if one is currently dormant, so
explicit authoring records intent without duplicating renderer degree logic in commands.
Other atoms, topology, coordinates, saved/source snapshots and normal history stay exact.

The existing native editor, CLI, initialized stdio and authenticated loopback call that
same registry operation. No new parameter, control, dialog or Save/close authority.
Hidden endpoints remain fresh Alt targets; Cancel preserves data, Apply/history/Unicode
Save/reopen retain it. App evidence remains distinct from actual OS/monitor execution.

## Independent acceptance and continuation

Observe JSON/XML runtime failures before data adoption; keep new Rust-field compilation
qualification separate. Original property/Expat/Decimal controls, strict invalid values,
native-v0 exactness, unknown/root/Text source retention, both MOL loss guards and retained-
SMILES drift refusals must cover missing/false/true and both-flag combinations.

After data exists, observe actual renderer/explicit-label failures before their changes.
Analytic multi-carbon chains require full hidden stroke endpoints/pixels/hits/CPU-GPU
agreement and retained literal H. Include methane, isolated C2 Single/Double/Triple,
unrelated/disconnected context, noncarbon/dormant flags, physical H, known/resolved versus
unknown/invalid C4, both-flag precedence and old near-collinear/two-Double labels.
Actual Unicode files/history and CLI/MCP/socket/App editor paths use independent raw
properties plus unchanged chemistry consumers. No self-writer-only or regenerated golden.

Run local fmt/clippy/workspace/layer/license/corpus checks; preserve any natural MCP bind
failure without unchanged retries. Then lock the complete chain-tool policy from the
source proposal, keeping Add Chain/repeat/Snaking/settings and all M0–M5 requirements
visible. Actions stays off; no publication or user-draft replacement.
