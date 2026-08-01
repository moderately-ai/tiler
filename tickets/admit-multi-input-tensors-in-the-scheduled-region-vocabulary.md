---
id: admit-multi-input-tensors-in-the-scheduled-region-vocabulary
title: Admit multiple input tensors in the scheduled-region and physical scalar vocabulary
status: todo
priority: p1
dependencies: []
related: [admit-multi-input-elementwise-programs-at-the-compiler-boundary, prototype-inline-aot-integration-proof]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal, implementation/build, implementation/frontend, contracts/artifacts, contracts/navigation]
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

## Outcome

**Design — the indexed input is a payload on `TensorRole::Input`, not a new variant and not a sibling field.** Four candidates were tested against identity encoding, correctness, and what a widened consumer would have to carry.

A *new variant* beside `Input` was eliminated on redundant encoding: a single-input region would be expressible two ways, so two structurally identical regions could carry different identity bytes, and canonicalizing that would be an unstated invariant nothing enforces. A *sibling field on `Access`* (`input_ordinal: Option<InputOrdinal>`) was eliminated on both representable nonsense — `Some` on an output access — and on cost, because the ordinal must reach every consumer the role reaches anyway (`BufferParameter`, `BoundsProof`, `BoundaryRequirement`, opaque-call bindings), so it buys a checkable invariant and saves no plumbing. A *boundary-tensor table* on `IndexRegion`, mirroring `tiler_ir::index`'s `TensorId` arena, was eliminated as the ordinal with indirection: the scheduled region's accesses are already an ordered list bound positionally at every layer below, so a table entry would carry nothing but the role and the ordinal, while adding a handle-validity obligation to verify.

What survived is the payload. The decisive fact is that the *role is what travels*: no consumer carries anything else that could separate two reads. `tiler_ir::index` reaches the same separation already — `identity.rs:96-106` computes a positional ordinal among same-role tensors while encoding — so this states what the sibling layer derives.

The ordinal is positional and region-local, never an `InputKey`: a scheduled region carries no semantic correlation (ADR 0070), and keying on the caller's names would make renaming an input recompile the kernel. A tensor's *components* share an ordinal and stay separated by `EncodedComponentRole`, which is what keeps the strict-affine U4 path a three-component read of one tensor.

**End-to-end evidence.** `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` compiles the approved region against `TargetProfile::governed()` and resolves the per-target outcome, so each pass is a complete verified plan: three access reads with ordinals 0/1/2, one bounds proof each, a four-buffer kernel signature, one load per input, and a four-value program binding one external allocation per `InputKey` in declaration order. It passes under `StrictF32`, `FlushSubnormalsToZeroF32`, and `ReassociateF32`.

**It does not pass under `RelaxedF32`, and that wall is not this one.** `RelaxedF32` is the contract that permits arithmetic contraction, and `fusion_legality`'s `ArithmeticContraction` obligation returns `unrealized-contraction` for any region holding a multiply adjacent to an add under such a contract. `a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction` records that the widening which would discharge it was *eliminated rather than deferred*, with a measurement: a permitting realization carries no `NoFloatingPointContraction` obligation into the artifact and the measured Apple row fuses a written multiply/add pair under `-ffp-contract=fast`. Reopening it needs new evidence, not this ticket.

Measured, this worktree, `nightly-2026-07-19`: `(a * b) + c` (three inputs) and `(a * 2.0) + 3.0` (one input) refuse identically under `RelaxedF32` and compile under the other three; `(a * 2.0) * 3.0` (one input, same family) compiles under all four. The refusal reads the multiply/add adjacency, at any input count. `admit-the-fused-multiply-add-pointwise-body-under-a-contracting-contract` owns the remainder.

**Transition demonstrated.** Run against the unwidened boundary the flipped test fails at the vocabulary, not at the assertion: `the_three_input_region_compiles_under_every_contract` reported `Err(UnsupportedCapability { rule: "signature" })` where it expected `Ok(())`, and `the_physical_pointwise_expression_names_each_input_tensor` failed to compile at all — `expression.input()` takes no argument and `PointwiseF32ExpressionAdmissionError::DuplicateInput` does not exist in the widened vocabulary. Both halves moved in this change.

**Identity domains.** `tiler.schedule.v2` → `v3` and `tiler.kernel.v4` → `v5`. Both ordinals land inside repeated records, so an earlier reader loses framing and every region and kernel ever encoded maps to different bytes; a cache or artifact holding an earlier identity must miss rather than match. `tiler.kernel-program.v6`, `tiler.artifact-program.v12`, and the neutral manifest schema deliberately do **not** move: their own record layouts are unchanged, they fold the stepped identities by reference, and the artifact binding table is keyed by interface *name* rather than by role. `docs/artifact-abi.md` and `docs/status.md` carry the ledger step.

**Pins rebaselined, each with its reason recorded beside it.** `STRICT_F32_REGION_IDENTITY_HEX` (schedule domain plus the ordinal bytes); the governed target descriptor `GOVERNED` (one byte of the `buffer-bindings` row); the explain request qualifier `bddeaf899938ede4` → `0b7759de2d9b5756`; four `crates/tiler-metal/goldens/*.metal` (kernel and region digests, plus a rendered role comment — the emitted MSL bodies are unchanged).

**Beyond the stated scope, and flagged rather than absorbed.** Two changes the closes-when forced:

- The governed target profile's declared `max_buffer_bindings_per_entry` rises from 2 to 4. A three-input region binds four buffers and the old bound refused it at target feasibility. Four is the widest signature the bounded profile can assemble and is what the governed `buffers` budget already admitted; it remains a compiler-governed prototype guarantee, not a device measurement, and Metal's documented per-stage argument table bounds it far above. **This is a target-profile authority row and wants Tom's eye.**
- `normalize_pointwise` now admits a *mixed* operation body. The old recognizer required the child to repeat the root's key, so `(a * b) + c` — the approved region's own shape — was refused as `pointwise-association` regardless of input count. `NormalizedPointwise` carries `root_operation` and `child_operation` separately.

**Two checks are reservations, not tested guarantees**, and say so at their sites: `pointwise-leaves` and the per-input arm of `pointwise-shape` cannot fail in this profile, because a frozen program retains only output-reachable declarations and the semantic schema requires operand shapes to agree. They are kept because the first widening admitting a deeper body or a broadcast operand makes them reachable, and both failures they name are silent.
