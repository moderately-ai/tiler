---
id: name-the-capability-api-version-authority-or-retire-the-requirement
title: Name the capability-API version authority or retire the requirement
status: todo
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

**Fact — no component mints either.** Exact check on the commit this was split from: `grep -rni "api_version\|api version" crates/` returned hits only inside `tiler-artifact`, and after the split it returns none at all. `tiler-compiler` publishes `SelectedCapability::capability_revision` and nothing else version-shaped; there is no compiler-version constant either. Neither value has an authority, a mint site, or a stated definition of what it versions.

**Fact — what a producer did with the field while it existed.** `prototypes/serial-sum-compile/src/bundle.rs` narrowed the compiler's `u32` capability revision into the `u16` `capability_api_version` slot through a checked conversion that refused rather than truncating, with the conflation named at the call site. That was the honest form of an unanswerable question, and it still put a value into artifact identity under a name that meant something else.

**Inference — an absent component is safer than an invented one, and it is still a gap.** Removing the field means an artifact no longer asserts a version nothing established. It does not mean the contract's requirement was satisfied, and a reader of `docs/operation-extensions.md` alone would still expect the value to travel. Both documents now say so explicitly; this ticket closes the difference between saying so and answering it.

## The decision this ticket owes

One of two, with the derivation rather than the preference:

1. **Something mints a capability-API version.** Then name it: which component owns it, what it is a version *of* — the Rust calling contract a capability implementation is compiled against, per `docs/research/extensions/operation-extension-api.md:162`, is the candidate — how a provider learns it, and what a mismatch between the version an artifact records and the one a reader implements must do. A version that cannot be violated is not an identity component. The same question then applies to the compiler version the sentence pairs it with, and it may not have the same answer.
2. **The requirement is retired.** Then `docs/operation-extensions.md` says so and states what covers the risk instead: whether the provider revision and the capability revision together distinguish two artifacts that a capability-API change would separate, and what a provider is obliged to do when the API it was compiled against moves under it.

Do not add a field before answering. A slot every producer must fill with something is the exact defect the split removed, and reinstating one under a different name would repeat it.

## Closes when

`docs/operation-extensions.md` either names an authority for a capability-API version and a compiler version, with the mismatch behaviour each implies, or records that the requirement is retired and what covers it instead; `docs/artifact-abi.md`'s selected-provider record agrees; any field added has a producer that can supply it; and `uv run --locked python scripts/check_repository.py` passes.
