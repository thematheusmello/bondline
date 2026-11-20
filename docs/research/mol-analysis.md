# Analysis of MOL structures

## Selection

The Analysis window follows the selection. With no atoms selected, the report
covers the drawing. A selected atom contributes its own attached hydrogens
from its complete fragment. Selecting ethanol's oxygen gives `HO`. Choosing
both the oxygen and adjacent carbon gives `CH3O`. The guide leaves the
treatment of cut bonds unspecified. Bondline counts the drawn contributions
and identifies a partial selection in the panel. [Vendor support](https://support.revvitysignals.com/hc/en-us/articles/4408239739668-ChemDraw-Where-is-the-Analyze-structure-command-and-icon) · [Guide, pp. 221–224](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf).

Opening a completely represented MOL enables Marquee and Select All.
Selection, inspection and deselection use the existing revision, page, atom
ID, selection epoch and source identity checks. A selected atom can be
inspected with no coordinate write. The native keyboard and pointer and
the headless translation command keep the original movement restriction.
Other source formats retain their own admission checks. [Selection and source editing](../verification-native-selection.md).

## Missing valence in an elemental connection table

The V2000 atom valence column and V3000 `VAL` attribute include bonded
and implicit hydrogen. Explicit zero has a distinct encoding. The input
codec already resolves declared valence against the bond-order sum. Analysis
retains that result even when it is zero. No derived count enters the document
or the saved MOL. [BIOVIA Chemistry 2026 CTfile Formats, pp. 9–13 and 64–70](https://www.3ds.com/assets/invest/2026-01/biovia_chemistry_2026_ctfile_formats.pdf).

For an omitted valence, the modern BIOVIA table assigns one or more allowed
valences to a main-group atom. Bondline uses the first sufficient value after
counting incident single, double and triple bonds over the complete fragment.
The calculation uses atom IDs and connectivity. Coincident coordinates and
isotope labels have no effect on that calculation. An explicit hydrogen atom
contributes its own isotope and occupies a bond on its neighbour. A doublet
occupies one additional valence. [BIOVIA Chemical Representation Guide, Default Valences and Rules for Calculation](https://manualzz.com/doc/44003874/biovia-chemical-representation) · [Sayle, original comparison of the two published conventions](https://www.inchi-trust.org/wp/wp-content/uploads/2017/11/1.-Sayle_InChI_2017.pdf).

Supported neutral atoms include B, C, N, O, F, Si, P, S, Cl, As, Se, Br,
Te, I and At, and the zero-valence noble gases through Rn. The supported
singly charged period-two atoms use the appropriate same-row isoelectronic
entry where defined. Isolated halide ions contribute zero attached H.
Unmarked metals, unhandled charge states, singlets and triplets, excessive
bond sums and ambiguous bond types leave the affected atom unresolved.
An independently resolvable selected atom in another component can still be
inspected. The original file contains no edition selector to distinguish
several incompatible historical metal defaults. [Comparison and historical
ambiguity](https://www.inchi-trust.org/wp/wp-content/uploads/2017/11/1.-Sayle_InChI_2017.pdf).
