---
id: bind-section-purpose-and-schema-into-the-section-descriptor
title: Bind a section's purpose and schema into its descriptor and digest
status: done
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec, record-the-implemented-artifact-envelope-in-the-contract, prototype-metal-bundle-assembly]
scopes: [implementation/artifact, contracts/artifacts]
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

## Outcome

Both disagreements are closed, and the contract was the correct side of both.

**The section digest now binds the purpose and the content schema.** The pre-image is the domain separator, the purpose tag, the content schema, and then the exact section bytes; the qualifiers are fixed width and precede the variable-length content, so the pre-image is unambiguous without a length prefix between them. `equal_bytes_under_different_purposes_have_different_section_digests` pins the property the change exists for — that a section digest is usable as a *standalone* content address, which is what content-addressing a backend code section does.

**The descriptor carries all four declared fields.** It is now an ordered identifier, a purpose tag, the purpose's required/optional disposition, the purpose's content schema, the exact byte length, and the content digest. One narrowing remains and is deliberate rather than owed: the digest algorithm stays named once in the header, because one envelope is digested under one governed algorithm and a per-descriptor spelling would admit an envelope whose sections disagreed about it. `docs/artifact-abi.md` records that as a decision instead of a gap.

**Why derivable fields are written anyway, and how they stay honest.** Disposition and schema are properties of the purpose, not the instance, so a reader that recognizes a purpose can derive both. The reader that *needs* them carried is exactly the one that does not recognize the purpose — which is the mechanism the contract's optional-section skip rule will require. Writing a derivable field creates the hazard of two producer-supplied fields agreeing with each other and with nothing else, so the decoder compares each against **its own table** rather than against another encoded field: a descriptor that disagrees is asserting a schema or a skip permission rather than reporting one, and is rejected by `UnsupportedSectionSchema` or `SectionDispositionMismatch`.

**The ticket's suggested versioning mechanism was wrong, and taking it would have been worse than doing nothing.** It said to bump the manifest schema *minor*, on the grounds that the existing `UnsupportedManifestSchema` rejection handles the skew. It does not: the reader admits `minor <= implemented`, so a minor bump would have left it accepting a `1.0` manifest whose descriptors it can no longer parse — silently misreading a descriptor table rather than refusing it. A field added inside a fixed-width record is not additive. The manifest schema moved to **2.0**. The envelope format and canonical encoding profile in the header are untouched at `{1, 0}`: the manifest's layout moved, not the framing around it.

Every purpose this build writes is `Required` and an unrecognized purpose is still refused outright, so no skip path exists yet — item 2 of the contract's narrowing list is unchanged and still deferred behind exposing the format outside a lockstep release.

The ticket's scope note asked the taker to claim `implementation/artifact` and `contracts/artifacts`; both are now declared. It also forbade deciding whether a bundle's identity is content-addressed over compilation inputs or payload bytes — that was already decided by `prototype-metal-bundle-assembly` (compilation inputs), and nothing here revisits it.

**Verification.** Five new cases: the standalone-address property, a disposition contradicting its purpose, a content schema contradicting its purpose, an unrecognized disposition tag, and an exhaustive sweep requiring every governed purpose to declare both fields and round-trip both tags. `cargo nextest run --workspace --no-fail-fast` — 612 passed, 0 skipped; `cargo clippy --workspace --all-targets` clean, with `parse_manifest`'s new length extracted into `parse_section_descriptors` rather than allowed; `cargo fmt --all --check` clean. One pre-existing test hard-coded the manifest major and now derives it from `MANIFEST_SCHEMA`.
