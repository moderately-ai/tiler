---
id: state-the-region-contract-statement-in-the-frontends-contract
title: State the region's `contract` statement in the frontends integration contract
status: in-progress
priority: p2
dependencies: []
related: [state-the-numerical-contract-in-the-region-grammar, decide-the-inline-frontend-numerical-contract]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, frontend, numerics, inline-dx]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785685340
---
## User-visible outcome

`docs/integration/frontends.md` describes the region grammar a frontend expands, including the `contract` statement every region now states, so a reader of the contract is not told a region has four statements when it has five.

## Why this is a separate ticket

**Fact.** `state-the-numerical-contract-in-the-region-grammar` landed the statement under the `implementation/frontend` scope, which reaches `crates/tiler/**` and `crates/tiler-macros/**` and not `docs/integration/**` — a separate scope (`contracts/integrations`) that the ticket did not hold. The grammar paragraph was therefore left untouched rather than edited off-scope.

**Fact — what the frontend now does**, read from `crates/tiler-macros/src/numerics.rs` at the landing commit: a region states `contract <name>;` in its declaration block beside `sym`, `in`, and `deliver`, at most once; the admissible names are `strict_f32`, `flush_subnormals_to_zero_f32`, `relaxed_f32`, `reassociate_f32`, and `flush_and_reassociate_f32`, each naming the `tiler_compiler::session::NumericalContract` constant of the same name; an unpublished name is refused at the name with the admissible list; a region stating none is refused at the invocation with a diagnostic naming the statement to add; and whether the delivered target can honour the stated contract stays the compiler's own feasibility refusal, reported at the `deliver` keyword.

**Fact — provenance.** Tom decided on 2026-08-01, at the live session, relayed through `decide-the-inline-frontend-numerical-contract`: no default, the region states its contract explicitly.

## Implementation keys

- Sweep `docs/integration/frontends.md` for every sentence whose truth depended on the frontend choosing a contract, not just the grammar listing — the eight-step expansion description names the contract as a frontend input.
- Check whether any other document under `docs/integration/` shows a `tensor!` region; a shown region that states no contract is now a region that does not compile.

## Corrections — 2026-08-02, from reading the two sources this ticket cites

**Fact — there are three refusals, not two.** This ticket's body names the two `crates/tiler-macros/src/numerics.rs` owns (`ContractRefusal::Unstated` at the invocation, `ContractRefusal::UnknownContract` at the name). A third lives one layer up, in `crates/tiler-macros/src/grammar.rs`: `SyntaxError::RepeatedContractStatement`, reported at the second `contract` keyword, because `parse` admits the statement at most once. Reading only the vocabulary module hides it, since a repeat never reaches vocabulary resolution. All three are documented.

**Fact — the eight-step expansion flow does not name the contract as a frontend input**, contrary to this ticket's first implementation key. `## Expansion-time AOT flow` lists parse, construct/verify/normalize/optimize/schedule, emit, identify, look up, compile-on-miss, read, and embed; none of the eight mentions a numerical contract, and step 1 already covers reading the statement. Nothing there needed correcting. The sentences that did are the two grammar listings — `sym`/`in`/`deliver` in the accepted-spelling section and in the fusion-visibility paragraph — plus the two shown regions that stated no contract.

**Fact — no sentence under `docs/integration/` said the frontend chooses the numerical contract.** The near miss is the `## Frontend responsibilities` bullet on resolving "ergonomic accuracy presets into complete canonical per-operation contracts", which is the transcendental-accuracy axis of a source library's intrinsics rather than the region's numerical contract, and it stays true. `docs/integration/candle.md` mentions numerical contracts only as claims about whose kernels compute what, and is untouched.

## Closes when

The frontends contract describes the `contract` statement, its vocabulary, all three of its refusals, and the feasibility split, and no sentence in `docs/integration/` still says the frontend chooses the numerical contract.
