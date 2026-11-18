# SDF prior-page test readiness — 2026-11-18

This is a test-only qualification on base783a632, not an SDF production or listener fix.
Full M0–M5 remains active/incomplete; Actions OFF.

The receiving qualified aggregate44695 fails only
apply_before_previous_fields_refreshes_then_installs_the_requested_prior_page in the
UI library:330 passed/1 failed/1 ignored. The failed assertion requires Field1 visible
after Apply→Previous. Its edit bytes/revision/history/dirty/original-file checks had passed.
The wait before that assertion checks revision2 and !sdf_fields.is_busy only.

Independent source review establishes a premature-readiness possibility:
SdfFieldsWorkflow::is_busy tracks pending/job/editor work, while fields.show runs during
show_workspace and later sync/settle_editor can retire/install the requested page AFTER
that frame's accessibility nodes were produced. Backend revision2/no-busy can therefore
coexist with the previous frame's spinner/Field65 nodes. This code ordering is not a
captured causal proof of44695's failure.

The same existing wait now additionally requires Field1 present and Field65 absent.
It preserves revision2, no-busy, the ten-second deadline, frame pumping, every later
positive/negative assertion and all production code. It waits for the workflow the test
already asserts, rather than accepting backend completion as rendered completion.
Focused26864 passes1/0. The final receiving aggregate will report its own result separately.
No timeout/retry/socket policy, field pagination/authoring behavior or old assertion is
weakened. Initial patch context matched a different similar wait; it was corrected to
this exact test before any execution, leaving the other wait unchanged.
