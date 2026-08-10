---
id: reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section
title: Reconcile the artifact ABI's hashing-site count with its own governed-digest section
status: done
priority: p3
dependencies: []
related: [cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check, decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim, reconcile-the-artifact-abis-four-hashing-sites-with-the-fifth-it-names]
scopes: [contracts/artifacts, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
`cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check` reconciled eight count sites against the true population of eighteen. This ticket originally found two further ordinal claims in a different vocabulary — *hashing sites* rather than *governed domains*. Re-reading the contract at this ticket's base distinguishes the already-repaired ADR 0074 block from a separate live schema-15 Fact, and the encoder comment also remains live.

**Ticket repair — 2026-08-08 at base `689c5ccc0e4b8fa5087c5a91feeafd24360c5012`.** The original first Fact is false only for the ADR 0074 convention-2 block it named: commit `8e024560` already corrected that block, leaving its retired wording quoted only in the dated correction. Its separate schema-15 Fact remains live and says the identity digest is under a "fourth governed digest domain"; this is the same stale ordinal and requires a narrow contract edit to "its own governed digest domain". The live "The governed digest" authority says the envelope has seven governed domains, of which five are digest arguments. The encoder Fact is verified in substance but its "fourth" ordinal is stale. `crates/tiler-artifact/src/domains.rs` enumerates eighteen total domains: seven envelope, four proof-sidecar, and seven artifact-program identity/key domains. Within the envelope, `identity_digest` is the fourth of five digest arguments (and the fifth envelope domain overall, after the manifest framing tag), but both repairs remove the ordinal rather than replacing it. The ticket's outcome is therefore one contract-prose edit, one encoder-comment edit, and this repair; ADR 0103 remains a non-goal.

**Correction — 2026-08-10.** The 2026-08-08 repair paragraph above mis-ranked `identity_digest` as the fifth digest argument. Under "The governed digest", the five digest arguments in stated order are manifest, section, envelope, identity, then payload_identity; identity is the fourth digest argument. Status and delivered outcome are unchanged.

**Retired ticket Fact — false as a live claim at this base.**

1. **`docs/artifact-abi.md`, in the ADR 0074 convention-2 block.** The paragraph anchored `"Hashing occurs at exactly four sites, all of them envelope framing"` names `manifest_digest`, `section_digest`, `envelope_digest`, and `identity_digest`. The same document's governed-digest section says the opposite, far earlier, at the anchor `"A fifth is a digest argument reached through a carried payload"`, and specifies `payload_identity = H("tiler.artifact-envelope.payload-identity.v1\0" || …)`. The crate agrees with the second: `crates/tiler-artifact/src/program/codec/payload.rs` hashes under it at `.digest(PAYLOAD_IDENTITY_DOMAIN, metadata)`. So "exactly four" is false, and the follow-on Fact's `"it is envelope framing like the other three rather than a layered digest"` inherits the same undercount. Commit `8e024560` already corrected this block; its retired wording is now quoted only in the dated correction.

   **A separate live contract Fact, found on re-read, needs the same repair.** The manifest-schema-15 paragraph anchored `"The manifest ended with the artifact's canonical identity in full"` says the trailing digest is under a "fourth governed digest domain". The schema step, fixed width, and measurements do not depend on that ordinal; the statement should say it is under its own governed digest domain, preserving the separate-domain meaning without claiming its position in the envelope's growing digest set.

   The undercount predates the identity digest: `git show 09d1666a~1:docs/artifact-abi.md | grep -n 'Hashing occurs at exactly three sites'` shows the block said "three" before ADR 0103 and stepped to "four" with it, never counting the payload identity digest at either step.

2. **`crates/tiler-artifact/src/program/codec/encode.rs`, on `IDENTITY_DIGEST_DOMAIN`.** The doc comment opens `"It is a fourth domain rather than a reuse"`. This live wording is stale; replace it with `"It is a separate domain rather than a reuse"`. The separate-domain reasoning is correct because `MANIFEST_DIGEST_DOMAIN` covers the bytes this digest is written into.

**What is *not* wrong, so a worker does not over-correct.** The live encoder comment's reasoning does not depend on its number: a separate domain is owed because `MANIFEST_DIGEST_DOMAIN` covers the bytes this digest is written into. Domain bytes, schema versions, identity pins, code behavior, and the contract's schema and measurement reasoning remain unchanged.

**A neighbouring hazard, already checked so nobody re-checks it.** ADR 0103's rejected alternative "Frame the digest behind a length prefix" claimed the codec was "inconsistent with the three existing digest sites, none of which frames"; that clause was withdrawn on 2026-08-08 because the payload descriptor's digest *is* length-prefixed — `push_slice(bytes, payload.digest.as_bytes())` in `encode.rs`, and `tiler_ir::identity::push_slice` reserves `8 + value.len()` and writes an eight-byte prefix. **`docs/artifact-abi.md` does not repeat that generalization**: `grep -n 'unframed\|unprefixed\|length prefix' docs/artifact-abi.md` returns no claim that governed digests are uniformly unframed. Its relevant framing statements are scoped: the canonical-manifest and schema-15 Facts describe the identity digest as unframed, and the governed-digest section names the unframed header and section-descriptor digests without generalizing to every digest. This ticket changes only the schema-15 ordinal, not those framing claims or ADR 0103 decision 1's matching statement.

## Why this is a separate ticket

`decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim` already maps ADR 0103's related ordinal question in this ticket's frontmatter and owns the decision-record scope. This ticket holds the remaining `implementation/artifact` edit and the separate `contracts/artifacts` contract-prose edit. Shared `project/tickets` covers this corrected ticket record.

## Closes when

The live encoder comment and schema-15 contract Fact state no ordinal into a set that can grow, and each retains its separate-domain reasoning. The already-corrected ADR 0074 block remains unmodified. `docs/artifact-abi.md` under "The governed digest" and `crates/tiler-artifact/src/domains.rs` remain the population authorities.

## Outcome audit — 2026-08-09

Delivered by `492685365feae2e22eda38a505280516d05ad665`. The live schema-15 Fact now says the trailing identity digest is under `its own governed digest domain`, and the encoder says `It is a separate domain rather than a reuse of MANIFEST_DIGEST_DOMAIN`. Both retain the reason that the manifest digest covers the bytes containing the identity digest; neither carries a growing-set ordinal. The already-corrected ADR 0074 block stayed untouched, and no domain byte, schema version, encoded byte, identity pin, or runtime behaviour moved.
