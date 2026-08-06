---
id: apply-the-declared-numerical-conformance-on-every-reference-evaluation-path
title: Apply the declared numerical conformance on every reference evaluation path
status: review
priority: p2
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies, correct-the-silu-subnormal-fact-that-covers-only-the-negative-tail, carry-a-bf16-subnormal-realization-the-reference-can-be-told]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, conformance]
claimed_from: todo
assignee: agent-conformance-thread
lease_expires_at: 1785989597
---
## User-visible outcome

Every reference path that answers for a compiled candidate honours the two subnormal dimensions the candidate's contract resolved, instead of one of three paths honouring them and the other two silently computing the strict reading.

## Why this exists, and why it bites today

**Fact — the declared conformance is applied at three sites in the workspace, all in one module.** `grep -rn 'apply_to_operand\|apply_to_result' crates/ --include='*.rs'` returns fourteen lines at `4cf593e7`: eleven inside `crates/tiler-reference/src/conformance.rs` itself (the two method definitions and nine of its own test assertions) and `crates/tiler-reference/src/oracle.rs:754`, `:755`, `:761`. No other file matches.

**Fact — the semantic evaluator was never told a contract.** `grep -c 'ReferenceNumericalConformance' crates/tiler-reference/src/evaluate.rs` returns `0`. `ReferenceEvaluator::evaluate` computes the strict reading unconditionally, so `ReferenceNumericalConformance::from_realization`'s refusal — which exists precisely so an oracle cannot silently answer a question it was not asked — cannot fire on that path at all.

**Fact — the declared-order reduction oracle takes no conformance either.** `strict_partial_sums` and `strict_partitioned_sum` (`crates/tiler-reference/src/evaluate.rs:484`, `:615`) have signature `(input, axes, partitions, contributors_per_partition)` and fold with a bare host `+` through `canonicalize_arithmetic_f32`.

**Inference — so under `FLUSH_AND_REASSOCIATE_F32` the oracle in use discharges the reassociation dimension correctly and drops the two subnormal dimensions the same contract resolves to a sign-preserving flush.** That contract flushes on both dimensions; the oracle preserves. It fails closed rather than open — a preserving reference disagrees with a flushing device — so the risk is a correct implementation refused and the disagreement misattributed to the grouping.

**Fact — the obligation is already written down, as prose in a test header rather than as an object.** `crates/tiler-reference/tests/contraction_conformance.rs:44-46` states that "A device comparison against this oracle is a comparison against the strict reading, and the flushing dimension has to be declared on the comparison rather than absorbed here." Nothing in the tree is that declaration.

**Fact — it is invisible in every case that exists.** The M4 Max row's operands (`0x3f400000, 0x3e800000, 0x33400000, 0x33000000`) and the CPU-side `REGROUPING_SENSITIVE_INPUT` scaled by `2x + 1` are all normal, so the preserving and flushing readings agree on every value any current case produces.

## What this ticket must produce

- Every reference path that can be asked about a compiled candidate either carries a `ReferenceNumericalConformance` or documents, at the definition, why its subject cannot be affected by either subnormal dimension. Answering by omission is what this ticket exists to end.
- A subnormal-producing case at the reduction shape, which is the population that proves the change can fail: a partial sum that is subnormal under one declared split and not under another, with the exact bit patterns written out.
- The check watched failing before it is believed — revert the threading and confirm the new case refuses.

## Explicit non-goals

Widening `from_realization`'s acceptance (its refusal is correct and [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) says why); any order-witness object; any new dimension; any device run.

## Closes when

No reference path silently answers a contract it was not told, the subnormal-sensitive reduction case exists with exact bits, and the new refusal has been observed.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md), which found the three-site count while deriving what object bounds a program under a permissive contract.

## Outcome — 2026-08-05

**Answering by omission ends because the contract now reaches every capability by construction rather than by discipline.** An evaluator carries one `ReferenceNumericalConformance` and hands it to every registered capability through the request; each capability then either applies it at its arithmetic sites or states at its own definition why neither dimension has a site there. The threading is behaviour-preserving at the strict reading — the whole 282-test reference suite passes unchanged — and the two dimensions become separately observable the moment a caller states another contract.

**Fact — the boundary that decides every row is one sentence, and this crate had already written it.** The two dimensions are functions on an *arithmetic operand* and on a *newly produced arithmetic result*. That is the boundary `canonicalize_arithmetic_f32` is already drawn at ("It applies to an *arithmetic result*: a value that is only read, or an exact constant payload, keeps its bits"), so a family that transports, selects, or reproduces a bit pattern reaches neither site — and applying a flush there would model a device flushing a load rather than an arithmetic unit.

### Per-path decision table, exhaustive over what the crate registers and exports

| Path | Verdict | Why |
| --- | --- | --- |
| `ReferenceEvaluator` | **carries** — `under`, `conformance()`, threaded into every `ReferenceEvaluationRequest` | the gap the ticket named: `grep -c 'ReferenceNumericalConformance' crates/tiler-reference/src/evaluate.rs` returned `0` and now returns a nonzero count |
| `tiler::multiply-f32@1`, `add-f32@1` | **applies** (`binary`) | elementary binary32 arithmetic; the tensor-level twin of the already-threaded scalar binary |
| `tiler::strict-serial-sum-f32@1` | **applies** (`strict_sum`) | each contributor and the accumulator are operands, each sum a produced result |
| `tiler::strict-tensor-contraction-f32@1` | **applies** | each factor, the product, and every accumulation |
| `tiler::silu-f32@1` | **applies** | its declared fact does *not* cover the case that reaches the dimensions — see below |
| `tiler::rms-norm-f32@1` | **applies** | `RMS_NORM_F32_FACT_SUBNORMALS` records the input-flush divergence at the squaring |
| `tiler::softmax-f32@1` | **applies** | `SOFTMAX_F32_FACT_SUBNORMALS` records the subnormal exponential `0x00b33687` |
| `quantize-strict-affine` | **applies** | `value / scale` at a subnormal `value` and a minimal normal `scale` crosses a whole code step |
| `strict_partial_sums` / `strict_partitioned_sum` | **carries** — `_under` entries added, the existing two are those at the strict reading | the second gap the ticket named |
| `StagedStrictTensorContractionF32` | **carries** — `under(conformance)` | the staged fold is the same arithmetic as the registered one |
| `silu_f32`, `certified_exp_f32`, `certified_rsqrt_f32`, `rms_norm_f32`, `softmax_f32` | **carry** — `_under` entries added | the direct entries are the same functions the registered evaluators call, not second copies |
| `tiler::constant-f32@1`, `tiler.scalar::constant-f32@1` | **documented immune** | no operands, and the result is a declared payload rather than a produced value; flushing it would stop the region materializing a subnormal pattern the definition promises verbatim |
| `tiler::reindex/broadcast/slice/concatenate-f32@1` | **documented immune** | elements are cloned byte for byte; no operation, so a transported subnormal is neither an operand nor a result |
| `tiler.scalar::canonicalize-nan-f32@1` | **documented immune** | a conversion; the scalar counterpart of a reduction committing a lone contributor without an addition |
| `assemble-strict-affine`, `dequantize-strict-affine`, `tiler.scalar::…-u4-dequantize` | **documented immune, and the type system is what discharges it** | the value contract's scale domain is `positive-normal-f32` and the code difference is an exact integer of magnitude zero or at least one, so `\|difference * scale\| >= scale >= 2^-126`; no operand and no product can be subnormal |
| `tiler::constant-bf16@1`, `multiply-bf16@1`, `add-bf16@1` | **documented out of reach, with the gap filed** | the object's two dimensions are binary32 functions and this family's arithmetic is exact-rational over BF16's value set; its own `BF16_FACT_SUBNORMALS` declares preservation. → `carry-a-bf16-subnormal-realization-the-reference-can-be-told`, `deferred` |
| `decide_predicate` / `decide_contract` | **documented immune** | their subject is step three of ADR 0042's composition, defined as the value *before* the result-subnormal mapping; absorbing the modes would answer for step four under a step-three metric |
| `IndexRegionEvaluator` scalar binary | unchanged — already applied | the three pre-existing sites |

**Fact — the one path the derivation's Part 1 table did not reach, checked and found immune rather than assumed.** The index-region oracle's strict-affine dequantize performs a multiply and applies no conformance, which looked like a fourth silent site. Reading `read_scale_value`'s contract shows the subnormal range is excluded from the scale domain precisely so "the decode's multiply cannot produce a subnormal", so both dimensions are vacuous there. It is now documented rather than left as a coincidence a reader must re-derive.

### Measurement — the subnormal case at the reduction shape, exact bits

Two rows at `[1, 4]`, each driven under all four resolutions of the two dimensions. Both are in `a_declared_split_and_the_declared_subnormal_modes_are_answered_independently`.

**Row `normal-cancelling`, contributors `0x00800001, 0x80800000, 0x00800001, 0x80800000`** — all four operands **normal**:

| input / result | 2×2 partials | 2×2 total | 4×1 partials | 4×1 total |
| --- | --- | --- | --- | --- |
| Preserve / Preserve | `0x00000001, 0x00000001` | `0x00000002` | `0x00800001, 0x80800000, 0x00800001, 0x80800000` | `0x00000002` |
| Flush / Preserve | `0x00000001, 0x00000001` | `0x00000000` | unchanged | `0x00000001` |
| Preserve / Flush | `0x00000000, 0x00000000` | `0x00000000` | unchanged | `0x00000000` |
| Flush / Flush | `0x00000000, 0x00000000` | `0x00000000` | unchanged | `0x00000000` |

This is the ticket's requested population exactly: `0x00800001` is `2^-126 (1 + 2^-23)` and `0x80800000` is `-2^-126`, so the two-by-two split's partial is their exact Sterbenz difference `2^-149` — **subnormal under that declared split** — while the four-by-one split performs no addition in its first pass and its partials are the four normal operands, **not subnormal under that one**. One shape, one operand set, two declared splits, opposite answers.

**Row `subnormal-operands`, contributors `0x00400000` ×4** runs it backwards: the four-by-one partials are the subnormal operands and the two-by-two partial `0x00800000` is normal. It also separates the dimensions in the opposite direction — here the input dimension zeroes the two-by-two partials (`0x00000000`) and the result dimension leaves them alone (`0x00800000`), where row one is the reverse.

The whole-program path has its own case, `the_semantic_evaluator_applies_each_declared_subnormal_dimension_it_is_told`: `0x00000001 * 1.0` is decided by the input dimension and `0x00800000 * 0.5` — two **normal** operands, subnormal product `0x00400000` — only by the result dimension, with an overflow, a signed zero, and an invalid-infinity NaN invariant across all four columns so a passing row cannot be a flush applied indiscriminately.

### Measurement — the check watched failing, twice

Both perturbations were applied, run, and reverted.

1. `strict_partial_sums_under` reverted to fold with a bare host `+` → `FAIL … normal-cancelling under Preserve/FlushToZero { zero_sign: PreservesSign }: the two-by-two partials  left: [1, 1]  right: [0, 0]`.
2. `ReferenceEvaluator`'s request construction reverted to hand every capability `ReferenceNumericalConformance::strict()` → `FAIL … FlushToZero { zero_sign: PreservesSign }/Preserve: a subnormal operand entering a multiply  left: 1  right: 0`.

Each is the ticket's own failure mode reintroduced, and each case refused it.

### Fact — an out-of-scope declaration defect found on the way, filed

Deciding whether SiLU could be documented immune required checking its declared fact rather than trusting it. `SILU_F32_FACT_SUBNORMALS` says `preserved-and-unreachable-no-binary32-silu-result-or-intermediate-is-subnormal`, justified from the large-negative tail alone; near zero the reference is `x / 2`, so `silu(0x007fffff) = 0x00400000` — a subnormal result from a subnormal operand. The family therefore applies the modes, and the declaration defect is filed as `correct-the-silu-subnormal-fact-that-covers-only-the-negative-tail` (`todo`, `implementation/ir`, outside this branch's scopes).

### Parked for Tom — the public boundary, implemented as a draft and not self-accepted

Eleven additive public items. **Every one is additive**: no existing signature moved, so no caller outside `crates/tiler-reference/**` changed, which is also what kept the branch inside its exclusive scope.

- `ReferenceEvaluator::under`, `ReferenceEvaluator::conformance`
- `ReferenceEvaluationRequest::conformance`
- `strict_partial_sums_under`, `strict_partitioned_sum_under`
- `silu_f32_under`, `certified_exp_f32_under`, `certified_rsqrt_f32_under`, `rms_norm_f32_under`, `softmax_f32_under`
- `StagedStrictTensorContractionF32::under`, `::conformance`

**Why this shape and not the alternatives.** Changing the existing signatures to take the contract as a parameter is the smaller surface and was rejected on scope, not taste: `certified_exp_f32`, `certified_rsqrt_f32`, and `StagedStrictTensorContractionF32::governed` have callers in `crates/tiler-compiler`, which this ticket may not edit — and a stop condition fires there rather than a signature change. A single `ConformedReference` facade carrying every family as a method was rejected on architecture: it would collapse families the crate deliberately keeps separate and would create two spellings of the same value. The `_under` suffix and the `under` constructor are the crate's own accepted precedent, `IndexRegionEvaluator::new`/`under`, applied uniformly — so the decision Tom is being asked for is one shape repeated, not eleven.

**What acceptance would settle**, stated so a refusal is cheap: whether a conformance-carrying entry is spelled as a sibling function (`_under`) or as a signature parameter on the existing one. Nothing has been released on it; the strict spellings are unchanged and every caller in the tree still uses them.

### Pin survey — no pinned identity moved, verified rather than assumed

`compute_reference_identity` (`crates/tiler-reference/src/identity.rs:22`) folds the semantic snapshot identity, the value validators, and each capability's op key, signature, semantic authority, provider identity, and revision. This branch adds no capability, changes no signature, and moves no revision — the conformance is a per-evaluation parameter of the same kind as `iteration_step_allowance`, which is likewise absent from the identity. `registry_identity_is_deterministic_and_revision_complete` and `registry_identity_budget_is_exact_at_boundary` pass unchanged, and the provider revision stays at 7 because every registry-identical evaluation at the strict reading is bit-identical to before.

### Commands run

`cargo fmt --all -- --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-reference --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-reference --no-deps`; `cargo nextest run --workspace`; `cargo test --workspace --doc`; `tkt lint`; `git diff --check`; `tkt guard`; `make full`.

### Measurement boundary

Every bit pattern above is host binary32 arithmetic under the pinned `nightly-2026-07-19` toolchain, reproducible on any host by the two named tests. **No device ran.** Nothing here observes what a flushing target computes; it establishes what the reference answers when told a contract, which is the object a device comparison needs and not evidence about the device. The four-column shape is exhaustive over the two dimensions' resolutions at `PreservesSign`; `AlwaysPositive` is exercised by `conformance.rs`'s own unit corpus and not re-run per family.
