---
id: prototype-semantic-normalization
title: Implement bounded semantic normalization
status: todo
priority: p0
dependencies: [prototype-typed-explain-infrastructure, correct-reference-value-and-authority-contracts]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, normalization]
---
Introduce the deterministic normalization stage before region formation. The
first profile may be identity-only or contain a deliberately tiny proved rule
set, but it must establish termination, traversal order, budgets, semantic and
reference revalidation, transactional failure, canonical identity, and typed
explain records. Normalization must not imply the later alternative-producing
rewrite engine.

This stage owns one relocated obligation. Tom accepted on 2026-07-18 that
identical referentially transparent operation invocations normalize to one
semantic value before computation identity — equality requiring the same
operation key, operands, canonical attributes, numerical contract, and inferred
result types, with source origins preserved for explanation but excluded from
equality. ADR 0064 later placed common-subexpression elimination outside
commitment compaction and in "existing later layers"; Tom confirmed on
2026-07-23 that this relocated the obligation rather than cancelling it, and
that this normalization stage is its home. Implementing it here is in scope
whenever the first profile's proved rule set admits it; deferring it is
acceptable only with an explicit note recording that the obligation remains
open and unowned elsewhere. Physical planning may still recompute a shared
value independently when that is cheaper than reuse or materialization.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

`NormalizeSemantics` is implemented in `crates/tiler-compiler/src/normalize.rs` and runs in `compile` after `VerifySemanticRequest` and before region formation, replacing the placeholder `normalize.serial-sum.v1` explain record.

**What the first profile normalizes.** Exactly one proved rule: identical referentially transparent operation invocations collapse to one semantic value. Two occurrences are equal when they share an operation key, canonical attributes, ordered operand identities taken after congruence, and ordered inferred result types (`ResolvedValueType` plus `Shape`). Eligibility requires the frozen registry to declare the operation family `OperationEffect::Pure`. Source origin — declaration position, handles, and graph ownership — is excluded from equality and preserved for explanation as the canonical/merged operation ordinals in the typed records. This discharges the obligation Tom accepted on 2026-07-18 and relocated here on 2026-07-23; it is a semantic-identity normalization only, and physical planning remains free to recompute a shared value when that is cheaper than reuse.

**What the first profile does not normalize.** Nothing else. It does not resolve axis names or ellipses, canonicalize reductions or output-axis policy, compose permutations or split/merge chains, canonicalize broadcast/repeat mappings, eliminate identity reindexes or no-op casts, fold or canonicalize constants and dtypes, or remove dead values — `SemanticProgramBuilder::build` already compacts to output-reachable structure, so dead-value removal is not this stage's job. It never reorders, reassociates, or contracts floating-point arithmetic, and it never produces alternatives: the alternative-producing engine remains `implement-transactional-rewrite-engine`.

**Guarantees established.** Termination is structural — detection is one forward pass over a finite verified operation list with no fixpoint loop. Traversal order is verified topological order by ascending graph-local ordinal, results by ascending result position, so the earliest occurrence of a congruence class is always its representative. The explicit budget `DeterministicBudgets::normalization_rewrites` bounds committed rewrites; exhaustion abandons the entire rewrite and keeps the verified input, so a budget never yields a partially canonicalized graph, and it emits a typed `BudgetStop` at `ExplainStage::Normalization`. Failure is transactional: the input `SemanticProgram` is immutable and never mutated, and a candidate is adopted only after every postcondition passes. Semantic revalidation is unconditional — the candidate is rebuilt through the ordinary checked `SemanticProgramBuilder`, so the frozen authority re-infers and re-validates every operation, and the rewritten program independently re-enters `verify_request`. Postconditions additionally check operation count, ordered input/output interface keys, shapes and types, preserved reached definitions, admission provenance and registry snapshot, changed graph identity, and the declared fixpoint. Every failure is `CompileError::InvalidCompilerOutput(CompilerOutputError::Normalization(_))` — a hard error, never a silent fallback. Canonical identity of the normalized result is its `SemanticIdentity`, which flows into the request subject and therefore into explain identity.

**Reference revalidation boundary.** Reference equivalence is proven by checked differential tests in `normalize.rs`, not inside the stage. `tiler-reference` is an executable oracle whose cost is proportional to the materialized element count, and the compiler admits programs with billions of elements (an existing pipeline test compiles `[70_000, 70_000]`), so evaluating every rewrite at compile time is not a viable contract. It is also a dev-dependency of `tiler-compiler`; promoting it to a build dependency would invert the oracle's direction. This is a deliberate deviation from a literal "reference revalidation in-stage" reading and is recorded here for review.

**Collateral change.** The serial-sum strategy recognizer previously required exactly five operations. Because normalization can legitimately produce the four-operation shared-constant form, the check now bounds the count to the recognized range up front and then pins it against the exact set of distinct operations the structural walk reached. That is strictly stronger than the previous constant — it rejects any operation outside the recognized structure — while admitting the normalized spelling of an already-admitted program. Adding `normalization_rewrites` to `DeterministicBudgets` changes the explain compilation subject, so the golden request qualifier in `explain.rs` was updated.

**Verification.** `uv run --locked python scripts/check_repository.py` passes. 65 `tiler-compiler` tests pass, including bitwise reference-differential equivalence over duplicated constants with signed-zero, NaN, and infinity payloads; convergence of the duplicated and pre-shared spellings on one graph identity and one compiled portfolio; idempotence; determinism; budget abandonment; and typed causally chained explain records.
