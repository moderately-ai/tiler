---
id: decide-whether-the-bundle-envelope-section-digest-is-redundant
title: Decide whether the bundle envelope-section digest is redundant
status: todo
priority: p2
dependencies: []
related: [measure-the-expansion-cache-hot-path-efficiency, decide-whether-the-canonicity-re-encode-is-redundant]
scopes: [implementation/cache]
shared_scopes: [contracts/decisions]
paths: []
tags: [performance, cache, correctness]
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
