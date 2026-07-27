---
id: name-the-capability-api-version-authority-or-retire-the-requirement
title: Name the capability-API version authority or retire the requirement
status: done
priority: p2
dependencies: []
related: [record-the-capability-revision-in-selected-provider-identity, name-the-resolved-lowering-capability]
scopes: [contracts/foundation, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, artifact, identity, needs-tom]
---
Split from `record-the-capability-revision-in-selected-provider-identity`, which put the *capability revision* into artifact identity and removed the field that had been standing in for a capability-API version. This ticket owns the half that removal deliberately did not answer.

**Fact — the contract requires it.** `docs/operation-extensions.md`: "Compiler and capability-API versions also participate in identity." That sentence sits beside the provider-revision trust contract, so it is naming two further identity components rather than restating the provider revision in other words.

**Fact — no component mints either.** Construction-site inspection shows that
`SelectedCapability` and the artifact selected-provider row carry provider
identity/revision plus capability key/revision. No production type carries a
capability-API or Tiler compiler version, and no producer mints or compares
either. Textual mentions remain in artifact schema history, so a substring
search is not an absence check.

**Fact — what a producer did with the field while it existed.** `prototypes/serial-sum-compile/src/bundle.rs` narrowed the compiler's `u32` capability revision into the `u16` `capability_api_version` slot through a checked conversion that refused rather than truncating, with the conflation named at the call site. That was the honest form of an unanswerable question, and it still put a value into artifact identity under a name that meant something else.

**Inference — an absent component is safer than an invented one, and it is still a gap.** Removing the field means an artifact no longer asserts a version nothing established. It does not mean the contract's requirement was satisfied, and a reader of `docs/operation-extensions.md` alone would still expect the value to travel. Both documents now say so explicitly; this ticket closes the difference between saying so and answering it.

## The decision this ticket owes

Ensure artifact identity changes whenever a compiler or capability-interface
change can change executable meaning or bytes. Either name enforceable version
authorities and mismatch behavior, or prove that existing provider/capability
revisions cover that risk and retire the extra requirement.

One of two, with the derivation rather than the preference:

1. **Something mints a capability-API version.** Then name it: which component owns it, what it is a version *of* — the Rust calling contract a capability implementation is compiled against, per `docs/research/extensions/operation-extension-api.md:162`, is the candidate — how a provider learns it, and what a mismatch between the version an artifact records and the one a reader implements must do. A version that cannot be violated is not an identity component. The same question then applies to the compiler version the sentence pairs it with, and it may not have the same answer.
2. **The requirement is retired.** Then `docs/operation-extensions.md` says so and states what covers the risk instead: whether the provider revision and the capability revision together distinguish two artifacts that a capability-API change would separate, and what a provider is obliged to do when the API it was compiled against moves under it.

Do not add a field before answering. A slot every producer must fill with something is the exact defect the split removed, and reinstating one under a different name would repeat it.

## Closes when

`docs/operation-extensions.md` either names an authority for a capability-API version and a compiler version, with the mismatch behaviour each implies, or records that the requirement is retired and what covers it instead; `docs/artifact-abi.md`'s selected-provider record agrees; any field added has a producer that can supply it; and `make full` passes.

## Outcome — option 2, retired, with the derivation (2026-07-27)

Recorded in `docs/operation-extensions.md` and agreed by `docs/artifact-abi.md`. **No field was added**, which the ticket asked for explicitly.

**The capability-API half is eliminated by this document's own scope statement.** Its "Initial trust and linkage model" states that providers are "trusted native compiler code, statically linked into the process", and that "native dynamic loading, hot reload, a stable Rust plugin ABI, untrusted plugins, and cross-process providers are deferred". A provider and the capability API it is written against are therefore compiled into one binary. A provider not rebuilt against a changed API does not produce a mismatched artifact — **it fails to compile.** There is no reader implementing one version and no producer implementing another, so no mismatch a recorded version could detect, and the ticket's own test applies: a version that cannot be violated is not an identity component.

The requirement becomes live again only if one of those deferred linkage models is admitted, which is where it should be reconsidered rather than pre-emptively met.

**The compiler half is discharged by content addressing, and this is the part I had wrong first.** My initial reading was that the payload digest covers only the compilation *subject* and not the emitted object, so two Tiler builds emitting different MSL from one semantic program would share an identity — a real gap. Reading `crates/tiler-artifact/src/program/codec/payload.rs` in full corrected it: the subject **contains the exact source that was compiled**, along with the target, the compile and link flags, and the toolchain provenance. A Tiler build that emits different source therefore yields a different artifact identity by construction, and one that emits the same source, flags, ABI, and schedule cannot change executable meaning. A backend-toolchain change is covered by the folded provenance.

**Inference — recording a compiler version would be weaker.** It asserts what produced an artifact rather than what the artifact is, so two builds emitting identical source would get different identities and lose a legitimate cache hit, while the case it claims to catch is already caught by the source it alters.

**Not covered, and recorded as such in the contract.** A provider changing output-affecting behaviour without bumping its declared revision remains a provider bug — and no capability-API version would have caught it, since the API is unchanged. Separately, two bundles built from one subject by a non-reproducible linker share an identity and differ in envelope digest; that is an already-decided property of content addressing over inputs, not a gap this opens.
