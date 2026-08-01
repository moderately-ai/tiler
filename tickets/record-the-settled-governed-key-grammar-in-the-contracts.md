---
id: record-the-settled-governed-key-grammar-in-the-contracts
title: Record the settled governed-key grammar in the contracts
status: in-progress
priority: p2
dependencies: []
related: [reconcile-the-two-target-profile-key-grammars]
scopes: [contracts/artifacts, contracts/decisions, contracts/foundation, research/extensions]
shared_scopes: []
paths: []
tags: [identity, documentation]
claimed_from: todo
assignee: worker-key-grammar
lease_expires_at: 1785581736
---
## User-visible outcome

No contract or accepted decision still describes the artifact layer's governed-key grammar as unsettled or as length-only, and the shared `TargetProfileKey` spelling is indexed where a reader looking up an ambiguous name will find it.

## Why this slice exists

**Fact.** `reconcile-the-two-target-profile-key-grammars` settled the question: the artifact layer's six `governed_key!` types now enforce the same alphabet as `tiler_compiler::target::TargetProfileKey` (ASCII lowercase, digits, `.`, `-`, `_`), while the byte bounds stay deliberately different — 128 is one producer's minting bound, 256 is the artifact layer's admission bound. That ticket held only implementation scopes, so three sentences it invalidated are still in the tree:

- `docs/artifact-abi.md:101` records the asymmetry as one that ticket "owns and that record deliberately does not settle".
- `docs/artifact-abi.md:286` says a governed key "is bounded at 256 UTF-8 bytes because this layer governs what a producer may name" without mentioning that the layer now also governs the spelling.
- `docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md:103` states as **Fact** that "there is no alphabet, case, separator, or namespace check in the crate", with a reproduction (`grep -rn "is_ascii\|InvalidByte" crates/tiler-artifact/src/`) that no longer returns nothing; `:149` lists "whether the artifact layer should enforce the compiler's key alphabet" as an open question.

**Inference.** ADR 0090's item 10 itself is not superseded — it governs namespace *minting*, not spelling, and the record already separates the two ("and separately from the namespace question"). What changed is the Fact paragraph beside it and the open question it deferred, so this is a correction and a closed question rather than a new decision.

The glossary row is the second half. `docs/glossary.md` already indexes names denoting several unrelated subjects, and `TargetProfileKey` — one type in `tiler-compiler` that a compilation is assessed against, one in `tiler-artifact` that a packaged program carries, with different bounds and, until now, different grammars — is not among them. The research record that surfaced this (`docs/research/extensions/backend-provider-composition.md`) got it wrong precisely by using one of each and describing them in one sentence, which is the failure mode a glossary row exists to prevent.

## Implementation keys

- The rustdoc at `crates/tiler-artifact/src/program/keys.rs` and `crates/tiler-compiler/src/target.rs` is the settled contract for the code-level subject and already carries the derivation; these edits cite rather than re-derive it, and must not restate it in a way that can drift.
- Re-run each reproduction command before rewriting the sentence that prints it. ADR 0090's is now a positive control rather than a negative one, and a record whose stated reproduction does not reproduce is worse than one that is merely stale.
- Correct the research record's finding 2 in the same pass; it is the origin of the conflation.
- Do not restate the 128-versus-256 difference as an unresolved gap. It is a decided asymmetry with a direction argument, and a contract sentence that reopens it invites a future worker to "fix" it by narrowing the artifact bound to one producer's number.

## Closes when

Every sentence above states current behaviour, the ADR's open question is closed by pointing at where its answer lives, and the glossary indexes the shared `TargetProfileKey` name with both subjects.
