---
id: implement-index-domain-predicates
title: Implement typed index-domain predicates and proof exchange
status: todo
priority: p1
dependencies: [implement-shapeenv-index-bindings]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, proof, mature-product]
---
Add the accepted bounded typed predicate language, semantic obligations, durable proof evidence, and sound Unknown outcomes to verified index regions. Extend bounds and write-ownership proving beyond the static structural and finite fallback profile without converting semantic predicates into physical guards.

## Not startable as written — no stated outcome (2026-07-27)

**This ticket has no `## Closes when` and no sections** — thirteen lines, of which two are substance. It names four deliverables at once: the bounded typed predicate language, semantic obligations, durable proof evidence, and sound `Unknown` outcomes. Each is a separate design with its own identity and validation consequences, and the ticket says nothing about which comes first or what any of them looks like when done.

**What it needs before it is claimable.** It should be split along its own four deliverables, with the predicate language first because the other three are expressed in it. The split cannot be written from the ticket alone — it needs the accepted predicate design it refers to as "the accepted bounded typed predicate language", which is not linked from here.

**One constraint stated in the body and worth keeping at the front of any split:** extending bounds and write-ownership proving must not convert semantic predicates into physical guards. That is the failure the whole ticket exists to avoid and it is easy to do accidentally.
