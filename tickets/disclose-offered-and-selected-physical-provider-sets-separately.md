---
id: disclose-offered-and-selected-physical-provider-sets-separately
title: Disclose offered and selected physical-provider sets separately
status: in-progress
priority: p1
dependencies: [drive-an-external-physical-implementation-provider-through-compilation]
related: [accept-the-public-backend-provider-composition-boundary]
scopes: [implementation/compiler]
shared_scopes: [contracts/decisions, project/tickets]
paths: []
tags: [implementation, compiler-api, backend-providers, explainability, public-boundary]
claimed_from: todo
assignee: coord
lease_expires_at: 1786188969
---
## User-visible outcome

Explain output distinguishes which providers were *offered* a compilation from which were *selected* for it, and a physical-implementation provider's identity can appear in both — so a caller can see that its installed provider was consulted and lost, rather than reading an empty set and being unable to tell that from never having been asked.

## Why this exists

**Fact — the offered set is populated from the lowering registry alone.** `crates/tiler-compiler/src/session.rs:1513` constructs `offered_providers: Arc<[ProviderIdentity]>` from the lowering capability registry and passes it to `into_compilation_batch` (`:1520`); it reaches the compilation through `:1841` and is read back through the accessor at `:761`. No physical-implementation provider contributes to it, so no physical provider's identity can appear in explain output at all.

**Fact — this is item 5 of an accepted ADR, and it is the one item with no ticket.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):19 records, in the accepted status paragraph, that "the item-2 physical-provider registry and the item-5 disclosure accessors remain unimplemented", and `:143` states that implementation follows item by item. Item 2 has [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md) and item 11's orchestration promotion landed on 2026-08-01; item 5 had nothing.

**Inference — the separation is a correctness property of explain, not a convenience.** AGENTS.md concentrates correctness scrutiny on "explain output for accepted and rejected rewrites, candidates, guards, capabilities, and assumptions". Collapsing offered into selected makes a rejected candidate indistinguishable from an absent one, which is precisely the distinction explainability exists to preserve.

## Public boundary — draft, do not self-accept

The accessors are a public boundary. ADR 0090:19 states it in terms: "every concrete public surface named here — the provider registry and its installation method, **the offered-versus-selected disclosure accessors**, the promoted `assemble_artifact` boundary — still comes to Tom at implementation time under [ADR 0075]". [`accept-the-public-backend-provider-composition-boundary`](accept-the-public-backend-provider-composition-boundary.md) is `done` and routed exactly these to him. Land a reviewed draft with an out-of-crate fixture exercising it, and put the exact accessor shapes, their ownership, and their naming to Tom before acceptance.

## Closes when

An installed physical-implementation provider that was consulted and not selected is visible as offered-and-not-selected in explain output from an out-of-crate caller; a provider never installed is distinguishable from one installed and rejected, with a check observed failing when the two are conflated; ADR 0090's status paragraph is corrected to stop naming item 5 as unimplemented; and the accessor shapes have gone to Tom.
