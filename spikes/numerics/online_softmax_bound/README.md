---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.online-softmax-bound"
kind: "experiment"
title: "Online-softmax rescaling bound probe"
topics: ["numerics", "accuracy", "reductions", "measurement"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.numerics.certified-bounds-as-rewrite-permissions"]
entrypoints: ["spikes/numerics/online_softmax_bound_probe.py"]
last_verified: "2026-08-05"
ticket: "connect-certified-rounding-error-bounds-to-rewrite-permissions"
---

# Online-softmax rescaling bound probe

This probe checks the hand-derived worst-case bound on the online-softmax rescaling fold against exact evaluation, over a named finite corpus. The derivation is in [the certified-bounds record](../../../docs/research/numerics/certified-bounds-as-rewrite-permissions.md); this directory holds only what was measured.

**What it establishes.** That the derived bounds are not violated on 22 adversarial cases, and — the part the derivation cannot supply — how loose they are, which is the quantity that decides whether a caller's stated tolerance can actually be met.

**What it does not establish.** That the bounds hold universally. A finite corpus refutes a bound by exhibiting a counterexample and never proves one, which is the `Empirical` boundary [the region-accuracy contract](../../../docs/research/numerics/region-accuracy-contract.md) defines. The record labels the derivation `sound-proof` on its algebra and this probe `bounded-measurement` on its numbers, and the two claims are deliberately not merged.

## How the simulation avoids depending on the host's arithmetic

Every operation is computed exactly in `Decimal` at 120 digits and then rounded once to binary32, so no host floating-point rounding, optimization level, or compiler flag participates. `exp` is correctly rounded to binary32 — the strongest admissible implementation, and therefore the case in which any observed looseness belongs to the fold rather than to the elementary function. The reference is the exact real `sum_j exp(x_j - max_k x_k)` over the exact binary32 inputs, which is the governed real lift the region-accuracy contract names, deliberately not "whatever the two-pass fold computed": a rewrite compared against another implementation measures a difference rather than an error.

Bounds are computed in exact rational arithmetic through `Fraction`, never through a host float. A bound on rounding errors that was itself rounded would be a category error, and it is the kind that passes review.

## Reproduce

From the repository root, both modes:

```sh
python3 spikes/numerics/online_softmax_bound_probe.py
python3 -O spikes/numerics/online_softmax_bound_probe.py
```

Standard library only — no dependency to pin and none claimed. Verdicts are explicit checks rather than `assert`, so optimized Python cannot discard them; either command exits nonzero instead of publishing JSON when a derived bound is violated, when the observed price exceeds the derived price, or when the evaluated population does not match the declared one.

[`results.json`](results.json) is the byte-for-byte output retained from both modes on the recorded host, verified identical between them. It binds the probe's own SHA-256, the oracle library and precision, the format and its unit roundoff, the `exp` implementation and its `eps_exp`, the declared and evaluated case counts, and the recorded Python implementation, version, machine, and system. It does not identify the Python executable or the complete interpreter build. Another environment may produce a new bounded record; it must not silently overwrite the provenance of this one.

## Watched failing — 2026-08-05

Four perturbations were applied in a scratch copy rather than in place. The unperturbed run returned exit 0 before and after.

| Perturbation | Result |
| --- | --- |
| The rescale factor's sign is flipped, `exp(m_j - m_{j-1})` for `exp(m_{j-1} - m_j)` — a real defect in the rewrite | exit 1, `32 check(s) failed over 22 cases`: 16 `online error exceeds its derived bound` and 16 `observed price exceeds the derived price` |
| The derived price is divided by 100 | exit 1, `5 check(s) failed over 22 cases`, all `observed price exceeds the derived price` |
| A declared case group is removed from `corpus()` | exit 1, `population mismatch: evaluated 20 cases, expected 22` |
| `gamma` is asked for `h = 2**24`, where `h*u = 1` | raises `ValueError: gamma is undefined at h=16777216: h*u = 1.0 >= 1` rather than returning a negative bound every later comparison would pass |

**The third perturbation found a defect in this probe rather than confirming it, and it is recorded because that is the reason to run the exercise at all.** The population check originally compared `len(rows)` against `len(corpus())`. Both sides came from the same function, so deleting a case left the check agreeing with itself and the probe exited 0 — a check that could not say no, which is the shape this repository is required to distrust. It now compares against a literal `DECLARED_CASES = 22`, the same discipline `docs/research/numerics/sources/verify-sources.sh` states at its own top.

**What this corpus cannot detect, so the evidence is not overread.** Removing the elementary-function factor from the online bound, and removing its summation term entirely, both still pass: the bound's headroom over these inputs is large enough to absorb a missing first-order term. The corpus refutes structural errors in the fold and gross understatements of the price; it does **not** pin the bound's constants, and no reading of `results.json` should suggest otherwise.

## The corpus, and why each member is in it

Twenty-two cases, stated rather than sampled, because a worst-case bound is refuted by a worst case and a random draw does not know where one is.

- **`increasing-v{2,8,64,512}-span{1,20,80}`** — strictly increasing logits move the running maximum at every step, making all `V-1` rescale factors non-trivial roundings. This is the regime the rewrite is most expensive in, crossed with three logit spreads because the spread governs the argument-perturbation term.
- **`decreasing-v{8,64,512}`** — the maximum is found first and never moves, so every rescale factor is exactly 1. The observed price is exactly zero here, which is the measurement behind the record's observation that a proved monotonicity precondition would collapse the bound.
- **`uniform-v{8,64,512}`** — every term is exactly 1 and the reference is exactly `V`, so any departure is entirely the fold's. Both folds are exact on these.
- **`dominant-tail-v{64,512}`** — one logit far above a long flat tail. The reference is within rounding of a single term, so both folds are nearly exact and the bound-to-observed ratio reaches `10^19`. This is roughly the regime an attention softmax lives in, which is why it is in the corpus and why the record reports it as the bound's least useful case rather than as a success.
- **`sawtooth-v{64,512}`** — the maximum moves on some steps and not others, so the rescale factors alternate. The intermediate case, and the one that catches a defect the two extremes miss.

## Traceability

- **Supported claim:** [Certified rounding-error bounds as rewrite permissions](../../../docs/research/numerics/certified-bounds-as-rewrite-permissions.md).
- **Derivation sources:** the classical results are read in `acta-numerica-fp-2023` and the algorithm in `online-softmax-arxiv-1805.02867v2`, both recorded in [the numerics source manifest](../../../docs/research/numerics/sources/README.md).
- **Normative owner:** [Numerical semantics](../../../docs/numerical-semantics.md).
- **Work record:** [`connect-certified-rounding-error-bounds-to-rewrite-permissions`](../../../tickets/connect-certified-rounding-error-bounds-to-rewrite-permissions.md).
