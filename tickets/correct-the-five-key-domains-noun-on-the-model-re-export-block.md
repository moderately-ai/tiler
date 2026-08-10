---
id: correct-the-five-key-domains-noun-on-the-model-re-export-block
title: Correct the five key domains noun on the model re-export block
status: in-progress
priority: p3
dependencies: []
related: [reattach-the-scalar-arithmetic-block-and-correct-its-accessor-count]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift]
claimed_from: todo
assignee: terra-artifact-domain-comment
lease_expires_at: 1786406376
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
