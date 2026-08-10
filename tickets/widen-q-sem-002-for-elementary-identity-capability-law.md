---
id: widen-q-sem-002-for-elementary-identity-capability-law
title: Widen Q-SEM-002 for the elementary-identity capability law
status: in-progress
priority: p3
dependencies: []
related: [accept-adr-0101-elementary-identity-dimension, decide-whether-to-admit-an-elementary-identity-permission]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, open-questions]
claimed_from: todo
assignee: sol-qsem002-elementary-law
lease_expires_at: 1786412164
---
## User-visible outcome

[Q-SEM-002](../docs/open-questions.md) (Built-in algebraic capability declarations) names three capability laws in its close condition: the existing reassociation and commutativity matrix, plus ADR 0101 decision 3's third, *parameterized* elementary-identity capability law (an operation-owned functional equation together with that equation's real-domain side condition). The open-questions index no longer reads as closed solely over the two pre-0101 laws.

## Why this exists

**Fact.** [ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) Open questions states: decision 3 proposes a third, parameterized capability law; while the record was only `proposed`, Q-SEM-002 was deliberately not widened so a proposal would not enter an index of commitments; and **if the record is accepted, widening Q-SEM-002 is part of that acceptance's own sweep** (anchor: `widening Q-SEM-002 is part of that acceptance's own sweep`).

**Fact.** [`accept-adr-0101-elementary-identity-dimension`](accept-adr-0101-elementary-identity-dimension.md) is `done` for the catalog, research disposition, and numerical-semantics dimension definition. It did **not** edit Q-SEM-002. The close condition still reads only "complete operation/dtype/signature reassociation and commutativity matrix with verifier tests."

**Fact.** No typed elementary-identity capability vocabulary has landed: `OperationAlgebraicCapabilities` still carries only `ordered_associativity: bool`. This ticket does not implement that vocabulary; it updates the commitments index so the ADR's accepted decision 3 is visible as close work rather than a dangling self-obligation.

## Required delivery

- Edit Q-SEM-002's close condition in `docs/open-questions.md` so it includes ADR 0101 decision 3's third, parameterized elementary-identity capability law (functional equation + real-domain side condition), without claiming any typed field, permission, or verifier matrix is delivered.
- Optionally point ADR 0101 Open questions at this ticket so the sweep sentence names a board owner rather than implying the widen already rode acceptance.

## Non-goals

- Admitting a typed elementary-identity permission (owned by [`decide-whether-to-admit-an-elementary-identity-permission`](decide-whether-to-admit-an-elementary-identity-permission.md)).
- Implementing `OperationAlgebraicCapabilities` parameterization, verifier tests, or any crate change.
- Reopening or re-accepting ADR 0101; the dimension definition and reservation already stand.

## Closes when

Q-SEM-002's close condition in `docs/open-questions.md` explicitly includes the elementary-identity capability law from ADR 0101 decision 3 (functional equation + real-domain side condition) alongside reassociation and commutativity, and a reader cannot take the close condition as limited to the two pre-0101 laws alone.
