---
id: carry-a-source-correspondence-from-region-text-to-emitted-msl
title: Carry a source correspondence from region text to emitted MSL
status: deferred
priority: p3
dependencies: []
related: [retain-and-attribute-a-real-msl-failure-through-an-expansion]
scopes: [implementation/frontend, implementation/metal, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [deferred, diagnostics, frontend]
---
## Deferred — the triggers that reactivate this

**This is a deferral, not scheduled work.** It is filed at `deferred` so the scheduler never offers it. Either trigger below reactivates it; neither has fired as of 2026-08-04.

1. **Invocation-controlled text reaches the emitted MSL.** Today none does — `tiler_metal`'s emitter derives every entry point, helper, and staging name from an identity digest, names buffers `b<ordinal>`, and emits scalar constants as hexadecimal bit patterns, so no `InputKey`, `OutputKey`, or region token appears in the translation unit. A frontend that ever emits a consumer-chosen identifier, an inline MSL escape hatch, or a consumer-supplied literal into the source makes an MSL diagnostic attributable to something the consumer wrote, and this becomes real work.
2. **A `metal` diagnostic must be actionable by a consumer rather than by a Tiler developer.** See below.

## Why this exists

**Fact — the ask.** `docs/integration/frontends.md`'s remaining-checks list asked for "MSL text attributed to region source": an MSL line and column mapped back to the `out` sub-expression, operand, or `deliver` token that produced it.

**Fact — nothing carries the correspondence, at two independent points.** `tiler_ir`'s semantic program holds no frontend spans, and must not: the compiler core stays independent of any frontend's tokens, so a `proc_macro::Span` cannot travel through it. And `tiler_metal`'s emitter attaches no per-statement provenance to the text it writes — the only correspondence in an emitted unit is the entry point's `// Entry point` comment and the kernel and scheduled-region identity digests beside it, which name a *kernel*, not a source construct.

**Inference — building it is a public-boundary design, not an implementation detail.** A correspondence would be a new public structure produced by `tiler-macros`'s region parser, carried through `tiler-ir` and the compiler without becoming a frontend dependency, and consumed by `tiler-metal`'s emitter to annotate emitted lines. That crosses three crates and adds a public type on each, which is Tom's boundary rather than a worker's.

**Inference — and today it would attribute to the wrong thing.** `retain-and-attribute-a-real-msl-failure-through-an-expansion` derived that a `metal` rejection of the emitted *source* is unreachable from any region text and therefore a defect in Tiler's own emitter, whose reader is a Tiler developer; and that the one route a consumer can hit without a Tiler defect is a build host whose `metal` predates the declaration's measured language standard, whose remedy is the toolchain rather than any construct in the region. Pointing a diagnostic at an `out` sub-expression in either case would name something that is not at fault. `crates/tiler-macros/src/aot.rs`'s "Reaching the `metal` stage's own refusal" carries the derivation, and `a_retained_msl_diagnostic_carries_the_emitted_source_position` keeps the fallback honest by holding that a retained MSL diagnostic still carries its own emitted-source position.

## What reactivation would have to answer first

- Which span a consumer acts on when one MSL line comes from several fused semantic operations, and what is reported when the answer is "all of them".
- How the correspondence survives fusion, splitting, and reassociation without becoming a second authority over what produced which line — the parent ticket's standing constraint is that a span must never be inferred by matching MSL text against region tokens.
- Whether the correspondence participates in artifact identity. It must not change the compiled bytes, and something must hold that.
