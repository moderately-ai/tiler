---
id: correct-the-five-key-domains-noun-on-the-model-re-export-block
title: Correct the five key domains noun on the model re-export block
status: done
priority: p3
dependencies: []
related: [reattach-the-scalar-arithmetic-block-and-correct-its-accessor-count]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift]
---

## User-visible outcome

The domain re-export comment in `crates/tiler-artifact/src/program/mod.rs` names the five `model` `cfg(test)` re-exports accurately: four program key domains and the artifact-identity separator, so a reader counting "key domains" from the comment alone does not misclassify `ARTIFACT_DOMAIN`.

## Finding (Neighbouring census on reattach-the-scalar-arithmetic-block-and-correct-its-accessor-count)

**Fact.** The block anchored `// five key domains re-exported from \`model\`, and \`ROUTE_REQUIREMENT_DOMAIN\`.` lists five names under the `#[cfg(test)] pub(crate) use model::{…}` re-export: `ARTIFACT_DOMAIN`, `DEFERRED_KEY_DOMAIN`, `PAYLOAD_KEY_DOMAIN`, `PROVIDER_KEY_DOMAIN`, `STAGE_KEY_DOMAIN`. The count five is right.

**Fact.** `crates/tiler-artifact/src/domains.rs` documents `ProgramArtifact` as `Separator opening the canonical artifact-program identity` and the four others as `Separator of one … canonical key` variants. `ARTIFACT_DOMAIN` is the identity separator, not a key domain. The noun "key domains" covers four of the five.

## Work

Re-read the domain block and the `domains.rs` `Program*` docs at the worker base. Substitute the live `five key domains` wording so the comment distinguishes the identity separator from the four key domains without changing the re-export list, domain values, or `cfg(test)` visibility. Prefer ADR 0106 substitution if the clause was never true at any commit. Do not expand scope to other census claims.

## Closes when

The live comment no longer calls all five model re-exports "key domains", the four key separators and the identity separator remain named accurately, and a re-read of `domains.rs` still agrees.

## Worker audit (2026-08-10)

**Fact verdicts at `313afe61758a62f7ca5672ee430a1dddb279fae1`: verified.**

- The `program/mod.rs` comment anchored `five key domains re-exported from \`model\`` preceded a `#[cfg(test)]` model re-export of exactly `ARTIFACT_DOMAIN`, `DEFERRED_KEY_DOMAIN`, `PAYLOAD_KEY_DOMAIN`, `PROVIDER_KEY_DOMAIN`, and `STAGE_KEY_DOMAIN`; five is therefore the correct count.
- In `domains.rs`, `ProgramArtifact` is anchored by `Separator opening the canonical artifact-program identity.`, while `ProgramStageKey`, `ProgramPayloadKey`, `ProgramProviderKey`, and `ProgramDeferredKey` are each documented as a separator for one canonical key. The original collective noun was consequently imprecise, not a basis to change values, visibility, or the re-export population.

## Outcome

The comment now distinguishes the one artifact-identity separator from the four key domains. No model constants, identity bytes, values, visibility, or re-export list changed.

## Review correction (2026-08-10)

Exact-hash review of `4a7fd05e6a5cb3d85f0316f76c7a6a82fd6a9a8c` found the wrapped comment read `these seven, the the artifact-identity`. This follow-up removes only the retained first `the`; the live clause remains `the artifact-identity separator and four key domains`.
