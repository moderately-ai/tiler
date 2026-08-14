---
id: implement-the-truthful-explain-capacity-budget-refusal
title: Implement the truthful explain-capacity budget refusal
status: todo
priority: p1
dependencies: [decide-the-truthful-public-class-for-complete-explain-capacity-refusals]
related: []
scopes: [implementation/compiler, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, correctness, explain, public-boundary]
---

## User-visible outcome

An otherwise valid compile request that reaches a complete-explain retention ceiling returns `CompileFailureClass::BudgetExhausted` naming the exact report-only explain resource, the build's limit, and the attempted retained-prefix lower bound. It no longer falsely reports `InvalidCompilerOutput`.

## Accepted surface

Tom accepted this exact public boundary on 2026-08-14 in the live Codex conversation, relayed by the coordinating agent through [`decide-the-truthful-public-class-for-complete-explain-capacity-refusals`](decide-the-truthful-public-class-for-complete-explain-capacity-refusals.md):

- retain `CompileFailureClass::BudgetExhausted { resource, limit, reported }`;
- add report-only `BudgetResource::{ExplainDetailRecords, ExplainDetailCanonicalBytes}`;
- add closed `BudgetRefusal::ConstructionLowerBound`;
- keep both existing explain limits as build constants outside `DeterministicBudgets` and request identity; and
- preserve explain capacity as an outer, request-wide atomic abort with no candidate, numerical-contract, or target fallback and no partial output.

For the record arm, `limit = 4_096` and `reported = retained_detail_records + 1`. For the byte arm, `limit = 1_048_576` and `reported = retained_detail_bytes + encoded_refused_record_bytes`. `reported` is the exact attempted retained prefix including the refused record and only a lower bound on the complete trace demand.

## Required work

- Re-audit every decision Fact and every error-conversion consumer at the exact implementation base before editing production source. Repair drift; stop if it changes the accepted surface.
- Extend `ExplainError::DetailCapacity` with the exact arm, limit, and attempted-prefix quantity at the construction check. Preserve record-first precedence when one attempted record exceeds both ceilings.
- Preserve the typed `DetailCapacity` source through a distinct internal carrier that remains `TargetCompileFailure::Outer` through candidate, numerical-contract, and target orchestration. Map only that carrier to the accepted public `BudgetExhausted` payload at the session boundary.
- Do not route the refusal through `RequestError`, reuse or generalize the current candidate-local `CompileError::BudgetExhausted`, retry another candidate or contract, continue to another target, or retain an earlier target as partial output.
- Keep genuine verifier, ledger, identity, provider-authority, event-class, and stale-identity explain failures mapped to `InvalidCompilerOutput`.
- Add the two public resource keys and the closed refusal provenance, update the total macro renderer and public documentation, and explain that the reported value is a lower bound rather than required capacity.
- Keep the terminal trace record, both ceiling values, `DeterministicBudgets`, canonical request/evidence bytes, request qualifier, explain schema/renderer versions, plan identity, artifact identity, and cache identity unchanged.

## Required negative controls

- Force the record and byte arms independently one unit below their attempted prefix, then force both simultaneously to prove deterministic record-first precedence.
- Perturb only resource, limit, reported, and `ConstructionLowerBound` in turn; each unchanged public assertion must fail with the moved subject named.
- Force capacity in the first of multiple otherwise viable semantic candidates and before a viable fallback numerical contract; prove later work is never reached.
- Force capacity after an earlier target could succeed; prove no later target is reached and no earlier success survives in a partial compilation.
- Drive a genuine verifier-produced compiler-output error and prove it remains `InvalidCompilerOutput`.

## Non-goals

Changing either explain ceiling, selecting an active-provider cardinality, promising full provider-slot activity, adding request budget fields, compacting or truncating the trace, changing provider admission, or refactoring the general candidate-local budget authority.

## Closes when

The accepted mapping is the only reachable public classification for complete-explain capacity, every internal path preserves request-wide atomic abort semantics, all subject perturbations fail for the intended reason, and the dependent evidence ticket can exercise the exact landed carrier.
