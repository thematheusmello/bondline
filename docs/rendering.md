# Rendering evidence and scope

The delivered scene contains an RGBA8 background and ordered, axis-aligned integer
rectangles. Later rectangles replace earlier pixels, including alpha; there is no alpha
compositing or antialiasing in this reference path. Chemical primitives and text are missing.

The CPU reference clips rectangles and writes straight RGBA8 bytes. The GPU path clips before
float conversion, uploads a checked storage buffer and draws instanced triangle pairs with
an original WGSL shader. RGBA8Unorm readback is tightly packed after removing row padding.
The GPU path never silently substitutes a CPU result. Pipeline validation is returned as
a typed error. Tested native backend: Apple M3 Max / Metal.

Bounds: readbacks are 1..2048 pixels per axis; scenes have at most 4096 rectangles. Empty or
overflowing rectangle dimensions fail before GPU initialization. Geometry-only page layout
accepts native viewports up to 100000 units per axis without allocating a pixel frame.

The page producer uses physical model dimensions. It fits the page inside a margin of
min(32, width/8, height/8), rounds each dimension to the nearest integer and centers it.
An extreme aspect ratio still occupies at least one unit. The original paper style has a
gray 232 background, gray 210 offset shadow, gray 196 one-unit border and white interior.
This is a Bondline design decision, not a claim about another application's appearance.

The native UI consumes those same opaque page rectangles in logical egui display units;
the OS/egui transform applies display scale. Offscreen reference dimensions are pixels.
These fit-to-viewport previews do not establish print scale or physical-resolution export.

The geometry assertion accounts for the declared integer rounding: each outer dimension
has at most 0.5 unit error, so |width - aspect_ratio * height| <= (1 + aspect_ratio)/2.
The interior is exactly one unit inside that border. The former continuous float-layout
assertion was replaced with these pixel-grid invariants before wiring the shared producer.
An independent, hand-authored 8x8 pixel grid additionally checks every byte on CPU and GPU.

Other evidence covers layered source replacement (including alpha), clipped geometry,
overflow and primitive-count rejection, one-pixel pages, and odd readback row pitches.
Actual native page rotation was visually/accessibility checked after the shared-scene change.
Human-approved goldens, native Windows/Linux execution, chemical rendering, blending,
antialiasing and publication-quality color/output remain unverified or missing.

Render workers accept a shared cooperative cancellation flag. Pre-cancelled work fails
before frame allocation or adapter creation; CPU drawing checks between scanlines and GPU
readback uses nonblocking device polling with a ten-second deadline. Cancellation cannot
preempt an operating-system driver call or an already submitted GPU command. These limits
must remain explicit when adapters move captures off the document lock.
