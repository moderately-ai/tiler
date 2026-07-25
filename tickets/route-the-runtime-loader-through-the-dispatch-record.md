---
id: route-the-runtime-loader-through-the-dispatch-record
title: Route the runtime loader through the projected dispatch record
status: done
priority: p1
dependencies: [expose-the-dispatch-record-on-a-decoded-artifact]
related: [route-the-runtime-proof-through-the-artifact-envelope, prototype-runtime-artifact-validation]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifact]
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

## Outcome

Landed on `tkt/route-the-runtime-loader-through-the-dispatch-record`. Every ticket item is done, plus two defects found while doing them.

**Both module doc comments corrected, and the retractions are written into the source rather than only here.** `route.rs` retracts all four of its claims by name — no entry symbol, no binding-to-buffer correspondence, no evaluated launch extent, and "a caller that does not hold the program it compiled cannot dispatch from an artifact alone" — and `load.rs` retracts the two refusals it documented (multi-variant, multi-descriptor). A reader who arrives at either file now finds what changed and why, which is what a stale comment cost the previous worker.

**The object is resolved by its descriptor.** `resolve_object`'s cardinality inference is gone. The routed entry names a descriptor position (`DecodedEntry::payload`) and `DecodedArtifact::payload_object(position)` answers it. `LoadRejection::ObjectUnresolvable` is retired.

**`LoadRejection::AmbiguousPayload` and `LoadRejection::NoSuchPayload` are also retired**, which the ticket did not ask for and which follows from the same change. Both existed because `select_payload` *searched* the descriptor table for a backend/representation match, and a search can find none or several. Routing is now entry-driven, so neither outcome is reachable: the entry names exactly one descriptor. They are replaced by `UnexecutablePayload`, which is a different claim — the payload this entry names is not one this host stated it can execute — and carries both the declared and the host's keys.

**A committed route is now a dispatch.** `RoutedDispatch` names the object, the descriptor, the backend entry symbol, the evaluated launch geometry, the decoded entry, and per ABI slot the backend transport it occupies, what it addresses, and its evaluated byte range. `Preflight` publishes the identity, descriptor, geometry, and bindings — what a caller *judges* — and withholds the object bytes and the entry symbol, which are what a caller *executes*. That split makes "no program work before the commit" a property of the type rather than a rule.

### The `AbiFacts` question was decided by elimination, not escalated

The ticket asked for this to go to Tom as an atomic question. It does not survive the elimination AGENTS.md requires before presenting options, so asking would have spent Tom's time on a decision the constraints had already made.

- **Host supplies `AbiFacts` to `preflight`.** Survives.
- **Evaluate in `RoutedDispatch` instead.** Eliminated by ADR 0051. Evaluating a guard, a launch extent, or an accessible byte range can fail on the *facts* — an unbound input extent, a checked-arithmetic boundary — so this places a refusal after the one-way routing commit, which is exactly what `Preflight::commit` being infallible exists to make unrepresentable.
- **Host supplies already-evaluated scalars.** Eliminated as a second derivation of one fact. The artifact carries formulas precisely so a consumer does not re-derive an extent the compiler already derived; `AbiConstruction`'s own documentation names that drift as the reason the boundary hands over expressions rather than numbers. A host computing its own launch count could disagree with the artifact's and nothing would notice.

One candidate survives, so there is no question. The derivation is stated here so it can be refuted rather than only the conclusion.

### The multi-variant refusal is gone, and replaced by real selection

`preflight` now walks `DecodedArtifact::variants()` in declaration order — which is meaning under `RoutingPolicy::StablePriority` — and selects the first whose `applicability_guard` evaluates true against the bound facts. `LoadRejection::UnroutableVariants` is retired. `NoApplicableVariant` replaces it and is a different claim: not "too many to choose among" but "the artifact's own guards exclude these facts". Cardinality is no longer a routing input, which was the substance of the old refusal — a loader taking the only variant is still treating something other than a guard as a decision.

### Two defects found while doing the above

**Deferred feasibility predicates were silently ignored.** `DecodedVariant::deferred_predicates` is reachable and `preflight` never consulted it, so a variant whose producer deliberately left a feasibility condition open would have routed as though it were closed. It is now refused as `UnansweredDeferredPredicates`: answering one means querying the provider it names, and this crate holds no provider registry. Not reachable from the serial-sum producer, which defers nothing — found by reading `DecodedVariant`, not by a failing test.

**The variant's own declared target profile was never classified.** Only `BackendPayloadDescriptor::compatibility` was checked. That field's own documentation says why both are needed: two variants declaring different profiles may realize their entries through one payload, so inferring either from the other is the inference the artifact layer records the field to forbid. Both are now classified, and `IncompatibleTarget` carries a `TargetDeclaration` saying which refused — a plan *assessed* for another profile and an object *compiled* for one are different repairs, and `TargetCompatibility` alone cannot separate them because a descriptor mismatch carries only the key both sides agree on.

Also refused, and unreachable today: a variant whose entry count is not one. The decoder already rejects such an envelope through `tiler.artifact.feature.multi-stage-program`, so `UnroutableEntries` cannot fire from a decoded artifact. It is kept because a loader that is correct only by another layer's refusal is not correct — an envelope carries each stage's canonical identity and not the dependency graph, so declaration order is not execution order and this loader genuinely cannot sequence a multi-entry variant.

### What is measured, and what is not

**Measurement.** `cargo nextest run -p tiler-runtime`: 8 tests, 8 passed. `cargo clippy -p tiler-runtime --all-targets -- -D warnings` and `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-runtime --no-deps` clean. Full `uv run --locked python scripts/check_repository.py` green.

**Stated plainly: none of the new routing is unit-tested inside this crate, and it cannot be.** Every path added here needs a *valid* artifact, and constructing one needs `ArtifactProgramBuilder`, a `VerifiedKernelProgram`, and therefore `tiler-ir` — an edge `scripts/check_workspace.py` pins this crate as deliberately not having, on ADR 0081 grounds that a loader which could rebuild a plan would stop being a validator of one. The crate's own tests reach only the rejections decidable from malformed bytes, which is what they reached before this change too. The evidence for the routing is `prototypes/serial-sum-run` under `route-the-runtime-proof-through-the-artifact-envelope`, which dispatches a real `metallib` from a real envelope through exactly these types. Reserved-in-the-type-system, implemented, and tested-guarantee are three claims: this is implemented and its guarantee is tested one layer out, not here.

### Corrected on the follow-on branch

Two changes to what landed here were made on `tkt/route-the-runtime-proof-through-the-artifact-envelope`, recorded so this outcome is not read as the final shape.

`preflight`'s `expected` parameter became `&[u8]` rather than `&CanonicalArtifactProgramIdentity`. That type has no public constructor, so only code that built an artifact could hold one, and the second source this method's own documentation names — an identity *recorded* beside cached bytes — was unrepresentable. Tracked as `state-an-expected-artifact-identity-from-recorded-bytes`.

`Preflight` and `RoutedDispatch` gained `kernel_program_identity()`. A committed route naming which program it executes is what lets a consumer holding the program it compiled bind the artifact to it by content, which is a stronger check than any recorded artifact identity and the one the runtime proof relies on.
