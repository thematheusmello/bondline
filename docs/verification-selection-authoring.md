# Persistent selection and translation receiving

2026-11-16. The full M0–M5 goal remains active. The [locked original contract](research/selection-translation-plan.md)
extends the public guide's selection/movement roles; it does not establish exact26.1
screens or modifier precedence. Actions remains off. Native interaction is not yet verified.

## Atomic model batch

`Document::replace_fragments` validates every target and the complete candidate once
before publishing replacements. Input order cannot reorder the page. Unrelated Arc
fragments stay shared; missing/duplicate replacement IDs and collisions with replacements
or retained atoms refuse the entire batch. The old single replacement delegates to
this same implementation. No model fields, file syntax or dependency changes.

An independent2722-byte declaration has SHA256
`6f7af4c25c5eb4f9e73fcf36044eda60766dd70cb9e3657102bb324a6be8b3a3`.
Root48686 observes two intended E0599 missing-method errors before implementation.
Root79851 then passes all19 model tests, including the new literal multi-fragment case
and unchanged old identity/budget/storage cases. This is a model API pass, not evidence
of a native selection tool or completed M2.

## Initial command and adapter REDs

Root45335 reaches real runtime UnknownCommand for both `ui.selection` and
`selection.translate` (0/2), then actual CLI unknown_command for `ui.selection` (0/1).
The initial2402-byte core declaration SHA256 is
`25eab4539ae3be39df72adac889f17e7b1fec144c0b9db232aa17ffad7b67c3d`;
CLI2204B SHA256
`83ef911fcb419c9fed15d739f1c65d92aa0df116f9870bab4c86b65fc56b662f`.
No future Rust API is required just to reach those runtime failures.

Root48686 separately reaches actual-App0/1: no current actionable Marquee selection
control. Before any receiving or product implementation, the initial App declaration's
Button/full-visible-name assumption is qualified to the existing compact selectable
tool/glyph pattern. Both originals remain frozen. Qualified2247B SHA256
`9ebe810656c6e26e9e45998f1e8ed39e015c069f07199c5581f7f32dd85839e7`.
No runtime assertion is weakened to conceal an implementation failure.

Ignored declarations/logs live under `target/selection-authoring/`; committed tests are
the durable executable expectations. The command/native/adapter implementation and
expanded receiving continue after the model checkpoint. No full-gate or platform pass
is claimed by this initial record; the earlier MCP shutdown failure remains unresolved.
