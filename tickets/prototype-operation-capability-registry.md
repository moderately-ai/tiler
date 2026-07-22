---
id: prototype-operation-capability-registry
title: Implement operation capability registration and resolution
status: todo
priority: p0
dependencies: [reconcile-implementation-work-graph-after-authority-audit, correct-reference-value-and-authority-contracts]
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-foundation, extensions]
---
Add versioned typed registration, deterministic resolution, ambiguity/missing
diagnostics, and provenance for index/access and scalar-lowering capability
families. Providers consume narrow checked contexts and emit only through the
canonical builders; no placeholders, opaque payloads, downcasting, or
provider-owned IR. Registration does not prove that emitted index work refines
an operation occurrence: `prototype-semantic-index-refinement` owns that
separate checked authority.

Semantic effects remain authoritative in the semantic registry. The bounded
P0 physical frontier owns scheduled-kernel provider registration only. The
later reviewed `implement-opaque-physical-call-providers` ticket owns opaque
registration after optimizer conformance and mature boundary/cost authorities.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
