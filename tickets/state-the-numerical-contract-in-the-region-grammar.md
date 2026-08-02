---
id: state-the-numerical-contract-in-the-region-grammar
title: State the numerical contract in the region grammar, explicitly
status: in-progress
priority: p1
dependencies: []
related: [decide-the-inline-frontend-numerical-contract, denote-a-reduction-region-in-the-inline-macro-grammar, calibrate-and-activate-parallel-reduction-selection]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, inline-dx, numerics, public-boundary]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785634941
---
## User-visible outcome

A `tensor!` region states the numerical contract it compiles under, in its own text, and a region that states none is refused at expansion with a diagnostic naming what to write — no silent default chooses a program's numerical meaning.

## Decision provenance

Tom decided on 2026-08-01 at the live session, recorded on [`decide-the-inline-frontend-numerical-contract`](decide-the-inline-frontend-numerical-contract.md): avoid assuming defaults and be specific; sane defaults can return later when the project's shape is known. The bound declaration admits two contracts (`FLUSH_SUBNORMALS_TO_ZERO_F32` and `FLUSH_AND_REASSOCIATE_F32`), the two are different meanings rather than two settings, and the choice belongs to the consumer's text.

## Implementation keys

- The spelling is a **public syntax surface** and goes to Tom under ADR 0075 as a concrete draft, exactly as the `strict_serial_sum` trio did. Prefer a declaration-block statement beside `sym`/`in`/`deliver` naming the contract's constant-style name; the grammar denotes a *contract*, never a plan.
- Only contract names the composed vocabulary defines are admitted, refused at the token otherwise with the admissible names listed; whether the target honours the stated contract stays the compiler's feasibility question, refused downstream with its own typed diagnostic — the grammar does not pre-answer it.
- A region that states no contract is refused at expansion — after this lands there is no `CONTRACT` constant left, and the refusal names the statement to add. Update every in-tree consumer (tests, trybuild goldens, the facade pass/fail suites, `deliver_compiles_embeds_and_routes`) to state their contract explicitly; the goldens gain the refusal case watched failing.
- `tiler_macros::aot` module documentation records the decision, its date, and venue; `the_bound_declaration_admits_the_two_flushing_contracts` keeps pinning the admissible pair.

## Closes when

A region states its contract and compiles under it, a contract-less region is refused with the naming diagnostic (watched failing first), an unknown contract name is refused at the token, Tom has accepted the syntax, and targeted tests plus the batch gate pass.
