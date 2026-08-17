---
id: decide-the-algebraic-capability-authority-for-contraction-splits
title: Decide the algebraic capability authority for contraction splits
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/ir, implementation/compiler, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, numerics, identity, public-boundary, needs-tom]
---
## User-visible outcome

Tiler decides which operation-owned algebraic evidence makes a fixed contiguous or lane-strided contraction split legal, so a caller's numerical permission can never authorize a regrouping or permutation the operation did not declare sound.

This is a prerequisite discovery/decision ticket, not a decision-ready packet. Do not add it to the decision queue until the Pareto-complete gate below has been independently reviewed.

## Discovery — exact main `155046bb3050e554329e1f45dfe3ece549ed7f31`

**Fact — permission is only half the accepted legality rule.** [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md), anchor `Each transformation requires two independent facts`, and [Numerical semantics](../docs/numerical-semantics.md), anchor `Reassociation requires both an operation capability`, require an applicable operation-declared algebraic capability and an independently resolved numerical permission. Reassociation needs the first for ordered regrouping; permutation additionally needs a commutativity capability.

**Fact — the standard contraction deliberately declares neither capability.** `register_standard_contraction` in `crates/tiler-ir/src/semantic/contraction.rs` supplies `OperationAlgebraicCapabilities::none()`, and `the_contraction_declares_no_algebraic_capability` pins that absence. The current public capability record has only `ordered_associativity`; no commutativity vocabulary exists. The only compiler consumers of `algebraic_capabilities()` are logical normalization rules, not the contraction physical-proposal path.

**Fact — exact implementation attempt `648a372f8cbb306df43a4edfc4e14a6211cac7b1` over `07aca5cd8f67824019d8c183fd3a9584ce84b670` exposed the gap and was not merged.** Its `contraction_split_region` checks only request permissions. Its positive `contraction_membership_permission_is_decided_before_construction` therefore admits a contiguous split for a real standard contraction even though that operation declares no ordered-reassociation capability; the synthetic lane-strided path additionally consumes a commutativity capability nothing can state. Independent review classified both as a P1 semantic-authority blocker. Reproduce the contradiction with:

```sh
cargo test -p tiler-ir the_contraction_declares_no_algebraic_capability -- --nocapture
cargo test -p tiler-compiler contraction_membership_permission_is_decided_before_construction -- --nocapture
rg -n 'algebraic_capabilities\(\)|declares_ordered_associativity' crates/tiler-compiler/src
```

**Fact — the physical carrier is already accepted but does not settle semantic authority.** [`decide-the-fixed-strided-contributor-membership-vocabulary`](decide-the-fixed-strided-contributor-membership-vocabulary.md) accepts `ContributorMembership::{Contiguous, LaneStrided}` and `ReductionTopology::CooperativeContractionSplit`; it says which permissions each topology consumes. It does not supersede ADR 0014 or decide which operation/combiner owns the prerequisite capabilities.

## Required decision packet

- Re-audit the complete contraction definition/registry, `OperationAlgebraicCapabilities`, normalization rules, numerical-contract resolution, schedule verification, compiler proposal path, reference semantics, canonical identity, and every exhaustive capability consumer at the packet's exact base.
- Enumerate the nondominated frontier, including status quo typed refusal, a capability declared on the contraction family, a capability projected from an explicit typed reducer/combiner authority, a narrower contiguous-only surface if it is independently sound, further bounded numerical proof, and deferral. Eliminate any option that infers capability from permission or from a definition-fact string.
- Define exactly what ordered associativity and commutativity mean for the contraction's per-contributor `left * right` followed by F32 addition under its canonical-NaN and subnormal realization. State the dtype/signature/value-domain population and whether the proof is unconditional or realization-specific.
- If a public capability vocabulary changes, fix the exact Rust spelling, exhaustive census, canonical tags/framing, operation-registry identity movement, request/explain/cache cascade, old-byte invariants, refusal precedence, and unsupported population. Do not treat an append-only field as identity-neutral merely because a domain string can stay fixed.
- Specify the verifier/compiler join that requires both the declared capability and the resolved permission before either split proposal exists. Contiguous must name missing ordered-reassociation capability separately from forbidden reassociation; lane-strided must additionally separate missing commutativity capability from forbidden permutation.
- Include independent numerical derivation and subject perturbations for each capability, permission, membership, and identity field. State the strongest counterargument and reversal evidence for every surviving candidate.
- Present at most one concrete Tom question after independent review; until then keep this ticket out of `.ticketsplease/decision-queue.md`.

## Consequences and non-goals

This ticket does not implement either split, change the already accepted physical carrier, grant any numerical permission, admit distributivity/FMA/atomics/nondeterministic arrival, claim device performance, or repair the implementation attempt's separate witness/explanation coverage gap. The dependent implementation ticket retains that repair explicitly.

## Closes when

Tom has accepted an exact public/identity-bearing algebraic authority and the complete operation/dtype/signature matrix needed by both fixed contraction memberships, or has accepted a narrower fail-closed outcome with the excluded membership and reopening trigger stated. The accepted result must leave a mechanically unique verifier/compiler implementation that checks capability and permission independently.
