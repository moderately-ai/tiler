---
id: refuse-an-enclosure-precision-the-grid-arithmetic-cannot-express
title: Refuse an enclosure precision the grid arithmetic cannot express
status: done
priority: p2
dependencies: []
related: [bound-the-certified-exponential-s-cost-in-its-admitted-argument-region]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics]
---
## The finding

**Fact — a public input panics instead of refusing.** `EnclosurePrecision::new` is a public `const fn` over any `u32`, and `exp_enclosure` turns the grid width into a signed exponent at `crates/tiler-reference/src/accuracy.rs`:

```
let threshold = ExactRational::power_of_two(
    -i32::try_from(precision.fraction_bits().saturating_add(2))
        .expect("a bounded grid width fits i32"),
);
```

A width past `i32::MAX` fails that conversion and panics. Probed at the base of `bound-the-certified-exponential-s-cost-in-its-admitted-argument-region`, one `exp_enclosure(&ExactRational::one(), EnclosurePrecision::new(bits))` per row:

| `bits` | Outcome |
| --- | --- |
| `100_000` | `reference.enclosure.precision-unreachable` |
| `2_147_483_646` | panic — `a bounded grid width fits i32: TryFromIntError(PosOverflow)` |
| `u32::MAX` | panic — same |

**Fact — the doc comment claims the opposite.** `exp_enclosure`'s `# Panics` section says it panics only if "a grid width bounded by the caller's own `EnclosurePrecision` leaves `i32`, which those bounds make unreachable". Nothing bounds `EnclosurePrecision`: `new` accepts any `u32` and returns `Self` infallibly, so the claimed bound does not exist. The source wins and the comment is the defect.

**Inference — same fail-closed family as the argument bound, different axis.** `bound-the-certified-exponential-s-cost-in-its-admitted-argument-region` bounded the *argument* region so every admitted argument has a bounded cost. This is the remaining unbounded axis on the same function, and it is worse in kind: an over-large argument now returns a typed refusal a caller can explain, where an over-large precision aborts the process. A reference oracle whose contract is "fail closed with typed, explainable errors" must not have a public input that panics.

**Fact — no caller in the tree reaches it.** Every construction site passes `EnclosurePrecision::binary32_corpus()` (256) or a small literal in a degradation test; the widest in the tree is `EnclosurePrecision::new(12_000)` in `a_precision_the_series_cannot_reach_is_refused`. The exposure is the public boundary, not a live path.

## What to decide

This is a public-boundary question and belongs to Tom, which is why it is filed rather than absorbed:

1. **Validate at construction.** `EnclosurePrecision::new` becomes fallible, or gains a checked constructor beside an infallible one bounded by a governed maximum. This puts the refusal where the value is written rather than where it is used, which is the shape ADR-adjacent validation elsewhere in this crate prefers — and it changes an accepted public `const fn`'s signature.
2. **Refuse in `exp_enclosure`.** A new `EnclosureError` variant with its own stable diagnostic code, refusing a grid the arithmetic cannot express. Keeps `EnclosurePrecision` a plain newtype and needs a diagnostic code decided rather than invented.
3. **Bound the type's domain silently.** Clamp or saturate. Rejected on inspection: a clamp answers a question the caller did not ask at a precision they did not request, which is the shape this module refuses everywhere else.

## Closes when

`exp_enclosure` has no reachable panic on any `(ExactRational, EnclosurePrecision)` pair its public signature admits; the refusal — wherever it is placed — carries a typed error with a stable diagnostic code and a test that watches it fire *and* watches the admitted neighbour; and `exp_enclosure`'s `# Panics` section states what is actually true rather than a bound that does not exist.

Filed at `awaiting-decision` rather than `todo` because every option above moves a public boundary — a `const fn`'s signature or a new governed diagnostic code — and the board must not offer a ticket whose first step is a decision it cannot make. Tom's answer to "What to decide" is what makes it dispatchable.

## Decided — defence in depth, 2026-08-05

Tom decided at the live review (witnessed first-hand by the coordinator): both layers, not either. (1) `EnclosurePrecision` gains a validated construction bound so the overflowing grid width is unrepresentable — the primary repair. (2) The consumption site's `i32` conversion becomes a checked conversion returning the typed `EnclosureError` refusal rather than a panic — the second layer, kept even though the bound makes it unreachable through the validated constructor, because defence in depth is the stated preference. The second layer's watched-failing evidence comes from perturbing the construction bound (the pattern the exp-bound landing used), not from a wildcard test; a check that cannot be demonstrated failing under a stated perturbation does not land. Both surface changes return for acceptance as one delta. Status moves to `todo`: this is now a decided implementation ticket awaiting dispatch.

## Outcome

**Both layers landed, and the bound is the arithmetic's rather than a new number.** `crates/tiler-reference/src/accuracy.rs` gains `MAX_GRID_FRACTION_BITS = i32::MAX - SQUARING_GRID_MARGIN = 2_147_483_637`, `EnclosurePrecision::new` returns `Result<Self, EnclosureError>` against it, and the tail threshold's `i32` conversion returns `EnclosureError::GridWidthUnrepresentable` — code `reference.enclosure.grid-width-unrepresentable` — instead of asserting. `exp_enclosure` has no reachable panic on any `(ExactRational, EnclosurePrecision)` pair its signature admits, and its `# Panics` section says so rather than documenting the gap.

**The bound's derivation, and the three candidates it eliminated.** The authority is the `i32` exponent arithmetic the width feeds, so the only question was *which* width it has to protect.

1. `u32::MAX` — no bound. That is the defect.
2. A fresh cost-motivated ceiling, some small number of thousands of bits. Rejected: it is not derived from anything the arithmetic requires, it would narrow the admitted region on an authority no code here states, and `MAX_SERIES_TERMS` already answers the cost question with `PrecisionUnreachable`. The measurement below removed the last reason to want one.
3. `i32::MAX` — the widest width `ExactRational` can express *directly*. Rejected on inspection: `exp_enclosure` does not round on the caller's grid, it rounds on one `REDUCED_ARGUMENT_BITS + 2` bits finer, so this bound would admit precisions whose *derived* grid the same arithmetic cannot express — pushing the refusal back onto the consumption site the bound exists to make unreachable.
4. `i32::MAX - (REDUCED_ARGUMENT_BITS + 2)` — the greatest stated width at which every width derived from it is still an exponent. Survives, and is what landed.

The margin is now the named constant `SQUARING_GRID_MARGIN`, carrying the squaring-grid rationale that used to sit as an inline comment, and the derived grid is `EnclosurePrecision::squaring_grid` — private, total by construction, the one width in the module allowed past what a caller may state. `the_grid_bound_is_the_exponent_limit_less_the_squaring_margin` pins `MAX_GRID_FRACTION_BITS + SQUARING_GRID_MARGIN == i32::MAX`, so neither constant moves alone, on the pattern `the_argument_bound_is_the_result_budget_in_argument_units` uses for the argument axis.

**Layer one, watched failing.** `a_grid_width_the_arithmetic_cannot_express_is_refused` refuses `u32::MAX`, `2_147_483_646` — the two widths the finding's probe table recorded as panics — and `MAX_GRID_FRACTION_BITS + 1`, and admits the bound itself. With `MAX_GRID_FRACTION_BITS` widened to `u32::MAX - 10` it fails on `left: None, right: Some("reference.enclosure.grid-width-unrepresentable")` at the `2_147_483_646` row, and the tie test fails on `left: 4294967295, right: 2147483647`. Widened all the way to `u32::MAX` the tie test does not merely fail, it stops compiling — `attempt to compute u32::MAX + 10_u32, which would overflow`, under the deny-by-default `arithmetic_overflow` lint — which is why the recorded perturbation is the `- 10` one that lets both checks run.

**Layer two, watched firing and watched failing.** Under that same perturbation, `exp_enclosure(1, EnclosurePrecision::new(2_147_483_646))` returns `reference.enclosure.grid-width-unrepresentable`. With the checked conversion reverted to its original `.expect("a bounded grid width fits i32")` and nothing else changed, the identical call panics with `a bounded grid width fits i32: TryFromIntError(PosOverflow)` — the finding's own panic, reproduced. The evidence is a perturbation of the construction bound, as decided, and there is no wildcard test: through the validated constructor layer two cannot fire, because a stated width is at most `i32::MAX - 10` and the conversion adds two.

**The whole admitted region is total, observed rather than argued.** `the_widest_admitted_grid_refuses_rather_than_aborting` calls `exp_enclosure` at `MAX_GRID_FRACTION_BITS` itself and gets `reference.enclosure.precision-unreachable`. It costs about two seconds and allocates a quarter-gigabyte threshold, which is the price of exercising the extreme instead of reasoning about it; the package suite went from 29.0 s to 31.3 s, and it is the only test in the module that reaches that width.

**Measurement — M3 Pro (Mac15,6), macOS 27.0, `nightly-2026-07-19`, nextest test profile, one `exp_enclosure(1, EnclosurePrecision::new(bits))` per row, single process, 2026-08-05.**

| `bits` | Time | Outcome |
| --- | --- | --- |
| `256` — the corpus precision | 0.41 ms | ok |
| `4_000` | 23.5 ms | ok |
| `8_000` | 123 ms | ok |
| `8_483` | 120 ms | `precision-unreachable` |
| `1_000_000` | 121 ms | `precision-unreachable` |
| `16_000_000` | 135 ms | `precision-unreachable` |
| `2_147_483_637` — the bound | 2.04 s | `precision-unreachable` |

**This is why no cost ticket was filed, and it corrects an expectation the work started with.** The precision axis looked like the argument axis before the exp-bound landing — an admitted region with no stated cost bound — and the plan was to file it. It is not one: `MAX_SERIES_TERMS` caps the term count and the terms' magnitudes depend on the reduction depth rather than on the grid, so a wider grid only moves where the loop stops and widens the threshold it compares against. The growth is linear in the width rather than quadratic, and the worst admitted case is two seconds. The table is recorded on `MAX_GRID_FRACTION_BITS` in the source, where the next reader wondering the same thing will find it. The absolute figures carry that host's load and are not a portable guarantee; the shape — flat until the threshold's own magnitude dominates, then linear — is what the conclusion rests on.

**The oracle is byte-identical.** `certified_exp_f32` and `silu_f32` over 8,000 arguments spanning `[-104, 89]` digest to `0x6657f406300fa256` and `0xe634bf0789fcf00c` at the base commit `55d1d09f` and after the change, with zero refusals in both sweeps. Those are the exact values the exp-bound landing recorded, so its convention was *reconstructed* rather than restated: FNV-1a (offset basis `0xcbf29ce484222325`, prime `0x100000001b3`) over each result's little-endian binary32 bits, one digest per function, arguments `(-104.0 + 193.0 * i / 8000.0) as f32` computed in `f64` for `i` in `0..8000`. Three other plausible parameterizations of the same interval were run in the same process and produced different digests; only this one reproduces the recorded pair. The harness is not preserved — no scope on this branch reaches `spikes/` — but the sweep is stated exactly enough to rebuild in one sitting, which is how the landing it came from left it.

**The acceptance delta — two public surface changes, both for Tom.**

1. `EnclosurePrecision::new(u32) -> Self` becomes `EnclosurePrecision::new(u32) -> Result<Self, EnclosureError>`, still `const`. Every call site in the tree is a test literal and takes `.expect("a stateable grid")`, which is this crate's existing idiom for a validated constructor (`ExactRational::from_ratio(1, 2).expect("valid")`, `OpKey::new(...).expect("valid")`). `binary32_corpus()` is unchanged and still infallible.
2. `EnclosureError` gains `GridWidthUnrepresentable { fraction_bits: u32 }` with the new stable code `reference.enclosure.grid-width-unrepresentable`. The payload follows the variants beside it: the observed value, with the governed limit left to the `Display` message. The enum is `#[non_exhaustive]`, so the addition is not itself breaking; the diagnostic code is new governed vocabulary and is the part that needs deciding rather than noting.

Named `GridWidthUnrepresentable` rather than `PrecisionUnrepresentable` because the sibling codes already spend the word "precision" twice — `precision-unreachable` for a grid the series cannot converge to, `precision-too-coarse` for one that cannot bracket away from zero — and a third would differ from the first by four letters in the middle of a long word. This one names the quantity that is out of range instead.

**One doc comment corrected beyond the two the ticket names.** `CertifiedEnclosure::coarsen` is public, takes any `EnclosurePrecision`, and delegated to a grid rounding that panics above `i32::MAX` — an undocumented panic on the same axis, which layer one also closes. It now records why it cannot panic. `rsqrt_enclosure` was exposed the same way through `sqrt_enclosure` and is closed by the same bound.

**Scope.** The branch touches `crates/tiler-reference/**` (`implementation/reference`) and this ticket file. `project/tickets` was added to `shared_scopes` because recording this outcome writes under `tickets/`, which `ticketsplease.toml` maps to `project/tickets`; the declaration is scheduling metadata for work this ticket already authorizes, not a new outcome.

**Commands run.** `cargo fmt --check`; `cargo check -p tiler-reference --all-targets`; `cargo check --workspace --all-targets --locked`; `cargo clippy -p tiler-reference --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-reference --no-deps`; `cargo nextest run -p tiler-reference` (277 passed, 2 skipped); `cargo test -p tiler-reference --doc`; `git diff --check`; `tkt lint`; `tkt guard`.

## Landed — the surface is the decided shape

The delta (`EnclosurePrecision::new` fallible + `EnclosureError::GridWidthUnrepresentable`) implements Tom's defence-in-depth decision verbatim; it returns at the next decision round as a confirmation item rather than a new question.
