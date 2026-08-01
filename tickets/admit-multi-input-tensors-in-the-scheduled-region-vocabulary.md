---
id: admit-multi-input-tensors-in-the-scheduled-region-vocabulary
title: Admit multiple input tensors in the scheduled-region and physical scalar vocabulary
status: todo
priority: p1
dependencies: []
related: [admit-multi-input-elementwise-programs-at-the-compiler-boundary, prototype-inline-aot-integration-proof]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, schedule]
---
## Why this exists

`admit-multi-input-elementwise-programs-at-the-compiler-boundary` set out to widen the compiler's strategy recognizers so the approved inline region `sym n; in a, b, c; out (a * b) + c` would compile. It cannot be done there. The recognizers are where the refusal is *observed*, but the vocabulary that makes multi-input inexpressible lives in `tiler-ir`, below every choice `tiler-compiler` is able to make. This ticket owns that widening; the compiler-side admission is its dependent, not its peer.

**Measurement — worktree at base `1b2a776`, 2026-07-31, nightly-2026-07-19.** The approved three-input region refuses with `UnsupportedCapability { rule: "signature" }` under all four `NumericalContract` values, with no explain trace, while the one-input control `(a * 2.0f32) * 3.0f32` compiles under all four against the same governed profile. Both halves are now permanent tests in `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs`; that file replaces the hand re-derivation this measurement had already been through twice (`b623670`, `e6a47d9`).

**Fact — the physical scalar vocabulary names one input.** `PointwiseF32Node::Input` (`crates/tiler-ir/src/schedule/pointwise.rs:71`) is a nullary leaf meaning "the single `f32` tensor element read by this scalar invocation". `PointwiseF32Expression`'s fields are private and `PointwiseF32ExpressionBuilder::build` (`:251`) is its only constructor — verified by reading every signature in the file that returns the type. The builder refuses a second input with `PointwiseF32ExpressionAdmissionError::DuplicateInput` (`:188`), and `is_valid` (`:141`) independently requires `inputs == 1`. No recognizer `tiler-compiler` could write routes around this: a widened recognizer would produce a normalization the physical layer cannot express, which is admission-then-mid-pipeline-failure — strictly worse than the current refusal.

**Fact — the region boundary names one input tensor.** `TensorRole` (`crates/tiler-ir/src/schedule/model.rs:22`) is `Input | Intermediate | Output` with no ordinal, so two reads of two distinct program inputs are indistinguishable at the region boundary. `kernel::lower::plan` (`crates/tiler-ir/src/kernel/lower.rs:143`) takes `reads.first()` and derives `read_tensor`, `read_elements`, `read_bounds`, and `addressing` from that one access; `emit_pointwise` (`:459`) binds exactly one `input: KernelValueId`. The several-read shape that does exist is the U4 dequantize path, and it is several *components of one encoded tensor* — `EncodedComponentRole` is documented at `crates/tiler-ir/src/semantic/types.rs:1220` as "semantic schema data, not an ABI slot or graph operand position", so it is not a spelling for independent input tensors and must not be reused as one.

**Inference.** The program and artifact layers are already N-input: `MaterializedOrigin::ProgramInput { key: InputKey }` admits any number of externally bound values, and `tiler-compiler`'s assembly binds exactly one only because the region it assembles for reads exactly one. The widening is therefore concentrated at the scheduled-region and physical-expression layer, and the layers above follow from it rather than needing independent redesign.

## User-visible outcome

A scheduled region can read several distinct program input tensors, and a physical `f32` pointwise expression can name which input each leaf reads, so an N-input elementwise DAG has a representable region, kernel, and program.

## Boundaries and what to watch

- `TensorRole` and `PointwiseF32Node` are both deliberately non-`#[non_exhaustive]` ADR 0074 convention 5b types whose own documentation states that adding a variant is *expected* to break every total map, and that the break is the design. Give a new variant its own identity tag at each encoder rather than widening an existing one; `TensorRole::Input` alone has 157 occurrences across 6 crates (`tiler-ir`, `tiler-compiler`, `tiler-artifact`, `tiler-reference`, `tiler-metal`, `tiler-build`), and the schedule/kernel/program identity encoders (`push_component_role`, the `PointwiseF32Node` tag bytes at `model.rs:985`/`:994`) are the sites where a silent mis-encoding would be worst.
- Canonical identity bytes will move. Every pinned digest downstream — the explain request qualifier `bddeaf899938ede4` in `crates/tiler-compiler/src/explain.rs:3730` included — must be rebaselined deliberately, with a comment, and each movement flagged rather than absorbed.
- Whether the indexed input is a new `TensorRole` variant, an ordinal field on the existing one, or a separate boundary-binding type is a genuine design choice with different identity-encoding consequences; it is the first thing to derive, not to assume.
- An unsupported program must still refuse with a typed reason naming what was not recognized. Widening the vocabulary is not licence to accept an unrecognized program.

## Closes when

The approved three-input region compiles end to end under all four `NumericalContract` values — reaching a complete verified plan, not merely passing strategy selection — and `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` is updated in the same change, its refusal expectation becoming a compilation and its `DuplicateInput` expectation becoming an indexed input leaf. That edit is what makes the transition demonstrated rather than asserted.
