---
id: prototype-operation-compilation-capabilities
title: Implement operation compilation capabilities
status: todo
priority: p0
dependencies: [prototype-canonical-index-region-slice, correct-semantic-identity-layering, reconcile-implementation-work-graph-after-authority-audit]
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, compiler-foundation, extensions]
---
Add versioned typed provider capabilities for index/access lowering, scalar
lowering, materialization constraints, and numerical/effect refinement. Lowering
callbacks must consume narrow checked contexts and emit through the canonical
`tiler-ir` index builders; do not introduce placeholder traits, opaque payloads,
downcasting, or a duplicate provider-owned IR. Resolution and provenance must
be deterministic. Different providers may supply disjoint capability families,
while competing providers for a singular family remain an explainable
ambiguity. Missing, ambiguous, or invalid capability output fails closed.
Selected provider revisions enter checked refinement and compilation
provenance, never semantic graph or pure index-structure identity. Reached
provider-independent definitions and admission-provider provenance remain
separate inputs under ADR 0072. Capabilities must consume a
`ScalarAuthorityEvidence` receipt bound to the exact emitted index region;
they must not re-infer scalar authority through an untracked provider path or
mistake that receipt for semantic lowering equivalence.

Physical implementation and opaque-call capability registration is deferred to
`prototype-physical-implementation-frontier`, where the checked schedule and
program proposal surfaces exist. Semantic effects remain authoritative in the
semantic registry; compilation providers supply preservation or refinement
evidence rather than redeclaring operation semantics.
