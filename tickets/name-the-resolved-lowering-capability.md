---
id: name-the-resolved-lowering-capability
title: Carry the resolved lowering capability's governed key into the plan
status: done
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

## Outcome

**Done.** A caller outside `tiler-compiler` can now name the capability that lowered each occurrence, with no invented value.

**Ownership decision: the compiler mints the governed key.** The ticket required this to be stated. The alternative — expose the family and let the consumer compose a key — was rejected because the key enters artifact identity through the selected providers ADR 0072 folds in, so a consumer assembling it from parts would be a second derivation of one identity. That is the same drift hazard `relocate-abi-expressions-into-tiler-ir` was closing one layer up. A consumer now wraps an opaque string in its own key type rather than composing one.

**Spelling.** `tiler.capability.<family>.<op-namespace>.<op-name>.v<op-version>`, minted by `governed_capability_key` in `crates/tiler-compiler/src/lowering.rs` from the `ResolvedLoweringCapability` both resolution paths already hold. `LoweringFamily::key_token` gives the family token by exhaustive match rather than by `Display`, which renders prose for diagnostics; a capability key is durable identity, so a new family is a build error instead of a silently unnamed one (ADR 0074 convention 3). The operation's semantic version is in the key, so two versions of one operation never share it.

**`LoweringFamily`'s public surface did not widen beyond the decision.** It stays `#[non_exhaustive]` and its discriminant is untouched; `key_token` is an added method, not a new variant or a changed tag.

**Signature deliberately excluded, and the exclusion is owned rather than assumed away.** A capability is registered under family, operation, signature, and provider, so one provider could register two signatures for one operation family and both would mint this key. A `CapabilityKey` is bounded at 256 bytes and `LoweringSignature` is an unbounded structural value, so folding one in would either truncate — silently colliding, which is worse than the conflation because it would look distinguishing — or need a digest, a second identity to keep in agreement. Consumers record provider and revision beside the key, and the governed registry registers one signature per family and operation, so no colliding pair exists today. `resolve-capability-key-signature-conflation` owns widening the key or adding a check that fails closed when a second signature appears.

**Evidence.** New test `session::tests::an_alternative_names_its_capabilities_and_exposes_its_abi_inputs` asserts the key spelling against real resolution rather than against a fixture — it requires the reduction's capability to be named `tiler.capability.index-access.tiler.strict-serial-sum-f32.v1`, so a silently blank or misspelled key fails. It also pins the ABI arena property an assembler depends on: every operand precedes the node naming it, and every position the entries name is in range. 185 `tiler-compiler` tests pass, including the artifact-receipt and `provider_only_revision_changes_provenance_and_not_structure` conformance cases that compare `LoweringProviderIdentity` by value.
