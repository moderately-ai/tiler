---
id: separate-the-rescaling-price-from-the-observed-fold-divergence
title: Separate the rescaling price from the observed fold divergence
status: review
priority: p3
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions, derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy, reductions]
claimed_from: todo
assignee: agent-rescale-price
lease_expires_at: 1786049402
---
## User-visible outcome

The online-softmax rescaling *price* is stated as what it is — the ratio of the two folds' derived bounds, and therefore the extra relative budget the rewrite consumes against the shared real reference — rather than as an upper bound on how far the two folds' computed results can lie from each other, which it is not.

## Why this exists

**Fact.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) states in its outcome that the fold's "price over the two-pass fold it replaces is at most `(1 + eps_exp)^(V-1) * (1 + gamma_{2(V-1)}) / (1 + gamma_{V-1}) - 1`", and [its probe](../spikes/numerics/online_softmax_bound/README.md) checks `abs(online - two_pass) / reference` against exactly that quantity, failing the run when it is exceeded.

**Inference.** `P` is defined by `1 + B_online = (1 + B_2)(1 + P)`, so it bounds the additional *bound* the rewrite carries. It does not bound the realized divergence: the two folds' errors are independent perturbations that may land at opposite ends of their own brackets, and the rigorous bound on `abs(online - two_pass) / reference` is `B_online + B_2`, which is about twice `P` at every shape measured. Derived in [the tree-fold record](../docs/research/numerics/tree-fold-online-softmax-bound.md) Part 5.

**Inference — this does not destabilize the admission rule.** That rule compares a rewrite's *bound* against a caller's tolerance, and `B_online` is that bound. `P` is presentational and is sound in its intended reading. What is missing is a sentence saying which reading is meant, in the two places a reader takes the other one.

**Measurement.** The divergence check is a good detector and a bad theorem: over the tree probe's 91 rows it fired on 55 under a sign-flipped rescale factor and on 20 under a hundred-fold understated price, so removing it would lose real refutation power. It is retained and relabelled in the tree probe already; the sequential probe still carries the unqualified wording.

## What this ticket must produce

- The certified-bounds record's outcome sentence and Part 2 Step 4 stating which quantity `P` bounds, without re-deriving anything.
- The retained sequential probe's check renamed or annotated so a reader knows a violation would be a signal to investigate rather than a refutation, and the rigorous `B_online + B_2` check added beside it. Regenerating `results.json` is an identity step for that spike: its `probe_sha256` moves, and the record's cited table must be re-read against the new file in the same change.
- The tree probe and its README already carry both checks labelled; check they still agree with whatever wording lands here.

## Non-goals

Changing any derived constant; touching `docs/numerical-semantics.md`; relitigating the admission rule's shape.

## Closes when

Both records and both probes state one reading of `P`, and the rigorous divergence bound is checked where the heuristic one is.

## Outcome — 2026-08-06

**Fact — the wording landed in both record sites.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md)'s outcome now carries a paragraph labelled `Inference` stating that `P` is the ratio of the two folds' derived bounds — `1 + B_1 = (1 + B_2)(1 + P)` — so it bounds the extra relative budget the rewrite consumes against the shared real reference and **not** the realized divergence `|online - two_pass| / R`, whose rigorous bound `B_1 + B_2` is cited to [the tree-fold record's](../docs/research/numerics/tree-fold-online-softmax-bound.md) Part 5 rather than re-derived. Part 2 Step 4 carries the same statement at the point the derivation produces `P`, plus a paragraph naming what the probe checks with what authority. The outcome's "is at most `(1 + eps_exp)^…`" became "is `P(V) = (1 + eps_exp)^…`", and "the spread … appears in each fold's own bound and not in the difference between them" became "and not in their ratio", because `P` is a ratio and the spread does not cancel from a difference. **No derived constant moved, `docs/numerical-semantics.md` was not touched, and the admission rule's shape was not reopened.**

**Fact — the sequential probe now carries both checks under the tree probe's names.** `spikes/numerics/online_softmax_bound_probe.py` renames the published row field `observed_price` to `observed_divergence`, adds `sum_of_bounds` (`B_1 + B_2`), and replaces the single check with two: `divergence exceeds the sum of the two derived bounds` (derived, rigorous) and `divergence exceeds the derived price (heuristic detector)` — the message text, field names, and comment framing matching `online_softmax_tree_bound_probe.py` exactly, so the two probes cannot be read as making different claims. No arithmetic changed: every number in `results.json` is byte-identical to its predecessor.

**Measurement — the run, from the documented commands, repository root.**

```sh
python3 spikes/numerics/online_softmax_bound_probe.py
python3 -O spikes/numerics/online_softmax_bound_probe.py
python3 spikes/numerics/online_softmax_tree_bound_probe.py
python3 -O spikes/numerics/online_softmax_tree_bound_probe.py
```

All four exit 0; both modes byte-identical per probe by `diff`; both committed `results.json` files reproduce the run byte for byte. 22 cases and 91 rows, 0 failures.

**Fact — the identity step, both pins moved together.** `online_softmax_bound/results.json`'s `probe_sha256` moved `4ae534b8a203f61f9840f42de21f493ee701f079c49725c7dc871657245be1ea` → `0ba915749bc09bdc325fbe110e24e5b6d4b669f386dd030c3937ec6bfcf0fb97`. The tree probe **imports** the sequential probe and pins its source digest, so `online_softmax_tree_bound/results.json`'s `imported_probe_sha256` moved to the same value in the same commit; its own `probe_sha256` (`40cafa1916bec…`) did not move and that file changed in that one line and nowhere else, confirmed by diff. Both spike READMEs now state the coupling so the next editor of either probe regenerates both.

**Measurement — the record's cited table re-read against the new file, and nothing moved.** Every number in the certified-bounds record's Part 2 measurement table was regenerated from the new `results.json`: all eight rows' bound-over-observed ratios, divergences, and derived prices are unchanged, the `observed price` column is renamed `observed divergence`, and a `B_1 + B_2` column was added from the new field. The prose figures were re-checked too — `4.8` at `V = 2` and `2.7e29` are both present (the latter at `increasing-v2-span80`, `online_bound_over_observed = 2.774E+29`), and `P = 6.09e-5` at `V = 512` is unchanged. **One cited claim did move, and it was in the tree-fold record rather than this one:** Part 5 said `B_tree + B_2` is "about twice `P` at every shape in the table". Measured, the ratio runs **2.00 to 50.0** over the 77 tree rows with a non-zero price and **2.00 to 83.0** over the sequential probe's 22 cases — twice is the floor, not the typical value, because the shared `exp(A*u)` factor cancels from the ratio `P` and survives in the sum. Part 5 now carries the measured range with the mechanism, and the certified-bounds record quotes the same ranges rather than "about twice".

**Measurement — watched failing, four perturbations in a scratch copy, unperturbed exit 0 before and after each.** Recorded in the spike README, re-run on 2026-08-06 because the added check changed what a defect produces.

| Perturbation | Result |
| --- | --- |
| `fl_exp(fl_sub(previous_peak, peak))` → `fl_exp(fl_sub(peak, previous_peak))`, the sign-flipped rescale factor | exit 1, `48 check(s) failed over 22 cases`: 16 `online error exceeds its derived bound`, **16 `divergence exceeds the sum of the two derived bounds`**, 16 `divergence exceeds the derived price (heuristic detector)`, on the same 16 cases |
| `rewrite_price` divided by 100 | exit 1, `5 check(s) failed`, all the heuristic detector — the rigorous check correctly silent, since understating a constant moves no computed value |
| the `dominant-tail` group removed from `corpus()` | exit 1, `population mismatch: evaluated 20 cases, expected 22` |
| `gamma` asked for `h = 2**24` | `ValueError: gamma is undefined at h=16777216: h*u = 1.0 >= 1` |

**The new rigorous check was watched failing under the sign-flipped rescale factor**, on 16 of 22 cases. Two further perturbations confirm the README's retained "cannot detect" claim against the new check set: removing the elementary factor from `online_bound`, and removing its summation term, both still exit 0.

**Inference — the two checks separate by authority, not by sensitivity, and the README says so.** Because `P < B_1 + B_2` on every row of this corpus, the detector is the strictly more sensitive of the two and the rigorous check catches nothing here it does not. What the rigorous check adds is that on those 16 rows it is a *refutation* where the detector is only a signal; what the detector adds is the 5 catches on a wrong price, which no computed value could reveal. That asymmetry is why the ticket's instruction to retain the detector was right and is now recorded rather than assumed.

**Fact — the tree probe and its README agree with the wording that landed.** The tree probe already checked both quantities under the names this change adopted; nothing in it was edited. Its README's provenance paragraph, which claimed the retained probe "is not edited and its own `results.json` provenance is untouched", now states the import coupling and this change's moved digest. The tree record's Part 5 and its open axis are updated, and the certified-bounds record gained a closed open-axis entry pointing here.

**Fact — two other tickets carried the same conflated phrase and were corrected**, both in `project/tickets`: `reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller` (the ticket Tom's decision reads, "the observed price is exactly zero" → "the observed divergence between the two folds is exactly zero", and "not in their difference" → "not in their ratio") and `derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause`.

**Scope and gate.** The diff touches only `docs/research/numerics/`, `spikes/numerics/`, and `tickets/` — nothing under `crates/`, `prototypes/`, `Cargo.*`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh` — so no Rust gate content changed. `tkt lint` and `git diff --check` clean; `tkt guard` reports the diff inside `research/numerics` and `project/tickets`.
