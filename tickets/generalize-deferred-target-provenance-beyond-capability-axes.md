---
id: generalize-deferred-target-provenance-beyond-capability-axes
title: Generalize deferred target provenance beyond quantitative capability axes
status: todo
priority: p1
dependencies: []
related: [admit-an-atomic-subgroup-realization-subject-to-target-profiles, decide-the-prepared-subgroup-width-equality-gate, carry-subgroup-width-through-exact-prepared-entry-equality]
scopes: [implementation/compiler, implementation/build, implementation/ir, contracts/optimizer, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, feasibility, provenance, public-boundary, identity]
---
## User-visible outcome

The compiler can defer an exact target obligation without pretending every obligation is an independently declarable quantitative capability axis.

## Fact — 2026-08-11

`DeferredPredicate`, `DeferredSet`, `EntryDeferredPredicate`, explain generation, and public `PreparedEntryTargetRequirementRef::capability_axis` all assume a `CapabilityAxis`. Subgroup width must instead be confirmed from the complete accepted `SubgroupRealizationSubject`; adding an independent axis would decompose that atomic subject and permit unsupported partial conjunctions.

## Required delivery

- Introduce a required typed deferred subject distinguishing the existing quantitative-axis case from subgroup-width confirmation derived from the complete atomic subgroup subject.
- Keep canonical sorting, deduplication, explanation, public borrowed views, and exact-entry forwarding exhaustive over the new subject. Replace any accessor that falsely remains total over capability axes.
- Lower both subject families into the existing generic artifact `PreparedEntryTargetRequirement`; do not add a subgroup-specific artifact row or an independently satisfiable subgroup fact.
- Encode every identity-bearing subject field and step only the compiler/public domains whose grammar actually changes. Recompute all explanation and request pins.
- Perturb subject kind, atomic subgroup width/arithmetic/transfer, entry, query, and relation independently with unchanged checks.

## Closes when

The compiler can carry a subgroup-width confirmation without inventing a capability axis, and every producer/consumer is exhaustive over the typed provenance.
