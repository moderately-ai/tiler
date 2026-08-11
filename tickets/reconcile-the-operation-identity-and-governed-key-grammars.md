---
id: reconcile-the-operation-identity-and-governed-key-grammars
title: Reconcile the operation-identity and governed-key grammars
status: done
priority: p2
dependencies: []
related: [reconcile-the-two-target-profile-key-grammars, replace-flat-selected-lowering-capability-keys-with-structured-subjects, frame-provider-identities-before-using-them-as-explain-keys]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, implementation/runtime, contracts/foundation, contracts/artifacts, contracts/decisions, research/artifacts, research/cache, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [identity, validation, extensions, decision, needs-tom, public-boundary]
---
## User-visible outcome

A selected lowering capability is identified by its structured family and exact operation key. No legal operation key is narrowed to fit a downstream text grammar, and no delimiter join, case fold, truncation, default, or fallback may make two capabilities compare as one.

## Source-first audit at the decision base

The accepted decision was rederived at exact base `b03d1e7699d4f7cfbfb6ee7a903e2d2fbe16af18` before this record changed.

- **Verified.** `governed_capability_key` in `crates/tiler-compiler/src/lowering.rs` infallibly formats `tiler.capability.{family}.{namespace}.{name}.v{version}`. The only live family is `index-access`.
- **Verified.** `OpKey` uses the shared identity-component validator in `crates/tiler-ir/src/semantic/types.rs`: each component is nonempty, at most 255 bytes, and admits ASCII alphanumeric bytes, including uppercase, plus internal `.`, `_`, and `-`.
- **Verified.** Artifact `CapabilityKey` is at most 256 bytes and admits only lowercase governed text.
- **Historical and imprecise.** The ticket's `scalar` measurements predated retirement of that family. The live `index-access` spelling puts the first uppercase byte at 30 rather than 24 and reaches 544 rather than 538 bytes for two maximum components. The mismatch remains, but the old absolute measurements were not current facts.
- **False census.** There are seven downstream `CapabilityKey::new` conversions, not three: five `.expect()` assertions, one correctly propagated `?`, and one lossy remapping. The five assertions are brittle, but the audited governed spike/prototype paths do not demonstrate reachability from an external registration; the production build path already propagates a typed error.
- **False bound claim.** The former 66-byte statement was copied from a source scan of governed key literals, not a typed census of legal `OpKey` values.
- **False as complete options.** Neither former Option A nor Option B makes the composed identity injective. Legal distinct keys `OpKey::new("a.b", "c", 1)` and `OpKey::new("a", "b.c", 1)` both flatten to `tiler.capability.index-access.a.b.c.v1`.
- **Verified failure mechanism.** `ConflatedCapabilityKey` compares equal structured operations with differing signatures; it does not reject two distinct operations whose flattened strings collide. Both registrations can therefore succeed under the same provider and capability revision.
- **Verified consequence.** The lowering-registry identity remains sound because its encoder length-frames the operation components. The defect begins in `LoweringProviderIdentity { provider, capability_key: String, capability_revision }`. Both resolved-provider populations sort and deduplicate that lossy record, and public `SelectedCapability` plus artifact `SelectedProvider` retain only the flattened string. The artifact cannot recover a boundary already erased by the compiler projection.

The last discovery changes the purpose from choosing where a text-grammar refusal occurs to defining an injective selected-capability identity. Preserving the old packet would have replaced one false claim with another.

## Decision — accepted 2026-08-11

**Accepted by Tom in the live decision round, relayed by the coordinating agent: carry a structured selected-lowering capability subject end to end.**

The compiler-owned subject is the closed lowering-family tag plus the exact `OpKey`. Provider identity and capability revision remain separate fields. The existing one-signature-per-`(family, operation, provider)` guard remains load-bearing; this decision does not add the signature to the human capability subject or reopen signature multiplicity.

The neutral artifact representation carries a governed capability-family key plus the exact structured `OpKey`. It length-frames family, operation namespace, operation name, and semantic version independently. A display spelling may exist for diagnostics, but it is never equality, ordering, deduplication, cache, receipt, or artifact identity authority.

This is intentionally liberal at the semantic boundary: every `OpKey` legal today remains legal. The family key retains its own governed bound, each operation component retains its own bound, and the enclosing artifact budgets bound the complete record. The lossy 256-byte combined-text ceiling is removed rather than imposed retroactively on operation identity.

No legacy parser may reinterpret the ambiguous flattened string. There is no lowercase fold, delimiter escape fallback, truncation, digest substitution, default family, or late packaging normalization.

## Ranked alternatives

1. **Accepted — structured fields.** Injective and fail-closed, removes repeated formatting/revalidation, and keeps semantic identity independent of one artifact text projection.
2. **Opaque compiler-minted canonical bytes.** Correct and strict, but less readable and duplicates minting/receiving bounds across layers.
3. **Versioned injective escaped text.** Correct if every delimiter, uppercase byte, and escape marker is encoded, but adds a second grammar and may still refuse otherwise legal maximum keys under the old combined bound.
4. **Operation-specific narrowing plus an aggregate bound.** Can be made correct, but couples the semantic extension vocabulary to one downstream projection and conflicts with the alpha preference to avoid arbitrary limits.
5. **Registration-time collision scanning.** Local and increasingly costly; it does not establish a globally injective representation.
6. **Former Option B, former Option A, folding, truncation, or status quo.** Rejected because the dotted-component collision survives or identity is silently changed.

## Identity and compatibility consequences

The implementation must replace public `SelectedCapability::capability_key() -> &str`, the private lossy lowering-provider record, and artifact `SelectedProvider.capability` coherently. At the implementation base it must rederive the complete identity population; the audited expectation is:

- lowering-registry, semantic-graph, refinement, and kernel-program identity grammars do not step merely for this carrier replacement;
- artifact provider-key domain `v2` moves to `v3`;
- artifact-program identity `v16` moves to `v17`;
- manifest schema `16.0` moves to `17.0` because the repeated selected-provider row changes shape;
- derived artifact, manifest, proof-sidecar artifact subject, envelope digest, and expansion-cache artifact facet are deliberately rebaselined;
- no old-domain fallback is admitted.

These versions are an audited expectation, not permission to copy stale absolute values: the implementation worker must rederive them at its exact base and reconcile concurrent schema changes.

## Delivery graph

[`replace-flat-selected-lowering-capability-keys-with-structured-subjects`](replace-flat-selected-lowering-capability-keys-with-structured-subjects.md) owns the public/API, codec, identity, panic-removal, fixture, and contract implementation.

The audit also found a separate delimiter join in explain-only provider references: `ProviderRef::registered` joins provider namespace and name with `.` even though both components admit dots. [`frame-provider-identities-before-using-them-as-explain-keys`](frame-provider-identities-before-using-them-as-explain-keys.md) owns that independently scoped correctness repair; it is not silently folded into artifact capability identity.

The selected-physical-provenance artifact migration depends on the structured capability migration so two branches do not independently claim the same next artifact schema version.

## Outcome

The exact public direction is accepted and the stale Facts, option packet, scope population, close condition, and identity consequences are repaired. Implementation remains ticketed work; this decision does not authorize a partial text-validation patch.
