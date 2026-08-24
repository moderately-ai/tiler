---
schema: "tiler-doc/v1"
id: "tiler.research.verification.conformance-claim-universe-by-owner"
kind: "research"
title: "The closed-world conformance claim universe by owner"
topics: ["verification", "conformance", "identity", "registries", "compiler", "runtime"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.correctness-and-testing"]
ticket: "inventory-the-closed-world-conformance-claim-universe-by-owner"
---

# The closed-world conformance claim universe by owner

**Status:** complete owner audit; system universe remains open

**Reviewed:** 2026-08-24 at `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a`

## Result

**Fact.** Tiler has no singular, closed, owner-emitted conformance universe at this base. The retained [36-row owner matrix](../../../spikes/verification/conformance-claim-universe-by-owner/inventory.tsv) separates ten demonstrated fail-loud typed vocabularies from bounded base snapshots and explicitly unknown populations. A current registry can be exact about what it contains without proving that every system subject must enter it; this report no longer conflates those statements.

**Fact.** The system universe is distinct from a goal profile. Semantic definitions, compiler rules, verifier invariants, runtime outcomes, and retained performance claims are system subjects. A goal profile later selects obligations and evidence standards; it cannot define subjects out of existence. Tests and receipts are evidence, not feature rows.

**Fact.** No stable candidate system-universe identity is derivable today. The TSV checksum binds only one raw file. It is not a canonical projection and says nothing about field authority, unknown encoding, or revision policy. Those are unresolved by [the authority/change-policy ticket](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md).

## Exact-base Fact audit

| ticket Fact | verdict | source anchor and meaning |
| --- | --- | --- |
| `FrozenSemanticRegistry::operation_definitions` exposes a canonical exact semantic-operation inventory | **imprecise** | [`semantic/registry.rs`](../../../crates/tiler-ir/src/semantic/registry.rs), `FrozenRegistryData` and `operation_definitions`: exact for the frozen registry instance, but no independent owner manifest proves all system semantic operations must enter that instance. |
| `FrozenReferenceRegistry` retains exact operation/signature capabilities without a public iterator | **verified, narrowly** | [`reference/registry.rs`](../../../crates/tiler-reference/src/registry.rs), `FrozenReferenceRegistry`: ordered capability and validator maps are folded into registry identity, with no public capability iterator. This is an exact instance snapshot, not a closed system denominator. |
| `RuleRegistry::rules` is the complete canonical compiler-private rewrite inventory | **false** | [`rewrite.rs`](../../../crates/tiler-compiler/src/rewrite.rs), `RuleRegistry::rules`, and [`pipeline.rs`](../../../crates/tiler-compiler/src/pipeline.rs), `canonical_semantic_baseline_rule`: construction is split across a pipeline baseline, CSE, and conditional algebraic registries. Four production identities are source-known; compiler-wide population is unknown. |
| `tiler-conformance` owns cross-layer evidence rather than layer-local meaning/tests/performance | **verified** | [`tiler-conformance/src/lib.rs`](../../../crates/tiler-conformance/src/lib.rs), anchor `owns portable cross-layer conformance orchestration and machine-checkable evidence`, plus its explicit exclusions. |
| root-spike counts and paths must be reproduced | **verified with corrected commands** | The retained [reproduction record](../../../spikes/verification/conformance-claim-universe-by-owner/README.md) executes the intended source buckets and returns `484/182/152/115/94/265/137/954/10/5`. The rejected draft's different commands were false even where their recorded numbers happened to be intended values. |

The first and third verdicts repair ticket premises without changing the inventory goal: current-container exactness is weaker than closed-world completeness, and fragmented construction is positive evidence of an unknown denominator.

## Demonstrated exact typed vocabularies

Only these rows are called complete. Each was independently widened with a temporary `AuditProbe` subject and rejected by the owner build; exact commands and diagnostic text are retained in the [perturbation record](../../../spikes/verification/conformance-claim-universe-by-owner/README.md#executed-subject-perturbations).

| family | population | fail-loud mechanism |
| --- | ---: | --- |
| lowering families | 1 | exhaustive `LoweringFamily` mappings (`E0004`) |
| feasibility capability axes | 7 | `CANONICAL_AXES`/`variant_count` assertion (`E0080`) and exhaustive mappings |
| budget resources | 15 | `variant_count`-sized `ALL` (`E0308`) and exhaustive mappings |
| explain dispositions | 16 | exhaustive owner mappings and legal-event census (`E0004`) |
| program availability phases | 5 | exhaustive ABI tags and consumers (`E0004`) |
| numerical dimensions | 11 | exhaustive key/tag/behaviour mappings (`E0004`) |
| Metal-AOT compile stages | 2 | `variant_count`-sized `ALL` (`E0308`) |
| compiler call-failure stages | 7 | exhaustive key/fallback/ordinary mappings (`E0004`) |
| runtime route failures | 7 | exhaustive fallback/display/source mappings (`E0004`) |
| cache publication kill phases | 9 | `variant_count`-sized `KILL_POINTS` (`E0308`) |

The cache phase row is a test vocabulary providing crash evidence; it is not a cache feature row and does not close the cache obligation universe.

## Bounded snapshots and unknown populations

The audit corrected six ownership/identity errors that would otherwise poison later identity work:

- the semantic catalog owns 34 definitions keyed by `ValueTypeDefinitionKey`; strict-affine is a separate quantization-owned definition, not a 35th catalog row;
- `tiler-cache`, admitted by [ADR 0082](../../../docs/decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md), owns cache identity/publication;
- `TargetProfile` is in `tiler-compiler`, and `tiler.target-profile.declaration.v11` is distinct from `tiler.target-profile.descriptor.v11`;
- `tiler.artifact-program.stage.v4` is the artifact stage identity, separate from `tiler.kernel-program.v13`; and
- physical-provider provenance uses `tiler_ir::semantic::ProviderIdentity` with namespace, name, and nonzero output-affecting revision; changing provider output steps that revision, and two revisions are distinct installable identities; and
- rewrite identities number four in the bounded production source search, while physical strategy constants number six with the `_STRATEGY` suffix. Neither search closes its system family.

Current owner snapshots remain useful but are not complete censuses: 19 semantic operations; 34 catalog definitions; one quantization definition; 19 algebraic declaration fields with two positive ordered-associativity declarations; 17 index laws; 28 reference capabilities; seven reference validators; 22 governed lowering capabilities; four feasibility outcomes; 14 deterministic budget fields; and four budget-refusal variants. Each row names its construction, identity, refusal, and missing owner-manifest check.

The `BudgetRefusal` negative control is especially important: adding a fifth variant left `cargo check -p tiler-compiler` green, producing only a missing-doc warning. Therefore a closed enum and a remembered count are not themselves completeness evidence.

Entire obligation populations remain unknown for compiler-wide rewrites, extensible physical providers/strategies, feasibility predicates, schedule/KIR/program verifiers, artifact/ABI/proof/publication, machine-wide execution outcomes, runtime completion, global target declarations/descriptors, cache contract obligations, and retained performance claims. No measurement-record count is retained as authority.

## Decision-packet gate

| candidate | verdict | reason |
| --- | --- | --- |
| status quo inference from tests/grep/one build | eliminated | silently omits private, conditional, dynamic, and prose subjects |
| one hand-maintained authority manifest | eliminated | a new owner subject can land without changing it |
| owner-derived typed/registry manifests | frontier destination | subject additions can fail compilation/construction and identities stay with owners; unresolved private boundaries need decisions |
| bounded source census | frontier bridge only | useful exact-base evidence when vocabulary is stated, never a future negative proof |
| blanket deferral | dominated for demonstrated vocabularies; retained for unresolved lanes | it discards real fail-loud owner evidence, while explicit unknown rows preserve correctness |

The nondominated transition is hybrid: consume demonstrated owner vocabularies, create owner manifests where current snapshots stay green, and refuse any overall completeness result while an unknown row remains. The strongest counterargument is distributed maintenance. Evidence that would reverse this conclusion is an existing singular owner whose iterator demonstrably rejects an undisposed subject from every matrix family; the row-by-row audit found none.

## Follow-up graph

Immediate owner work remains in the existing optimizer/planner, structural, stage, explain, and receipt tickets. Decision-blocked boundaries remain in:

- [`decide-how-owner-private-conformance-inventories-cross-crate-boundaries`](../../../tickets/decide-how-owner-private-conformance-inventories-cross-crate-boundaries.md);
- [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md);
- [`derive-artifact-proof-and-publication-conformance-obligations`](../../../tickets/derive-artifact-proof-and-publication-conformance-obligations.md);
- [`derive-runtime-route-completion-and-cache-obligations`](../../../tickets/derive-runtime-route-completion-and-cache-obligations.md); and
- [`define-retained-performance-claim-authority-and-identity`](../../../tickets/define-retained-performance-claim-authority-and-identity.md).

No row invents a public boundary or selects the first goal profile. Unknowns remain explicit prerequisites of [`assemble-the-first-versioned-conformance-goal-profile`](../../../tickets/assemble-the-first-versioned-conformance-goal-profile.md).

## Evidence boundary

This is a source-derived owner audit, not an executable conformance suite. The raw TSV checksum is provenance only. No production API, schema, identity domain, or goal profile changed. The report is complete as an audit of named families at the named base; it explicitly does not claim that the system universe is closed.
