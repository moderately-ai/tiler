---
id: resolve-capability-key-signature-conflation
title: Decide whether a governed capability key must distinguish signatures
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, compiler, artifact, identity]
---
`name-the-resolved-lowering-capability` minted the governed capability key as `tiler.capability.<family>.<op-namespace>.<op-name>.v<version>` (`crates/tiler-compiler/src/lowering.rs`, `governed_capability_key`). It deliberately excludes the resolved signature, and this ticket owns that exclusion.

**Fact.** A capability is registered under family, operation, signature, and provider — `LoweringCapabilityKey` at `crates/tiler-compiler/src/capability.rs`, whose duplicate check raises `LoweringRegistryError::DuplicateCapability` only when all four match. So one provider may register two capabilities for one operation family that differ only in signature.

**Fact.** Those two would mint the same governed key today.

**Fact — why the signature was left out.** A `CapabilityKey` is bounded at `MAX_GOVERNED_KEY_BYTES = 256` (`crates/tiler-artifact/src/program/keys.rs`). `LoweringSignature` is an unbounded structural value. Folding one in would either truncate — silently colliding, which is worse than the present conflation because it would look distinguishing — or require a digest, which introduces a second identity that must be kept in agreement with the signature it summarizes.

**Inference — currently unreachable, not currently harmful.** Consumers record the provider and the capability revision beside the key (`SelectedProvider`), so two capabilities collide in artifact identity only when one provider registers two signatures for one operation family at one revision. The governed registry registers one signature per family and operation, so no such pair exists today.

**What makes this reachable.** Any of: a provider registering per-shape or per-attribute signatures; the bounded profile admitting a second signature for an existing operation; or a capability key becoming a lookup key rather than only recorded evidence.

## Closes when

Either the key is widened to distinguish signatures with a stated bounded encoding, or the conflation is accepted with a check that fails closed when a second signature is registered for one family and operation — so the assumption cannot become false silently. `uv run --locked python scripts/check_repository.py` passes.
