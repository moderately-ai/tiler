---
id: record-the-capability-revision-in-selected-provider-identity
title: Record the capability revision in selected provider identity
status: in-progress
priority: p1
dependencies: []
related: [carry-the-metal-payload-in-an-artifact-envelope, name-the-resolved-lowering-capability, resolve-capability-key-signature-conflation]
scopes: [implementation/artifact, contracts/artifacts, contracts/foundation, implementation/metal-aot, research/artifacts]
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

## Outcome

**2026-07-25. The API version was replaced, not joined.** `SelectedProvider` is now `{provider: ProviderIdentity, capability: CapabilityKey, capability_revision: u32}`. The `u16` `capability_api_version` is gone.

**Why replaced rather than added beside, which is the half the decision left open.** This ticket's own `## Scope` fixed the test: "If the API version survives, name its authority: which component mints it, and what it is a version *of*. If nothing can mint one today, say so and remove it rather than leaving a field every producer must fill with something." Nothing can. **Exact check, reproducible in one line:** `grep -rni "api_version\|api version" crates/` returned six hits before this change, every one inside `tiler-artifact` — the field's declaration, its canonical-key fold, its encode, its decode, one fixture, and one doc example — and none in any component that could supply a value. After this change it returns nothing. Keeping the field would have kept a slot in artifact identity whose only possible content is a guess, which is the defect the ticket was filed about rather than a second one to tolerate beside it.

The `## Closes when` clause makes the same call: "records everything `docs/operation-extensions.md` requires a selected plan to record, with **no conflated and no invented value**". A field with no producer cannot be filled with anything but an invented value.

**The requirement is not abandoned, it is made legible.** `docs/operation-extensions.md` still says compiler and capability-API versions participate in identity, and that sentence is now marked in place as a requirement rather than a description, pointing at `name-the-capability-api-version-authority-or-retire-the-requirement`. `docs/artifact-abi.md` states the same from the artifact's side: what the provider row carries, what it deliberately does not, and why absence is the fail-closed reading.

**`u32`, not `u16`.** `LoweringCapabilityRevision` is a `u32`, so a `u16` field could not hold every value the compiler can mint, and the assembler's checked narrowing was a refusal path that existed only because of the width mismatch. It and `BundleError::CapabilityRevisionWidth` are both gone; the assembler now passes `selected.capability_revision()` through unchanged.

**Not validated nonzero here, deliberately.** `tiler-compiler` documents the revision nonzero and this layer does not re-check it, exactly as the `FeasibilityRuleSetRef::revision` beside it does not. Adding a refusal on one and not the other would make two received revisions mean different things at one boundary; adding it to both is a separate change with its own argument.

### Versioning — three constants moved, and each for a stated reason

- `MANIFEST_SCHEMA` `3.0` → **`4.0`**. Major, and for a stronger reason than the previous two steps: the field's *width* moved from two bytes to four, so a `3.0` reader does not merely misinterpret a value, it loses framing for every row after it. The reader admits `minor <= implemented`, so a minor step would have left it accepting a manifest it can no longer parse.
- `ARTIFACT_DOMAIN` `v2` → **`v3`**, and `PROVIDER_KEY_DOMAIN` `v1` → **`v2`**. Both, not one: a provider key is folded into the artifact identity *and* sorted and deduplicated against its siblings on its own, so the record has to be self-describing rather than relying on the enclosing domain to separate it. Without the retag, a `v2` provider key and a `v3` provider key of two different selections could differ only in bytes a reader of either domain would have consumed as something else.

`ENVELOPE_FORMAT` and `CANONICAL_ENCODING` stay at `{1, 0}`: the manifest's contents moved, not the framing around them.

**Every existing artifact identity changes.** That is the intended consequence and not a migration cost to soften — the whole defect was that a capability revision change produced an unchanged identity. There is nothing to migrate: no artifact is persisted anywhere in this repository, and a stale one now fails closed on the manifest schema rather than decoding into a plausible wrong program.

### The defect was untested, and now is not

`a_reached_capability_provider_revision_changes_identity` varies the **provider's** revision and passed before and after. Nothing exercised the capability's own revision, which is exactly why the drift could exist. `a_reached_capability_revision_changes_identity` now builds two artifacts differing only in `capability_revision` and asserts their canonical identities differ, that a rebuild at the original revision reproduces the original identity, and that the provider identity is byte-equal across the pair — so the test fails if the assertion ever passes for the wrong reason.

### What landed

- `crates/tiler-artifact/src/program/model.rs`: the field, its documentation of why two revisions are independent and that this one is received rather than derived, its fold into `canonical_key`, and the two domain bumps with the reason at each site.
- `crates/tiler-artifact/src/program/codec/{encode,decode}.rs`: the `u32` wire field and the `MANIFEST_SCHEMA` step with its reason.
- `crates/tiler-artifact/src/program/{mod,tests}.rs`: the doc example, the fixture, and the regression case.
- `prototypes/serial-sum-compile/src/bundle.rs`: `capability_version`, its retraction comment, and `BundleError::CapabilityRevisionWidth` are gone; the module's own summary of what enters identity now names the capability revision.
- `docs/artifact-abi.md` and `docs/operation-extensions.md` as described above.
- Split: `name-the-capability-api-version-authority-or-retire-the-requirement`.
