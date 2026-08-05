---
id: derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound
title: Derive the tree-fold form of the online-softmax rescaling bound
status: todo
priority: p3
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions, derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy, reductions]
---
## User-visible outcome

The online-softmax rescaling bound covers the fold shape a real flash-class kernel uses — a tree of `(m, d)` pair merges across lanes and workgroups — rather than only the sequential recurrence the published algorithm states.

## Why this exists

**Fact.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) derives the bound for Algorithm 3 exactly as Milakov and Gimelshein state it: a sequential scan, giving `2(V-1)` roundings and `V` calls to `exp` along the worst path. It says explicitly that the derivation is the sequential case.

**Fact.** A parallel realization does not have that shape. It merges partial states pairwise: `(m, d) merge (m', d') = (max(m, m'), d*exp(m - max) + d'*exp(m' - max))`, which is what the two-level subgroup and workgroup reduction work would compose.

**Inference.** The bound does not transfer by substituting `ceil(log2 V)` for the height. A merge applies **two** multiplies, **two** exponentials, and one add, where a sequential step applies one multiply, two exponentials, and one add — so the per-level operation count differs. More importantly the telescoping argument that made the logit spread cancel must be redone, because along a tree path the rescale arguments are differences between *partial* maxima rather than a chain of consecutive ones. The telescoping is likely to survive, since the partial maxima along any root path are still non-decreasing, but **that is a conjecture in this ticket and not a result** — it is exactly the step a careless derivation would assume, and assuming it is how a bound becomes wrong rather than loose.

## What this ticket must produce

- The bound for a balanced binary merge tree of height `h`, with the per-merge operation count derived rather than assumed, and the telescoping argument either re-established or replaced.
- The comparison against the sequential form. A tree fold should be *cheaper* in `gamma` (height `log2 V` rather than `V-1`) and *more expensive* in `exp` count per path, and which term dominates is the answer a scheduler needs.
- The unbalanced case, or an explicit statement that the bound is stated for balanced trees and that an unbalanced one takes the height of its deepest path.
- An extension of [the existing probe](../spikes/numerics/online_softmax_bound_probe.py) to the tree fold, with its corpus counted against a declared literal and at least one watched failure, matching the discipline the sequential probe already carries.

## Non-goals

Selecting a schedule; implementing a kernel; deriving bounds for fold shapes no capability set names.

## Closes when

The tree-fold bound is derived and checked, the record's sequential-only caveat is replaced by a statement covering both, and any divergence between the two forms is stated with the shape that decides which is preferable.
