---
id: name-ordering-and-synchronization-in-the-boundary-property-list
title: Name ordering and synchronization in the optimizer boundary-property list
status: in-progress
priority: p2
dependencies: []
related: [implement-boundary-property-model, implement-boundary-property-enforcers]
scopes: [contracts/optimizer]
shared_scopes: []
paths: []
tags: [contract, optimizer, boundary-properties]
claimed_from: todo
assignee: agent-optimizer-list
lease_expires_at: 1785042783
---
`implement-boundary-property-model` landed eight typed boundary-property dimensions in `crates/tiler-compiler/src/boundary.rs`. The accepted contract's list is shorter than what the model now implements.

**Fact.** `docs/compiler/optimizer.md` states the boundary-property list with "include" rather than "are", so nothing in it is contradicted by the implementation carrying more. This is an incompleteness, not a conflict, and the distinction decides how it is fixed.

**Fact.** The implemented dimensions are `StorageLayout`, `StorageEncoding`, `Alignment`, `Materialization`, `ExecutionAffinity`, `MemoryDomain`, `Availability`, and `Visibility`. The contract's list is two entries short of these, and the missing pair — ordering and synchronization — is exactly the pair `AGENTS.md` names as physical contracts rather than node annotations.

**The question this ticket must answer before editing anything:** whether the list is deliberately open ("include") because the property set is expected to grow with targets, or whether it was intended as closed and the wording is loose. Those lead to different edits. An open list should say so normatively and state what admits a new member; a closed list should be completed and closed. Do not convert one into the other silently — `AGENTS.md` requires accepted decisions, proposals, and future work to stay visibly distinct.

**Do not simply mirror the implementation into the contract.** The compiler module is a `pub(crate)` draft under ADR 0074 convention 7 and several of its dimensions carry type-system reservations that reject explicitly rather than implemented support. A contract that lists a dimension states that the optimizer *has* that property; the implementation currently reserves some of them. Distinguish the four maturity claims — type-system reservation, architectural seam, implemented support, tested guarantee — rather than promoting a reservation into a contract sentence.

## Closes when

The list either names ordering and synchronization or states why they are excluded; the open-versus-closed question is answered normatively; each named dimension's maturity is not overstated; and `uv run --locked python scripts/docs.py render` and the full gate pass.
