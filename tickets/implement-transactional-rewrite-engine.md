---
id: implement-transactional-rewrite-engine
title: Implement the external transactional rewrite engine
status: todo
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: [implement-first-algebraic-rewrite-portfolio]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, rewrites]
---
Implement the bounded external rule-provider and transactional alternative
engine after the ordinary optimizer path is proven. Preserve exact rule and
provider identity, termination/budget contracts, semantic revalidation,
rollback, deterministic traversal, and typed explain. Unknown provider behavior
is never optimizable merely because it is registered.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Not startable as written — no stated outcome (2026-07-27)

**This ticket has no `## Closes when` and no sections.** Its body names six properties the engine must preserve — rule and provider identity, termination and budget contracts, semantic revalidation, rollback, deterministic traversal, typed explain — and nothing that says what "implemented" is. Nobody can land it, because nobody can tell when it is done, and the six properties are each a substantial subsystem rather than a checklist.

**What it needs before it is claimable.** A stated first slice: which single rewrite the engine runs end to end, what a rule provider's registration looks like, and what one transaction commits and rolls back. The `Unknown provider behavior is never optimizable merely because it is registered` line is a constraint on the answer, not the answer.

Its dependency `prototype-optimizer-conformance-gate` is the natural place that slice comes from: the gate defines what an optimizer path must not break, and the engine's first transaction should be one the gate already covers.
