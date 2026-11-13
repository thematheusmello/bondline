# Approved single-bond reference

single-bond.png is an actual 1024×1024 Metal capture from Bondline's native macOS
session, not a mockup or reference-product asset. The same authored document produced
byte-identical CPU/GPU PNGs. The fixture JSON records its physical page and endpoint
coordinates; the test constructs validated model data, not a purported native project file.

SHA-256: 1b650231527f47db5dc7c368644939907d2b038d32f25e77578318f483521bbc

On 2026-11-13 the user saw these exact bytes and answered “Approve this reference” to
approval of this specific scene. This does not approve other scenes, general chemical
rendering, platform backends or publication styles. The earlier blank-page candidate
remains unapproved. Do not regenerate this reference to accommodate a failing test;
a changed reference needs renewed human review of its exact bytes.

The test decodes the PNG with external Pillow 12.3.0, verifies its hash and compares every
RGBA byte with CPU and actual GPU output. Deliberately corrupted pixels must fail too.

The original scene/image is licensed Apache-2.0 and uses the Apache-2.0 Roboto font already
attributed in assets/manifest.json and assets/fonts/LICENSE-Roboto.txt (repository root).
Original test/source code remains MIT OR Apache-2.0. No proprietary artwork is included.
