---
schema: "tiler-doc/v1"
id: "ADR-0072"
kind: "decision"
title: "Separate semantic meaning from provider provenance"
topics: ["identity", "ir", "extensions", "artifacts"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.operation-extensions", "tiler.contract.artifact-abi"]
evidence: ["tiler.research.extensions.semantic-foundation-api-v2", "tiler.research.artifacts.target-neutral-envelope"]
ticket: "correct-semantic-identity-layering"
---

# 0072: Separate semantic meaning from provider provenance

**Status:** accepted

## Context

The first semantic implementation included the reached registry-provider
projection in `SemanticProgram` identity. That made a provider revision appear
to change tensor meaning even when the graph, resolved types, operation keys,
attributes, shapes, sharing, and named interface were identical. The same
coupling would make pure index or schedule structure depend on where it occurs
and which compiler implementation admitted it.

Provider revisions remain correctness-critical. They identify the authority
whose inference and validation admitted a graph and the capabilities selected
to transform or realize it. They are provenance and cache-invalidating inputs,
but they are not semantic graph meaning.

## Decision

Tiler assigns distinct, domain-separated identities to distinct subjects:

- `SemanticGraphIdentity` covers only canonical graph meaning and its ordered
  interface. It excludes provider revisions and unrelated registry entries.
- `SemanticDefinitionProjectionIdentity` covers the provider-independent type
  and operation definitions reached by that graph.
- `SemanticAdmissionProvenanceIdentity` covers the reached provider identities
  and revisions whose mandatory semantic capabilities admitted the graph.
- `SemanticRegistrySnapshotIdentity` covers the complete frozen registry
  environment. It is compilation-request provenance, not graph meaning.

`SemanticProgram` owns these four typed subjects in one non-forgeable
`SemanticIdentity` bundle with named borrowed accessors and no public
constructor. Private compiler request, target, and artifact-construction types
retain that bundle atomically; they inspect an individual subject only when a
verification rule needs its particular equality. A provider-only revision
therefore preserves `SemanticGraphIdentity` and the reached-definition
projection while changing admission provenance and the registry snapshot. A
change to graph meaning requires changed canonical graph bytes, normally
through a new semantic type or operation key version when the specification
itself changes.

Later layers follow the same rule:

- region semantic content is separate from the region occurrence and exact
  graph-value boundary binding;
- index-region identity covers pure symbolic index/access structure;
- selected scalar-definition authority revalidates an exact index region and
  yields a separate region-bound receipt containing reached
  provider-independent definitions and provider-attributed admission
  provenance;
- a compiler-owned checked refinement binds index structure to one region
  occurrence, exact value/access mappings, reached definitions, selected
  providers, and required evidence;
- schedule and structured-kernel identities cover their canonical structural
  content and refine the immediately lower structural layer;
- a complete program identity covers the semantic graph, bound region
  implementations, coverage, dependencies, materializations, buffers, ABI,
  guards, and routing; and
- manifest, envelope, and runtime-cache identities add the selected artifact,
  target, toolchain, payload, and live-context provenance appropriate to their
  exact subjects.

Whole-program semantic identity must not be nested inside pure index, schedule,
or structured-kernel content identity. Equal structural content may be reused
at several occurrences; the checked binding and complete plan preserve where
and how it is used. Selected provider identities remain in refinement, plan,
and artifact provenance. Unused providers may affect the compilation-request
environment but do not poison an otherwise identical selected artifact.

The dependency direction is:

```text
semantic keys and descriptors -> semantic graph meaning
registry providers -> admission and compilation-environment provenance
region content + occurrence + index structure -> checked refinement
index structure -> schedule structure -> structured kernel structure
semantic graph + bound implementations + complete coverage + buffers + ABI + guard + routing commit -> kernel program
kernel program + selected provenance + target/toolchain/payload -> artifact
artifact + live device/context/specialization -> runtime cache
```

## Consequences

- Semantic equivalence and provider provenance can be tested and cached for
  their actual subjects instead of through one overloaded digest.
- Provider upgrades invalidate admission and selected implementation evidence
  without falsely changing a graph's tensor meaning.
- Structural index, schedule, and kernel content can be shared across graph
  occurrences while occurrence-specific verification remains explicit.
- Scalar registry revisions can be rechecked against the same structural index
  region without changing that region's identity; definition changes and
  provider-only changes remain distinguishable evidence subjects.
- Every cache or artifact key must compose the identities and provenance needed
  by its own contract; no lower-layer digest is a substitute for complete plan
  coverage or selected-provider evidence.
- The semantic identity split and the index region's separate scalar-authority
  receipt are implemented for their bounded profiles. The completed semantic
  program owns definition and admission subjects computed by one deterministic,
  iterative, cycle-safe transitive authority closure with incremental root and
  unique-subject bounds. Bound values remain private implementation policy while
  typed failures expose the resource, active limit, and rejected count.
  Borrowed program validation runs the same closure and
  preserves a typed registry-error source;
  compiler requests and artifact-construction plans retain their non-forgeable
  four-subject bundle atomically. Region occurrence, semantic checked refinement,
  schedule, structured-kernel, complete-program, and artifact identities are now
  implemented for the bounded profiles, with each retaining its distinct subject.
- The complete-program identity of this decision's list is implemented for the bounded profile. `CanonicalKernelProgramIdentity` folds the semantic graph, every stage's bound implementation, complete canonical stage ownership (proof-bound occurrence claims with their realization-stage ordinals, or exact publication claims), materializations, buffers, the entry ABI, the applicability guard, and the routing-commit lifecycle. `complete-program-identity-with-abi-guards-and-routing` moved the domain from `tiler.kernel-program.v1` to historical v2 because the last three subjects were previously invisible; later encoding and ABI-completeness changes moved the same subject through v3 and v4 to v5, folding the declared split-reduction contracts moved it to v6, replacing storage-order coverage with canonical semantic occurrences moved it to v7, folding the published outputs in the semantic interface's order rather than sorted by record content moved it to v8, binding each covered occurrence to its refinement evidence moved it to v9, folding the declared publishing-copy contracts moved it to v10, and folding the declared staged-realization contracts moved it to v11. Complete owner grammar — including the exact split occurrence and each continuation ordinal — moves the current program identity to `tiler.kernel-program.v12`. The portfolio-level sense of "routing" is a different subject and is not folded in: a variant's priority rank orders variants against each other, and one program in isolation has no rank to carry.
- The `v9` step is where this decision's separation is load-bearing rather than descriptive, and the choice it forced is recorded here because getting it wrong would have inverted the rule above. A completed `IndexRefinementReceipt` owns two identities: its complete receipt identity, which restates the frozen semantic, scalar, and law registry snapshots, and the reached-only `tiler.ir.index-refinement-executable-coverage.v1` projection, which restates only the definitions and admission provenance the occurrence reached. Program and artifact stage coverage folds the second. Folding the first would have made an unused semantic or scalar provider revision change kernel-program and artifact identity — provenance invalidating meaning, which is exactly the coupling this decision removes — and `an_unused_semantic_provider_revision_does_not_change_identity` in `tiler-artifact` is the check that holds it, over receipts minted by the real verifier rather than over a fixture that could not tell the two apart.

## Alternatives considered

Including providers in graph identity is conservative for cache invalidation
but conflates meaning with implementation provenance and prevents reuse across
equivalent providers. Excluding providers everywhere would permit stale or
misattributed inference, lowering, and code generation. One universal digest
would be easy to thread through APIs but could not state which equality it
proves and would make unrelated changes invalidate every layer.
