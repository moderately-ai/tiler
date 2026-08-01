---
id: realize-the-contraction-through-the-appendable-direct-path
title: Realize the contraction through the appendable direct path
status: done
priority: p1
dependencies: [admit-the-contraction-normative-reference]
related: [realize-the-strict-contraction-on-metal, broaden-governed-physical-support-for-reassociated-programs, bound-the-reference-contraction-comparison-for-the-profile-cells]
scopes: [implementation/compiler, implementation/ir, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, language-model]
---
## User-visible outcome

A contraction of the workload's projection structure compiles through the ordinary entry point and executes bit-identically to the reference — through the `direct` realization, whose every enabling step is an appended tag or a widened check, so the delivery owes no identity-domain step and does not touch the retired synchronization axis.

## Why this ticket exists, and what it is not

**Fact — from `realize-the-strict-contraction-on-metal`'s recorded stop (2026-08-01, commit `cd0a4e7`).** The `tiled` realization the L3 record selects stages tiles through threadgroup memory behind a barrier, the structured-kernel verifier refuses any barrier as `UnexpectedSynchronization` (`crates/tiler-ir/src/kernel/verify.rs:341`), and no schedule can authorize one: the synchronization axis was deliberately retired (`feasibility.rs:93-95`, tag `0x08` reserved), and restoring it inserts a field into the kernel identity — `tiler.kernel.v5→v6` plus a feasibility step. That work is owned by `admit-the-first-typed-synchronization-point-and-atomic-target-authority`, which the tiled ticket now depends on.

**Inference — the recognizer, lowering, and assembly half is realization-independent and every step appends.** The stop report's decomposition, verified against the cited sites: `request.rs:2223`'s `input_count() != 1` widening around verified semantic occurrences (the reassociated-programs precedent); a `NormalizedContraction` beside `NormalizedPointwise`; an eighth `GovernedIndexAccess` capability with a binary `[f32,f32]->[f32]` signature (`governed.rs:206`); `ScalarProgram` tag `0x27`; `LogicalAccess` tag `0x05`; the two-read widening of `verify.rs:403`; single-region assembly. None needs a barrier and none inserts into a repeating record.

**This is not the substitution ADR 0076 forbids.** `direct` is byte-identical to `tiled` at all six workload cells in the L3 record and consumes no numerical permission — the record eliminates realizations that *weaken* the contract to gain speed, and `direct` is the slower kernel, not a weaker contract. The tiled ticket stays open as the performance-selected realization and builds on this one; when it lands, `direct` remains a retained alternative rather than a superseded path, because it is the realization with no synchronization requirement and no K-precondition.

## Required delivery

- The appendable steps above, extended together so every retained alternative covers the exact semantic program.
- `direct`'s precondition is `K >= 1` and nothing else; assert that no K-multiple refusal exists on this path, because a check that cannot fire must not be delivered as if it could.
- No fused multiply-add on the accumulation path, held by the per-statement emission rule (the flags are insufficient — finding 16), verified the way the existing emission tests pin such properties.
- Bit-comparison against the reference evaluator at the profile cells the evaluator admits — `w_decode_kv` and `w_vocab_slice` today — with the retained `result_sha256` values as the drift check, and an explicit boundary statement that the four refused cells are owned by `bound-the-reference-contraction-comparison-for-the-profile-cells`.
- The explain digest will move if a governed capability or scalar key registers; rebaseline with the established comment idiom and verify nothing else moved.

## Non-goals

The tiled realization and anything needing synchronization; structures 2 and 3; the split alternatives; the matrix-instruction route; opaque calls; cost models.

## Closes when

A contraction of the profile compiles through the ordinary entry point and its results are bit-identical to the reference at every admitted profile cell, the emitted module carries no fused multiply-add on the accumulation path, and the boundary of the four unreached cells is stated rather than absorbed.

## Outcome

A two-operand tensor contraction compiles through `compile()` to an emitted Metal entry point, as the `direct` realization: one invocation per output element, folding its own contracted sequence in ascending order from the first product. Recognition, an eighth governed index-access capability, a scheduled region, structured-kernel verification, and single-region program assembly moved together, and every enabling step is an appended tag or a widened check.

### Recognition, against the precedent

`normalize_contraction` in `crates/tiler-compiler/src/request.rs` is a third whole-program strategy beside `normalize_serial_sum` and `normalize_pointwise`, and follows `broaden-governed-physical-support-for-reassociated-programs`'s landed shape: generalized recognition around verified semantic occurrences, with a `NormalizedContraction` of its own rather than a projection into `NormalizedSerialSum`. `select_supported_strategy` now attributes a refusal by the operation key the program actually contains rather than by enumeration order, so a rejected contraction reports the contraction recognizer's reason and a rejected sum reports the sum's.

The precedent's asymmetry is deliberately inverted in one place. That ticket kept its recognizer *narrower* than its physical representation, admitting one three-leaf chain; this one admits **every well-formed binary index structure** the semantic registry validates, because the realization addresses each operand axis by whichever output or contracted coordinate the structure binds it to, so refusing a general structure would be a check with no correctness content. `a_structure_whose_contracted_index_sits_at_different_axes_compiles` compiles `abc,b->ac`, whose summed index sits at axis 1 of one operand and axis 0 of the other — the case a realization keyed on "the last axis of both operands" gets wrong while `td,od->to` still passes. What stays narrow is everything else: exactly two operands, exactly one reachable operation, `f32` throughout, no attribute beyond the index structure. `operand_positions` maps each declared input ordinal to the structure operand supplying it, so a caller whose declaration order differs from its operand order is admitted rather than refused for a spelling.

### Appends only, with the reasoning per tag

Four tags were appended and no field was inserted into any repeating record. `STRICT_F32_REGION_IDENTITY_HEX` (`crates/tiler-ir/src/schedule/builder.rs`) and every artifact envelope and sidecar golden are unmoved, which is the check that the claim holds rather than the argument for it.

- **`ScalarProgram::StrictTensorContraction`, `0x27`** — after RMS's `0x26`, as the brief derived. It carries the contracted iteration shape, the contributor order, and the canonical NaN payload, and deliberately **no empty-domain identity**: the registered family declares `refused-an-unseeded-fold-has-no-empty-result`, so a field carrying one would be a value that can never be correct.
- **`LogicalAccess::ContractionOperand`, `0x05`** — after `0x04`, as the brief derived. It is not a `ReductionContributor` with a wider shape: a reduction's output is its input's shape with the reduced axes removed, and neither contraction operand stands in that relation to the output, so the equality the reduction bounds proof checks has no analogue.
- **`ReductionTopology::Contraction`, `0x34`** — **not in the brief's list, and the brief's derivation is corrected rather than followed.** `ReductionTopology::Serial` carries `axes: Vec<Axis>`, which names axes of *one* read tensor; a contraction's summed index generally sits at a different axis of each operand (`abc,b->ac` again), so one `Vec<Axis>` cannot state it and reusing `Serial` would give one field two meanings while leaving the general structure unstatable. The new variant carries the contracted shape instead. It is an append on the same argument as `0x33`: `None`, `Serial`, and `MultiPass` keep their tags and their field positions.
- **The two-read widening of `crates/tiler-ir/src/kernel/verify.rs:403` happened in substance and not at that line.** With a topology of its own, the widening is a new `verify_reduction` arm that admits exactly two reads and takes its contributor count from the contracted shape, rather than a relaxation of the `[read]` destructure at 403 — which still holds for the single-read families and is stronger there for staying exact. The arm calls the *same* `verify_contributor_loop`, so the `start == 1` first-product seed is shared and not re-stated.

`BoundsProofKind` gained nothing. A contraction operand's proven domain is the contiguous linear range of its own elements, which is what `LinearRange` already means and what it already serves for `ScalarBroadcast` and `PackedU4LsbZeroTail`; which of those positions the access touches is the map's statement, and `verify_contraction` proves every derived coordinate is in range by requiring per-axis extent agreement.

### Emission: no fused multiply-add, held per statement

`crates/tiler-metal/src/emit.rs` was **not changed at all**, which is the result rather than an omission: the fold is already three separate structured operations per step — a multiply, a NaN canonicalization, and an add — and the emitter writes every structured operation as one statement over already-named locals. `the_contraction_kernel_emits_no_fused_multiply_add_on_its_accumulation_path` pins that in the text: no `fma(`, `mad(`, `simdgroup`, or `multiply_accumulate` token; no single `float` statement carrying both a `*` and a `+`; exactly two products and one accumulation among the float statements; and each product and sum committing the canonical payload. The flag obligation is still recorded, as a second line of defence rather than as the guarantee — which is the point, because the L3 probe measured `-ffp-contract=off` failing to reach a fused instruction the source asked for.

The canonicalization between the multiply and the add is what makes the pair unfusable even in principle: the backend sees a helper call between them. That placement is the declared `after-every-combine-and-at-the-result-boundary` rule, not a defence bolted on.

**No result-boundary conversion is emitted, and its absence is derived.** The serial sum needs one when its contributor sequence is a singleton, because its seed is a raw load. A contraction's seed is a *product*, which this emission canonicalizes, so every path out of the fold already carries the canonical payload and a second conversion would be a provable identity in a body the refinement gate compares structurally.

### `K >= 1` and nothing else

`no_k_multiple_refusal_exists_on_the_direct_path` compiles contracted extents 1, 2, 3, 5, and 7 — every one a tile or split width would reject — and asserts each succeeds. `tiled` refuses `K` not a multiple of sixteen and the splits refuse `K` not a multiple of their width; `direct` has neither, so a K refusal here would be a check that can never fire, shipped as if it could.

The empty contracted domain is not a second precondition. The registered family refuses it at construction, so `an_empty_contracted_domain_is_refused_before_a_request_exists` asserts the refusal at `build()`; the recognizer and the schedule verifier each carry the same check as a stated precondition a reader can find locally rather than infer from an inferencer three crates away.

### The two-cell comparison, with sha256 confirmation

`governed::contraction_conformance` reconstructs the L3 probe's own SplitMix64 operands — `WORKLOAD_SEED = 0x5445524D`, the right operand at `seed ^ 0xA5A5A5A5A5A5A5A5` — so every digest is computed over the same bytes the device consumed.

- **`w_decode_kv`** (1×1024×1024): the reference evaluator's result digest is `79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f`, the retained value. The **emitted index region**, executed by `tiler-reference`'s independent index-region oracle, equals that result element by element, and its own digest is the same — so the region Tiler emits reproduces a measured device result bit for bit at a whole profile cell.
- **`w_vocab_slice`** (1×8192×1024): the reference's digest is `88b01ae776f42bdb2f2d1092ddfd039e20e652d28393a6e2ec19e5cc1d9803c8`, the retained value. The emitted region is **not** compared here, and the reason is `tiler-reference`'s own `MAX_EVALUATION_STEPS` bound of 16,777,216, which this cell's 8,388,608 contracted points exceed. `the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget` asserts that refusal rather than routing around it: raising the budget belongs to `implementation/reference`, which this work does not own.

`sha2` is a workspace dependency, but adding it here would edit `Cargo.lock`, which this work does not own either; the digest is therefore computed by a local FIPS 180-4 implementation checked against the published empty-string and `"abc"` vectors before any comparison rests on it.

### The four refused cells — boundary statement

`w_prefill_q` (20,971,520 fold steps), `w_prefill_mlp_in` and `w_prefill_mlp_out` (402,653,184), and `w_prefill_o` (268,435,456) all exceed `MAX_REFERENCE_TENSOR_ELEMENTS`, so `contract_operands` refuses them under `IterationStepsExceeded`. No operand or output tensor exceeds a limit; only the fold's step count does. **That boundary belongs to `bound-the-reference-contraction-comparison-for-the-profile-cells` and was deliberately not settled here** — raising the bound, staging the comparison in slabs, and restating the deliverable as the retained digests are three decisions with different costs. `the_four_prefill_cells_are_refused_by_the_references_work_bound` asserts each refusal, so "four cells are uncompared" is a checked fact with a named reason rather than an omission a reader has to notice.

### Digest and pin movements

The explain request digest moved from `b8ffa37f3d2dc86b` to `4d9f4773575b6679`, rebaselined with the established comment idiom. Only the *compiler* half of the request subject moves — the exact inverse of the step that first registered the contraction family: the semantic snapshot already admitted it and did not move again, and the lowering-registry identity now covers one further index-access capability. The governed scalar registry gained no key, because the emission reaches `multiply-f32` and `add-f32`, both already registered.

Nothing else moved, and that was checked rather than assumed: `STRICT_F32_REGION_IDENTITY_HEX`, every selected and materialized artifact envelope and sidecar golden, the target-profile descriptor, and the Metal golden fixtures are all unchanged, which the full workspace suite proves by passing without a second rebaseline.

### R5 is separately unmet, and a ticket owns it

`FusionNumericalCapabilities::governed` registers no `FusionOperationRole` for the contraction, so a cover region holding a contraction *and* another operation still fails closed to `Unknown`. The whole-program shape never asks — `derive_fusion_legality` is skipped for a region with fewer than two members, and the recognized contraction shape is exactly one operation — so registering a role here would have been a declaration with no reachable consumer and no evidence, the mirror image of the K refusal this ticket refused to ship. `admit-a-fusion-role-for-the-tensor-contraction` was filed and the support matrix records the rung as R6 with R5's criterion explicitly skipped rather than met.

### Public items, for review

None is self-accepted. In `tiler-ir`:

- `tiler_ir::schedule::ContractionAxisSource`, a new public enum with variants `Output { position: u32 }` and `Contracted { position: u32 }`. Deliberately **not** `#[non_exhaustive]`, under ADR 0074 convention 5b: the identity encoder and `tiler-compiler`'s region construction and subject binding map it totally.
- `ScalarProgram::StrictTensorContraction { contracted_shape, order, canonical_nan_bits }` — a variant of a non-`#[non_exhaustive]` enum, so it is a build error at every out-of-crate total map.
- `LogicalAccess::ContractionOperand { operand_shape, output_shape, contracted_shape, sources, order }` — a variant of a `#[non_exhaustive]` enum.
- `ReductionTopology::Contraction { contracted_shape, order, permits_reassociation, permits_permutation }` — a variant of a `#[non_exhaustive]` enum.

No public item was added to `tiler-compiler` or `tiler-metal`. Inside `tiler-compiler` the crate-private additions are `NormalizedContraction`, `NormalizedProgram::Contraction`, `NormalizedProgramSubject::Contraction` (both boxed), `NormalizedProgram::{contraction, input_elements_at, max_input_elements}`, `VerifiedTargetRequest::contraction`, `physical::{contraction_region, contraction_operand_sources, contraction_accesses_match}`, and the eighth `GovernedIndexAccess` row. `NormalizedProgram::input_elements` was **removed** and replaced by the two accessors above, because a contraction's two operands have different extents and the removed method's single answer would have sized an opaque call against the wrong tensor — the exact hazard `frontier.rs`'s own comment named as needing a per-ordinal resolution.

### Watched failures

Every new check was run against a case that must fail, and each produced the failure it was supposed to. Sources were restored between perturbations and the final tree is unperturbed.

1. **The FMA token check** — `emit_binary` perturbed to emit `fma({lhs}, 1.0f, {rhs})` for `+`: `fma( must not appear on a path whose contract forbids contraction`.
2. **The one-statement check** — the same site perturbed to emit `{lhs} + {rhs} * 1.0f`: `one statement carries both operators, so the pair is fusable: float v31 = v16 + v30 * 1.0f;`.
3. **The contracted-axis source derivation** — `contraction_operand_sources` perturbed to map a contracted index to `Output { position: 0 }`: every compiling case became `InvalidCompilerOutput`.
4. **The operand ordinal binding** — `contraction_region` perturbed to bind both reads to `Input { ordinal: 0 }`: every compiling case became `InvalidCompilerOutput`.
5. **The contributor-loop seed** — `emit_contraction`'s loop perturbed to `start: 0`: `bounded contraction fixture lowers: Verification(ReductionContract)`, which is the `start == 1` first-product obligation firing.
6. **The subject binding's canonical NaN** — `contraction_region` perturbed to a non-contract payload `0x7fc01234`: every compiling case became `InvalidCompilerOutput`.
7. **The retained-digest reconstruction** — the SplitMix64 increment perturbed from `…DD1D` to `…DD1F`, which is the spike's own perturbation 2: `w_decode_kv: the reference does not reproduce the retained direct result`, observed `c3afeba1f6f8df58ed2f91534e95104d0166d32ae743736922b143a3cfa8173b`.

`a_single_perturbed_contributor_breaks_every_comparison` is the eighth, retained as a permanent test rather than run by hand: advancing the last contributing element by one representable value changes the reference's result, the emitted region's result, and the digest. The *last* element is deliberate — a fold that stopped early, or one seeded at `+0.0` and therefore ignoring its first contributor, would still be caught by a first-element perturbation, so the last position is the one that discriminates the fold's completeness.

Two checks are **reservations rather than tested guarantees**, and are recorded as such: the recognizer's `contraction-operands` arm (an operand that is not a declared input is unconstructible for a one-operation program) and `verify_contraction`'s "an output coordinate no operand reads" arm (ADR 0087's first rule already refuses that structure at construction).

### Verification run

`make full` green: `cargo fmt --all --check`; `cargo check --workspace --all-targets --locked`; workspace Clippy with `-D warnings`; `cargo nextest run --workspace --locked` at 2,087 tests run, 2,087 passed, 5 skipped; `cargo test --workspace --doc --locked`; warning-denied rustdoc; `cargo nextest run --release --locked -p tiler-reference -p tiler-compiler` at 714 run, 714 passed, 1 skipped; `ticketsplease lint` `ok: no problems found`; shellcheck. `git diff --check` clean.

Three clippy findings were fixed rather than allowed: `emit_reduction`'s eighth argument became a `(buffer, bounds)` pair matching the contraction's own shape; the two whole-program subjects' fail-closed arms merged into one pattern; and both contraction enum variants are boxed, because a contraction's payload is roughly twice the serial sum's and every value of those enums would otherwise pay for the widest variant.
