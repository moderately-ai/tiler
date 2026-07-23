---
id: make-resolved-value-type-ordering-explicit
title: Make resolved value type ordering explicit
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, determinism, diagnostics]
---
Sibling of the `ValueTypeDefinitionKey` ordering fix, found and correctly scoped out during that work. `ResolvedValueType` also derives `Ord`, so its ordering follows Rust variant declaration order and a semantically neutral variant reorder would silently change behaviour.

The exposure is narrower than the key's was and must not be overstated: concrete-instance closure sets are never serialized, so this ordering reaches no durable identity encoding. What it does decide is which missing-authority error `freeze` reports first when several are candidates — diagnostic determinism, not identity stability. AGENTS.md requires stable diagnostics, so it is still worth closing.

Replace the derived `Ord`/`PartialOrd` with explicit implementations that preserve the current order exactly, following the pattern established for `ValueTypeDefinitionKey` (an explicit family discriminant shared between ordering and any encoding, so rank and tag cannot drift apart). Add a test that pins the family order and fails under a variant reorder. Confirm and state in the outcome whether this ordering reaches any durable encoding; if it turns out it does, that is a stronger finding than this ticket assumes and should be reported rather than quietly fixed.
