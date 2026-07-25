---
id: record-the-capability-revision-in-selected-provider-identity
title: Record the capability revision in selected provider identity
status: in-progress
priority: p1
dependencies: []
related: [carry-the-metal-payload-in-an-artifact-envelope, name-the-resolved-lowering-capability, resolve-capability-key-signature-conflation]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, identity]
claimed_from: todo
assignee: agent-artifact
lease_expires_at: 1785017446
---
`SelectedProvider` cannot record the value `docs/operation-extensions.md` says a selected plan records, and asks instead for one no producer can supply. The first artifact assembler hit this and carried a real value into an adjacent slot rather than inventing a plausible one; that stopgap is in code and named, and this ticket closes it.

**Fact — the normative contract.** `docs/operation-extensions.md`: "a selected plan records the `{provider identity, capability revision}` pair each occurrence resolved, and the compiler re-derives that set from the installed registry rather than trusting what a plan recorded." The same section separately says "Compiler and capability-API versions also participate in identity", so the revision and the API version are two facts rather than one fact spelled twice.

**Fact — the compiler supplies the revision and not the API version.** `crates/tiler-compiler/src/capability.rs:108-133` defines `LoweringCapabilityRevision(u32)`, documented as "a nonzero output-affecting revision of one registered lowering capability ... distinct from the admitting `ProviderIdentity` revision", and `tiler_compiler::session::SelectedCapability::capability_revision() -> u32` exposes it. `grep -rni "api_version" crates/` returns six hits, every one inside `tiler-artifact`: there is no capability API version anywhere in `tiler-compiler`.

**Fact — the artifact model has one slot and it holds the other fact.** `crates/tiler-artifact/src/program/model.rs:271-278`: `SelectedProvider { provider: ProviderIdentity, capability: CapabilityKey, capability_api_version: u16 }`, the third field documented as "version of the capability API the selection was made against". `canonical_key` (`model.rs:281-290`) folds all three and `codec/encode.rs:212` writes the `u16`, so the field is in artifact identity and in the wire format.

**Inference — two defects, not one.** The capability *revision* is dropped, so a provider that changes its output-affecting lowering revision without changing its provider revision produces an identical artifact identity — exactly the drift the revision exists to catch. And the capability *API version* has no producer, so whatever an assembler writes there is a claim nothing established.

**What the assembler does today, and why it is not the answer.** `prototypes/serial-sum-compile/src/bundle.rs::capability_version` narrows the compiler's revision into the `u16` slot with a checked conversion that refuses rather than truncating, and its doc comment names the conflation and this ticket. It was chosen over the two alternatives: hard-coding a constant would be an invention, and dropping the value would remove a real identity component. It is still a conflation, and an artifact currently asserts an API version where it means a revision.

## Scope

Decide whether `SelectedProvider` gains a `capability_revision` field beside the API version, or whether the API version is the wrong field and should be replaced. Either changes `canonical_key`, the manifest encoding, and therefore every existing artifact identity, so the encoding version and the migration posture are part of the decision.

If the API version survives, name its authority: which component mints it, and what it is a version *of*. If nothing can mint one today, say so and remove it rather than leaving a field every producer must fill with something.

`u16` versus `u32` is part of the question. `LoweringCapabilityRevision` is `u32`, so a `u16` field cannot hold every value the compiler can mint, and the assembler's checked narrowing is a refusal path that should not need to exist.

## Closes when

An out-of-crate assembler records everything `docs/operation-extensions.md` requires a selected plan to record, with no conflated and no invented value; `prototypes/serial-sum-compile/src/bundle.rs::capability_version` and its retraction comment are gone; the encoding change is versioned; and `uv run --locked python scripts/check_repository.py` passes.

## Decision — Tom, 2026-07-25

**Approved: promote.** ADR 0075 reserves public-surface promotions to the owner; this one is granted.

`SelectedProvider` gains a slot for the capability revision. The assembler currently carries the real `u32` through a checked narrowing into a `u16` that refuses rather than truncates, with the conflation named at the call site — honest, but still a conflation, and artifact identity should record which capability revision actually lowered rather than a narrowed proxy.
