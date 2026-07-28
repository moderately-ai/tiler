---
id: expose-the-numerical-contract-preference-list
title: Expose the numerical contract preference list on the public compiler boundary
status: done
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

## Outcome (2026-07-27)

`CompileRequest::preferring(program, contracts)` states an ordered preference and refuses an empty list with a `CompileFailureClass::InvalidRequest`. `CompileRequest::new` is now that path with one entry, so the two cannot drift. Both `#[allow(dead_code)]` naming this ticket are gone: `NumericalContractPreference::ordered` and `VerifiedTargetRequest::numerical_contracts` are both on the live path.

Readers: `Compilation::stated_numerical_contract_keys` and `resolved_numerical_contract_key`.

**Keys rather than the public `NumericalContract` enum, and the reason is a correctness one.** Mapping a resolved contract back onto that enum needs an inverse of `NumericalContract::resolve`, and every total spelling of it absorbs an unrecognized key into one of the two variants — a silently wrong answer about which numerics a program was compiled under, in an accessor a caller would reasonably trust. ADR 0076 already makes the key a contract's governed name, so the key is what identifies one. This was the second design attempted; the first carried the enum and was withdrawn when the inverse turned out to need a fallback.

**Test covers what a caller-side retry could not.** The stated list survives compilation in full, not just the winner; a reversed list is a different stated list even though it names the same two contracts; and an empty list is refused as an invalid request. The middle assertion is the one that matters — it is what proves order reaches the request subject rather than being normalized away, which is exactly what a caller looping over contracts itself could never establish.

## Surface added by `implement-first-profile-numerical-policies`, for review here

That ticket widened the contract and added the named preset family. None of what it added falls in ADR 0075's always-ask categories — no new `pub mod`, no new `pub trait`, no `pub(crate)`→`pub` promotion, and no source-breaking change to an existing public signature — so it landed without approval. It is recorded here because this ticket owns the public numerical-contract spelling and should not be reviewed against a surface that has since moved.

**`tiler_compiler::session` — the preset names.** `NumericalContract` gains `RelaxedF32`, additive growth of an already-`#[non_exhaustive]` enum. Its `resolve` now routes through the crate-internal `policy::NumericalPolicyPreset` table rather than naming each constructor again, so a preset registered in one and not the other fails to compile. The three public names now denote `tiler.strict-f32.v1`, `tiler.flush-f32.v1`, and `tiler.relaxed-f32.v1`.

**`tiler_ir::schedule` — new public behaviour-space types, all in the existing public module.** `ArithmeticType` (`F16`/`Bf16`/`F32`/`F64`, with `canonical_type_key` returning the same namespaced versioned spelling a `TypeKey` renders), `ValueDomainProvenance`, `ExceptionalValueAssumption`, `ApproximationEnvelope`, and `MaterializationRounding`. `NumericalRealization` is **unchanged** — it still carries four dimensions and its constructor's signature is untouched, which is what kept `tiler-metal` and `tiler-metal-aot` out of the change.

**`tiler_reference` — the oracle can be told a contract.** `ReferenceNumericalConformance` and `UnsupportedReferenceContract` are new public types; `ReferenceEvaluator::{under, standard_under, conformance}`, `IndexRegionEvaluator::{under, conformance}`, `ReferenceEvaluationRequest::conformance`, and `ScalarReferenceRequest::conformance` are new public items. `new` and `standard` keep their signatures and bind the strict reading, which is what those constructors already computed.

**What is still crate-private and still owed here.** `NumericalContractPreference`, `CompilationRequest`, and `pipeline::compile` are unchanged and unexposed. (The branch's original text also claimed the two `#[allow(dead_code)]` attributes naming this ticket were still in place — that was true on its stale base and is false on the merged tree, where the Outcome above records them gone and the items live.) `crate::policy` is a private module under ADR 0074 convention 7; whether `NumericalPolicyPreset` should be the public preset type instead of `session::NumericalContract` is a question for this ticket's review, and the two currently duplicate a three-name vocabulary that one type could carry.
