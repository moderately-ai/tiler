---
id: separate-the-rescaling-price-from-the-observed-fold-divergence
title: Separate the rescaling price from the observed fold divergence
status: in-progress
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
