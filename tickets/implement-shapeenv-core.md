---
id: implement-shapeenv-core
title: Implement the core ShapeEnv authority
status: done
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: [shape-environment-contract]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing, mature-product]
---
Implement the accepted graph/session-owned ShapeEnv: typed root symbols,
constraints, exact mathematical integers, binding/source phases, canonical
identity, validation, and explicit unresolved/ambiguous errors. It must not
depend on index IR. Index bindings and predicates consume this authority in
separate downstream tickets.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

**Partly done, and the remainder is split rather than implied.** `crates/tiler-ir/src/shape/env.rs` implements the scoped symbol and typed root-binding half of the authority `docs/ir.md` specifies. The constraint environment is `implement-shapeenv-constraints`.

**Why the split falls there.** The contract makes contradiction rejection normative — "contradictory semantic constraints reject the graph" — so a constraint set that stored relations without deciding contradiction would be a type-system reservation wearing the name of an implemented authority. Landing the symbol half alone is a smaller claim that is fully true; landing both halves with one of them undecided would have been a larger claim that is not.

**What the symbol half establishes, each traced to a contract clause.**

- **Scope is part of the symbol.** "Equal spelling in different scopes never implies equality." `ShapeSymbol` pairs a name with a `SymbolScope`, so the rule holds at comparison, in identity bytes, and in binding lookup rather than being a thing every consumer must remember.
- **One declaration and one root binding per symbol.** A second declaration and a second binding are distinct typed rejections, neither last-write-wins, and both leave the draft unchanged.
- **Free symbols are invalid, not deferred.** An unbound symbol fails `build` naming the first offender in canonical order, so the diagnostic does not depend on authoring order.
- **A binding states its source class and its availability phase.** The verifier requires "exactly one typed binding whose source class and availability phase are supported by every semantic factor that consumes it", so both travel on the binding. `RootBinding::new` rejects a phase earlier than the source class admits — a binding claiming a target property is readable at `CompileProfile` would let a consumer evaluate it before any device exists.
- **Availability phases are ADR 0043's.** The module uses `crate::program::abi::AvailabilityPhase` rather than declaring a shape-local copy. That is the defect `relocate-abi-expressions-into-tiler-ir` closed twice this week, and re-introducing it here would have been the third.
- **Identity is canonical.** Domain-separated and length-prefixed through `crate::identity`, over entries sorted at `build`, covering declarations and root-binding provenance and nothing derived.

**Draft status, deliberately.** The module is `pub(crate)` under ADR 0074 convention 7, because this ticket itself says any consequential boundary "remains a draft until Tom reviews and accepts the exact implementation commit". Nothing is reachable outside `tiler-ir`, so no reserved boundary was crossed and promotion stays a separate reviewed step. It carries a module-level `dead_code` allow with the reason stated: its consumers do not exist yet — `implement-shapeenv-index-bindings` is what makes index lowering read it — and the bounded profile's shapes are static literals with no symbols at all, so wiring a premature consumer to satisfy the lint would make the authority look adopted while proving nothing.

**Evidence.** Six tests, each asserting a contract clause rather than the implementation. The scope test deliberately checks that two same-spelled symbols in different scopes can *both* be declared and bound independently, which an inequality assertion alone would miss. The identity test asserts that declaration order does not change identity while a differing bound value and a differing *provenance* both do — provenance participating in identity is a contract requirement, and a same-value different-reason pair is the case that would silently collide if it did not. Full repository gate green.
