---
id: admit-multi-input-elementwise-programs-at-the-compiler-boundary
title: Admit multi-input elementwise programs at the public compiler boundary
status: todo
priority: p1
dependencies: []
related: [prototype-inline-proc-macro-frontend, prototype-semantic-normalization, prototype-inline-aot-integration-proof]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api]
---
## Why this exists

`prototype-inline-proc-macro-frontend`'s ticket text settles that the frontend "invoke[s] the ordinary compiler boundary". It does not, and the reason is a measured property of that boundary rather than a decision the frontend took.

**Measurement — worktree at base `b623670`, 2026-07-31, nightly-2026-07-19.** A temporary integration test in `tiler-compiler` built the approved region as a `SemanticProgram` — three `f32[4]` inputs, `F32Multiply`, `F32Add`, one output — and called `session::compile_governed` under each of the four `NumericalContract` values and `session::compile` under the governed target profile. All five refused identically:

```
UnsupportedCapability { rule: "signature" }
```

before any target-qualified explain trace.

**Fact.** `crates/tiler-compiler/src/request.rs` recognizes exactly two program shapes, and both require a single input. `normalize_pointwise` opens with `program.input_count() != 1 || program.output_count() != 1 || program.operation_count() != 4` and `normalize_serial_sum` with `program.input_count() != 1 || program.output_count() != 1 || !(RECOGNIZED_OPERATIONS_MIN..=RECOGNIZED_OPERATIONS_MAX).contains(&program.operation_count())`. The recognized pointwise program is one input against constants; a program with three tensor inputs matches neither. Reproduce with `grep -n "input_count() != 1" crates/tiler-compiler/src/request.rs`.

**Inference.** Wiring `session::compile` into the expansion today would make the approved region — `sym n; in a, b, c; out (a * b) + c` — an unconditional `compile_error!` on every invocation, because `docs/integration/frontends.md` requires that "target-neutral parse, semantic, optimizer, verifier, and envelope failures become unconditional `compile_error!` diagnostics". That is why the frontend constructs and verifies the public logical program and stops there, and why the edge from `tiler-macros` to `tiler-compiler` was not added.

## User-visible outcome

A semantic program with several tensor inputs and an elementwise expression over them compiles through `tiler_compiler::session`, so an inline region reaches the optimizer instead of being refused at strategy selection.

## Boundaries

- The recognizers are a bounded prototype profile, not a contract; widening one is not licence to accept an unrecognized program silently. An unsupported program must still refuse with `UnsupportedCapability` and a rule that names what was not recognized.
- Whether "recognize a shape" survives at all, or is replaced by a general normalization over the semantic graph, is the design question — `prototype-semantic-normalization` is the neighbouring work.

## Closes when

The measurement above is re-run against the widened boundary and the approved three-input region compiles, or the ticket records with evidence why a frontend must not invoke the compiler for such a program.
