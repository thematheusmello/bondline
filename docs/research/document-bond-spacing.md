# Explicit document relative spacing

Contract2026-11-17, baseb377456. Full foreign styles and M0–M5 remain the target;
this increment completes one drawing-settings path without widening unknown semantics.

The public [BondSpacing](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/properties/BondSpacing.htm)
property is0x0804, INT16, stored as10×percentage in CDX and plain percentage in XML.
It applies to Document and Bond. The SDK relates it to preferred BondLength and says
Abs takes precedence; unresolved owner/Abs/default cases stay guarded. A preliminary
unsigned-domain assumption was corrected against this source before tests/code.

The existing [public evidence](bond-spacing-evidence.md) is now supplemented by actual
execution of pinned Apache-2.0 Octet-Draw on original skeletal CDXML, with rejecting
font/measure shims never called. Twenty-eight analytic records pass201 comparisons;
fourteen capability observations are separate. Fixed preferred40/p18 gives centreline
separation7.2pt at actual20/40/80; preferred20/80 at actual40 gives3.6/14.4. Changing
width leaves centreline distance fixed and changes the visible gap. That program ignores
tested Abs values and falls back at0/100, so it is not an oracle for those behaviors or
all26.1 semantics. No reference assets/fonts/binaries or upstream implementation ship.

Authored `DrawingStyle` gains independent optional bond_spacing_percent with preserving
width/spacing builders. `new(width)` and absent JSON/native bytes retain old behavior.
The initial interpreted data domain is finite0..3276.7 (nonnegative signed encodings),
not a universal XML validity assertion. Negative/unsupported values remain source-backed
and opaque. True signed zero is distinct from negative nonzero lexical underflow in XML
source certification. The native setting and shared authoring command allow1..100.

Explicit root spacing resolves s=p*preferred_length/100 in document points. Original
source-free/proven-native absence of preferred length uses the documented own48pt
creation convention; this is not a foreign style-sheet default. Missing spacing keeps
the exact original3pt path and guards. Camera transforms scale s and ink independently
of glyph/em units. Plain centered Double uses±s/2, Triple uses-s/0/s, sided Double uses
the existing bisector trim with s. Explicit spacing uses actual geometry/ink/paper/gap/
hit guards, not the old arbitrary one-third fallback restriction; that legacy restriction
stays exact when no spacing is authored. No clamping, thinner ink, moved atoms or lost H.

`document.set_document_settings` remains command46 with four required keys. Add one
optional nullable scalar: omission preserves, null clears, number sets spacing. Keep all
45 prior schemas/order exact. The46th schema/description intentionally expands while
old wire requests remain valid; Rust struct literals add the omission field. Typed
nonfinite values must fail serialization rather than turn into null. Native Apply3/pair
remain preserving; a four-field Apply helper shares the same weak request/currentness.

Construct the final settings/style candidate once and certify once when ink/spacing or
an active spacing reference length changes. Do not reject a valid combined edit because
an intermediate preference-only candidate would be invalid. Preserve no-op Arc sharing,
redo, source/history/Save leases, raw guards and one normal revision/transaction.

The existing form gains one bounded384-byte percentage field using the same ordered
prefix/deferred-suffix/currentness/modal/cancel rules. Request presentation may expose
the current scalar; it grants no authoring or Save authority. Whole-owner display,
preview, capture, paste and candidate certificates must pass current settings as well
as style, overriding stale caller values. CDXML changes are per-property source splices;
unaddressed LineWidth lexemes remain exact. MOL loss/CDX/SMILES drift guards still apply.

Test-first acceptance: independent numeric/Expat/Decimal source vectors, actual external
geometry, analytic CPU/GPU pixels, actual/preferred contrast, width/label/DPI invariance,
legacy/schema/history/error/source/no-op/atomic cases, real CLI/stdio/loopback files and
actual-App four-field input. New native interaction must be inspected when access exists.
Fresh controller startup fails now; native acceptance remains explicitly unverified.
No dependency, font, asset, golden, workflow or listener-policy change belongs here.
