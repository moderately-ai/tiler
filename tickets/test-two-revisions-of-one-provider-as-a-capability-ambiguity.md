---
id: test-two-revisions-of-one-provider-as-a-capability-ambiguity
title: Pin two revisions of one provider as a capability ambiguity
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [testing, capability, extensions]
---
The lowering-capability registry treats two revisions of one provider as a contradiction rather than a version choice, and nothing tests it.

**Fact.** `LoweringCapabilityKey` in `crates/tiler-compiler/src/capability.rs` is `{family, operation, signature, provider}`; `ProviderIdentity` in `crates/tiler-ir/src/semantic/registry.rs` is `{namespace, name, revision}` with derived `Eq` and `Ord`; and `FrozenLoweringCapabilityRegistry::resolve` filters candidates on family, operation, and signature only.

**Inference.** Two registrations that differ only in provider revision are two distinct keys, so both insert successfully — `DuplicateCapability` fires only on an exact key repeat — and both match one selector, so resolution returns `AmbiguousCapability` listing both. No newer-wins, no supersession.

**Fact — the gap.** `contradictory_providers_resolve_to_a_deterministic_ambiguity` registers two providers with different *names* at the same revision. `duplicate_registration_of_one_provider_is_a_collision` re-registers an identical key. Neither covers the two-revision case, so the behaviour above rests on reading the code rather than on a test.

Add one regression test in `crates/tiler-compiler/src/capability.rs` that registers the same `(family, operation, signature)` for one provider namespace and name at two revisions, asserts both registrations succeed, and asserts resolution returns `LoweringResolveError::AmbiguousCapability` whose candidates are both provider identities in canonical ascending order. ADR 0078 cites this ticket as the owner of the gap; update its claim if the measured behaviour differs from the inference above.

## Outcome — the inference measured and pinned (2026-07-27)

`capability::tests::two_revisions_of_one_provider_resolve_to_an_ambiguity` registers the same `(family, operation, signature)` for one provider namespace and name at revisions 1 and 2, **in both registration orders**, and asserts what the ticket inferred: both registrations succeed, resolution returns `LoweringResolveError::AmbiguousCapability`, and the candidates are both identities in canonical ascending order. The measured behaviour matches the inference exactly, so ADR 0078's claim stands and needed no correction — only its closing sentence, which said no test pinned the case, moved to a measurement.

**Both registration orders, deliberately.** A newer-wins rule that happened to keep whichever arrived last would pass a single-order test.

**The test's own coverage claim was checked rather than asserted.** Adding a test that duplicates existing coverage is the easy failure here, so a newer-wins rule was simulated inside `resolve` — collapsing the ambiguity when every candidate shares a namespace and name. Under it, `contradictory_providers_resolve_to_a_deterministic_ambiguity` and `duplicate_registration_of_one_provider_is_a_collision` both stay **green**, and only the new test **fails**. That is the evidence that it closes a gap: one sibling registers two provider *names* at one revision, the other re-registers an identical key, and neither can see a revision-based supersession. The simulation was reverted.

The reasoning is recorded on the test itself, so a reader deciding whether it is redundant has the answer without re-deriving it.
