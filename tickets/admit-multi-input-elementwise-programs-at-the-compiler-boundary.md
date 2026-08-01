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

## Independently reproduced at `e6a47d9`, 2026-07-31

**Measurement.** `prototype-inline-aot-integration-proof` re-ran this ticket's measurement from a fresh temporary integration test at base `e6a47d9` on `nightly-2026-07-19`. The approved three-input region still refuses under all four `NumericalContract` values with `UnsupportedCapability { rule: "signature" }` and `explain: "absent (refused before a target-qualified trace)"`. The same probe compiled `(a * 2.0f32) * 3.0f32` — one input, two `F32Constant` operations, four operations total — successfully, so the boundary refuses this program specifically rather than refusing everything.

**Fact, and the consequence that raises this ticket's priority.** The gap is wider than "the frontend cannot invoke the compiler for the approved region": *no* region the approved grammar can express is admitted, for any operand count. `crates/tiler-macros/src/grammar.rs:113-128` gives the region body exactly two productions, `Expression::Operand` and `Expression::Binary` over `*` and `+`, so a region has N tensor inputs and zero constant operations; both recognized normalizations require exactly one tensor input plus constants (`normalize_pointwise`, `crates/tiler-compiler/src/request.rs:1959-2079`) or a `strict_serial_sum_f32_op` reduction the grammar cannot spell (`normalize_serial_sum`, `:2092`). The intersection of "expressible" and "compilable" is empty.

**Inference.** This ticket is therefore the critical path for `prototype-inline-aot-integration-proof`, which now declares it as a dependency: the AOT, cache, embedding, and runtime halves that proof needs are all verified ready, and the only missing piece is a program a consumer can write that the compiler will accept. The alternative route — giving the region grammar a scalar-literal production so a consumer can write the one-input shape that already compiles — is a change to `tensor!`'s observable public surface and is Tom's under ADR 0075, not a worker's.
