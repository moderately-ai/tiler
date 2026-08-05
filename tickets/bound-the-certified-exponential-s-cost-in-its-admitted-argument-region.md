---
id: bound-the-certified-exponential-s-cost-in-its-admitted-argument-region
title: Bound the certified exponential's cost in its admitted argument region
status: review
priority: p2
dependencies: []
related: [cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, performance]
claimed_from: todo
assignee: agent-exp-bound
lease_expires_at: 1785960717
---
## The finding

**Measurement — Apple M4 Max, dev profile, one `exp_enclosure(2^b, EnclosurePrecision::binary32_corpus())` per row, 2026-08-05.**

| Argument | Time |
| --- | --- |
| `2^0` | 0.33 ms |
| `2^5` | 0.38 ms |
| `2^10` | 0.71 ms |
| `2^13` | 4.1 ms |
| `2^16` | 157 ms |
| `2^20` | 37.5 s |
| `2^22` | over 9 minutes, not run to completion |

**Fact — the governed bound bounds the wrong quantity.** `MAX_ARGUMENT_HALVINGS` in `crates/tiler-reference/src/accuracy.rs` refuses an argument whose binade exceeds 22, and `EnclosureError::ArgumentTooLarge` is that refusal. It bounds the *halving count*, which is the loop trip count, and says nothing about the magnitude of the result those halvings then square up to: `exp(2^22)` needs a numerator of roughly `2^22 * log2(e)` — about six million bits — and every squaring multiplies and normalizes numbers of that size. So an argument the module admits can cost minutes, and the admitted region has no stated cost bound at all.

**Inference — this is a fail-closed gap, not a performance nit.** Everything else in this crate bounds its work before doing it: `MAX_REFERENCE_TENSOR_ELEMENTS`, `MAX_REFERENCE_ELEMENT_BYTES`, `MAX_SERIES_TERMS`, and the evaluator's own `iteration_step_allowance` all refuse with a typed error naming the limit and the observed value. The enclosure's argument bound is the one that admits a case it cannot cost. A caller that reaches it does not get a refusal it can explain; it gets a process that appears to hang.

**Fact — no caller in the tree reaches it today.** `certified_exp_f32` guards at `+89` and `-104` before calling `exp_enclosure`, so the binary32 SiLU and softmax references never present an argument above binade 7. The exposure is `exp_enclosure`'s own public boundary, which is re-exported from `crates/tiler-reference/src/lib.rs` and takes any `ExactRational`.

**Fact — this is pre-existing and not introduced by the reduction-depth change.** `cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock` restated `MAX_ARGUMENT_HALVINGS` as `23 + REDUCED_ARGUMENT_BITS` specifically so that the admitted binade stayed at 22 either way; the table above is a property of the result's magnitude, which that change does not move.

## What to decide

The question is which bound the module should state, and it is a public-boundary question rather than an implementation detail:

1. **Bound the result's magnitude.** Refuse when `argument * log2(e)` exceeds a governed bit width, with a typed error naming the width and the observed one. This is the shape the rest of the crate uses, and it makes the admitted region cost-bounded rather than merely trip-count-bounded.
2. **Bound the total work.** Carry a step or magnitude allowance the way `ReferenceEvaluator::with_iteration_step_allowance` does, so a caller can authorize a large enclosure deliberately and a caller that did not is refused.
3. **Leave it, and say so.** Record that the admitted region above some binade is unbounded in cost, with the reason and the argument that no caller can reach it. This is only tenable if the public boundary is narrowed so that no caller *can*.

Option 3 conflicts with `exp_enclosure` being public and general; options 1 and 2 change what the function refuses, which is a semantic change to a governed refusal and needs its diagnostic code decided rather than invented.

## Closes when

The admitted region has a stated cost bound or a recorded reason for not having one; any new refusal carries a typed error with a stable diagnostic code and a test that watches it refuse *and* watches the admitted neighbour, on the pattern `an_over_large_argument_is_refused` now uses; and the measurement table above is either reproduced against the change or superseded by it.

## Outcome

**Option 1, and the diagnostic code was preserved rather than invented.** `MAX_ARGUMENT_HALVINGS` is replaced by two constants in `crates/tiler-reference/src/accuracy.rs`: `MAX_RESULT_MAGNITUDE_BITS = 2048` bounds the width of the result the enclosure will compute, and `MAX_ARGUMENT_MAGNITUDE = 1419` is `floor(2048 * ln 2)`, that budget expressed in argument units. `EnclosureError::ArgumentTooLarge` and `reference.enclosure.argument-too-large` are unchanged — the refusal is still that the argument is too large, and only what makes it too large moved — so no new code had to be decided and the vocabulary ticket's `2^40` row still holds. The variant's payload is unchanged too: the enum carries the observed value and leaves the governed limit to the message, as `PrecisionUnreachable` already does, because these limits are compile-time constants rather than the per-evaluator allowances `error.rs` carries.

**Why the result's width rather than a work allowance.** The halving count is the loop trip count; the cost is in the magnitude the halvings square back up to, which is what `x * log2 e` measures and what every gcd normalization is quadratic in. Option 2's allowance would let a caller authorize the expensive region deliberately, but it buys that at the price of a second axis on every call site for a region no reference can use — past the budget the exponential exceeds every IEEE binary format at or inside binary64, so the reference decides nothing about any candidate. Option 3 was unavailable: `exp_enclosure` is public, general, and takes any `ExactRational`.

**Why 2048.** Binary64 spans `[2^-1074, 2^1024]`, which `exp` reaches at `|x| <= 745`, or 1,075 bits; twice that admits every reference for a format at or inside binary64 with room. Binary128 reaches `2^16384` and would raise the constant deliberately, with its own measurement.

**The admitted region is now bounded by construction, not by extrapolation.** At most `MAX_SERIES_TERMS` series terms over `T_i = y^i / i!` with `y <= 2^-8`, whose magnitudes depend on neither the argument nor the precision; then at most `10 + 1 + REDUCED_ARGUMENT_BITS = 19` squarings over endpoints carrying at most `MAX_RESULT_MAGNITUDE_BITS` plus the squaring grid's width. The magnitude check runs before the binade is read, so it also bounds every exponent the reduction derives.

**Measurement — M3 Pro (Mac15,6, 11 cores), macOS 27.0, `nightly-2026-07-19`, nextest test profile, load average 3.0–4.9, one `exp_enclosure(|x|, EnclosurePrecision::binary32_corpus())` per row, best of five for the integer rows. Supersedes the table above.**

| Argument magnitude | Time | Ratio to `104` | Status now |
| --- | --- | --- | --- |
| `1` | 0.53 ms | 0.57× | admitted |
| `104` — the greatest the oracle presents | 0.93 ms | 1.00× | admitted |
| `745` — binary64's full range | 1.13 ms | 1.23× | admitted |
| `1419` — the bound | 1.36 ms | 1.47× | admitted |
| `2^12` | 2.64 ms | 2.85× | **refused** |
| `2^13` | 6.48 ms | 7.0× | **refused** |
| `2^14` | 18.4 ms | 19.9× | **refused** |
| `2^16` | 229 ms | 247× | **refused** |
| `2^18` | 3.55 s | 3,830× | **refused** |

The rows past the bound were taken with `MAX_ARGUMENT_MAGNITUDE` temporarily widened, since they are refusals on the landed code; a refusal itself costs about 40 ns. Each doubling past the bound roughly quadruples the cost, which is a quadratic normalization over a linearly growing width, and extrapolating four doublings from `2^18` puts `2^22` near fifteen minutes — consistent with the nine-minute non-completion recorded above on an M4 Max. The absolute figures carry that host's load and are not a portable guarantee; the ratios were taken in one process so load reached every row alike. The bound therefore sits before the quadratic region rather than inside it.

**The oracle is byte-identical.** `certified_exp_f32` and `silu_f32` over 8,000 arguments spanning `[-104, 89]` digest to `0x6657f406300fa256` and `0xe634bf0789fcf00c` at the base commit and after the change, with zero refusals in the sweep, on both the coordination host and the M3 — the same values on two hosts, which is the profile independence the module claims. The convention is FNV-1a over each result's little-endian binary32 bits, one digest per function; the wall-clock landing's harness was not preserved, so its `0x2f1bd946...` is not comparable and this states its own convention rather than claiming to reuse one.

**Watched firing, both directions.** With `MAX_ARGUMENT_MAGNITUDE` widened to `100_000_000`, `an_over_large_argument_is_refused` fails (`2^16` no longer refuses) and `the_argument_bound_is_the_result_budget_in_argument_units` fails on "the greatest admitted argument's result must fit the budget". Narrowed to `50`, the latter fails on "and it must be the greatest such argument" and `every_argument_the_guards_admit_is_inside_the_enclosure_bound` fails with "the enclosure must bracket 0x42b1ffff, which the guards admit". Each check was watched failing before being trusted.

**The reachability claim, re-verified and stated as the boundary.** `certified_exp_f32` decides `+inf` at or above `+89` and `+0.0` at or below `-104` before consulting the enclosure, so every argument that reaches `exp_enclosure` has `|t| <= 104` — an order of magnitude inside the admitted `1419`. The refusal is a bound on the magnitude, so admitting the greatest magnitude the guards pass admits all of them; `every_argument_the_guards_admit_is_inside_the_enclosure_bound` watches the two representable extremes (`0x42b1ffff`, and the neighbour just above `-104`) bracket, and watches `-f32::MAX` refuse from outside the guards. The exposure that remains is `exp_enclosure`'s own public boundary, which now refuses instead of appearing to hang.

**One finding out of scope, filed rather than absorbed.** `exp_enclosure` panics rather than refusing on an `EnclosurePrecision` whose grid width leaves `i32` — `EnclosurePrecision::new` admits any `u32` and the width is negated into an exponent — and the function's `# Panics` section asserted a bound on that type which does not exist. Probed: `100_000` refuses with `precision-unreachable`; `2_147_483_646` and `u32::MAX` panic. Pre-existing, unreachable from any caller in the tree, on the precision axis rather than the argument one, and every repair moves a public boundary, so it is filed at `awaiting-decision` as [`refuse-an-enclosure-precision-the-grid-arithmetic-cannot-express`](refuse-an-enclosure-precision-the-grid-arithmetic-cannot-express.md). The `# Panics` section now states what is true and cites it.

**One stale assertion left alone.** `implement-the-typed-accuracy-contract-vocabulary.md` describes the enclosure's refutation criterion as "the governed halving or term bound", which this change makes stale in its first half. It is another ticket's recorded outcome and editing it here would rewrite history that ticket owns; its `2^40` diagnostic-code row remains correct either way.

`project/tickets` was added to this ticket's shared scopes because recording this outcome and filing the ticket above both write under `tickets/`; the declaration is scheduling metadata for work this ticket already authorizes.
