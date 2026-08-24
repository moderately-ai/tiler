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

**Status:** complete source inventory; candidate universe only

**Reviewed:** 2026-08-24 at `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a`

## Result

**Fact.** Tiler does not yet have one closed, owner-emitted conformance universe. It has several exact sub-universes and several submerged or extensible families whose complete populations cannot be recovered without inventing authority. The retained [owner matrix](../../../spikes/verification/conformance-claim-universe-by-owner/inventory.tsv) therefore carries 25 positive family rows: exact populations where the owner exposes one, and an explicit `unknown` plus a follow-up owner everywhere else. An unknown row is part of the candidate universe; it cannot disappear merely because a source search found nothing.

**Fact.** The system universe is not a goal profile. Semantic definitions, compiler rules, verifier invariants, runtime outcomes, and retained performance claims enter the former. A later profile selects and qualifies a subset; it does not define what exists. Tests and cross-layer receipts are evidence about claims, not feature rows.

**Inference.** The only fail-closed path is a hybrid transition: use owner-derived typed or registry manifests for exact families; retain bounded source censuses only as labelled bridges; and refuse an overall completeness verdict while any family remains `unknown`. A hand-maintained list can be a review aid, but cannot become the authority.

**Proposal.** Treat the matrix byte stream as candidate-system-universe input, not an accepted public format. Its current SHA-256 is `192aafe506547f4ec9d730cf9ff2c5b1a869dfa23cfa038e29d9759c10f4487a`. The normative universe authority and change policy remain a separate decision.

## Per-Fact audit before the inventory

The ticket's Facts were re-read against their complete owners before this report was written.

| ticket Fact | verdict | source evidence |
| --- | --- | --- |
| `FrozenSemanticRegistry::operation_definitions` is a canonical exact semantic-operation inventory | **verified** | [`registry.rs`](../../../crates/tiler-ir/src/semantic/registry.rs), symbols `operation_definitions` and `FrozenRegistryData`; the iterator walks the frozen `BTreeMap`, and the same data enters semantic-registry identity. |
| `FrozenReferenceRegistry` retains exact operation/signature capabilities without a public iterator | **verified** | [`registry.rs`](../../../crates/tiler-reference/src/registry.rs), symbols `FrozenReferenceRegistry`, `references`, and `validators`; exact keys are retained in ordered maps and folded into registry identity, but there is no public capability iterator. |
| `RuleRegistry::rules` is the complete canonical compiler-private rewrite inventory | **imprecise** | [`rewrite.rs`](../../../crates/tiler-compiler/src/rewrite.rs), symbol `RuleRegistry::rules`, is exact only for one instantiated registry. [`request.rs`](../../../crates/tiler-compiler/src/request.rs) constructs a CSE registry and conditionally constructs a separate algebraic registry. There is no construction-independent compiler-wide owner. The ticket wording is repaired by this verdict and the matrix's `compiler.rewrite-rules` unknown row. |
| `tiler-conformance` owns cross-layer executed evidence, not layer-local meaning, tests, or performance | **verified** | [`lib.rs`](../../../crates/tiler-conformance/src/lib.rs), anchor `owns portable cross-layer conformance orchestration and machine-checkable evidence`; its explicit exclusions match the ticket. |
| root-spike counts and paths must be reproduced at this base | **verified and reproduced** | The commands in the retained [reproduction record](../../../spikes/verification/conformance-claim-universe-by-owner/README.md) produce 484 semantic tests, 182 reference source tests, 152 reference integration tests, 115 compiler pipeline tests, 94 compiler external tests, 265 schedule tests, 137 kernel tests, 954 backend-side tests, ten Metal conditional early returns, and five Metal-AOT conditional early returns. These are evidence envelopes, not the feature denominator. |

The corrected rewrite premise does not change the ticket's purpose. It changes one row from falsely exact to explicitly unknown.

## Exact owner populations

The detailed construction, consumption, refusal, identity, revision, profile, and perturbation fields are in the TSV. The exact owner populations at this base are:

| family | exact population | why the vocabulary is complete |
| --- | --- | --- |
| standard semantic operations | 19 definitions | `FrozenRegistryData.operations` is the owning ordered map and `operation_definitions` exposes every row. |
| standard semantic type-definition families | 35 definitions | The owning construction consists of 27 `BUILT_IN_SCALARS`, one complex family, six microscaling schemes, and one strict-affine family. These are definition families, not the unbounded set of parameterized instances. |
| algebraic declarations | 19 operation declarations, two positive ordered-associativity declarations | Algebra is a field of every exact standard operation definition; absence means no permission. |
| index-realization laws | 17 laws | The frozen owner map is exact and its owner test fixes the standard population. The boundary is crate-private. |
| reference capabilities | 28 operation/signature rows and seven value validators | The frozen reference owner retains both complete ordered maps. The capability population expands three registration loops; source call-site count alone is not the population. |
| lowering | one `LoweringFamily` and 22 governed capabilities | The family is a closed owner enum internally; the frozen capability owner retains the exact governed vector and asserts its population. |
| search budgets | 15 resources, 14 deterministic budget fields, four refusal variants | `BudgetResource` uses `variant_count`-sized exhaustive evidence; the request record and refusal enum are fixed typed owners. |
| explain dispositions | 16 | `ExplainDisposition` has a `variant_count`-sized exhaustive test population. This does not make all explain events enumerable. |
| numerical policy dimensions | 11 | `CANONICAL_DIMENSIONS` is sized by the owner's `DIMENSION_COUNT`; dimensions are input/result subnormals, contraction, reassociation, permutation, signed zero, reciprocal, approximations, NaN, infinity, and materialization rounding. |
| local compilation and route stages | two AOT `CompileStage` variants, seven compiler `CallFailureStage` variants, seven runtime route failures | Each local type is exhaustive in its owner. They do not close the machine-wide stage or completion universe. |
| top-level cache contract | five properties | The accepted contract requires complete identity, validation on every hit, immutable entries, atomic-rename publication, and defined crash/race behavior. Detailed refusal and invariant rows are not enumerated. |

The feasibility ruleset is identified as `tiler.feasibility.phased-capability-and-numerical-honourability.v7@1` and declares seven quantitative axes, synchronization, subgroup atomics, and all eleven numerical dimensions. Its result vocabulary is `Proven`, `Rejected`, `Unknown`, or `Deferred`; those words are not interchangeable.

## Explicitly unknown or open populations

These are unsupported as claims of completeness today:

- compiler-wide rewrite rules: three governed rule identities are source-known, but separate registry construction prevents a complete owner census;
- physical providers and strategies: one governed provider and six governed strategy strings are present, while callers may install providers and provider implementations own their strategies;
- complete feasibility obligations: the declared capability classes are known, but there is no owner-emitted obligation row per predicate and phase;
- all explain events and invariants beyond the exact 16 dispositions;
- schedule, KIR, and kernel-program vocabularies and intrinsic verifier invariants;
- artifact program/ABI, proof-sidecar, codec, and publication obligations;
- machine-wide compilation stages and outcomes beyond the exact local enums;
- runtime completion claims: route fallback is exact, but adapter completion is an associated type with no common owner identity;
- global target-profile instances: each checked descriptor is exact, while the system population is dynamically constructed and authority is profile-specific;
- detailed cache refusal and publication invariants beyond the five accepted top-level properties; and
- retained performance claims: 46 spike records declare `bounded-measurement`, but metadata enumerates records, not the individual performance claims in their prose.

The six source-known physical strategy names and the 46 measurement-bearing records are bounded censuses, not complete feature universes. The inventory states their search vocabulary and deliberately refuses to turn them into authority.

## Decision-packet gate

| option | verdict | decisive reason |
| --- | --- | --- |
| status quo: infer from tests, registries reached by one build, or grep | **eliminated** | It silently omits private, conditional, dynamically installed, and prose-retained claims. |
| one manual manifest as authority | **eliminated** | A new owner feature can land without changing the list; correctness would fail open. |
| owner-derived typed/registry manifests | **frontier destination** | New variants or registrations can fail compilation, construction, identity, or a population assertion. It requires boundary/owner decisions for private and fragmented families. |
| bounded source census | **frontier bridge only** | It can preserve a base-specific snapshot when its vocabulary, expansion rules, and subject perturbation are explicit. It cannot prove a future global negative. |
| defer the whole inventory | **eliminated for exact rows; retained for unresolved lanes** | It preserves correctness but throws away exact owner information. Explicit unknown rows plus bounded descendants dominate blanket deferral. |

The nondominated result is therefore the retained hybrid matrix. It admits no scalar “complete” outcome while any `unknown` row remains. Its strongest counterargument is maintenance cost across many owners. Evidence that could reverse the recommendation would be a single existing authoritative owner whose exact iterator covers all 25 families; the full source audit found none. A negative control is to add one rule through a separately constructed registry: any purported global manifest that stays unchanged is fail-open.

## Candidate identity derivation

The current TSV is the retained snapshot and is intentionally simple. A future owner-derived encoding should use a distinct domain such as `tiler.conformance.system-universe-candidate.v1\0`; sort rows by ASCII `family_id`; length-frame the family id, owner identity/revision, enumeration class, and population descriptor; then encode either exact ordered subject identities or an explicit `Unknown(followup-ticket-id)` marker. The goal profile is not an input.

This yields useful properties before a normative authority is chosen:

1. reordering construction sites cannot change the universe;
2. changing an owner revision or exact subject changes the candidate identity;
3. an unresolved family contributes positive bytes and cannot vanish as “zero found”; and
4. choosing a goal profile cannot rewrite system history.

The proposed domain and encoding are not an accepted schema or public boundary. [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md) owns that decision.

## Follow-up graph

Immediate work whose owner is already clear:

- [`derive-the-optimizer-and-planner-capability-obligation-manifest`](../../../tickets/derive-the-optimizer-and-planner-capability-obligation-manifest.md) — rewrite, lowering, physical planning, feasibility, budgets, schedule/KIR/program, and measured-cost obligations;
- [`derive-the-five-family-structural-conformance-manifest`](../../../tickets/derive-the-five-family-structural-conformance-manifest.md) — the already scoped structural family;
- [`classify-machine-compilation-and-execution-outcomes-by-stage`](../../../tickets/classify-machine-compilation-and-execution-outcomes-by-stage.md) — cross-backend stages and conditional early returns;
- [`make-explain-dispositions-assertable-by-a-conformance-suite`](../../../tickets/make-explain-dispositions-assertable-by-a-conformance-suite.md) — owner-supported disposition observation; and
- [`define-the-canonical-conformance-receipt-join-and-freshness-model`](../../../tickets/define-the-canonical-conformance-receipt-join-and-freshness-model.md) — evidence joins after owner manifests exist.

Decision-blocked work:

- [`decide-how-owner-private-conformance-inventories-cross-crate-boundaries`](../../../tickets/decide-how-owner-private-conformance-inventories-cross-crate-boundaries.md) — index laws, reference rows, lowering capabilities, and explain dispositions cannot acquire a consequential public surface here;
- [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md) — normative universe identity and revision policy;
- [`derive-artifact-proof-and-publication-conformance-obligations`](../../../tickets/derive-artifact-proof-and-publication-conformance-obligations.md) — artifact/ABI/proof/publication owner manifest;
- [`derive-runtime-route-completion-and-cache-obligations`](../../../tickets/derive-runtime-route-completion-and-cache-obligations.md) — runtime completion and detailed cache/publication owner manifest; and
- [`define-retained-performance-claim-authority-and-identity`](../../../tickets/define-retained-performance-claim-authority-and-identity.md) — claim-level identity inside retained measurement records.

No ticket here chooses the first goal profile. [`assemble-the-first-versioned-conformance-goal-profile`](../../../tickets/assemble-the-first-versioned-conformance-goal-profile.md) remains downstream of the universe and authority work.

## Evidence boundary

This is a source-derived inventory, not an executable conformance suite and not a performance measurement. Every exact row carries a subject perturbation design in the TSV and the reproduction record. No production API, identity domain, schema, or goal profile changed. The report's exactness is bounded to the named base; a future suite must consume manifests from the owning layers and make new subjects fail loud.
