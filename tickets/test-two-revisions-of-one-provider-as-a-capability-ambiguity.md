---
id: test-two-revisions-of-one-provider-as-a-capability-ambiguity
title: Pin two revisions of one provider as a capability ambiguity
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [testing, capability, extensions]
---
The lowering-capability registry treats two revisions of one provider as a contradiction rather than a version choice, and nothing tests it.

**Fact.** `LoweringCapabilityKey` in `crates/tiler-compiler/src/capability.rs` is `{family, operation, signature, provider}`; `ProviderIdentity` in `crates/tiler-ir/src/semantic/registry.rs` is `{namespace, name, revision}` with derived `Eq` and `Ord`; and `FrozenLoweringCapabilityRegistry::resolve` filters candidates on family, operation, and signature only.

**Inference.** Two registrations that differ only in provider revision are two distinct keys, so both insert successfully — `DuplicateCapability` fires only on an exact key repeat — and both match one selector, so resolution returns `AmbiguousCapability` listing both. No newer-wins, no supersession.

**Fact — the gap.** `contradictory_providers_resolve_to_a_deterministic_ambiguity` registers two providers with different *names* at the same revision. `duplicate_registration_of_one_provider_is_a_collision` re-registers an identical key. Neither covers the two-revision case, so the behaviour above rests on reading the code rather than on a test.

Add one regression test in `crates/tiler-compiler/src/capability.rs` that registers the same `(family, operation, signature)` for one provider namespace and name at two revisions, asserts both registrations succeed, and asserts resolution returns `LoweringResolveError::AmbiguousCapability` whose candidates are both provider identities in canonical ascending order. ADR 0078 cites this ticket as the owner of the gap; update its claim if the measured behaviour differs from the inference above.
