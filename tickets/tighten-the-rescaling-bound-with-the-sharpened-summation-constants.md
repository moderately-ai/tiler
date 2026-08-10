---
id: tighten-the-rescaling-bound-with-the-sharpened-summation-constants
title: Tighten the rescaling bound with the sharpened summation constants
status: deferred
priority: p3
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy, deferred]
---
## User-visible outcome

The online-softmax rescaling bound is stated with the sharpest constants the literature offers, so that a caller whose tolerance sits between the loose bound and the sharp one is admitted rather than refused.

## Why this exists

**Fact.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) derives with the classical `gamma_h = h*u/(1 - h*u)` and says so deliberately: `gamma`'s composition rule `gamma_h + gamma_k + gamma_h*gamma_k <= gamma_{h+k}` is what lets the fold's bound compose with the elementary function's inside one algebra.

**Fact.** Sharper constants exist and are preserved. Rump (2012) replaces `gamma_{n-1}` by `(n-1)*u` for recursive summation under round-to-nearest, removing both the second-order term and the `(n-1)*u < 1` side condition; Lange and Rump give `(n-1)*2u` for faithful rounding subject to `n - 1 <= 1/(2u)`. Both are restated in the vendored `acta-numerica-fp-2023` at its Section 4.6.2, and `jeannerod-rump-simax-2013` is the metadata-only row for the inner-product line.

**Measurement.** The probe records bound-over-observed ratios from 4.8 to 2.7e29, typically 20–600 in the regimes that matter. **The sharpening removes only the second-order `O((h*u)^2)` remainder (and the `h*u < 1` side condition); first-order coefficients of classical `gamma_h` and sharp `h*u` agree**, so the relative classical-vs-sharp gap is ~`h*u` itself (~3e-5 at binary32 `h = 511`, `V ≤ 512`) and **would not close that gap**, which is the honest reason this is deferred rather than scheduled: the dominant looseness is `gamma` being a worst case that positive-term summation does not approach, not `gamma` carrying an `O(u^2)` term.

**Correction — 2026-08-10.** Prior wording claimed the sharpening was "worth roughly a factor of two in the first-order term." That was wrong: `gamma_h = h*u/(1 - h*u)` expands as `h*u + (h*u)^2 + …`, so replacing it by `h*u` leaves the first-order coefficient unchanged and only drops the second-order remainder (and the side condition). A factor-of-two relation exists between Lange–Rump faithful `(n-1)*2u` and RN `(n-1)*u`, but that is not classical-versus-sharp first-order of the same model. The Measurement sentence above is the corrected claim.

## Why this is deferred rather than todo

Tightening a bound that nothing consumes yet is work whose value depends entirely on a caller being refused by the loose one. No tolerance vocabulary exists, no rewrite is admitted on a bound, and no caller has stated a tolerance. Doing this now would produce a sharper number nobody compares against, and would spend the composition-rule simplicity the record deliberately bought.

## Trigger

**Fires when a caller's stated tolerance is refused by the loose bound and would be admitted by the sharp one.** That is a single concrete observation rather than a judgement: it needs a tolerance vocabulary to exist, a rewrite to be gated on a bound, and one refusal whose margin falls inside the factor the sharpening buys. A general wish for tighter bounds does not fire it, and neither does the ratio table in the record — which was measured before any caller existed and is the evidence *for* deferring.

## What it would take

Re-derive the fold term with `(n-1)*u`; establish the composition with the elementary-function factor by hand where `gamma`'s rule no longer applies; restate the side conditions Rump's proof needs, since it uses `|RN(a+b) - (a+b)| <= min(|a|, |b|)`, a round-to-nearest property that does not hold for directed rounding; and re-run the probe against both forms so the improvement is measured rather than asserted.

## Trigger check log

- 2026-08-05 — **not fired**, and two of its three preconditions do not exist, so it cannot yet fire for reasons that have nothing to do with judgement: there is no tolerance vocabulary and no rewrite admitted on a bound, so no refusal can have occurred. Filed `deferred` at creation for exactly that reason rather than being filed dispatchable and parked later. Recheck with `grep -c 'tolerance vocabulary' docs/numerical-semantics.md`, which answers `0` while no caller can state one.
- 2026-08-09 — **not fired.** The proposed rewrite-price vocabulary is still awaiting acceptance in [`accept-the-rewrite-price-tolerance-vocabulary`](accept-the-rewrite-price-tolerance-vocabulary.md), no rewrite is admitted by comparing its derived price to such a caller budget, and therefore no observed refusal can separate the classical and sharpened constants. General exact tolerances elsewhere in numerical conformance are not this trigger.
