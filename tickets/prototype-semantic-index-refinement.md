---
id: prototype-semantic-index-refinement
title: Verify semantic-to-index refinement
status: done
priority: p0
dependencies: [prototype-operation-capability-registry, prototype-generic-region-formation]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-foundation, refinement]
---
Verify capability output against exact semantic occurrences and canonical index
regions. Bind ordered values and accesses, numerical/effect evidence, scalar
authority, reached definitions, selected-provider provenance, and reusable
content separately from occurrence identity. Registration or successful
builder construction alone is not refinement evidence.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

**Fact.** Implemented the index-region refinement authority (the "compiler-owned legality evidence" of `docs/ir.md` Layer 2 and the `LowerIndexRegions` boundary of `docs/compiler/optimizer.md`) as a new draft module `crates/tiler-compiler/src/legality.rs`, composing the merged `capability.rs` (provider resolution and canonical-builder emission) and the `region.rs` content-vs-occurrence identity discipline. `refine_index_region` drives a resolved `IndexAccess` provider through the canonical `IndexRegionBuilder`, structurally verifies the emitted region, and then independently proves it realizes the exact `SemanticOccurrence`.

**Fact — what refinement proves beyond "a provider produced a valid region".** A successful builder construction is never accepted as evidence. After the region builds, refinement independently checks: (1) the resolved capability is the `IndexAccess` family, operation, and ordered signature of the occurrence; (2) the ordered operand values (with aliasing) match the region's input boundaries by element type and shape, and every ordered result matches an output root's tensor and written-value type; (3) each result is backed by a complete unique write (`WriteOwnershipProof`); (4) the region revalidates under the scalar authority and its reached provider-independent scalar definitions equal exactly the authority the capability was admitted to emit; (5) the capability's operation authority and the region's scalar/type authority agree on the semantic snapshot; (6) the effect is pure. A well-formed region with the wrong shape, wrong output arity, or that reaches an undeclared scalar operation is rejected with a typed error.

**Fact — content separate from occurrence.** `RefinementContent` (region canonical identity, operand aliasing pattern renumbered to first-occurrence-local positions plus types/shapes, result interface, effect, numerical-contract identity, provider-independent reached scalar/semantic definitions, and authority snapshots) carries its own `RefinementContentIdentity` and is site- and provider-independent. `IndexRefinement` binds that content to the opaque `SemanticOccurrenceIdentity` (the selected semantic source), the selected-provider `ProviderIdentity`, the capability revision, and provider-attributed admission provenance, yielding a distinct `IndexRefinementIdentity`. A test proves two occurrences at different sites lowered by one provider share content identity but differ in occurrence identity, mirroring `region.rs`.

**Fact — checkable against the reference oracle.** Refinement's structural/authority binding is what makes the candidate output checkable: a test refines the pointwise-square occurrence and then evaluates the same `VerifiedIndexRegion` through the independent `tiler-reference` `IndexRegionEvaluator`, feeding the input boundary named by `operand_bindings`, and confirms `out[i] = in[i]*in[i]`.

**Fact — determinism.** All identity bytes use length-prefixed big-endian encodings over deterministic `Vec`/first-occurrence orderings and the existing deterministic `tiler-ir` projections; no `HashMap` iteration enters any identity or diagnostic.

**Fact — tests.** Nine module tests pass, including the negative cases (wrong result shape, extra output/result arity, undeclared scalar authority, wrong family, operation mismatch), the oracle-checkability case, and a case that refines a real region-formation occurrence identity. The full `scripts/check_repository.py` gate and `tkt guard` pass.

**Fact — scope.** Edits are confined to `crates/tiler-compiler/` (`legality.rs`, `lib.rs`) and this ticket. No `tiler-reference`, `tiler-ir`, or docs changes were required; `tiler-reference` is used only as an existing dev-dependency in tests.

**Draft boundaries requiring Tom's review.** The new `pub mod legality` and its public items are drafts, not a stable API: `refine_index_region`; `SemanticOccurrence`/`OccurrenceOperand`/`OccurrenceResult`/`OccurrenceValueId`/`SemanticOccurrenceIdentity`/`NumericalContractIdentity`; `IndexRefinement`/`RefinementContent`/`RefinementContentIdentity`/`IndexRefinementIdentity`; `OperandBinding`/`ResultBinding`; and `RefinementError`. The module is public only so the not-yet-wired authority is reachable, mirroring the sibling `pub mod capability`; it takes an opaque `SemanticOccurrenceIdentity` rather than a `region.rs` type to keep that private module encapsulated.

**Deferred (out of scope, follow-up triggers).** Reached-scalar-authority conformance requires exact equality of the declared and reached projections; broadening to a subset relation (a provider that conditionally emits fewer scalar ops) needs the reached key sets exposed and is deferred. Refinement covers the `IndexAccess` family only. Selecting covers, physical implementations, schedules, and cost remain later stages.
