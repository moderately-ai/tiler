---
id: decide-whether-the-bundle-envelope-section-digest-is-redundant
title: Decide whether the bundle envelope-section digest is redundant
status: review
priority: p2
dependencies: []
related: [measure-the-expansion-cache-hot-path-efficiency, decide-whether-the-canonicity-re-encode-is-redundant]
scopes: [implementation/cache, research/cache]
shared_scopes: [contracts/decisions, project/tickets]
paths: []
tags: [performance, cache, correctness]
claimed_from: todo
assignee: agent-bundle-digest
lease_expires_at: 1785943617
---
**Measured: the bundle's two section digests are 19.4–24.0% of a validated cache hit — 10,875 ns of 55,833 ns at a 32,136-byte envelope and 16,125 ns of 67,458 ns at a 47,803-byte one, on Apple M4 Max, release, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, reproduced by a second run at the same commit.** Reproduce with the invocation `spikes/cache/hot-path-efficiency/README.md` records; the retained rows are `decompose … bundle-section-digests` in `spikes/cache/hot-path-efficiency/results/hot-path-efficiency-macos-27.0-2026-08-04.tsv`, and the derivation is [the hot-path efficiency note](../docs/research/cache/hot-path-efficiency.md).

## The question, stated so it can be answered rather than assumed

`bundle::decode` digests both framed sections on every hit: the compilation subject, and the artifact envelope. The subject digest has no other authority — nothing else looks at those bytes — so it stays regardless.

The **envelope** section digest is the one worth asking about, because `decode_artifact` then runs over the exact same byte run and, by inspection of `crates/tiler-artifact/src/program/codec/decode.rs`, verifies the framing header, the manifest digest, every artifact section's content digest, the canonical identity, and finally re-encodes the whole envelope and byte-compares it against the input.

**It is not obvious that the bundle digest is redundant, and the cheap reading of the paragraph above is exactly the trap.** Three things have to be established rather than argued:

1. **Coverage.** Does `decode_artifact` actually reject *every* single-byte perturbation of the envelope run? The re-encode comparison suggests yes and does not prove it — a byte inside a region the decoder normalizes, or one it reads into a model and writes back verbatim, needs checking one class at a time. This is the same experiment shape `decide-whether-the-canonicity-re-encode-is-redundant` used, and that one found the "only the backstop covers it" set to be non-empty.
2. **Classification.** The two checks refuse with different typed reasons. `BundleRejection::SectionDigest` is a *cache bundle* rejection; an artifact codec failure arrives as `EntryRejection::Payload`. Quarantine, the report vocabulary, and every diagnostic that distinguishes "this cache entry is damaged" from "this artifact is invalid" are built on which one fired. Removing the earlier check silently reclassifies real corruption.
3. **Contract.** ADR 0050 states that a hit validates every section's bounds and digest, and `expansion.rs`'s module documentation repeats it. Dropping one is a contract amendment, not an optimization, and needs an ADR rather than a commit.

## Why it is worth asking anyway

A fifth to a quarter of every hit is a real quantity, it is paid by every expansion in every process, and it grows per byte — 0.338 ns/B at one size and 0.337 at the other, which is SHA-256 over the whole bundle at 2.96 GB/s. If the coverage experiment comes back clean and the classification can be preserved (for instance by mapping the codec's own refusal onto the bundle-level reason where the payload run is the one that failed), the saving is free of any weakening. If it does not, the answer is recorded and nobody asks again.

## What this ticket must not do

Remove the check on the argument that "`decode_artifact` covers it". That is the shortcut whose cost is the part that made the answer correct.

## Closes when

Each class of single-byte and structural envelope corruption has been tested with the bundle-level envelope digest neutered, and the set only that digest catches is recorded; the typed-reason consequence is decided explicitly; and the digest is retained or retired on that evidence, with an ADR where the contract sentence moves.

## Added scope

`research/cache` (`docs/research/cache/**`, `spikes/cache/**`), read from `ticketsplease.toml`. The experiment this ticket's "Closes when" requires cannot live in `crates/tiler-cache`: neutering the digest and observing what `decode_artifact` then catches needs a *real* artifact envelope, which needs a `SemanticProgram` and therefore `tiler-ir`, which [ADR 0082](../docs/decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md) item 2 decides that crate does not depend on. The only place holding all four crates together is a spike workspace, so the experiment is [`spikes/cache/envelope-digest-coverage/`](../spikes/cache/envelope-digest-coverage/README.md) and the note whose open question it closes is [the hot-path efficiency record](../docs/research/cache/hot-path-efficiency.md). No live ticket held `research/cache` when it was added.

## Outcome

**Retained, on evidence rather than argument. The set of corruptions only the bundle's envelope-section digest catches is non-empty**, and its members are the ones that matter most: an envelope run replaced by a *different valid envelope* is caught by nothing else in either layer, and with the digest removed the cache serves it as a validated hit.

The experiment is [`spikes/cache/envelope-digest-coverage/`](../spikes/cache/envelope-digest-coverage/README.md). Three results are retained at this commit: [the shipped run](../spikes/cache/envelope-digest-coverage/results/envelope-digest-coverage-macos-27.0-2026-08-05.tsv), [its reproduction](../spikes/cache/envelope-digest-coverage/results/envelope-digest-coverage-macos-27.0-2026-08-05-reproduction.tsv) after a forced rebuild, and [the neutered run](../spikes/cache/envelope-digest-coverage/results/envelope-digest-coverage-macos-27.0-2026-08-05-neutered.tsv).

### Method

One real artifact envelope (113,303 bytes, the `[4, 3]` serial sum `spikes/cache/hot-path-efficiency` compiles) is published through the public `ExpansionCache::get_or_publish`. Each corruption is applied to the envelope run *inside the published bundle*, leaving the envelope section's declared digest as the publisher wrote it, and driven twice: through the public `lookup`, and through `decode_artifact` on the same run. The second is the neutered hit path, because `bundle::decode` derives the envelope span from the descriptor's offset and length and never from its digest, and `read_entry` hands that span straight to the pinned validator.

**That reduction was confirmed rather than asserted.** The whole table was re-run against a build with the comparison genuinely removed — `decode_sections`' digest guarded by `purpose != BundleSection::ArtifactEnvelope`, in a throwaway copy of `tiler-cache` outside the repository, with the spike's path dependency repointed for one run — and in the neutered result every class's public verdict equals its `decode_artifact` verdict, for all 35 classes. `crates/tiler-cache` was not modified to take it. The harness observes which build it is on before recording anything: one flipped envelope content byte is refused by the bundle digest in a shipped build and by the artifact decoder in a neutered one, so the refusing boundary names the build, and a run whose observed mode disagrees with `--mode` records nothing.

### Per-class verdicts

Every class names the exact bytes it perturbs, so the list can be refuted by naming a byte it misses. `decode_artifact`'s boundary is the one it reported with the digest neutered.

| Region | Classes | `decode_artifact` alone | Attribution |
|---|---|---|---|
| Framing header — magic, envelope format major/minor, canonical encoding major/minor, digest algorithm tag, total length, manifest length, section count, manifest digest | 10 | `BadMagic`, `UnsupportedEnvelopeFormat`, `UnsupportedCanonicalEncoding`, `UnsupportedDigestAlgorithm`, `TotalLengthMismatch`, `ManifestDigestMismatch`, `SectionCountMismatch` | both |
| Manifest — domain tag, schema version, component schema versions, an interior byte, a section descriptor, the carried identity | 6 | `ManifestDigestMismatch`, every one | both |
| Section stream — framing id, framed length, and content, for each of the three framed sections | 9 | `NonCanonicalSectionId`, `SectionLengthMismatch`, `SectionDigestMismatch` | both |
| Manifest with the header digest re-sealed — carried identity, descriptor id, descriptor length, descriptor content digest | 4 | `ArtifactIdentityMismatch`, `NonCanonicalSectionId`, `SectionLengthMismatch`, `SectionDigestMismatch` | both |
| Structural — truncated by one, extended by one, a trailing byte inside the declared length, two content bytes transposed | 4 | `TotalLengthMismatch`, `TrailingBytes`, `SectionDigestMismatch` | both |
| **Structural — the run replaced by a different valid envelope, at equal length and at a different length** | **2** | **accepted** | **only the bundle digest** |

Underneath the classes sits the part that owes nothing to an enumeration: **every byte position of the run, two perturbations each — 113,303 × 2 = 226,606 real decodes — and `decode_artifact` refused all 226,606.** So no single-byte corruption is in the only-bundle set; 257 of those positions were also driven through the public path, and all 257 were refused by the digest in the shipped build and none of them in the neutered one.

### Why the split falls exactly there

**Fact, derived from the two decoders and confirmed by the table.** `decode_artifact` proves an envelope is *internally* consistent, exhaustively: the manifest digest covers every manifest byte, each descriptor's digest covers its section's bytes, the identity is re-derived, and the re-encode backstop covers what the wire carries and the model does not. Nothing in it relates the envelope to the *entry it was stored as* — it has no way to, since it is handed bytes and nothing else. So the corruptions it is blind to are exactly the ones that produce another well-formed envelope, and the smallest such corruption is not small: it is the whole run.

**Nothing else in the cache closes that gap either, and this was checked rather than assumed.** The key is `CacheKey::derive_bytes` over the *subject* section alone, so a substituted envelope leaves the key derivation, the embedded-key check, and the path check all satisfied. `bind-the-cache-subject-to-the-carried-payload-provenance` states the same asymmetry from the other side — "a `tiler-cache` bundle … does not prove that subject describes the artifact beside it" — and puts the cross-check in `tiler-build` rather than in the cache. The envelope-section digest is therefore the only thing in this crate binding a stored bundle to the envelope its publisher framed, and the neutered run shows the consequence directly: `lookup` returned a hit whose `envelope_bytes()` are a different artifact, with a canonical identity the harness asserted differs from the published one.

### The classification consequence

Moot under retention, and recorded so the question is not re-derived. `BundleRejection::SectionDigest { purpose: ArtifactEnvelope }` keeps firing where it fires today, so no diagnostic, report, or quarantine behaviour moves. For the record, two facts about the vocabulary that a retirement would have had to work around were checked while the experiment ran: quarantine does **not** distinguish the two reasons — `resolve_retaining` sets `replacing_rejected_entry` on any `MissReason::Rejected`, so a `Bundle` and a `Payload` rejection quarantine identically — while `EntryRejection`'s own documentation does, and says the payload arm carries "the artifact codec's own, unchanged" classification. A mapping that reported an artifact-codec refusal as a bundle-level one would have had to violate that sentence.

### Cost, and why it is not the reason to remove it

The 19.4–24.0% [the hot-path record](../docs/research/cache/hot-path-efficiency.md) measured is real and is not disputed here. It is 0.337–0.338 ns/B, which is SHA-256 at 2.96 GB/s over the whole bundle — the same governed digest `decide-whether-the-canonicity-re-encode-is-redundant` found to be running at roughly a quarter of achievable speed because `Sha256::compress` rotates its working state with a slice `rotate_right` that lowers to a `memmove`. Making the governed digest fast is worth more than removing this check and costs no guarantee at all; that opportunity is `decide-whether-the-canonicity-re-encode-is-redundant`'s recorded finding and is not re-filed here.

### Measurement boundary

Exhaustive over **one** corruption shape — one changed byte — at **one** envelope of one shape and length, on one host, one toolchain, one release profile. The class list is an enumeration and is offered to be refuted; multi-byte corruption in general, a structurally different envelope, and a forger who re-seals every digest are covered only by the four re-sealed classes and the two substitution classes that are stated. Nothing here is timed and nothing here is a portable performance claim.

### Graph maintenance

`spikes/cache/envelope-digest-coverage/README.md` needs a line in `spikes/README.md`, and the hot-path note's line in `docs/research/README.md` now names one supporting experiment where it must name two. Both files are `contracts/navigation`, held by `govern-the-three-ungoverned-spike-records` throughout this work — and held for exactly the file that ticket exists to edit, so no file-level disjointness was available to verify. [`catalog-the-cache-envelope-digest-coverage-spike`](catalog-the-cache-envelope-digest-coverage-spike.md) carries the exact text of both, in the same shape `catalog-the-cache-hot-path-efficiency-records` used for the sibling spike, and notes the ordering between the two catalog tickets.

The hot-path record itself is edited here rather than deferred: Section 9's third outcome and the headline paragraph that filed the question now record the answer, and `research/cache` is the scope this ticket added. No contract sentence moved — ADR 0050's "readers validate … section lengths/digests … on every hit" and `expansion.rs`'s restatement of it are both still exactly true — so **no ADR is needed and none was drafted**. Retention is what the accepted record already says; retirement is what would have needed one.
