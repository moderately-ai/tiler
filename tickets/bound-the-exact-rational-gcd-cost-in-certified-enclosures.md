---
id: bound-the-exact-rational-gcd-cost-in-certified-enclosures
title: Bound the exact-rational gcd cost in certified enclosures
status: in-progress
priority: p2
dependencies: []
related: [cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock]
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [performance, numerics]
claimed_from: todo
assignee: agent-gcd
lease_expires_at: 1786125787
---
## What was found, and where

**Measurement — Apple M4 Max, `/usr/bin/sample` over `tiler-reference::decoder_layer the_layer_evaluates_end_to_end_at_the_c1_prefill_row`, dev profile, 2026-08-05.** About 80 % of non-idle samples land in `num_bigint`'s Stein binary greatest-common-divisor — `biguint_shr`, `Integer::gcd`, `sub_assign`, `trailing_zeros` — reached from `ExactRational::normalize`.

**Fact — the gcd is per operation, not per result.** `crates/tiler-ir/src/semantic/accuracy/rational.rs` keeps every `ExactRational` in lowest terms, and `add`, `subtract`, `multiply`, `divide`, `scale_by_power_of_two`, `power`, `floor_to_binary_grid` and `ceil_to_binary_grid` all funnel through `normalize`, which computes `numerator.magnitude().gcd(&denominator)` every time. That invariant is load-bearing — it is what makes the canonical encoding an identity rather than a serialization, so one number has exactly one spelling — and it is not the thing to remove.

**Fact — what makes it expensive here.** `num-bigint`'s `BigUint::gcd` is Stein's algorithm: a shift-and-subtract loop that runs proportionally to the operand's bit length, over operands whose word count is itself proportional to that length, so one gcd is quadratic in the magnitude. `exp_enclosure`'s series carries `T_i = y^i / i!`, whose denominator is `2^(m*i) * i!` — thousands of bits within a few dozen terms — so the series pays a quadratic gcd against a magnitude that grows with every term. Timed in-process, the series loop was 79 % of `exp_enclosure`, split 33 % in `add` and 46 % in `multiply`-then-`divide`.

**Inference — the lever is the representation, not the caller.** [`cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock`](cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock.md) took the caller-side lever as far as it goes from inside `crates/tiler-reference`: a deeper argument reduction and a cheaper interval product bought 1.65× on the package, and an attempt to bound the magnitudes by rounding the series onto a working grid each iteration produced byte-identical enclosures and ran 1.5× *slower*, because it quadruples the operation count and the per-operation cost is what dominates. What is left is the per-operation gcd itself, which lives in `crates/tiler-ir`.

## What to evaluate

Each of these is a hypothesis with an owner in `crates/tiler-ir/src/semantic/accuracy/rational.rs`, not a decision:

1. **A dyadic fast path in `normalize`.** Every value an enclosure rounds onto a binary grid has a power-of-two denominator, and `gcd(n, 2^k)` is `2^min(trailing_zeros(n), k)` — a shift, not a loop. Measure how many `normalize` calls in a certified-enclosure workload have a power-of-two denominator before building anything.
2. **A faster gcd than Stein's.** `num-integer`'s `BigUint` gcd is binary; Lehmer's or a subquadratic variant would change the exponent rather than the constant. This is a dependency question as much as a code one.
3. **Deferred normalization.** A representation that carries a not-yet-reduced pair and normalizes only where the canonical identity is actually taken. This is the one that touches the invariant, so it needs the identity obligation stated first: which operations may return an unreduced value and which must not.

Nothing here may weaken the lowest-terms guarantee at an encoding boundary — `from_sign_magnitude_ratio` refuses a value that is not in lowest terms precisely so that one number cannot acquire two identities, and `ExactRationalError::NotInLowestTerms` is that refusal.

## Closes when

A measured attribution of the gcd cost across a certified-enclosure workload, and either an implemented change with its before/after on a quiet host and its identity evidence — the exact-rational and enclosure test corpora unchanged, and `certified_exp_f32` byte-identical over a stated argument sweep — or a recorded finding that the cost is inherent to the invariant, with the reason.
