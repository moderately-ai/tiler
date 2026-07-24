---
id: bind-section-purpose-and-schema-into-the-section-descriptor
title: Bind a section's purpose and schema into its descriptor and digest
status: todo
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec, record-the-implemented-artifact-envelope-in-the-contract, prototype-metal-bundle-assembly]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization]
---
`docs/artifact-abi.md` states two things about a section descriptor that the implemented codec does not do. The contract is the correct side of both disagreements; the implementation is the narrower one.

**Fact — the descriptor is missing two of its four declared fields.** "Every section descriptor contains its required/optional meaning, schema, exact byte length, and digest." The descriptor `encode_section_descriptors` writes in `crates/tiler-artifact/src/program/codec/encode.rs` is an ordered identifier, a purpose tag, the exact byte length, and the content digest. There is no per-section schema version and no required/optional disposition.

**Fact — the section digest does not cover the section's purpose.** The contract's derivation is `section_digest[i] = H("tiler-section-v1" || section_type/schema || exact section bytes)`. The implemented pre-image is `H("tiler.artifact-envelope.section-digest.v1\0" || exact section bytes)` — the purpose is absent.

**Why this matters, and why it is not urgent today.** Inside a complete envelope the purpose is already bound one level up: the manifest descriptor names it and the manifest digest covers the descriptor, so a swapped purpose is caught. The gap opens the moment a section digest is used as a *standalone* content address, because two sections with equal bytes and different purposes would then share one address. `prototype-metal-bundle-assembly` is the first consumer that will want exactly that — content-addressing a metallib code section — so this should close before, or with, that work rather than after it.

**What closes this.** Add the section's schema version and its required/optional disposition to the descriptor, fold the purpose and schema into the section-digest pre-image, and state in `docs/artifact-abi.md` that the implemented derivation now matches the contract's. Adding fields to the descriptor changes the manifest bytes and therefore every envelope digest, which is free while the codec is `pub(crate)` and no artifact is persisted anywhere; doing it after a bundle format exists is not. Bumping the manifest schema minor is the mechanism, and the reader's existing `UnsupportedManifestSchema` rejection already handles the skew.

**Scope note.** Deliberately left unscoped: the fix touches `implementation/artifact` and `contracts/artifacts`, and the ticket that takes it should claim both. It must not decide whether a bundle's identity is content-addressed over compilation inputs or over payload bytes — that is `prototype-metal-bundle-assembly`'s.
