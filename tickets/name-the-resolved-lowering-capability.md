---
id: name-the-resolved-lowering-capability
title: Carry the resolved lowering capability's governed key into the plan
status: todo
priority: p0
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, compiler, artifact]
---
An artifact records which capability providers actually lowered its program, and ADR 0072 makes those selected providers part of complete program identity. The compiler cannot currently name them, so the first artifact assembler has no faithful value to record.

**Fact.** `crates/tiler-compiler/src/request.rs:201-204` defines `LoweringProviderIdentity` with exactly two fields, `provider: ProviderIdentity` and `capability_revision: LoweringCapabilityRevision`. Its accessors (`request.rs:218-231`) expose those two and nothing else. `ArtifactConstructionPlan::lowering_providers` is the only path from a plan to provider identity.

**Fact.** `tiler_artifact::program::SelectedProvider` requires a governed `CapabilityKey` alongside the provider and an API version, and `crates/tiler-artifact/src/program/verify.rs:26-28` raises `ArtifactDiagnostic::MissingSelectedProvider` when an artifact declares none. So the field cannot be skipped.

**Fact.** The information exists at resolution time and is dropped. `LoweringFamily` is defined at `crates/tiler-compiler/src/capability.rs:68` and recorded on `RegisteredLoweringCapability` (`capability.rs:259-263`) together with the revision and the authority, but only the revision reaches `LoweringProviderIdentity`.

**Inference.** Without this, an assembler must invent a capability key. That is not a cosmetic placeholder: the key enters artifact identity, so an invented one produces an artifact whose identity claims a capability that did not lower it. `AGENTS.md` requires unsupported cases to reject explicitly rather than be silently approximated.

## Scope

Carry the resolved capability's governed key — or the family and signature the key is derived from — from capability resolution into `LoweringProviderIdentity`, and expose it on `tiler_compiler::session` beside the provider identity and revision. Decide explicitly whether the governed key is minted in the compiler or derived by the consumer from an exposed family, and state which, because that decides who owns the key vocabulary.

Do not widen `LoweringFamily`'s public surface further than the decision needs; it is `#[non_exhaustive]` and its discriminant is load-bearing for durable identity ordering.

## Closes when

A caller outside `tiler-compiler` can construct a `SelectedProvider` whose capability names the lowering that actually ran, with no invented value; the choice of key ownership is stated; and `uv run --locked python scripts/check_repository.py` passes.
