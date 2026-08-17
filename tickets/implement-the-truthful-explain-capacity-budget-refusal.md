---
id: implement-the-truthful-explain-capacity-budget-refusal
title: Implement the truthful explain-capacity budget refusal
status: in-progress
priority: p1
dependencies: [decide-the-truthful-public-class-for-complete-explain-capacity-refusals]
related: []
scopes: [implementation/compiler, implementation/frontend, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, correctness, explain, public-boundary]
claimed_from: todo
assignee: sol-explain-capacity
lease_expires_at: 1786926435
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

## Exact-base documentation repair — 2026-08-16

The exact-base audit found one live contract sentence made stale by this accepted addition: [`docs/compiler/optimizer.md`](../docs/compiler/optimizer.md), anchor `is the sole authority and reports one of three provenances`, still enumerated only the pre-existing refusal classes. `contracts/optimizer` was therefore added as required scheduling metadata and that paragraph now includes `ConstructionLowerBound` and both report-only explain resources. Those rows report the explain writer's unchanged hard build constants; neither is a `DeterministicBudgets` field, so neither enters canonical request bytes, request identity, or the explain qualifier. This repair does not change the accepted public surface.

## Exact-base Fact audit — 2026-08-16, `7ad48e73c13f3953e67d1c3b95de252bce401498`

The purpose and accepted surface survive. The only drift since the decision audit is unrelated access/lowering identity work and explain schema/renderer versions 11/9 replacing 10/8; neither changes this refusal's authority or payload.

1. **Verified — measured public reachability.** A clean detached exact-base build of `cargo run --release -- request-boundary 7` reproduces six specialists succeeding with 102 invocations, 56 alternatives, 2,291 rendered record lines, and 650,099 rendered bytes. Seven reach 119 invocations and fail as `Some(InvalidCompilerOutput)` with 2,258 lines, 643,313 bytes, terminal reason `compiler-failure:explain-detail-capacity`, and cause 2,256.
2. **Verified — the old public class is false for this path.** `crates/tiler-compiler/src/session.rs`, anchors `This is always a defect in Tiler` and `unreachable by construction from a valid`, reserves `InvalidCompilerOutput` for a Tiler defect. `From<ExplainError> for CompileError` previously routed every explain error through `CompilerOutputError::Explain`, and `class_of` collapsed that wrapper to the defect class.
3. **Verified — complete-or-refused and host protection.** `crates/tiler-compiler/src/explain.rs`, anchor `A trace is complete or it is refused`, retains the 4,096-detail and 1-MiB-canonical-detail ceilings, incrementally encodes the attempted record, withdraws it on refusal, and previously returned unit `ExplainError::DetailCapacity`.
4. **Verified — the accepted earlier aggregation does not settle this population.** [`refuse-nothing-legal-on-the-explain-detail-ceiling`](refuse-nothing-legal-on-the-explain-detail-ceiling.md), anchors `summarized at its source` and `The third is the smallest and the least satisfying`, aggregated exact duplicate cover/region grounds. It neither proves the current provider/plan records duplicate nor permits this valid request to retain a defect class.
5. **Verified — the old typed cause could not populate the accepted payload.** `BudgetResource` had thirteen rows and no explain resource; `BudgetRefusal` had three closed provenances; unit `ExplainError::DetailCapacity` carried no arm, limit, or attempted prefix.
6. **Imprecise, with the accepted decision's repair still correct.** `VerifiedRequestSubject::canonical_explain_subject_bytes` encodes all `DeterministicBudgets` fields, so moving either cap there would move request/evidence identity and the explain qualifier. Adding report-only public resource rows while leaving both caps as unchanged build constants does not. `git diff --unified=0` shows no identity encoder or version constant change.
7. **Verified — public growth and the deliberate exhaustive break.** `CompileFailureClass` and `BudgetResource` are public `#[non_exhaustive]` enums. `BudgetRefusal` is deliberately exhaustive, and `tiler-macros::aot::rendered_refusal` totally maps each provenance, so the new accepted meaning must update that owner before the workspace compiles.
8. **Verified — the internal carrier is scope-bearing.** `compile_candidate_target`, anchor `CompileError::NoFeasiblePlan(_) | CompileError::BudgetExhausted(_)`, treats the old internal budget carrier as candidate-local. `ExplainError` conversion enters `TargetCompileFailure::Outer`; reusing the old carrier could retry a semantic candidate, numerical contract, or target and silently return a partial batch.

## Implementation evidence — 2026-08-16

- The post-change seven-specialist row is `Some(BudgetExhausted { resource: ExplainDetailCanonicalBytes, limit: 1048576, reported: 1048698 })`; its retained trace cardinality, rendered bytes, terminal reason, and terminal cause are unchanged from the exact-base build.
- Independent one-below controls report records as `{ limit: 37, reported: 38 }` and canonical bytes as `{ limit: 100, reported: 101 }`; simultaneous pressure reports records first. An exact-on-both-limits control is admitted.
- Production-subject perturbations failed independently with `the public resource did not preserve the detail-capacity arm`, `the public limit did not preserve the construction limit`, `the public reported value did not preserve the attempted retained prefix`, and `the report-only record resource lost its attempted-prefix provenance`; every mutation was restored before the next control.
- Candidate and contract counters each reached exactly `[capacity]`, not their otherwise viable later candidate/fallback. The target counter reached exactly `[earlier-success, capacity-stop]`, not the later target; the result was `Err`, so the genuine earlier compiled target did not survive in a `CompilationProduct`.
- The real `verify_semantic_output_type` compiler verifier produces `ProgramError::Structure { rule: "semantic-output-type" }` for a valid semantic `u8`-output probe, and that error still projects to `InvalidCompilerOutput`; a non-capacity `ExplainError::StaleIdentity` still enters `CompilerOutputError::Explain`.
- `EXPLAIN_SCHEMA_VERSION` remains 11 and `EXPLAIN_RENDERER_VERSION` remains 9. Both caps, `DeterministicBudgets`, canonical request/explain subject bytes, request qualifier, plan/artifact/cache identities, and provider admission are unchanged.

Verification passed at the working tree that became the implementation commit:

```sh
cargo fmt --all
cargo check -p tiler-compiler -p tiler-macros
cargo nextest run -p tiler-compiler -p tiler-macros # 1,126 passed, 2 skipped
cargo clippy -p tiler-compiler -p tiler-macros --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-compiler -p tiler-macros --no-deps
cargo test -p tiler-compiler --doc # 16 passed, including 14 compile-fail cases
cargo test -p tiler-macros --doc # 0 doctests
cargo run --release -- request-boundary 7 # from the retained calibration spike
tkt lint
make citations
git diff --check
tkt guard tkt/implement-the-truthful-explain-capacity-budget-refusal --base 7ad48e73c13f3953e67d1c3b95de252bce401498 --ticket implement-the-truthful-explain-capacity-budget-refusal --config-ref 7ad48e73c13f3953e67d1c3b95de252bce401498 --explain
```

The exact-base guard reports every changed file under the declared `implementation/compiler`, `implementation/frontend`, `contracts/optimizer`, or shared `project/tickets` scope with no under-declaration. Its verdict is `WARN` because open sibling branches declare broad overlapping areas; those are scheduling warnings rather than a proven file conflict, and the only other live claim in this dispatch shares `project/tickets` without touching this ticket. An independent read-only full-diff review found one medium contract defect in the first draft: macro advice claimed the region had no plan, although construction capacity proves only that compilation returned none. The landed wording makes that distinction and the reviewer returned no remaining correctness findings. The repository-wide `make full` gate is intentionally left to the coordinator's merged batch as directed; this branch changes only the named compiler/frontend/optimizer-contract surfaces and ran their stronger targeted gates above.

## Closes when

The accepted mapping is the only reachable public classification for complete-explain capacity, every internal path preserves request-wide atomic abort semantics, all subject perturbations fail for the intended reason, and the dependent evidence ticket can exercise the exact landed carrier.
