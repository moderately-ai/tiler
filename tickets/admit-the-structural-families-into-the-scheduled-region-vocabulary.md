---
id: admit-the-structural-families-into-the-scheduled-region-vocabulary
title: Admit the structural families into the scheduled-region vocabulary
status: review
priority: p1
dependencies: [admit-the-registered-unary-families-at-the-compiler-request-boundary]
related: [reach-a-verified-kernel-through-the-structural-families, admit-the-reindex-and-broadcast-operation-families]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, compiler, structural]
claimed_from: todo
assignee: agent-structural-vocab
lease_expires_at: 1785988645
---
## User-visible outcome

A program stating `tiler::reindex-f32@1` or `tiler::broadcast-f32@1` reaches the optimizer instead of refusing at the request boundary under `operation-set`, so the two families the pinned workload cannot be written without stop being statable-but-unrecognizable.

## Why the elementary admission did not carry these two with it

**Fact — the activation was admissible and these are not, and the difference is which vocabulary is missing.** [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](admit-the-registered-unary-families-at-the-compiler-request-boundary.md) admitted `tiler::silu-f32@1` by *projecting* its per-point body into `PointwiseF32Node`, from one shared statement the governed index-access lowering also drives. No `PointwiseF32Node` spells a sigmoid-weighted linear unit either — but the body is expressible in the nodes that exist, so a projection is available.

**Fact — what these two lack is the access relation, and `LogicalAccess` has no spelling for it.** `crates/tiler-ir/src/schedule/model.rs`'s `LogicalAccess` carries `LinearIdentity`, `ScalarBroadcast`, `PackedU4LsbZeroTail`, `ReductionContributor`, and `ContractionOperand`. There is no reindex map at all, and the only broadcast is `ScalarBroadcast` — "every invocation reads the single scalar parameter element", a rank-zero operand read once — which does not express the workload's `[1024]` to `[T, 1024]` widening. A projection into the per-point vocabulary cannot substitute for a missing coordinate map, because the two families compute nothing: each result element is an operand element with the same bits, and what varies is *which* element.

Reproduce the absence in one line: `rg -n 'enum LogicalAccess' -A 60 crates/tiler-ir/src/schedule/model.rs`.

**Inference — the shape of the widening, not yet the design.** `LogicalAccess` is `#[non_exhaustive]` under ADR 0074 convention 5a precisely so a new coordinate map lands additively, and no out-of-crate consumer classifies it by exhaustive match. So the additive seam exists; what does not exist is the map itself, its bounds-proof obligation, its write-ownership consequence, and its identity encoding. `crates/tiler-compiler/src/governed.rs`'s `GovernedReindexF32` already emits the *index-region* half for every admitted `ReindexForm`, and `GovernedBroadcastF32` for both many-to-one relations, so the derivation of the coordinate maps exists and is tested — it is the schedule-level vocabulary that is absent.

## Boundaries

- **A copy variant is not the answer, and the admission ticket's non-goals already ruled it out.** Adding a `ScalarProgram` copy would realize a standalone reindex as a materializing copy kernel. What should reach a kernel is a *fused* region where the structural occurrence contributes an access map and an arithmetic neighbour contributes the scalar program.
- Every new `LogicalAccess` variant owes what the existing ones owe: a bounds-proof obligation the region verifier can discharge, write-ownership consequences, an identity encoding, and a total map at every site inside `tiler-ir` that matches it exhaustively.
- The request boundary's refusal stays until the vocabulary lands. `a_family_outside_the_expression_vocabulary_refuses_with_a_typed_reason` in `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` and `perturbing_one_occurrence_out_of_the_vocabulary_refuses_by_name` in `crates/tiler-compiler/tests/composed_family_recognition.rs` are what keep it observed, and both now assert it *beside* an admitted elementary neighbour so the rule is attributable.

## Closes when

A program containing a `Reindex` or a non-scalar `Broadcast` is recognized at the request boundary and reaches a verified scheduled region, its result is bit-compared against the reference evaluator, and the two boundary tests above are updated to assert the new admission beside whatever still refuses.

## Outcome, 2026-08-05

**Both families reach a verified scheduled region and compile through the ordinary path; the schedule identity domain did not step, and the request subject did.** The wall this ticket was filed against — `LogicalAccess` having no coordinate map and no widening broadcast — is gone, and the four assertions that pinned it are flipped with their refusing neighbours preserved.

### What the vocabulary is, and why it is this small

**Fact — both relations are written in one decode per operand axis.** `AxisDecode { divisor, modulus, mirrored }` denotes `(linear / divisor) % modulus`, optionally mirrored to `modulus − 1 − c`. `LogicalAccess::ReindexBijection` and `LogicalAccess::BroadcastReplication` each carry an operand shape, a result shape, and one decode per operand axis.

**Inference — one decode per axis covers all six registered reindex mapping forms**, which is not obvious and is the reason there is no variant per form:

- `PermuteAxes`, `InsertUnitAxis`, `RemoveUnitAxis` read one result axis per operand axis, so the divisor is that axis's suffix product.
- `SplitAxis` replaces one operand axis by a **contiguous** run of result axes, and contiguous row-major axes linearize as one window ending at the run's last axis.
- `MergeAxes` decodes one result coordinate into a run of operand axes, and the two-level decode **collapses**: for `E` divisible by `s·m`, `((linear / R) % E) / s % m == (linear / (R·s)) % m`, because the discarded high part is a multiple of `E/s`, itself a multiple of `m`.
- `ReverseAxis` is the same decode mirrored — the one form needing the flag, and the affine map D-10 admits.

**The admission rules stay separate, because a bijection and a replication license different conclusions.** A reindex's decodes must *tile* the iteration domain — sorted by descending divisor they telescope, so the linear index decomposes uniquely, exactly as a mixed-radix numeral does. A broadcast's must name *distinct whole result axes* and leave at least one uncovered, must not mirror, and must actually widen — so a broadcast that widens nothing is refused rather than admitted as a second spelling of `LinearIdentity`.

### The identity determination, per encoding site

**Fact — appends-only at the schedule layer; `tiler.schedule.v5` deliberately does not step.** `push_logical_access` frames each variant behind its own leading tag byte. `0x01`–`0x05` keep their tags and their field layouts; the two new relations take `0x06` and `0x07`, each followed by two `push_shape`-framed shapes and a `push_len`-framed run of fixed 17-byte decodes. Injective in both directions: every decode field reaches the bytes, so two maps differing in meaning differ here; and nothing but those fields does, so two maps equal in meaning encode identically. A reader reaching `0x06` is reading an access the earlier vocabulary could not express, never an earlier one reinterpreted.

**Fact — the same argument at `tiler.kernel.v6`.** `BinaryOp::IndexSubtract` is appended at tag `0x0c`; `0x01`–`0x0b` keep their meanings and field positions.

**Fact — the request subject *did* step, at two sub-tags, and it was required rather than chosen.** `pointwise-f32.v3` → `v4` and `serial-sum-f32.v2` → `v3`, both inside an unchanged `tiler.compiler.request-subject.v5`. The relation is load-bearing for identity: `a * w` with both inputs at the region's shape and `a * broadcast(w)` widening a smaller `w` encode the same input keys, result shape, expression, and element count — only the access maps separate them. Leaning on the member list instead would be exactly the unstated invariant an identity encoder must not rest on. An append was unavailable because the run sits at each arm's end, so an old subject and a new one carrying no maps would differ only by a trailing framed zero and an old reader would consume the next output's tag as this arm's payload.

**Measurement — every pin, checked on this tree.**

| pin | verdict |
|---|---|
| `STRICT_F32_REGION_IDENTITY_HEX` + `_V4` (`schedule/builder.rs`) | **did not move** |
| `the_staging_relation_step_moves_only_the_domain_separator` | **did not move** |
| 7 × `crates/tiler-metal/goldens/*.metal` (kernel + region digests) | **did not move** |
| `GOVERNED_PROPOSALS` (`frontier.rs`) | **did not move** |
| `ARTIFACT_IDENTITY` / `CACHE_SUBJECT` (`metal_plan.rs`) | **did not move** |
| `explain.rs` `request=` | **moved** `b2d55d5a36e0159b` → `f3244b2242ebcb5c` |

The one moved pin is recomputed from the observed failing value on this branch tree, never copied, with a ledger entry recording both sub-tag steps and the reason. The "did not move" rows are not asserted from the design: `cargo nextest run --workspace` is green at 2717 tests, and those pins are what would have gone red.

### Correctness argument per relation

**Bounds are discharged structurally, not declared.** Every decoded coordinate is `(…) % modulus` where the modulus is required to equal that operand axis's own extent, so it is in range by construction — which is why both relations pair with `BoundsProofKind::LinearRange` over the operand's element count, exactly as `ContractionOperand` does. What separates them is what the *covering* proves: the reindex's telescoping proves every operand element is read exactly once, and the broadcast's distinct-axis rule proves the read is invariant in each replicated coordinate.

**A defect the oracle found.** `lower.rs`'s pointwise arm loaded at the invocation index directly, ignoring `plan.addressing` — correct while every pointwise read was `LinearIdentity`, and precisely the check that keeps passing for the wrong reason once a second relation exists. It returned an unreversed tensor; the reference-oracle comparison caught it. Both the `f32` and `bf16` arms now go through `emit_offset`, which emits the invocation itself for `Identity`, so every dense region's body is byte-identical.

### Watched-failing evidence

Every new admitting and refusing path was perturbed and observed failing:

1. **Appends-only claim** — moved `TAG_LINEAR_IDENTITY` `0x01` → `0x09`: both `the_strict_f32_region_has_its_recorded_canonical_identity` and `the_staging_relation_step_moves_only_the_domain_separator` went red. The real change leaves them green, so the claim is proven in both directions.
2. **Reindex derivation** — forced the permute divisor to `suffix[0]`: refused `structural-relation`, i.e. the bijectivity rule catches a wrong map rather than compiling it.
3. **Mirror emission** — dropped `mirror` from the emitted term: the oracle test returned the unreversed tensor.
4. **Broadcast widening rule** — removed `result_elements <= operand_elements`: initially **nothing failed**, which is a finding rather than a pass. The rule is unreachable from the compile path because `BroadcastAxisMapping` refuses a non-widening mapping at the semantic boundary, and it is still load-bearing because `tiler-ir` verifies regions from any producer. `structural_relation_tests` now covers both predicates directly, and the perturbation is red.
5. **Telescoping loop** — removed it: initially still green, because a two-axis violation always fails the total-window check first. A three-axis case (`[2,2,2]` with two windows claiming divisor `1`) is now covered beside its admitted neighbour, and the perturbation is red.

Items 4 and 5 are the ones worth carrying forward: two rules looked tested and were not, and only running the perturbation showed it.

### Scope

Branch stayed inside `implementation/ir` (`crates/tiler-ir/**`) and `implementation/compiler` (`crates/tiler-compiler/**`) plus its own ticket file under the shared `project/tickets`. No scope was added.

### Public surface — Tom's, not self-accepted

This ticket adds public API to `tiler-ir` and it is landed **as a draft**, not as an accepted boundary:

- `tiler_ir::schedule::AxisDecode` (struct, three public fields, `read`/`fixed`/`is_canonical`)
- `LogicalAccess::ReindexBijection` and `LogicalAccess::BroadcastReplication` (additive under `#[non_exhaustive]`)
- `tiler_ir::schedule::reindex_decodes_are_bijective` and `broadcast_decodes_are_replicating`
- `tiler_ir::kernel::BinaryOp::IndexSubtract` (additive under `#[non_exhaustive]`)

`accept-the-structural-region-access-vocabulary` carries the acceptance node at `awaiting-decision`.

### Not delivered here, and where it went

- **The broadcast's bit comparison against the reference evaluator.** The reindex is bit-compared end-to-end (`a_reindex_reaches_a_kernel_matching_the_reference_evaluator`), but the workload's broadcast occurrence is two-input and the KIR test interpreter binds one input buffer. That is [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md)'s stated deliverable ("Equivalence against the reference evaluator on the compiled result"), and it is now unblocked.
- **A standalone reindex compiles**, which the Boundaries section did not anticipate. It is *not* a `ScalarProgram` copy variant — none was added, and one is still unstatable. What admits it is the pre-existing `PointwiseF32Expression` whose root is its input leaf. Refusing it was considered and rejected: the region is verified, its bounds proof discharged, and its result the reference evaluator's, so a rule rejecting it would be a check with no correctness content. Flagged for review rather than absorbed silently.
- **`frontier.rs`'s `access_domain_shape` answers `None`** for both relations, so a fusion chain over a structural read fails closed rather than matching on a domain it would have to guess.
