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

## Source audit — 2026-08-10

- **Verified — ADR obligation.** [ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) decision 3 still requires an operation-owned, identity-encoded functional equation with its real-domain side condition (anchor: `It is one dimension, with per-function content carried at two other layers.`), and its Open questions still assigns the Q-SEM-002 widen to acceptance (anchor: `widening Q-SEM-002 is part of that acceptance's own sweep`).
- **Verified — undischarged acceptance remainder.** [`accept-adr-0101-elementary-identity-dimension`](accept-adr-0101-elementary-identity-dimension.md) remains `done` and explicitly records that it `does **not** discharge that open-question self-obligation`; before this change Q-SEM-002 still said `complete operation/dtype/signature reassociation and commutativity` and named no third law.
- **Verified — current absence boundary.** `OperationAlgebraicCapabilities` in `crates/tiler-ir/src/semantic/operation.rs` still has exactly the field `ordered_associativity: bool`; its constructors, consumers, and identity tests name only ordered associativity. The crate census for elementary identity and functional-equation spellings finds explanatory softmax prose but no typed capability, permission, dimension, or verifier. [Numerical semantics](../docs/numerical-semantics.md) still states `No elementary-identity permission is admitted`, and [`decide-whether-to-admit-an-elementary-identity-permission`](decide-whether-to-admit-an-elementary-identity-permission.md) remains `deferred`.

All three ticket Facts are verified at base `03f1c16bb73073ee5b850a9c58a09dbd073ff6ad`; none required repair, and the purpose, public-boundary authority, and identity boundary are unchanged.

## Required delivery

- Edit Q-SEM-002's close condition in `docs/open-questions.md` so it includes ADR 0101 decision 3's third, parameterized elementary-identity capability law (functional equation + real-domain side condition), without claiming any typed field, permission, or verifier matrix is delivered.
- Optionally point ADR 0101 Open questions at this ticket so the sweep sentence names a board owner rather than implying the widen already rode acceptance.

## Non-goals

- Admitting a typed elementary-identity permission (owned by [`decide-whether-to-admit-an-elementary-identity-permission`](decide-whether-to-admit-an-elementary-identity-permission.md)).
- Implementing `OperationAlgebraicCapabilities` parameterization, verifier tests, or any crate change.
- Reopening or re-accepting ADR 0101; the dimension definition and reservation already stand.

## Closes when

Q-SEM-002's close condition in `docs/open-questions.md` explicitly includes the elementary-identity capability law from ADR 0101 decision 3 (functional equation + real-domain side condition) alongside reassociation and commutativity, and a reader cannot take the close condition as limited to the two pre-0101 laws alone.

## Outcome

Q-SEM-002 now names the three capability laws its closure must cover: reassociation, commutativity, and ADR 0101 decision 3's parameterized elementary-identity law, including both the operation-owned functional equation and its real-domain side condition. The wording remains a close condition: it does not claim that a typed capability, permission, numerical-contract dimension, verifier, domain proof, or implementation has landed.

## Verification

- `git diff --check` passed.
- `tkt lint --format json` returned `ok: true` with no diagnostics.
- `make citations` resolved 1,189 pinned citations and 6,449 local links across the live population.
- The source-safe crate census `rg -n -i 'functional equation|elementary.function identit|elementary.?identity|identity capability|real-domain side condition' crates --glob '*.rs'` returned four explanatory occurrences, all in `crates/tiler-ir/src/semantic/softmax.rs`; reading those sites confirms that none is a typed capability, permission, dimension, domain proof, or verifier.
- The full gate is carried from green commit `0b0e6952aaa6c88f7c7be923c3158adba9d86add`: `git diff --name-only 0b0e6952` names only `docs/document-metadata.md`, `docs/open-questions.md`, `docs/operation-extensions.md`, and ticket files. None is in the repository's gate-carry exclusion set, and this change reran the required `tkt lint` and `make citations` checks.
