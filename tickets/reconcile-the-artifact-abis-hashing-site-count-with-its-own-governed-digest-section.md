---
id: reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section
title: Reconcile the artifact ABI's hashing-site count with its own governed-digest section
status: in-progress
priority: p3
dependencies: []
related: [cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check, decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim]
scopes: [contracts/artifacts, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
claimed_from: todo
assignee: w-artifact-terra
lease_expires_at: 1786223950
---
`cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check` reconciled eight count sites against the true population of eighteen. Two more state the pre-repair inventory in a different vocabulary — *hashing sites* rather than *governed domains* — so a sweep looking for domain counts does not reach them, and they now contradict the document that carries one of them.

**Fact — verified 2026-08-08 at base `97282def` by reading both files in full.**

1. **`docs/artifact-abi.md`, in the ADR 0074 convention-2 block.** The paragraph anchored `"Hashing occurs at exactly four sites, all of them envelope framing"` names `manifest_digest`, `section_digest`, `envelope_digest`, and `identity_digest`. The same document's governed-digest section says the opposite, far earlier, at the anchor `"A fifth is a digest argument reached through a carried payload"`, and specifies `payload_identity = H("tiler.artifact-envelope.payload-identity.v1\0" || …)`. The crate agrees with the second: `crates/tiler-artifact/src/program/codec/payload.rs` hashes under it at `.digest(PAYLOAD_IDENTITY_DOMAIN, metadata)`. So "exactly four" is false, and the follow-on Fact's `"it is envelope framing like the other three rather than a layered digest"` inherits the same undercount.

   The undercount predates the identity digest: `git show 09d1666a~1:docs/artifact-abi.md | grep -n 'Hashing occurs at exactly three sites'` shows the block said "three" before ADR 0103 and stepped to "four" with it, never counting the payload identity digest at either step.

2. **`crates/tiler-artifact/src/program/codec/encode.rs`, on `IDENTITY_DIGEST_DOMAIN`.** The doc comment opens `"It is a fourth domain rather than a reuse of [\`MANIFEST_DIGEST_DOMAIN\`]"`. It is the envelope's fifth. This is the same ordinal shape the originating ticket repaired twice in the contract, and its worker correction states the repair pattern: name the domain instead of its position, "because an ordinal into a set that grows is a count that goes stale without ever looking like one".

**What is *not* wrong, so a worker does not over-correct.** Neither site's reasoning depends on its number. Site 1's point is that every layered identity is canonical bytes and that the hashing that exists is envelope framing rather than a layered digest; the payload identity digest is envelope framing too, so adding it strengthens the claim rather than weakening it. Site 2's point is that a separate domain is owed because `MANIFEST_DIGEST_DOMAIN` covers the bytes this digest is written into. Both survive verbatim once the count is fixed.

**A neighbouring hazard, already checked so nobody re-checks it.** ADR 0103's rejected alternative "Frame the digest behind a length prefix" claimed the codec was "inconsistent with the three existing digest sites, none of which frames"; that clause was withdrawn on 2026-08-08 because the payload descriptor's digest *is* length-prefixed — `push_slice(bytes, payload.digest.as_bytes())` in `encode.rs`, and `tiler_ir::identity::push_slice` reserves `8 + value.len()` and writes an eight-byte prefix. **`docs/artifact-abi.md` does not repeat that generalization**: `grep -n 'unframed\|unprefixed\|length prefix' docs/artifact-abi.md` returns no claim that governed digests are uniformly unframed, and its one statement on the subject is correctly scoped — the identity digest is "written unframed exactly as the header's manifest digest and each section descriptor's content digest are". So this ticket is the count alone, and neither that sentence nor ADR 0103 decision 1's matching one should be touched.

## Why this is a separate ticket

Scope, twice over. `decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim` found both sites while auditing ADR 0103's counts and holds `contracts/decisions` only; `docs/artifact-abi.md` is `contracts/artifacts` and `crates/tiler-artifact/**` is `implementation/artifact`. Both were characterized there and neither was edited.

## Closes when

Both sites state the true inventory, neither states an ordinal into a set that can grow, and the reasoning at each is unchanged. `docs/artifact-abi.md` under "The governed digest" is the authority to reconcile against, and `crates/tiler-artifact/src/domains.rs` is the authority for the population.
