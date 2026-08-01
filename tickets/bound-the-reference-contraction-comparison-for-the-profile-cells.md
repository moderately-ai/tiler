---
id: bound-the-reference-contraction-comparison-for-the-profile-cells
title: Bound the reference contraction comparison for the profile cells
status: in-progress
priority: p1
dependencies: []
related: [realize-the-strict-contraction-on-metal, realize-the-contraction-through-the-appendable-direct-path, bound-the-reference-contraction-iteration-space]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, contraction, language-model]
claimed_from: todo
assignee: worker-oracle-bound
lease_expires_at: 1785589250
---
## User-visible outcome

The reference evaluator can serve as the bit-exact oracle for every one of the L3 profile's six contraction correctness cells, or the comparison procedure for the cells it refuses is stated and owned — so no realization ticket has to quietly shrink its evidence to the cells that happen to fit.

## The blocker, exactly

**Fact — from `realize-the-strict-contraction-on-metal`'s recorded stop (2026-08-01).** `MAX_REFERENCE_TENSOR_ELEMENTS = 16 * 1024 * 1024` (`crates/tiler-reference/src/lib.rs:90`) bounds `output_count * contracted_count` in the contraction evaluator (`contraction.rs:450-456`). Four of the six cells exceed it: `w_prefill_q` at 20,971,520 (1.2×), `w_prefill_o` at 268,435,456 (16×), `w_prefill_mlp_in` and `w_prefill_mlp_out` at 402,653,184 (24×). Only `w_decode_kv` and `w_vocab_slice` fit.

## What this must decide, with the elimination stated

Whether the bound moves or the comparison stages. Raising the bound must derive the memory and time cost at the largest cell rather than picking a bigger number; the existing `IterationStepsExceeded { limit, actual }` refusal and the bound's own rationale are the authorities to read first — the bound exists so a malformed program cannot ask the host for an unbounded fold, and a raise that discards that protection is not an option. A staged comparison — evaluating the contraction in output slabs the bound admits and comparing slab digests — keeps the bound and changes the procedure; it must state why slab boundaries cannot change any folded value (each output element's fold is independent, which is a property of the registered signature to cite, not to assume). A third candidate, comparing only the two admitted cells and calling the four others covered by the retained L3 `result_sha256` values, converts a live oracle into a frozen golden and must say so explicitly if chosen.

## Closes when

Every profile cell has a stated, executable comparison route — through the evaluator directly, through a staged procedure whose independence argument is written where the procedure lives, or through an explicitly-frozen golden with its drift boundary named — and the choice's elimination is recorded in the ticket outcome.

## Outcome

**The comparison stages. The bound does not move by one step.** All six L3 cells now reproduce their retained `direct` `result_sha256` through the reference's own fold, and the whole-program refusal that protected the host is byte-for-byte the one that was there before.

### The elimination

**Candidate A — raise the bound.** Eliminated, and the derivation is what eliminates it rather than the size of the number. The first thing reading the site establishes is that the bound is *not* a memory bound: the fold's storage is `output_count` elements whatever the step count, already bounded by `preflight_f32_output`, so the largest cell's 402,653,184 steps retain 393,216 elements and no raise buys memory headroom because none was needed. What the bound protects is host *time*. `MAX_REFERENCE_TENSOR_ELEMENTS` is also the stored-element bound at `Tensor::dense`, `preflight_f32_output`, and eight other sites, so raising *it* would raise the storage limit for the whole crate; raising only the fold means introducing a second, larger constant. Either way, every caller's ceiling moves — including `ReferenceEvaluator::evaluate` on an arbitrary verified program, which is the exact caller the bound exists for and which cannot be distinguished from a profile-cell test by a constant. Spending a 24× weakening of the whole crate's protection to serve four test cells is the shape AGENTS.md names: the cheaper option's saved cost *is* the part that made it correct.

**Candidate C — freeze the four cells on the retained digests.** Eliminated because it answers a different question. The retained `result_sha256` is a measurement of an M4 Max, not of the reference; freezing means the four cells stop testing the reference at all, so a moved signature field, a changed canonicalization site, or a reordered contributor sequence would go unnoticed there. It also cannot localize: a digest says "different", never which element. Retained as the fallback had B failed; B did not fail.

**Candidate B — stage the comparison in output slabs. Survives.** Correctness rests on an independence property that is a fact about the registered signature, cited below and executed by a test. Performance is measured, not assumed. Maintainability is the strongest part: **no constant changes**, so nothing about the whole-program path's protection has to be re-derived by a future reader — each staged call passes exactly the test the unstaged call passes, and the larger total is a `for` loop in caller code rather than a number in `lib.rs`.

### The independence citation, from the registered signature

Read out of `strict_tensor_contraction_f32_facts()` rather than from the shape of the loop. Written where the procedure lives, in `StagedStrictTensorContractionF32`'s rustdoc.

- **`CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE`** (field 5) declares `ascending-lexicographic-over-the-canonically-ordered-contracted-index-space`. The fold's domain is therefore the *contracted* index space alone. `ContractionIndexStructure`'s own derivation — "an index that appears in the operands and not in the output is summed over; every other operand index is free and must be a member of the output exactly once" (`crates/tiler-ir/src/semantic/contraction.rs`) — makes contracted and output disjoint sets. So no output coordinate is a member of any contributor sequence, and no output element appears in another's.
- **`CONTRACTION_F32_FACT_SEED`** (field 6) declares `none-the-accumulator-starts-at-the-first-product`. Each fold's initial value is its own first product, so no accumulator state crosses output elements. The conclusion does not depend on which seed is declared: the alternative the contract distinguishes is an explicit *constant* initial, not a carried value.
- **`CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED`** and **`..._PERMUTATION_PERMITTED`** (fields 8, 9) are both `false`, and slabbing exercises neither — it regroups nothing and reorders nothing *within* a contributor sequence. What it reorders is the traversal of whole output elements, and none of the fourteen fields declares an output traversal order, so there is no term for one to violate.
- **`CONTRACTION_F32_FACT_DETERMINISM`** (field 14) declares `plan-deterministic`: the value is a function of the declared plan, and a slab width is not a plan term.

The mechanical statement matching that reading — `evaluate_outputs` reads only immutable operand elements, writes only the result it returns, and re-seeds the accumulator inside the output loop — is recorded as the code that was *checked against* the signature, not as the argument.

### What was built

`ContractionFold` (crate-private) now holds one contraction's validated fold: elements, axis readers, strides, the output and contracted shapes and counts. Nothing in `plan()` is proportional to the step count, which is what lets a caller learn a fold's cost without paying it. Two callers walk it and **neither carries its own arithmetic** — a staged evaluator with a second fold would agree with the registered one for reasons that say nothing about either being right.

- `contract_operands` (the registered operation, reached by `ReferenceEvaluator`) plans, refuses above `MAX_REFERENCE_TENSOR_ELEMENTS` under the unchanged `IterationStepsExceeded`, then folds the whole result.
- `StagedStrictTensorContractionF32` plans, derives the widest slab the same bound admits (`MAX / contracted_count`), and folds one slab per `evaluate_slab` call.

The residual refusal is `slab_output_count == 0`, reached either by a caller asking for a zero-wide slab or by that division reaching zero — which needs a contracted space larger than the bound, meaning one output element's fold is already over. **That second path is unreachable and recorded as a reservation rather than a tested guarantee**: ADR 0087's rule two (`SummedIndexInOneOperand`) puts every contracted index in *both* operands, so `contracted_count` divides each operand's element count, which `Tensor::dense` already bounds by the same constant. Every well-formed operand pair therefore admits a slab of at least one output element. Refused rather than assumed away, on the precedent the `AxisReader` arm beside it already sets.

### Executed-cell evidence

`crates/tiler-reference/tests/contraction_profile_cells.rs`. Operands reconstructed from the probe's own `SplitMix64` stream (`spikes/scheduling/metal_contraction_vertical/host.m`), digests transcribed from the `direct` rows of the retained `workload.tsv`, SHA-256 written out locally and checked against both published FIPS 180-4 vectors before anything rests on it (`sha2` would edit `Cargo.lock`, which this work does not own).

**Measurement — Apple M4 Max, 2026-08-01, nightly-2026-07-19, dev profile.** All six cells reproduce their retained digest; total 10.78 s, 484 MB peak resident set (`/usr/bin/time -l`).

| cell | steps | slabs × outputs | wall clock | verdict |
| --- | --- | --- | --- | --- |
| `w_decode_kv` | 1,048,576 | 1 × 16,384 | 10 ms | `79810ce4…` matches |
| `w_prefill_q` | 20,971,520 | 2 × 16,384 | 198 ms | `1c54f5cd…` matches |
| `w_prefill_mlp_in` | 402,653,184 | 24 × 16,384 | 3,799 ms | `eb382840…` matches |
| `w_prefill_mlp_out` | 402,653,184 | 25 × 5,461 | 3,796 ms | `124571de…` matches |
| `w_prefill_o` | 268,435,456 | 16 × 8,192 | 2,525 ms | `b99eff90…` matches |
| `w_vocab_slice` | 8,388,608 | 1 × 16,384 | 79 ms | `88b01ae7…` matches |

Release profile, same run with `--release`: 5.53 s total, 4 ns per step against the dev profile's 9. A cost of this shape is a measurement of this host and this profile, not a rate any other fold inherits.

**What runs by default.** `the_staged_oracle_reaches_the_cheapest_refused_cell` folds `w_decode_kv` and `w_prefill_q` on every run, in 0.31 s, and asserts on the *same operands* that the unstaged evaluator still refuses with `iteration space has 20971520 steps, exceeding 16777216`. The whole-profile test is `#[ignore]`d for its 10.8 s, with its invocation recorded in the module documentation:

```text
cargo nextest run -p tiler-reference --run-ignored only --no-capture \
    -E 'binary(contraction_profile_cells)'
```

The `#[ignore]` costs less drift detection than it looks like: every helper is shared with the default test, and all six cells share one fold, so an arithmetic change that moved four of them would move the two the gate runs. What goes unchecked between deliberate runs is exactly the four extra retained digests.

### The protection, watched refusing

Four perturbations, each run against a case that must fail, each reverted; `git diff --stat` confirms the final tree carries none of them.

1. **The retained-digest comparison** — the `SplitMix64` increment perturbed from `…DD1D` to `…DD1F` (the spike's own perturbation 2): `w_decode_kv: the staged reference does not reproduce the retained `direct` result`, observed `c3afeba1…`. With the control comparison temporarily removed so the refused cell reports for itself: `w_prefill_q: … does not reproduce`, observed `e2b17898…`.
2. **The slab-boundary independence** — `evaluate_outputs` perturbed to start the contracted loop at 1 whenever `first_output != 0`, which makes a slab boundary observable. `slab_boundaries_do_not_change_any_folded_value` fails at `a slab width of 1 changed a folded value`, and `w_prefill_q`'s digest moves to `0c171c9e…` while `w_decode_kv` — a single slab — still passes. That discrimination is the point: only the multi-slab cell notices.
3. **The staged work bound** — `admitted`'s refusal disabled: `a_slab_wider_than_the_work_bound_admits_is_refused` fails with `called Result::unwrap_err() on an Ok value: StagedStrictTensorContractionF32 { … slab_output_count: 16385, slab_count: 1 }`.
4. **The surviving whole-program protection** — `contract_operands`'s bound disabled: the default test's `expect_err("the unstaged fold exceeds the reference's work bound")` panics on an `Ok`, and the pre-existing `an_iteration_space_over_the_bound_is_refused_as_iteration_work` fails with `left: Ok(Tensor(… Shape([Extent(2896), Extent(2897)]) …))` — confirming the refactor moved that site without weakening it.

`slab_boundaries_do_not_change_any_folded_value` is the permanent form of the independence check: five partitions (widths 1, 3, 5, 11, and one wider than the whole result) and the unstaged evaluator all produce identical bit patterns, then the same five agree again after one contributing element is advanced by a single representable value. Without the perturbation the equalities would hold for a constant result or for empty slabs.

### Refusal-ordering note

`contract_operands` now resolves axis readers and strides during `plan()`, before the work-bound check rather than after it. No reachable refusal reorders: the reader arm is invalid state for a validated structure, and `row_major_strides` can only fail on extents whose products already fit `usize` because the operand tensors exist. Checked by reading each failure path, and the existing bound regression's exact-error assertions still hold.

### Public items, for review — none self-accepted

- `tiler_reference::StagedStrictTensorContractionF32<'operands>` — a new public struct with `governed`, `governed_with_slab_output_count`, `output_shape`, `output_count`, `contracted_count`, `slab_output_count`, `slab_count`, `evaluate_slab`, and a hand-written `Debug` that renders the plan and never the borrowed operands. No `evaluate()` convenience that folds every slab in one call is offered, deliberately: the caller's loop *is* the authorization, and a single call that walked an arbitrary total would put back the unbounded ask the bound exists to prevent.
- `tiler_reference::StagedContractionError` — `#[non_exhaustive]`, two variants: `UnsupportedDeclaration(UnsupportedContractionDeclaration)` and `Operation(ReferenceOperationError)`. Two error causes rather than one because a moved contract and a mismatched pair of tensors are repaired differently.

No existing public signature changed, and no constant moved.

### Follow-ups filed rather than absorbed

1. **[`route-the-contraction-conformance-through-the-staged-oracle`](route-the-contraction-conformance-through-the-staged-oracle.md)** — `crates/tiler-compiler/src/governed/contraction_conformance.rs` is stale in one section. Its "The four cells nothing here compares, and why" paragraph names this ticket as the owner of an unsettled boundary; the boundary is settled and its four cells are reachable. Routing them through `StagedStrictTensorContractionF32` — and deciding whether that crate wants a 10 s comparison — is `implementation/compiler` work this ticket does not own.
2. **[`decide-the-index-region-oracle-route-past-its-step-budget`](decide-the-index-region-oracle-route-past-its-step-budget.md)** — the index-region oracle's budget is a different bound and is *not* unblocked. `the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget` cites `MAX_EVALUATION_STEPS` (`crates/tiler-reference/src/oracle.rs:49`), a running per-region counter over scalar, reducer-body, and index-expression evaluations — not a product computable before the walk. Staging it needs a partition of the emitted region's own output domain and a fresh independence argument, which is a separate design. Recorded so nobody reads this outcome as having reached the emitted-region comparison at `w_vocab_slice`; it has not.
