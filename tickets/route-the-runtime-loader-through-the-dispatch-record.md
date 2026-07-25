---
id: route-the-runtime-loader-through-the-dispatch-record
title: Route the runtime loader through the projected dispatch record
status: in-progress
priority: p1
dependencies: [expose-the-dispatch-record-on-a-decoded-artifact]
related: [route-the-runtime-proof-through-the-artifact-envelope, prototype-runtime-artifact-validation]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifact]
claimed_from: todo
assignee: agent-bypass
lease_expires_at: 1785011575
---
`expose-the-dispatch-record-on-a-decoded-artifact` projected the dispatch record onto `DecodedArtifact`. `crates/tiler-runtime` was written against the surface that existed before it, so three of its statements are now false and one of its workarounds is now unnecessary. This ticket is the runtime-side follow-through; it holds `implementation/runtime`, which that ticket did not.

**Fact — `crates/tiler-runtime/src/load/route.rs:17-31` is stale.** It states that a committed route "does not name an entry symbol, a binding-to-buffer correspondence, or an evaluated launch extent, because a decoded envelope publishes none of those: the payload-metadata section has no public parser, `BindingData` carries no value reference, and every expression accessor hangs off a `VerifiedArtifactProgram` that no decode produces." All three clauses are now wrong: `DecodedEntry::backend_symbol`, `DecodedBinding::target`, and `DecodedExpr::evaluate` exist. Its closing sentence — "until [that ticket] does, a caller that does not hold the program it compiled cannot dispatch from an artifact alone" — is the claim that changed.

**Fact — `crates/tiler-runtime/src/load.rs:36-57` is stale in one of its three refusals.** "More than one payload descriptor … the descriptor-to-section map is not published" was true and is not: `DecodedArtifact::payload_object(index)` publishes exactly that association per descriptor, so `resolve_object`'s one-descriptor-one-object-section cardinality reasoning (`load.rs:238-264`) can be replaced by a direct lookup and `LoadRejection::ObjectUnresolvable` retired. The multi-variant refusal is a *different* question — guard evaluation now exists through `DecodedVariant::applicability_guard`, so that refusal can also be reconsidered, but doing so means deciding how a host supplies `AbiFacts` before routing commits, which is a real design step rather than a deletion.

**A doc comment is a claim and it is load-bearing.** These were left rather than fixed because the artifact-side ticket held only `implementation/artifact`; correcting another scope's files would have been a scope escape. They are recorded here so the staleness is tracked rather than discovered.

## The work

- Correct both module doc comments against what the record now publishes.
- Replace `resolve_object`'s cardinality inference with `payload_object`, and retire `ObjectUnresolvable` if nothing else can produce it.
- Extend `Preflight`/`RoutedDispatch` to carry the entry's symbol, transport slots, binding targets and evaluated launch geometry, so a committed route is a dispatch rather than a pointer to bytes. Whether the host supplies `AbiFacts` to `preflight` is the one genuine interface decision here; it is what turns an evaluable expression into an evaluated extent, and it should be presented to Tom as an atomic question rather than chosen quietly.
- Reconsider the multi-variant refusal in light of `applicability_guard` being reachable, or restate it with its real remaining reason.

## Closes when

The loader's documentation describes what the record publishes, the object is resolved by its descriptor rather than by counting sections, and `uv run --locked python scripts/check_repository.py` passes.
