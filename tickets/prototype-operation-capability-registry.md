---
id: prototype-operation-capability-registry
title: Implement operation capability registration and resolution
status: in-progress
priority: p0
dependencies: [reconcile-implementation-work-graph-after-authority-audit, correct-reference-value-and-authority-contracts]
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-foundation, extensions]
claimed_from: todo
assignee: agent-prototype-operation-capability-registry
lease_expires_at: 1784832271
---
Add versioned typed registration, deterministic resolution, ambiguity/missing
diagnostics, and provenance for index/access and scalar-lowering capability
families. Providers consume narrow checked contexts and emit only through the
canonical builders; no placeholders, opaque payloads, downcasting, or
provider-owned IR. Registration does not prove that emitted index work refines
an operation occurrence: `prototype-semantic-index-refinement` owns that
separate checked authority.

Semantic effects remain authoritative in the semantic registry. The bounded
P0 physical frontier owns scheduled-kernel provider registration only. The
later reviewed `implement-opaque-physical-call-providers` ticket owns opaque
registration after optimizer conformance and mature boundary/cost authorities.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

Implemented a bounded compiler-side lowering-capability registry in the new `tiler-compiler` module `capability` (`crates/tiler-compiler/src/capability.rs`), composing the frozen semantic and scalar authorities from `tiler-ir`. It covers both required families — `LoweringFamily::IndexAccess` and `LoweringFamily::ScalarLowering` — with versioned typed registration, deterministic resolution, deterministic collision/ambiguity/missing diagnostics, and canonical provenance. No `tiler-ir` edits were required; the registry consumes the already-public canonical builders and authority projections.

Registration and resolution model:

- Capabilities are stored in a `BTreeMap` keyed by `(family, operation OpKey, LoweringSignature, admitting ProviderIdentity)`. The admitting provider participates in the key, so two providers may each claim one occurrence; that contradiction surfaces as a deterministic resolution ambiguity rather than a silent last-wins selection. This is the deliberate, justified divergence from the reference-evaluation registry (which admits one oracle per signature) needed to satisfy the ambiguity requirement, consistent with the operation-extension contract's "contradictory provider selections fail deterministically".
- Registration takes the admitting provider directly and validates fully before retaining anything, so a rejected registration leaves the builder unchanged (transactional). Validation projects the lowered operation's semantic authority (`FrozenSemanticRegistry::project_operation_authority`, which also validates the signature types) and the declared emitted scalar operations' reached authority (`FrozenScalarRegistry::project_reached`); both feed provenance.
- Resolution scans the ordered map for the `(family, operation, signature)` selector: zero matches yield `MissingCapability`, one yields the resolved capability, and more than one yields `AmbiguousCapability` with candidate providers in canonical ascending order.

Determinism: the frozen snapshot iterates the `BTreeMap` in total key order for its canonical identity, so the identity and every diagnostic candidate list are independent of registration order. This is directly tested (`snapshot_identity_is_independent_of_registration_order`, and `contradictory_providers_resolve_to_a_deterministic_ambiguity` which registers in both orders 32 times and asserts a fixed candidate order). No `HashMap`-iteration-order value participates in any diagnostic.

Provenance excludes identity-unstable data: it encodes only `ProviderIdentity` (namespace/name/revision), `OpKey`, `ResolvedValueType::canonical_encoding`, the capability revision, and the reached scalar-definition and semantic-authority projection bytes. No `TypeId`, vtable, function, or allocation addresses enter identity. A running byte budget accumulated at registration is `debug_assert`-checked against the frozen encoding at freeze.

Provider seam (the "narrow checked context / emit only through canonical builders" constraint): `ScalarLoweringProvider` and `IndexAccessLoweringProvider` receive only `ScalarLoweringContext` / `IndexAccessLoweringContext`, which delegate to the canonical `tiler-ir` `IndexRegionBuilder` (and `ScalarReducerBodyBuilder`). The contexts never expose the raw builder, region finalization/verification, or any way to construct provider-owned IR, carry an opaque payload, or downcast the host. Tests drive a resolved provider of each family through a real builder and verify a `VerifiedIndexRegion` is produced, proving real IR emission — without asserting refinement.

Scope boundary honoured: the registry resolves available lowering knowledge and its provenance only; it does not prove a provider's emitted index work refines the occurrence (owned by `prototype-semantic-index-refinement`), does not touch semantic-effect authority, and does not register scheduled-kernel or opaque physical providers.

Draft boundary for review: the module is introduced as `pub mod capability` (consistent with `lib.rs`'s stated plan for the capability slice to introduce the facade) and is explicitly a draft, not a stabilized compiler-session API.

Gate: `uv run --locked python scripts/check_repository.py`, `git diff --check`, and `tkt guard tkt/prototype-operation-capability-registry` all pass. Not merged, pushed, or marked done.
