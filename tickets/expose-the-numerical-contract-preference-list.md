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

**The decision this ticket carries to Tom.** ADR 0076's second open question is whether an ordered preference list is the right shape at all, or whether one contract plus an explicit caller retry is. The implementation gives evidence the record did not have: the list costs one `Vec` on the request, one resolution loop, and one length-framed run in the subject encoding, and it is what lets the *stated fallback* enter identity — a retry loop cannot, because the compiler never sees the alternatives the caller would have accepted. That is an argument for the list and not a settlement; the public spelling is the point at which it becomes hard to reverse.

## Closes when

Either the public boundary gains a reviewed way to state an ordered preference and both `#[allow(dead_code)]` attributes are removed, or ADR 0076's open question is settled the other way and the crate-internal list is withdrawn with the record amended to say so. `uv run --locked python scripts/check_repository.py` passes.
