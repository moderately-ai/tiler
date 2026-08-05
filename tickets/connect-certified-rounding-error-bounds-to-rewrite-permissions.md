---
id: connect-certified-rounding-error-bounds-to-rewrite-permissions
title: Connect certified rounding-error bounds to rewrite permissions
status: review
priority: p2
dependencies: []
related: [derive-the-capability-set-for-search-discovered-flash-class-attention-kernels, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller, name-the-elementary-identity-rewrite-dimension, expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate, derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound, tighten-the-rescaling-bound-with-the-sharpened-summation-constants]
scopes: [research/numerics, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-certified-bounds
lease_expires_at: 1785971429
---
## User-visible outcome

A rewrite whose worst-case rounding error is *certified* below a caller-stated tolerance can be admitted on that certificate, so numerical permissions stop being all-or-nothing grants and become quantitative — "the streaming softmax costs at most this many ULPs under this contract" — which is the mature form of the fail-closed differentiator: SOTA kernels change bits silently; this system would change them under a proven bound the caller priced.

## Why this exists

**Fact.** The contract vocabulary today admits a reassociation only by categorical permission (and [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) declined one such permission on the record). **Fact.** The machinery to do better half-exists: `tiler-reference`'s certified enclosure arithmetic bounds elementary functions with watched-failing precision refusals, and `spike-hermetic-fptaylor-certificate-checking` sits deferred with its trigger log. **Inference.** The unowned connection is the admission rule: derive a bound for the rewritten fold, check it against the contract's stated tolerance, admit or refuse on the certificate — with the certificate itself validated, not trusted.

## The literature-survey obligation

Preserve and read the primary sources per the source-record discipline: **FPTaylor** (the deferred spike's subject — this ticket may fire that deferral's trigger; check its log and say so), **Gappa**, **Daisy** (Darulová), **Herbie** (rewriting *for* accuracy — the inverse search, and its error estimation is directly reusable), **Precimonious/HiFPTuner** for the tuning-adjacent half, and the classical analysis this all rests on: **Higham's rounding-error analysis of summation** (the pairwise/blocked/reassociated fold bounds are exactly the streaming-softmax question in its simplest form). The survey states, per tool: what it bounds, what it cannot (input domains, transcendentals, conditionals), certificate checkability, and licence/preservability.

## What the record must decide or defer

Whether bound derivation is per-rewrite-rule (the rule ships with a parametric bound, instantiated per shape) or per-instance (derived at compile time — cost question); what the tolerance vocabulary on the numerical contract looks like without breaking any existing key (the BF16 sibling-domain precedent is the shape for additive contract vocabulary); what validates a certificate at admission time and what refuses an unverifiable one; the worked example — the online-softmax rescaling bound derived by hand from Higham-style analysis, as a Measurement-labelled sanity anchor for whatever tool the survey selects. Deferral with triggers is the expected outcome for the tooling choice; the worked example and the admission-rule shape are the parts likely to resolve now.

## Non-goals

Implementing tolerance vocabulary in `crates/`; accepting any contract change (contract text moves are proposals here); running FPTaylor beyond what the deferred spike's own trigger discipline admits; any claim that a bound exists for a rewrite nobody has derived one for.

## Closes when

The record exists under `docs/research/numerics/` with preserved sources, the admission-rule shape is stated with its trust boundary, the hand-derived softmax bound anchors the tool question, the FPTaylor deferral's trigger log carries this ticket's verdict on whether it fired, and every open axis ends in a filed ticket or a deferred question with a trigger.

## Scope additions and why they are required

**`contracts/navigation`, added 2026-08-05.** Landing a research record obliges the change that adds it to reconcile the catalog that indexes it, and `ticketsplease.toml` maps both catalogs this change touches — `docs/research/README.md` and `spikes/README.md` — to `contracts/navigation` rather than to any research scope. Read from the config rather than asserted: the `contracts/navigation` entry lists both paths explicitly. The precedent is exact — [`survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature`](survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature.md) declared `scopes: [research/region-search, contracts/navigation]` for the same reason, and its landing commit `542a2999` touched `docs/research/README.md` and `spikes/README.md` beside its record. This is declaration and scheduling metadata, not a product-scope expansion: the catalog rows describe work already authorized by this ticket's own outcome.

**Disjointness against the one live holder, verified rather than assumed.** [`admit-bf16-into-the-schedule-and-kernel-vocabulary`](admit-bf16-into-the-schedule-and-kernel-vocabulary.md) is the only non-terminal ticket holding `contracts/navigation` with an assignee (`agent-bf16-vocab`). It has produced no branch — `git rev-parse --verify tkt/admit-bf16-into-the-schedule-and-kernel-vocabulary` fails — so there is no branch diff to collide with, and its status is `todo` rather than `in-progress`. The two catalog edits this ticket makes are one added row in each generated block, both naming this ticket's own record and spike; a BF16 vocabulary landing would touch different rows in the same files, which is an ordinary textual merge rather than a semantic conflict. Recorded here because a scope another ticket holds is admissible only with the check written down.

**`contracts/numerics` was deliberately *not* added.** [Numerical semantics](../docs/numerical-semantics.md) is the normative owner of any tolerance vocabulary, and this ticket's non-goals forbid a contract change. The record's Part 4 is therefore a proposal identified for Tom in the shape the BF16 record used, and no contract sentence moved. Adding the scope would have made a contract edit reachable, which is precisely what the non-goal exists to prevent.
