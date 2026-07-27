---
id: expose-the-numerical-contract-preference-list
title: Expose the numerical contract preference list on the public compiler boundary
status: todo
priority: p2
dependencies: []
related: [compose-numerical-honourability-and-retire-the-strict-boolean]
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, numerics, api]
---
`compose-numerical-honourability-and-retire-the-strict-boolean` implemented ADR 0076 item 2's ordered caller preference list at the compilation-request boundary and stopped there, because everything that would make it reachable from outside the crate is a new `pub` item in `tiler_compiler::session` and therefore Tom's to review under ADR 0075.

**Fact — what exists.** `NumericalContractPreference` in `crates/tiler-compiler/src/request.rs` is `pub(crate)`, with `exactly` and `ordered` constructors; `CompilationRequest::governed_preferring` builds a request from one; `verify_request` resolves it per target by the caller's stated order against that target's honourability declaration; the stated list is bound into `VerifiedRequestSubject::canonical_explain_subject_bytes` beside the resolved entry, so two requests that resolve alike but declare different fallbacks are different requests; and `VerifiedTargetRequest::numerical_contracts` reads it back. Four tests in `request.rs` cover order-determinism, single-entry equivalence, subject separation, and the unhonourable-preference rejection.

**Fact — what is missing.** `session::compile_governed(program, contract: NumericalContract)` takes exactly one contract. No public path states a list. `NumericalContractPreference::ordered` and `VerifiedTargetRequest::numerical_contracts` each carry a targeted `#[allow(dead_code)]` naming this ticket as the reason, so removing the allow is part of closing it.

## Approved implementation outcome

Expose a nonempty ordered list of acceptable numerical contracts. Preserve
caller order in request identity and expose both the stated list and resolved
choice to readers. A caller-side retry cannot substitute because the compiler
would not see or identify the alternatives the caller accepted.

## Closes when

The public boundary accepts the approved ordered preference, readers expose the
stated and resolved values, both targeted `#[allow(dead_code)]` attributes are
removed, and `make full` passes.

## Decision — Tom, 2026-07-25

**Approved: promote.** The capability is implemented, resolved, subject-bound and tested; only the public spelling was missing. Remove the two `#[allow(dead_code)]` that name this ticket — implemented-and-tested capability sitting unreachable behind a dead-code allow is exactly the state that decays into nobody knowing whether it still works.
