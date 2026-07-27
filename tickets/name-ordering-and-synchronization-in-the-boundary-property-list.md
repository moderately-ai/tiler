---
id: name-ordering-and-synchronization-in-the-boundary-property-list
title: Name ordering and synchronization in the optimizer boundary-property list
status: todo
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
