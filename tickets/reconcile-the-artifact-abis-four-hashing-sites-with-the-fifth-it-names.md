---
id: reconcile-the-artifact-abis-four-hashing-sites-with-the-fifth-it-names
title: Reconcile the artifact ABIs four hashing sites with the fifth it names
status: done
priority: p2
dependencies: []
related: [date-the-two-v4-step-paragraphs-trailing-the-v5-block, reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section, decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, documentation]
---

`docs/artifact-abi.md` contradicts itself about how many sites hash. One passage says four and enumerates them; another names a fifth.

## Facts, coordinator-verified at the merge that found it

**Fact.** The document contains `Hashing occurs at exactly four sites, all of them envelope framing` and, separately, `A fifth is a digest argument reached through a carried payload` — the latter giving `payload_identity = H(…)` under an envelope domain. Both strings resolve, once each.

**Fact.** This is **substantive, not positional.** A sibling repaired two paragraphs in this file whose *referents* moved when a block was inserted above them; this is a different defect — the two counts disagree on their face, in the same document, about the same subject.

## Why it matters

"Exactly four, all of them envelope framing" is the kind of closed enumeration a reader builds an argument on — that every hash in the crate is accounted for and shares a shape. If a fifth exists and reaches through a carried payload, both the count and the *characterization* are wrong, and any downstream reasoning that leaned on "all of them envelope framing" needs re-examining.

## What closes this

The two passages reconciled — establish from source which is right before choosing, and say which construction you read. Do not assume the larger number wins; the fifth may be a different kind of site that the four-count deliberately excludes, in which case the fix is to say so rather than to renumber.

**Prefer naming the construction over restating a count.** This file has had figures replaced by references to their owners repeatedly this week, on the reasoning that a number in prose rots on a schedule nobody watches. If an enumeration exists in code that owns this, name it.

**Establish the treatment from history** with `git log -S` and `git show <commit>:<file>`: true when written → dated beside; never true → substituted with the retired wording quoted. Repository **practice**, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim stays greppable; say inline that a later hit lands inside your note.

**Two known defects in this file are not yours and must not be folded in:** the premise that every `tiler-ir` domain opens `tiler.ir.` (46 of 60 do not) and the "first differing byte after `tiler.`" variant beside it, both in the same sentence, already reported. Report if you meet them.

**Preserve `git log -S` anchors.** Several tickets locate text in this file by distinctive substring; a sibling deliberately made its edits **prefix-only** so every protected substring stayed byte-identical, then verified each anchor still resolved to its original commit and not to the repair. Meet that standard.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`** — anchors fail as absence four ways: a line break inside them, an emphasis or backtick marker the source lacks, unescaped brackets read as a character class, and a quoted sentence that never appeared contiguously in source.

Check the neighbouring claims and **name the count**; six sweeps of this file this week each found more than they were sent for.

## Per-Fact audit, re-verified 2026-08-08 at base `80837d0d`

**Fact 1 — verified.** `grep -cF 'Hashing occurs at exactly four sites, all of them envelope framing' docs/artifact-abi.md` and `grep -cF 'A fifth is a digest argument reached through a carried payload' docs/artifact-abi.md` each return 1. The first sat in the ADR 0074 convention-2 block; the second under "The governed digest", giving `payload_identity = H("tiler.artifact-envelope.payload-identity.v1\0" || …)`.

**Fact 2 — verified.** Substantive rather than positional: the two counts are about the same subject in the same document and disagree on their face.

## Which passage was right, and the source read to decide

**Fact — "The governed digest" is right and the convention-2 block was wrong; the larger number wins here, and the *characterization* survives.** The authority is `crates/tiler-artifact/src/domains.rs`, read in full. `GovernedDomain` enumerates eighteen variants sized by `core::mem::variant_count`; `DomainContainer::ENVELOPE` is `7`, `PROOF_SIDECAR` is `4`, `PROGRAM_IDENTITY` is `7`, and a `const` block refuses any split that does not total `variant_count`. The envelope's seven are five digest arguments — `EnvelopeManifestDigest`, `EnvelopeSectionDigest`, `EnvelopeEnvelopeDigest`, `EnvelopeIdentityDigest`, `EnvelopePayloadIdentity` — and two framing tags, `EnvelopeManifest` and `EnvelopePayloadMetadata`. The fifth digest site is `payload_identity` in `crates/tiler-artifact/src/program/codec/payload.rs`, which hashes at `.digest(PAYLOAD_IDENTITY_DOMAIN, metadata)`.

The fifth is **not** a different kind of site the four-count excluded. Its digest is the payload descriptor's content address over canonical metadata bytes, re-proven on decode as `PayloadIdentityMismatch`, and it is the identity of no *layer* — the five layered identities the block names are untouched. So it is envelope framing exactly as a section digest is, and only the number was wrong.

## Treatment, established from history

Never true → substituted, with the retired wording quoted verbatim so it stays greppable; a later hit for either retired string lands inside the correction note rather than in a live claim.

- `git log --oneline -S 'Hashing occurs at exactly' -- docs/artifact-abi.md` → `568645b5` alone (2026-07-24 18:40), which wrote "three sites". `git grep -l 'PAYLOAD_IDENTITY_DOMAIN' 568645b5 -- crates/` exits 1 with no output, so three was exact.
- `03a86ac3` (2026-07-24 18:57, not an ancestor of `568645b5`) added the payload identity digest; the count did not move.
- `git log --oneline -S 'Hashing occurs at exactly four sites' -- docs/artifact-abi.md` → `09d1666a` alone (2026-08-06), which stepped three → four for `identity_digest` and added "the other three" beside it. `git grep -l 'PAYLOAD_IDENTITY_DOMAIN' 09d1666a -- crates/` returns `crates/tiler-artifact/src/program/codec/payload.rs`, so **neither** four nor "the other three" was ever true.
- `96dfe333` (2026-08-08) corrected "The governed digest" to seven domains and five digest arguments and did not reach the convention-2 block.

## Outcome

Three passages in `docs/artifact-abi.md` repaired; no code changed.

1. The convention-2 block no longer states a hashing-site number. It names the construction instead: `tiler_artifact::domains::GovernedDomain` owns the population, `DomainContainer::ENVELOPE` the envelope's share, and `each_container_admits_the_number_of_domains_the_contract_records` fails naming this document when the two disagree.
2. `identity_digest` is "envelope framing like the envelope's other digest arguments" rather than "like the other three" — an ordinal into a set that grew.
3. **Neighbouring defect, found and repaired.** The proof sidecar's "Facade status" Fact read "The framing magic, the seven domain separators". Seven is the envelope's number. `96dfe333` changed it from "four", listing it as one of the envelope count sites it swept (see `cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check`, "once in the codec-promotion Fact and once in the wire-form Fact"), but the wire-form Fact is the *sidecar's* and its four was already right. Restored to four, with the retired wording quoted. Independent source agreement: `crates/tiler-artifact/src/proof/mod.rs`'s module header reads "The framing magic, the four versioned domain separators", and `DomainContainer::PROOF_SIDECAR` is `4`.

**The neighbouring census: six prose sites in this document state a governed-domain count** — the codec-promotion Fact ("all seven of the envelope's domain separators"), the governed-digest Fact ("**seven** domain separators"), the union-obligation Fact ("*eighteen*", "the envelope's seven", "the proof sidecar's four", "the artifact program's seven"), the sidecar facade Fact, the sidecar's governed-domain section heading, and the sidecar section's closing sentence ("These four, the envelope's seven, and the artifact program's seven … are the eighteen"). Five agreed with `DomainContainer`; the sidecar facade Fact was the one that did not, and is item 3 above. The three deliberately normative counts named in the brief are left exactly as written.

## Left for the coordinator — out of scope here

- **`docs/decisions/0103-…` (`contracts/decisions`).** Its consequence reads "the same category as the other three hashing sites, all of which the decision names and admits", and its later note rules that this is "a claim about what the 2026-07-27 decision block enumerates rather than about the tree". The block no longer enumerates, so the phrase now needs the same ordinal-to-name repair item 2 applied here. That note already anticipated this and assigned it to `reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section`.
- **`crates/tiler-artifact/src/program/codec/encode.rs` (`implementation/artifact`).** `IDENTITY_DIGEST_DOMAIN`'s doc comment opens "It is a fourth domain rather than a reuse of [`MANIFEST_DIGEST_DOMAIN`]". It is the envelope's fifth digest domain — the same ordinal defect, unrepaired.
- **`reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section` (`todo`, p3).** Its site 1 is discharged by this ticket; only its site 2 and the ADR follow-up remain. Its Fact carries a **false anchor**: it quotes `"it is envelope framing like the other three rather than a layered digest"`, and `grep -cF` for that returns 0 because the source reads `this is envelope framing…`. Left unedited because the ticket is unclaimed.
- **Not folded in, as instructed, and met.** The union-obligation Fact still carries "every domain the shared IR admits opens `tiler.ir.`" and the "first byte after the shared `tiler.`" variant in the same sentence. `crates/tiler-artifact/src/domains.rs` has already retired that exact premise in its own doc comment — "That claim is retired… It was never true at any commit" — and replaced it with the NUL-terminator argument, so the contract now lags its own source on this point.

## Completion correction — 2026-08-09

The three items above are no longer open coordinator work. The contract and
encoder ordinal repairs landed under
[`reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section`](reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section.md),
which now records its exact delivery hash. ADR 0103's never-true population
counts were repaired by
[`decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim`](decide-whether-adr-0103-s-eight-domain-count-is-a-dated-record-or-a-stale-claim.md).
Its remaining "other three hashing sites" phrase is explicitly bounded in the
same record as a claim about the three sites the 2026-07-27 decision block
enumerated, not a live census of the codec, so it is retained historical prose
rather than an unowned repair.

The separate cross-crate no-prefix explanation is now owned by
[`repair-the-artifact-abis-stale-cross-crate-no-prefix-argument`](repair-the-artifact-abis-stale-cross-crate-no-prefix-argument.md).
Nothing remains unfiled from this completed hashing-site reconciliation.
