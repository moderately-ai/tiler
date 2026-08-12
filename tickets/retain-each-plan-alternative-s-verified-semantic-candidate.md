---
id: retain-each-plan-alternative-s-verified-semantic-candidate
title: Retain each plan alternative's verified semantic candidate
status: todo
priority: p2
dependencies: [retain-the-selected-semantic-candidate-for-the-conformance-oracle]
related: [define-the-composed-realization-driver-subject-bridge, implement-the-composed-realization-evaluation-driver]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, conformance, correctness]
---
## User-visible outcome

Every retained physical alternative keeps the exact rewritten semantic program it implements, so a later conformance driver cannot lose the oracle subject or silently substitute the caller baseline.

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
