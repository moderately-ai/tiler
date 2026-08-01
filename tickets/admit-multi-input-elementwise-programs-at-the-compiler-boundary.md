---
id: admit-multi-input-elementwise-programs-at-the-compiler-boundary
title: Admit multi-input elementwise programs at the public compiler boundary
status: todo
priority: p1
dependencies: [admit-multi-input-tensors-in-the-scheduled-region-vocabulary]
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

## Outcome — the admission is not compiler-side work

Worked at base `1b2a776` on `nightly-2026-07-19`. The measurement is re-run, made permanent, and the ticket's design question is answered — but the answer moves the work rather than completing it, so this ticket stays open behind a new dependency.

**The design question, and why both of its candidates were wrong.** The ticket asks whether "recognize a shape" survives as a third recognizer for N-input elementwise DAGs, or is replaced by a general normalization over the semantic graph. Neither candidate survives, because both are `tiler-compiler` answers to a constraint that is not in `tiler-compiler`. `prototype-semantic-normalization` is `done` and is a *congruence* stage — it collapses identical pure operation invocations to one semantic value — so it neither recognizes strategies nor could be extended to; it does not compete with the recognizers and replacing them with it was never available. And a third recognizer, however written, can only emit a normalization the physical layer is able to express. It is not: `PointwiseF32Node::Input` is a nullary leaf meaning *the* single input element, `PointwiseF32ExpressionBuilder::build` is the type's only constructor (its fields are private), and it refuses a second input with `DuplicateInput` while `is_valid` independently requires `inputs == 1`. `TensorRole` carries no ordinal, so two reads of two distinct program inputs are indistinguishable at the region boundary, and `kernel::lower::plan` derives its addressing, bounds, and element count from `reads.first()` alone. A widened recognizer would therefore admit the region at the boundary and fail in the middle of the pipeline — the outcome this ticket's own boundaries call worse than the refusal.

The one mechanism that looks like it already binds several buffers is the U4 dequantize path, and it is not one: `EncodedComponentRole` is documented at `crates/tiler-ir/src/semantic/types.rs:1220` as "semantic schema data, not an ABI slot or graph operand position" — components of one encoded tensor, not independent inputs. Reusing it as an operand position would encode a false semantic claim.

**Measurement, now permanent.** `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` pins both halves: the approved three-input region refuses with `UnsupportedCapability { rule: "signature" }` and no explain trace under all four `NumericalContract` values, and the one-input control `(a * 2.0f32) * 3.0f32` compiles under all four against the same governed profile — so the refusal is specific to input cardinality rather than a broken profile. A second test pins the located cause, `PointwiseF32ExpressionAdmissionError::DuplicateInput`, and shows a constant is still admitted afterwards so the rejection is of the second *input* and not of any further node. Each assertion was perturbed and watched to fail: the refusal class (against `NoFeasiblePlan`), the control's compilation (by substituting the three-input program), and the admission-error variant (against `StructuralLimit`). This file replaces the hand re-derivation the measurement had already been through twice, at `b623670` and `e6a47d9`.

**What did not change, deliberately.** No recognizer was widened. Admitting the program at the boundary without a physical layer that can express it is the failure mode the ticket names, and a widening that lands in `tiler-ir` first makes the compiler-side change small and verifiable rather than speculative.

**Blocked on.** `admit-multi-input-tensors-in-the-scheduled-region-vocabulary` (`implementation/ir`), now this ticket's dependency, which carries the located evidence and the identity-encoding hazards. When it lands, this ticket's remaining work is the compiler-side recognizer and program assembly, and the permanent test above is what turns its closure into a demonstrated transition.

**Referred to Tom, not self-accepted.** The refusal rule `"signature"` is a single combined gate over input count, output count, operation count, and dtype, so the approved region's refusal does not name input cardinality as the unrecognized property — and `CompileFailureClass::UnsupportedCapability { rule }` is public observable behaviour, quoted verbatim in `crates/tiler-macros/src/region.rs:55`. Splitting it so a frontend can emit a diagnostic naming multi-input support is a public boundary change under ADR 0075 and is recommended, but it is Tom's to accept and was left alone.
