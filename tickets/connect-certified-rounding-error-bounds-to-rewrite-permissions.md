---
id: connect-certified-rounding-error-bounds-to-rewrite-permissions
title: Connect certified rounding-error bounds to rewrite permissions
status: in-progress
priority: p2
dependencies: []
related: [derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [research/numerics]
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
