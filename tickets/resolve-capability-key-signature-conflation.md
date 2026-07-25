---
id: resolve-capability-key-signature-conflation
title: Decide whether a governed capability key must distinguish signatures
status: done
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

## Outcome

**Decided: keep the exclusion, and enforce the assumption it rests on.** The key stays `tiler.capability.<family>.<op-namespace>.<op-name>.v<version>`, and `LoweringRegistryError::ConflatedCapabilityKey` now refuses the registration that would make the exclusion unsafe. Widening the key was rejected on the ticket's own grounds: a bounded encoding of an unbounded structural signature is either a truncation, which collides silently while *looking* distinguishing, or a digest, which is a second identity that must be kept in agreement with the value it summarizes. Neither is worth buying when the property the key needs can simply be required.

### The invariant, stated

A governed capability key is complete evidence exactly while, for one family, one operation, and one provider, at most one signature is registered. Every consumer records the provider beside the key, so the *pair* names one capability under that condition and nothing under its negation. `LoweringCapabilityRegistryBuilder::register` now rejects the second signature with a typed error naming the family, the operation, the provider, the signature already registered, and the one refused.

**Scope of the guard, and why it is not wider.** It is per provider and per family, not per operation.

- Per **provider**, because two providers claiming one operation with different signatures are still distinguishable by the recorded provider, and because ADR 0072 requires a contended claim to reach a deterministic resolution ambiguity rather than a registration failure. A global guard would break that and its existing test.
- Per **family**, because the family is in the key, so two families for one operation and provider are already distinguishable.

Both negatives are asserted in `a_second_signature_for_one_family_and_operation_is_refused` rather than left to inference, because a guard that over-tightens would look identical to one that is correct until an external provider hits it.

### A correction to this ticket's own inference

The ticket recorded: *"Inference — currently unreachable, not currently harmful. … The governed registry registers one signature per family and operation, so no such pair exists today."* The first half of that conclusion is **wrong**, and the reason is worth writing down because it was reached by inferring a validator's behaviour from its name rather than reading it.

`register` validates a signature through `FrozenSemanticRegistry::project_operation_authority`, which closes over the types and the operation the signature names and fails when one of them is absent from the registry. It does **not** compare the signature against the operation's own arity or type contract. A unary `f32` signature therefore registers against a binary `f32` multiply. `one_operation_admits_more_than_one_registrable_signature` asserts exactly that, as a premise test for the guard.

So the conflation was reachable at the registration boundary before this change, by any out-of-crate provider, with no diagnostic. What was true is the narrower statement the ticket also made: the *governed* profile registers one signature per operation, so no such pair exists in Tiler's own registry. That is a fact about one registry and never was a property of the boundary. The guard makes it a property of the boundary.

### What this deliberately costs

A provider cannot register per-shape or per-attribute signatures for one operation family — the first of the three things the ticket named as making the conflation reachable. That refusal is the point rather than a side effect: admitting those signatures now requires deciding a bounded encoding for the key first, which is a decision someone makes rather than a property that quietly stops holding. The rejection names the conflation, so a provider that hits it is told what the obstacle is.

The other two triggers the ticket named are unaffected and remain correct as written: the bounded profile admitting a second signature for an existing operation is now a build-time failure with a named cause, and a capability key becoming a *lookup* key rather than recorded evidence is a separate future change — the guard supplies the uniqueness such a lookup would need, but nothing here turns the key into one.

### Not touched

`crates/tiler-artifact/src/program/keys.rs` and its `MAX_GOVERNED_KEY_BYTES = 256` are unchanged. They were only ever the constraint that made widening unattractive, and the decision not to widen leaves them with nothing to say.

`LoweringRegistryError` gained a variant. It is `#[non_exhaustive]`, and ADR 0075's resolved open question puts additive growth of such an enum in the category a coordinator may merge — ADR 0074's amended convention 5 forbids `#[non_exhaustive]` on enums with an out-of-crate consumer that maps them completely, so the growth is additive in fact rather than only in form.

### Gate

`uv run --locked python scripts/check_repository.py` passes.
