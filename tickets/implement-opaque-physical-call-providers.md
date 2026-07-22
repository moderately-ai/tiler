---
id: implement-opaque-physical-call-providers
title: Implement opaque physical-call providers
status: todo
priority: p1
dependencies: [implement-analytical-component-cost-model]
related: [prototype-physical-implementation-frontier]
scopes: [implementation/compiler, implementation/ir, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, extensions]
---
After optimizer conformance and the mature boundary-property and analytical-cost
authorities, implement reviewed opaque physical-call registration and
verification. Cover typed ABI, effects, aliasing, placement, numerical
guarantees, exact resources, target/applicability requirements, failure stages,
provider provenance, additive coexistence with scheduled kernels, and
deterministic rejection/explain behavior. Every admitted proposal carries a
typed analytical cost estimate and model provenance. Unknown or unavailable
cost remains a deterministic typed state and cannot silently become zero,
infinity, or an arbitrary winner; calibration remains deferred to the separate
measurement/activation ticket. Opaque calls remain explicit physical
boundaries and may not smuggle unknown semantics or effects into logical IR.

Any consequential public or cross-crate crate, module, trait, type, or call-site
boundary remains a draft until Tom reviews and accepts the exact implementation
commit. This ticket does not preselect that interface.
