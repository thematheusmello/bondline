# Shortened sided-double rendering contract

2026-11-16, based on edeeddf/cde514c. This upgrades the temporary Left/Right render
refusal in sided-double-prerequisite.md. It is one required depiction step toward the
full roadmap, not an alignment command, skeletal policy or unsaturated-ring tool.
No reference binary, proprietary artwork, dependency or golden change is authorized.

## Evidence and original choices

[IUPAC2008 GR1.10](https://iupac.qmul.ac.uk/drawing/drawing.html#GR110) specifies an
unchanged axis stroke plus an offset stroke ending on adjoining-angle bisectors.
An unsubstituted end or substitution only across the axis does not shorten that end.
GR1.10.4/GR3.5 establish ring-context depiction; they do not set a universal spacing.
[The public SDK](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/Bond_DoublePosition.htm)
defines Left/Right looking from B to E. Absence stays unspecified; automatic/manual CDX
variants cannot be recovered from XML. Existing pinned source identities remain in
clean-room/sources.md and sided-double-prerequisite.md.

Keep the original spacing s=3w and 9w<=axis length, with no claim of a vendor default.
Unspecified and Center Double/Triple retain their entire existing centered path and
pixels. Specified placement on a non-Double still refuses. Only explicit Left/Right
Double uses this new geometry. Do not auto-select a side or change literal H/labels.

## Geometry

For authored A→B, L=|B−A| and unit t=(B−A)/L. Canvas y increases downward, so the
screen-right unit normal is R=(-t.y,t.x). n=R for Right and -R for Left. Keep the first
stroke exactly A→B. The second is A+s*n+a*t → B+s*n−b*t, same width/color.

At A use inward g=t; at B use g=-t, but the same physical n. For each other outgoing
adjoining unit ray v, c=n·v and d=g·v. Only c>0 contributes a trim. Its angle-bisector
distance is s*(1+d)/c; use the equivalent s*c/(1-d) when d<0 to avoid cancellation near
a straight continuation. No same-side ray gives zero trim. For multiple rays, use the
largest required trim (closest same-side angular neighbor), independent of storage order.
That tie/nearest-neighbor choice is explicit original policy, not observed vendor behavior.

Zero/invalid adjoining rays, exact co-directed overlap, nonfinite/out-of-range geometry,
a+b>=L, or collapsed/invalid rounded output refuse atomically. Opposite collinear
continuation has zero trim. Do not add a blanket minimum bond length or angle. Use
bounded adjacency rather than repeatedly scanning all bonds; cancellation remains first
and is checked through topology/geometry work. Existing primitive/hit/coordinate bounds
remain. Calculations use f64 before actual f32 stroke construction; test tolerances
must come from those representations and be declared before judging results.

## Complete scene guards

Actual component strokes feed painting, labels, visible polygons, hit testing, view
projection, raster density and paper/viewport bounds. Do not use the authored axis as
the visible secondary extent. Every component must retain unmasked visible ink.

For sided doubles, reserve the inter-line corridor only over the actual common
longitudinal span of the two strokes, plus all actual component ink (including primary
tails). Check against all nonincident components. A symmetric old width4w envelope
misses the offset side and incorrectly reserves the opposite side. Keep that old
envelope exactly for the centered profile. Search white gap pixels only where both
components are present; averaging unequal full endpoints overstates that range.
No new white pixel outside the true shared span may qualify a missing interior gap.

An actual authored-axis component of a sided Double may join an incident Single axis
or another sided Double's axis as ordinary Single–Single solid rays already do. This
is a component-role rule at the same authored atom, not a larger anchor mask or a
blanket shared-bond exemption. Store/derive the explicit style before choosing roles;
centered Double index0 is not an axis. Every pair involving an offset still faces the
existing exact anchor/mask overlap test. Legacy centered/Triple junctions stay unchanged.
Co-directed/zero-length adjoining geometry still refuses before this exception.

Source completeness, unknown data, Save/export loss guards, label placement/readability,
component/gap pixel requirements, CPU/GPU coverage, work budgets and cancellation are
preserved. No command, model/file field, font, shader or asset is added by this step.

## Independent acceptance before implementation

- Current public API runtime RED for a literal directed Right Double; no missing helper
  API is needed. Analytic horizontal strips and 90°/3–4–5/120° bisector cases, terminal/
  trans-only zero trim, Left/mirror/reversal and neighbor-order controls.
- Literal width3 120° axis–Single join must render. Sided offset–offset overlap outside
  the old anchor must still refuse. Collinear/acute exhaustion and cancelled/bounded
  inputs fail without modifying the Page. Old centered/Triple controls stay exact.
- Distinguish actual offset-side ink/corridor from empty opposite-side space and inset
  end space. A visible component and a true common-span white gap are both required.
  Verify actual shortened hit extents, rotation, physical density and view projection.
- Analytic butt-rectangle pixels and unchanged CPU/GPU/Pillow consumers verify accepted
  scenes; source/model data stay exact. Existing data-only refusal tests require an
  explicit positive contract upgrade, while dormant/cancellation/legacy assertions stay.
- Actual complete native CDXML Open/capture/history/Save/reopen through shared commands
  must preserve placement/source/H, with independent file/pixel consumers. No broad
  imported-source authority or MOL loss exception. Native OS interaction remains a
  separate check while the desktop is in use; keep all previews closed.

Run focused RED/GREEN, relevant full local gates and a bounded source review. Preserve
any natural existing MCP listener failure without retry/serialization/policy changes;
complete only not-started stages separately. Hosted Actions stays off. Then continue
skeletal presentation, weighted unsaturated candidates and native aromatic tools.
