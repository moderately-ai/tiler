---
id: carry-the-invocation-gather-requirement-through-refinement
title: Carry the invocation-gather requirement through refinement
status: todo
priority: p1
dependencies: [carry-the-gather-relation-through-the-compiler-vertical, lower-a-recognized-gather-through-a-governed-capability]
related: [admit-an-invocation-scoped-gather-index-validation-receipt, decide-the-data-dependent-index-representation-public-surface, carry-the-gather-relation-through-the-compiler-vertical]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, gather, validation, fail-closed, public-boundary, identity]
---
## User-visible outcome

A valid dynamic gather reaches the exact accepted invocation-validation requirement at the IR and compiler refinement boundary and stops under `gather-invocation-validation-required`; it is no longer mislabeled as fully refined and cannot mint timeless proof, executable coverage, a schedule, an artifact, or dispatch authority.

## Exact-base Facts — `6e713e12`

- **Fact — the accepted types are absent.** `rg -n 'InvocationGatherIndexValidationRequirement|PendingInvocationIndexValidation|GatherValidationRequirementMismatch' crates/` returns no matches. The accepted packet at `InvocationGatherIndexValidationRequirement exposes exactly` fixes its fields, accessors and identity domain, and the following code block fixes both third outcome arms.
- **Fact — the current verifier overlooks the intrinsic gather requirement.** `ResolvedIndexRealization::verify_sequence` in `crates/tiler-ir/src/index/refinement/verify.rs` decides only `unknown_index_domain_predicates`; with none, it mints `IndexRefinementReceipt` and `IndexRefinementExecutableCoverageIdentity` even when `GatherIndexBoundsResolution::InvocationValidationRequired` remains on the verified access.
- **Fact — the compiler has only two outcomes.** `IndexRefinementOutcome` in `crates/tiler-compiler/src/legality.rs` is `Refined` or `Pending`, and `refine_index_region` maps the matching two IR arms. The accepted packet requires `PendingInvocationIndexValidation` and the stable `gather-invocation-validation-required` stop before scheduling.
- **Fact — the predecessor recorded this as unlanded remainder.** `Remaining work - not landed here` in [`admit-the-selected-data-dependent-index-representation`](admit-the-selected-data-dependent-index-representation.md) names the exact four symbols and reason this ticket owns, but no prerequisite node isolated that compiler carrier before this repair; the broad receipt target bundled it with the still-undecided artifact and runtime layers.

## Exact implementation contract

Implement only the already accepted refinement surface: `InvocationGatherIndexValidationRequirement` and its opaque v1 identity; `GatherValidationRequirementMismatch`; `IndexRefinementVerificationOutcome::InvocationValidationRequired`; `PendingInvocationIndexValidation`; `IndexRefinementOutcome::InvocationValidationRequired`; all accepted accessors and exhaustive consumers; and the stable compiler stop after successful dynamic refinement. Derive and cross-check the requirement from the exact occurrence, region-local gather requirement, ordered `[source, index]` operand bindings, and one result binding. A mismatch is an IR verification error, never a pending proof or compiler plan miss.

Do not invent conditional schedule coverage, artifact carriage, snapshot bytes, a runtime receipt, cache identity, or dispatch. This ticket ends at the exact accepted wall and hands its concrete type to the conditional-coverage decision.

## Closing checks and negative controls

- Size both refinement outcome populations from `core::mem::variant_count`; widening either two-arm enum without adding every total consumer must fail the build.
- Perturb access, occurrence/refinement subject, source binding, index binding, result-binding count/order, region-local requirement identity, axis, shapes, extent, and logical types independently with unchanged assertions; each must report `GatherValidationRequirementMismatch` at the accepted precedence.
- Change a valid dynamic gather's outcome back to `Verified` without changing the test. It must fail by showing that no `IndexRefinementReceipt`, `IndexRefinementExecutableCoverageIdentity`, or `CoveredOccurrence` may exist for this population.
- Change a static proof to the invocation arm and a dynamic requirement to the static arm independently; each must fail at its own exact expected outcome.
- Run the IR and compiler package tests, Clippy with warnings denied, rustdoc with warnings denied, `tkt lint`, `git diff --check`, and exact-base `tkt guard`. Record the type census, failure text of every subject perturbation, exact commit/base, and unsupported cases.

## Closes when

The accepted third-arm carrier is implemented and tested through the named compiler refusal, every generic receipt/coverage construction is unreachable for it, no later layer is entered, and the exact landed hash is a dependency of the conditional-coverage decision.
