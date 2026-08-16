---
id: replace-flat-selected-lowering-capability-keys-with-structured-subjects
title: Replace flat selected lowering capability keys with structured subjects
status: done
priority: p1
dependencies: [reconcile-the-operation-identity-and-governed-key-grammars, decide-the-selected-lowering-capability-subject-rust-surface]
related: [reconcile-the-two-target-profile-key-grammars, package-selected-physical-implementation-provenance-in-artifact-identity, frame-provider-identities-before-using-them-as-explain-keys]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, implementation/runtime, contracts/foundation, contracts/artifacts, research/artifacts, research/cache, research/target-profiles, research/runtime, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [identity, validation, extensions, implementation, public-boundary, artifact, schema]
---
## User-visible outcome

Every legal selected lowering capability remains packageable and two distinct `(family, operation)` subjects can never collapse through a delimiter-composed string.

## Source-first audit at the implementation base

The implementation audit was rerun at exact base `98669e8ea9cafc91b3a9139ff821781560c526bd` on 2026-08-16 before any production source changed.

- **Verified.** `governed_capability_key` in `crates/tiler-compiler/src/lowering.rs` still formats `tiler.capability.{family}.{namespace}.{name}.v{version}` as one `String`, and `LoweringFamily` in `crates/tiler-compiler/src/capability.rs` still has exactly the live `IndexAccess` variant. Reproduce with `rg -n 'governed_capability_key|pub enum LoweringFamily|IndexAccess' crates/tiler-compiler/src/{lowering,capability}.rs` and read the named definitions.
- **Verified — corrected source boundary and reproducer.** `OpKey` is defined in `crates/tiler-ir/src/semantic/operation.rs`; its `new` and `from_owned` delegate to the structured key whose validator lives in `semantic/types.rs`. Each namespace and name is nonempty, bounded by `MAX_IDENTITY_COMPONENT_BYTES = 255`, and admits ASCII alphanumeric bytes (including uppercase) plus non-leading `.`, `_`, and `-`. Reproduce with `rg -n 'pub struct OpKey|pub fn from_owned|pub fn new' crates/tiler-ir/src/semantic/operation.rs` and `rg -n 'MAX_IDENTITY_COMPONENT_BYTES|fn validate_component|fn from_owned' crates/tiler-ir/src/semantic/types.rs`, then read both complete definitions.
- **Verified.** Artifact `CapabilityKey` remains one governed text value under the 256-byte lowercase artifact-key grammar in `crates/tiler-artifact/src/program/keys.rs`.
- **False — repaired below.** The current tree has **eight**, not seven, reconstructions from `selected.capability_key()`: five `.expect()` assertions, two direct `?` propagations, and one lossy `VerticalError::HostProfile` remapping. Reproduce with `rg -n -C 4 'CapabilityKey::new\(selected\.capability_key\(\)\)' crates prototypes spikes --glob '*.rs'`. The added eighth site is `spikes/runtime/backend-provider-portfolio/src/portfolio.rs`, so this ticket now also declares `research/runtime`.
- **Verified.** The dotted collision remains legal and lossy: `OpKey::new("a.b", "c", 1)` and `OpKey::new("a", "b.c", 1)` pass the component grammar and `governed_capability_key` maps both to `tiler.capability.index-access.a.b.c.v1`.
- **Verified.** The one-signature law is still enforced by the `ConflatedCapabilityKey` branch in `LoweringCapabilityRegistryBuilder::register`; it compares the exact family, `OpKey`, and provider and rejects only a differing signature for that same triple. Distinct dotted-boundary `OpKey` values therefore remain separately registerable.
- **Verified.** `encode_capability_key` still independently frames the exact `OpKey` components for the lowering-registry identity, while `LoweringProviderIdentity` still stores the downstream subject as `capability_key: String`. `ResolvedLowering::providers` and `resolve_capabilities` both sort and deduplicate that lossy record; public `SelectedCapability::capability_key` returns the text, and artifact `SelectedProvider.capability` retains only an artifact `CapabilityKey` decoded from one text field.
- **Imprecise at this base.** The decision-base identity expectation is no longer the current step. `PROVIDER_KEY_DOMAIN` remains `tiler.artifact-program.provider.v2`, but `ARTIFACT_DOMAIN` is already `tiler.artifact-program.v17` and `MANIFEST_SCHEMA` is already `17.0` after the retained shape-environment landing. The structured provider row therefore requires provider `v2` to `v3`, artifact `v17` to `v18`, and manifest `17.0` to `18.0`, subject to recomputation from the completed diff rather than copied pins. Reproduce with `rg -n 'PROVIDER_KEY_DOMAIN|ARTIFACT_DOMAIN|MANIFEST_SCHEMA' crates/tiler-artifact/src/program`.
- **Verified — public-boundary authority is now supplied.** [`decide-the-selected-lowering-capability-subject-rust-surface`](decide-the-selected-lowering-capability-subject-rust-surface.md) is `done`, and its `Accepted answer — 2026-08-14` records Tom's exact compiler and artifact subject types, constructor and accessor visibility, replacement `SelectedCapability::subject()` signature, no-`Display` rule, owned decode path, public malformed classification, eight-adapter projection, and explicit exclusions. The earlier audit at `72b1357f892335e4883494c0e1906be89998258b` stopped because that exact Rust surface had not yet been accepted; that stop condition is satisfied at this implementation base without changing this ticket's purpose.

## Required delivery

- Reverify every Fact and identity/version consequence from the accepted decision at the exact implementation base before editing production source.
- Replace the compiler's flattened capability string with one typed subject containing the closed lowering-family value and exact `OpKey`. Mint it once at registration/resolution and retain it through `LoweringProviderIdentity` and public `SelectedCapability`; keep provider and capability revision separate.
- Retire artifact text `CapabilityKey` and replace selected-lowering provenance with the exact artifact `LoweringCapabilitySubject` fixed by [`decide-the-selected-lowering-capability-subject-rust-surface`](decide-the-selected-lowering-capability-subject-rust-surface.md), containing a governed `CapabilityFamilyKey` and exact `OpKey`. Encode and decode the family, namespace, name, and version as separately framed fields. Consume the decoder's owned namespace/name strings through `OpKey::from_owned`; preserve `TypeIdentityError` as the source of crate-private `ArtifactCodecError::InvalidOperationKey`, and classify that internal rejection through the existing public `ArtifactCodecFailure::Malformed { detail }` arm without claiming the typed source survives the public classifier.
- Keep the existing one-signature-per-provider/family/operation rule. Do not add signature to the selected subject or permit a second signature under a subject that cannot name it.
- Remove all eight downstream reconstructions of `CapabilityKey` from a flattened compiler string. In every adapter apply the one governed projection contract: compiler `LoweringFamily::key_token()` is the sole family-token authority, `CapabilityFamilyKey::new` is the sole receiving validation, and the exact `OpKey` is cloned whole. Do not format, parse, match, default, or remap the family locally. Replace the five conversion assertions and the lossy target-profile remap with typed `ArtifactBuildError` propagation; an intentionally infallible fixture may assert only at its existing outer fixture boundary after its internal assembly path preserves that typed cause.
- Reconcile the exact artifact provider-row domain, artifact-program domain, manifest schema, identity ledger, ABI contract, fixtures, proof-sidecar subjects, envelope digests, cache subjects, and every derived pin. Do not step unrelated identity domains.
- Do not add `Display` to either new composite subject: no accepted contract or live consumer requires a composite spelling, while their typed components already support diagnostics. Assert that no equality, ordering, deduplication, cache, receipt, or artifact identity consumer formats the components as authority.
- Add a public-boundary record under ADR 0075 for the exact included and excluded `SelectedCapability` and artifact subject surfaces.

## Required negative controls

- Register `("a.b", "c", 1)` and `("a", "b.c", 1)` under the same provider and capability revision; both must remain distinct through resolved-provider census, public selection evidence, artifact rows, codec round-trip, and identity.
- Perturb only the family, namespace/name boundary, operation version, provider, and capability revision independently and prove each identity-bearing assertion fails with its subject named.
- Exercise uppercase and maximum-length legal operation components and prove they remain packageable without folding, truncation, or a late text conversion.
- Corrupt each decoded structured component independently. Preserve and inspect the crate-private typed cause, while asserting the truthful public result is `ArtifactCodecFailure::Malformed { detail }` with no source; no legacy flattened-key interpretation is allowed.
- Temporarily perturb only existing `LoweringFamily` or `OpKey` diagnostic display punctuation, prove canonical identity does not move, then restore it; the new composite subjects have no `Display` surface.

## Non-goals

Narrowing `OpKey`, changing the governed family-key grammar, permitting multiple signatures for one selected subject, hashing the subject in place of encoding it, retaining the ambiguous spelling as a compatibility fallback, or changing lowering/provider selection.

## Closes when

The structured subject is the only selected-capability authority across compiler and artifact layers, the collision pair remains distinct end to end, every legal current `OpKey` remains admitted, all schema and derived identities reconcile, exact-tip full gates pass, and an independent identity-sensitive review reports no findings.

## Outcome — 2026-08-16

Implemented at reviewed branch commit `554aa07e1cc8fd5064cba4545a02aba9ae0129fa` and integrated on main as `b23f07226b5c8d97bf0e3de14c906f9e09307198`. The compiler and artifact now carry layer-owned structured lowering-capability subjects; all eight adapters perform the checked family projection and clone the exact `OpKey`; provider key v3, artifact v18, and manifest 18.0 are coherent; the dotted-boundary pair remains distinct through selection, codec, proof, and envelope identity; and the legacy flat path is absent.

The production-identical branch passed the full repository gate. Independent exact-tip review reported no findings after 1,325 focused package tests, doctests, all seven nested spike checks, typed Display/domain/frame perturbations, Clippy, rustdoc, ticket lint, citations, diff-check, and scope guard. Three unrelated exact-base spike defects discovered during the work were preserved as separate P1 tickets rather than folded into this identity migration.
