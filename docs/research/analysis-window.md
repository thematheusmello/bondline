# Formula in the Analysis window

The vendor documents a View menu window which follows the selection and analyzes the entire drawing when selection is empty. The v25 guide allows a portion of a structure to be selected and shows a formula alongside mass, isotope and elemental-analysis fields. The optional Paste action inserts analysis information into the drawing. This increment implements the formula readout. [Vendor support](https://support.revvitysignals.com/hc/en-us/articles/4408239739668-ChemDraw-Where-is-the-Analyze-structure-command-and-icon) · [User Guide v25, printed pages 221–224](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf).

## What the result counts

Every included model atom contributes its own element, isotope and formal charge. Its recorded attached hydrogens contribute separately. Empty drawing, unspecified attached H, unsupported source and resource limit each have a distinct result. An explicitly recorded zero is valid. Formula output uses isotope-separated Hill order and ASCII charge notation. [Numeric composition](formula-composition.md) · [Hill and RDKit conventions](hill-formula.md).

A nonempty selection consists of unique atom IDs from its current page. Bondline reports the selected atoms’ literal contribution and checks whether each contributing fragment is selected in full. The panel labels an incomplete fragment **Selected-atom composition** and explains the uncapped bonds. Multiple complete fragments produce a combined formula. Captions and styling do not add atoms. The guide does not specify how to assign hydrogens across cut bonds, so a molecular formula for the excised structure remains unresolved.

The formula query requires both the current document revision and selection epoch. Each edit, Undo, reopened document or new selection retires the appropriate earlier request. The panel caches one report under the weak content identity and selection epoch. The host requests repaint after a published visibility transition or edit. Repeating Show and inspecting a formula preserve the document, history, dirty state, selection and file association.

## Source coverage and bounds

A source-free drawing supplies its entire atom model. Imported drawings must pass a closed source-profile check for the complete drawing. A detached CDX model, opaque extension, raw SDF or SMILES collection, and a drawing projected from selected SMILES records have no complete-source result. Formula analysis grants no file-export or authoring permissions. The bounded synchronous reader inspects at most 4,096 atoms and 4,096 fragments on the included pages. Larger inputs return an explicit limit result.

Certified skeletal CDX pages disclose only the literal model counts. Propane with written H counts of 3, 2 and 3 yields `C3H8`. When the middle count is absent, whole-drawing analysis identifies atom 2. Selecting a resolved terminal carbon still yields `CH3` as a partial-fragment contribution. With all H omitted the first atom is unresolved. A detached source loses its analysis grant. [Skeletal CDX contract](cdx-skeletal.md).

## Remaining Analysis work

Exact mass, molecular weight, isotope distribution, elemental percentages, decimals, repeating groups, mixtures with stoichiometry and live pasted captions need separate sources and acceptance tests. A structure with unresolved H currently cannot produce a whole-drawing formula. Source-derived valence and partial-selection boundary conventions are separate chemistry decisions.
