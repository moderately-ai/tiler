---
id: bound-the-exact-rational-gcd-cost-in-certified-enclosures
title: Bound the exact-rational gcd cost in certified enclosures
status: in-progress
priority: p2
dependencies: []
related: [cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
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

## Outcome

Hypothesis 1 measured, then taken. Hypotheses 2 and 3 measured out of scope by the same census and are not taken; see below.

### Workloads

Both are stated as populations rather than as durations, and both were counted before anything was built.

- **W1** — `certified_exp_f32(x)` over 401 binary32 arguments `x = -40 + 0.2 i`, `i` in `0..=400`, at `EnclosurePrecision::binary32_corpus()`. All 401 decided.
- **W2** — `rsqrt_enclosure` over 256 binary32 radicands spanning `[2^-8, 2^8]` at the same precision. All 256 bracketed.
- **W3/W4** — one `exp_enclosure(1, ·)` and one `exp_enclosure(104, ·)`, the second being the greatest argument `certified_exp_f32`'s own guards can present.

### Measurement — the counted attribution

**Metric: Stein loop iterations and limb operations**, not wall clock. The census computes, for each `ExactRational::normalize` call, the loop `num-bigint 0.4.8`'s `BigUint::gcd` would run on those exact operands, and classifies the call by operand shape. A counted quantity was chosen deliberately: this repository's coordination host runs concurrent agent builds, so a timing here is an upper bound rather than evidence, while an iteration count is the same on any host.

| W1 (401 `certified_exp_f32`) | calls | share | Stein iterations | share |
| --- | ---: | ---: | ---: | ---: |
| denominator is `2^k` | 42,220 | 63.9 % | 5,915,630 | 62.9 % |
| — of which denominator is exactly 1 | 13,757 | 20.8 % | — | — |
| magnitude is `2^k`, denominator is not | 480 | 0.7 % | 6,168 | 0.066 % |
| one side at or below 64 bits, neither dyadic | 2,270 | 3.4 % | 47,998 | 0.51 % |
| general | 21,080 | 31.9 % | 3,441,061 | 36.6 % |
| **total** | **66,050** | | **9,410,857** | |

Limb operations follow the same split: 20,910,280 of 33,880,795 (61.7 %) on dyadic denominators. Widest operand seen: 1,584 bits.

W2 is the extreme case — **every one** of its 1,280 normalizations carries a power-of-two denominator, and all 133,797 Stein iterations and 330,869 limb operations are on that path. W3: 72 of 132 calls, 10,244 of 12,313 iterations. W4: 108 of 171 calls, 18,241 of 21,632 iterations.

**Why the share is that high, and why it is structural.** A certified enclosure rounds every intermediate outward onto a binary grid, so every value it carries past that point has a power-of-two denominator, and so does every product of two such values — which is exactly the squaring chain. The series terms `T_i = y^i / i!` are the general rows, since `i!` contributes an odd factor.

**So the counted bound is:** the dyadic path removes 62.9 % of W1's Stein iterations and 100 % of W2's, replacing each with one `trailing_zeros` and two shifts. What remains — the 36.6 % in the series — is the residue, and it is the part inherent to `i!` in the denominator.

### The change

`crates/tiler-ir/src/semantic/accuracy/rational.rs`:

- `power_of_two_exponent(&BigUint) -> Option<u64>`, replacing the previous `is_power_of_two` free function, which becomes `power_of_two_exponent(…).is_some()` at its one call site.
- `reduction_divisor(magnitude, denominator) -> BigUint`, a drop-in for `Integer::gcd` at both of this module's gcd sites, identical at every input including the zero magnitude, and taking `2^min(v2(magnitude), k)` when the denominator is `2^k`.
- `normalize` calls it, and when the divisor is itself a power of two divides both components by shifting. The shift is exact on the signed side because both components are exact multiples of the divisor.
- `from_sign_magnitude_ratio`'s lowest-terms check calls it too. That is a decode boundary admitting up to `MAX_EXACT_RATIONAL_MAGNITUDE_BYTES` (4,096) per component, so the widest dyadic pair an outside caller may present used to cost roughly 32,760 Stein iterations over 512-limb operands and now costs a trailing-zero count.

No public surface added, widened, or narrowed. Both new functions are module-private.

### Identity evidence

- 664-point dump, **byte-identical** before and after — `sha256 d435436da3c0e6ab21543e6e0bc0818ec3db846e6d2bfb6c8bfa0a28f9eae756`, 99,147 bytes. It carries all 401 W1 `certified_exp_f32` results as bit patterns, all 256 W2 enclosure endpoints as exact rationals, and the exact endpoint pairs of `exp_enclosure` at `-104, -40, -1, 1, 40, 104, 1419`.
- The change cannot move a value by construction: `reduction_divisor` returns the same divisor `Integer::gcd` returns, so every `ExactRational` is the same pair of integers it was.
- `cargo nextest run --workspace` — 2,995 passed, 7 skipped. The three `tiler-build` `metal_plan` golden pins are unmoved at `ARTIFACT_IDENTITY 7a2bfe51619c05a13fe86cd973e1dfa85c7353da33e4e75af0531068b774357d`, `CACHE_SUBJECT 8bdcde644d7df6d4ca95736f445a011b2d163efdfb3ba93a5c0a954d139b1aa2`, `FIXED_CONTENT_BYTES 65_294`. No identity domain steps.

### Wall clock — supporting only, not evidence

Taken in-process, five rounds after a discarded warm-up, min reported, with the census instrumentation compiled in on **both** sides so the two are comparable. **Host: Apple `Mac16,6`, 14 cores, macOS 27.0.0, load average 2.88 / 4.09 / 8.24 at the time of measurement, with other agents building concurrently.** `AGENTS.md` requires CPU timing on the idle M3 Pro, so these are upper bounds and a clean measurement is still owed:

| workload | before (min of 5) | after (min of 5) | ratio |
| --- | ---: | ---: | ---: |
| W1, 401 `certified_exp_f32` | 217.340 ms | 95.107 ms | 2.29× |
| W2, 256 `rsqrt_enclosure` | 3.244 ms | 0.519 ms | 6.25× |

Round-to-round spread was under 1.5 % in every case. The W1 ratio exceeds what removing 62.9 % of the gcd alone predicts because the same branch also replaces two big-integer divisions with shifts.

### Hypotheses not taken, with the measurement that decided each

- **The symmetric dyadic case** — a power-of-two magnitude against an odd denominator — is 480 W1 calls (0.7 %) carrying 6,168 iterations (0.066 %). A branch paid on every call to remove work that is not there.
- **A word-sized fast path** — reducing `gcd(huge, small)` by one modulo when either side fits a machine word — is 2,270 calls (3.4 %) carrying 47,998 iterations (0.51 %). Below the noise of the thing it would fix.
- **Hypothesis 2, a faster gcd than Stein's.** The residue it would attack is the 36.6 % general share. Still open, but it is now a dependency question over a third of the cost rather than over all of it, and it should be re-costed against the residue rather than against the original profile.
- **Hypothesis 3, deferred normalization.** Untouched. It remains the only lever on the series terms themselves, and it is the one that puts the lowest-terms invariant at risk, so it needs its identity obligation stated first as this ticket's body says.

### Reproducing the census

The instrumentation is deliberately **not committed** — it is a counter in a hot path that exists to answer one question. To rerun it, add to `rational.rs` a `pub mod census` holding relaxed `AtomicU64` counters and a `stein(&BigUint, &BigUint) -> (iterations, limbs)` that replays `num-bigint 0.4.8`'s `BigUint::gcd` loop verbatim, call `census::record(numerator.magnitude(), &denominator)` as the first statement of `normalize` after its zero-numerator return, and drive it from a temporary `crates/tiler-reference/tests/` integration test over W1–W4:

```sh
cargo nextest run -p tiler-reference --test gcd_census_temp --no-capture
```

The counters are a property of the operand population rather than of the implementation, so they read the same before and after the change — which is itself the check that the change did not alter what the workload computes.
