---
id: name-ordering-and-synchronization-in-the-boundary-property-list
title: Name ordering and synchronization in the optimizer boundary-property list
status: done
priority: p2
dependencies: []
related: [implement-boundary-property-model, implement-boundary-property-enforcers]
scopes: [contracts/optimizer]
shared_scopes: []
paths: []
tags: [contract, optimizer, boundary-properties]
---
`implement-boundary-property-model` landed eight typed boundary-property dimensions in `crates/tiler-compiler/src/boundary.rs`. The accepted contract's list is shorter than what the model now implements.

**Fact.** `docs/compiler/optimizer.md` states the boundary-property list with "include" rather than "are", so nothing in it is contradicted by the implementation carrying more. This is an incompleteness, not a conflict, and the distinction decides how it is fixed.

**Fact.** The implemented dimensions are `StorageLayout`, `StorageEncoding`, `Alignment`, `Materialization`, `ExecutionAffinity`, `MemoryDomain`, `Availability`, and `Visibility`. The contract's list is two entries short of these, and the missing pair — ordering and synchronization — is exactly the pair `AGENTS.md` names as physical contracts rather than node annotations.

## Outcome

Complete the accepted contract's initial list with
availability/ordering and visibility/synchronization. Keep the list explicitly
extensible: a new dimension is admitted only when its requirement and guarantee
spaces, satisfaction or subsumption rule, child derivation, dominance,
identity, maturity, and value-preserving enforcer boundary are stated.

**Do not simply mirror the implementation into the contract.** The compiler module is a `pub(crate)` draft under ADR 0074 convention 7 and several of its dimensions carry type-system reservations that reject explicitly rather than implemented support. A contract that lists a dimension states that the optimizer *has* that property; the implementation currently reserves some of them. Distinguish the four maturity claims — type-system reservation, architectural seam, implemented support, tested guarantee — rather than promoting a reservation into a contract sentence.

## Closes when

The initial list names ordering and synchronization, states its admission rule,
does not overstate the maturity of reserved values, and `make full` passes.

## Outcome (2026-07-27)

`docs/compiler/optimizer.md`'s initial boundary-contract list now names availability and visibility, with the admission rule for a further dimension and a maturity table for the two added.

**The maturity distinction was the substantive part, and it is not uniform within a dimension.** The ticket warned against promoting a reservation into a contract sentence; reading `crates/tiler-compiler/src/boundary.rs` shows the split runs *inside* each new dimension rather than between them:

| value | maturity |
| --- | --- |
| availability *after producing dispatch* | implemented and satisfiable |
| availability *after observed host completion* | type-system reservation — ADR 0033 makes host observation a separate boundary and no guarantee in this vocabulary discharges it |
| visibility *readable on the requiring affinity* | implemented and satisfiable |
| visibility *requires an explicit coherence action* | reserved and **deliberately not satisfiable** |

The last is the one worth stating plainly: `VisibilityGuarantee::RequiresExplicitCoherenceAction` exists precisely so that it fails `satisfies`. ADR 0047 makes an affinity-to-domain edge declare its own coherence requirements, so a domain owing a flush or invalidate is guaranteed by its producer and *not* readable by a consumer until an enforcer supplies the action. Modelling it as satisfied at a higher cost is the substitution ADR 0043 forbids. Listing the dimension without that would have read as "coherence is handled".

The contract now says a reserved value is not a weaker form of support — it rejects, and the rejection is the guarantee, because the alternative is a plan silently reading a value nobody made visible.

**The list was incomplete rather than wrong**, as the ticket established: it said "include", so nothing in it was contradicted by the implementation carrying eight dimensions. The two added are the pair `AGENTS.md` names as physical contracts rather than node annotations, which is why their absence mattered more than the count did.

**Admission rule recorded:** requirement space, guarantee space, satisfaction or subsumption rule, child derivation, dominance, identity encoding, maturity class, and the enforcer boundary at which a value-preserving enforcer may discharge the requirement rather than the plan being refused. A dimension without a satisfaction rule is a label; one without an identity encoding is invisible to any consumer comparing two plans.
