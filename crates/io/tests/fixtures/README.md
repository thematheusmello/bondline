# Original native elemental profile

bondline-v0.cdxml is an independently authored Bondline 0.1.0 format vector, not a vendor
sample. Its field names/semantics come from the public CDXML SDK in the source register;
its two-carbon graph and coordinates were chosen for this test. It pins the native encoding
delivered in 23ffd33 so a future encoder/version change cannot quietly disable older files.

License: MIT OR Apache-2.0, like the original repository code.
Bytes: 408, no final newline.
SHA-256: ead553df9d8c6bf7a371e7e36909a9c8895f34b9231f20c3c3a796680d64bbc6

The independent Python oracle checks this hash and decoded revised-field expectations.
Keep this vector when adding profiles/migrations; do not refresh it just to pass a test.
It is not a rendering golden or evidence of the complete CDXML format.
