---
id: retain-each-plan-alternative-s-verified-semantic-candidate
title: Retain each plan alternative's verified semantic candidate
status: review
priority: p2
dependencies: [retain-the-selected-semantic-candidate-for-the-conformance-oracle]
related: [define-the-composed-realization-driver-subject-bridge, implement-the-composed-realization-evaluation-driver]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, conformance, correctness]
claimed_from: todo
assignee: worker-retain-semantic-candidate
lease_expires_at: 1786643165
---
## User-visible outcome

Every retained physical alternative keeps the exact rewritten semantic program it implements, so a later conformance driver cannot lose the oracle subject or silently substitute the caller baseline.

## Current Fact audit — exact base `b19762f0383d1789ad9c1ad853cd49ce1cfab852`

- **Verified — `SemanticCandidate` is a transient compile-time owner.** `crates/tiler-compiler/src/pipeline.rs`, anchor `struct SemanticCandidate`, holds `proposal: RewriteProposal<SemanticProgram>` plus origin/key/request. It is built in `compile_contract_group` and borrowed by `ExpectedCandidateOwner` while the flattened `ProgramAlternative` is assembled; it is not itself retained.
- **Verified — `build_alternative_for_origin` was the sole `ProgramAlternative` construction site.** `crates/tiler-compiler/src/pipeline/planning.rs`, anchor `pub(super) fn build_alternative_for_origin`. The test helper `build_alternative` delegates to it. No other `ProgramAlternative {` literal exists.
- **Verified — `ProgramAlternative` did not own a candidate.** Same file as the struct, anchor `pub(crate) struct ProgramAlternative`. Fields were stable id, identity, private `owner_key`, kind, plan, scheduled/KIR/artifact evidence, realization, cost, and equivalence. `#[derive(Clone, Debug, Eq, PartialEq)]` was in force.
- **Verified — `ProgramAlternativeIdentity::new` already folds the complete candidate identity.** Anchor `tiler.program-alternative.v2`. It length-frames origin, all five `SemanticIdentity` components, the numerical-contract key, and the selected-plan identity. Retention does not change those bytes.
- **Verified — final owner-binding ignored any retained program.** `verify_global_portfolio`, anchor `rule: "semantic-portfolio-owner-binding"`, re-derived identity from `ExpectedCandidateOwner.semantic` and compared `owner_key` plus stored identity. A swapped retained program would have been invisible.
- **Verified — `Compilation` / `PlanAlternative` expose no semantic-program accessor.** `crates/tiler-compiler/src/session.rs`, anchors `pub struct Compilation` and `pub struct PlanAlternative`. The view is a borrow over the crate-private alternative; its methods are compilation, stable id, fused, kernels, capabilities, physical providers, ABI, delivered realization, and prepared-entry requirements.
- **Verified — existing owner-binding perturbations cover deletion, owner key, and origin only.** `final_portfolio_verifier_rejects_deletion_owner_and_origin_misbinding` mutates `owner_key` and expected origin. It does not perturb a retained program.
- **Verified — `SemanticProgram` is `Clone` over `Arc<ProgramData>` and has no `Eq`.** `crates/tiler-ir/src/semantic/program.rs`, anchors `pub struct SemanticProgram` and `pub(super) data: Arc<ProgramData>`. Keeping a derived `Eq` on `ProgramAlternative` is therefore impossible without a side table or a forbidden pointer comparison.
- **Verified — `semantic(false)` and `semantic(true)` share one `SemanticIdentity`.** Graph identity is canonical; reversing constant construction order does not move it. Perturbations that need a distinct candidate must change constant payloads or use a rewrite (`tensor_add_chain` under the relaxed contract).

## Authority

Tom accepted the storage boundary on 2026-08-12 through [`retain-the-selected-semantic-candidate-for-the-conformance-oracle`](retain-the-selected-semantic-candidate-for-the-conformance-oracle.md): mandatory direct private retention on every `ProgramAlternative`, no standalone public accessor, no fallback, and no identity/schema change.

## Required delivery

- Perform a fresh per-Fact audit at the worker's exact base. Read `SemanticCandidate`, `build_alternative_for_origin`, `ProgramAlternative`, `ProgramAlternativeIdentity::new`, `verify_global_portfolio`, the public `Compilation`/`PlanAlternative` facade, and the ownership perturbation tests in full before editing.
- Retain the exact candidate as an Arc-backed `SemanticProgram` clone directly on every internal alternative. Missing retention must be unrepresentable.
- Re-derive and compare the retained candidate's complete `SemanticIdentity` at the final portfolio owner-binding check. A swapped candidate, a candidate from another owner, or an identity moved without the candidate must fail as invalid compiler output with a named rule.
- Resolve `ProgramAlternative`'s current `Eq` derivation explicitly. Do not compare Arc pointers and do not move retention into a side table merely to preserve a derive.
- Keep `PlanAlternative`'s public surface unchanged. The child bridge/driver owns any later exposure.
- Prove that `tiler.program-alternative.v2`, artifact bytes/identity, cache subjects, and existing successful plan identities do not move.

## Watched failures

Perturb the retained program independently from the existing owner key and identity; swap two candidates with different semantic identities; drop retention from one construction path; and change the retained program while restoring only the outer identity. Quote each failure. A test that edits only its assertion does not count.

## Non-goals

The composed driver, public accessors, reference pinning, split-test provenance repair, artifact serialization, rewrite replay, or cross-process reconstruction.

## Closes when

Every retained alternative structurally owns its verified candidate, the final verifier refuses each independent misbinding, public/API and canonical identities remain unchanged, and targeted compiler checks plus exact-base guard are green.

## Outcome

`ProgramAlternative` now structurally owns its verified `SemanticProgram` in a mandatory private field. `build_alternative_for_origin` clones the candidate (one Arc increment) at the only construction site. `ProgramAlternative` equality is written by hand and compares `semantic_identity()`, never the `Arc` pointer. `verify_alternative` refuses a dropped or swapped construction-time candidate as `portfolio-retained-semantic-binding`. `verify_global_portfolio` re-derives the retained program's complete `SemanticIdentity` and the identity that candidate would mint, still under `semantic-portfolio-owner-binding`. `PlanAlternative` is untouched. `tiler.program-alternative.v2` still has one `src/` occurrence; the governed fixture's successful plan labels remain `program-alternative:db1a4cbc46771083` and `program-alternative:eeaa29a40b81091d`.
