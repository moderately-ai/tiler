---
schema: "tiler-doc/v1"
id: "tiler.spike.cache.envelope-digest-coverage"
kind: "experiment"
title: "Expansion cache envelope-section digest coverage probe"
topics: ["cache", "artifacts", "correctness"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["exhaustive-finite", "executable-model"]
supports: ["tiler.research.cache.hot-path-efficiency"]
entrypoints: ["spikes/cache/envelope-digest-coverage/harness/src/main.rs", "spikes/cache/envelope-digest-coverage/harness/src/envelope.rs"]
last_verified: "2026-08-05"
ticket: "decide-whether-the-bundle-envelope-section-digest-is-redundant"
---

# Expansion cache envelope-section digest coverage probe

This harness answers one question with evidence rather than with a reading of two decoders: **which corruptions of a cached artifact envelope does the cache bundle's own envelope-section digest catch that [`decode_artifact`](../../../crates/tiler-artifact/src/program/codec/decode.rs) does not?** [The hot-path measurement](../../../docs/research/cache/hot-path-efficiency.md) put that digest at 19.4–24.0% of a validated hit and filed the question rather than acting on it; [`decide-whether-the-bundle-envelope-section-digest-is-redundant`](../../../tickets/decide-whether-the-bundle-envelope-section-digest-is-redundant.md) carries the answer.

```sh
cd spikes/cache/envelope-digest-coverage
cargo build --release
./target/release/cache-envelope-digest-coverage --record macos-27.0-2026-08-05
```

Nothing runs it automatically; no `make` target reaches `spikes/`. Run it from this directory, which is where `--record` resolves `results/` from. `--quick` strides the sweep for development and produces a result nobody should record. The whole run takes about ten seconds, almost all of it the 226,606 real decodes of the sweep.

## What it does

One real artifact envelope is compiled, assembled, and published through the **public** [`ExpansionCache::get_or_publish`](../../../crates/tiler-cache/src/expansion/store.rs). Every corruption is then applied to the envelope run *inside the published bundle* and driven twice:

- **the shipped column** writes the corrupted bundle to the cache's own entry path and calls the public `lookup`, so the verdict is the one a real reader gets from the real code;
- **the neutered column** calls `decode_artifact` on the corrupted envelope run directly, which is exactly what the hit path does next once the bundle's envelope-section digest is removed — `bundle::decode` derives the envelope span from the descriptor's offset and length and never from its digest, and `read_entry` hands that span straight to the pinned validator.

Nothing else about the bundle is touched. In particular the envelope section's declared digest stays the digest of the bytes the publisher framed, which is what makes each row a corruption rather than a forgery; the bundle's own descriptor length and total length do follow the run's length, so a length-changing corruption is decided by whatever inspects the envelope instead of by the bundle's contiguity chain.

Two things are corrupted, and the difference matters. A **class** names the exact bytes it changes, so the class list can be refuted by naming a byte it misses. The **sweep** needs no enumeration at all: every byte position of the run, two perturbations each, counted — 113,303 × 2 = 226,606 decodes in the retained run.

## What it found

Recorded in [the 2026-08-05 result](results/envelope-digest-coverage-macos-27.0-2026-08-05.tsv), reproduced byte for byte in [its reproduction](results/envelope-digest-coverage-macos-27.0-2026-08-05-reproduction.tsv), and taken again against a neutered build in [the neutered result](results/envelope-digest-coverage-macos-27.0-2026-08-05-neutered.tsv).

**Every single-byte corruption is caught by both.** All 35 classes and all 226,606 sweep decodes are refused by `decode_artifact` on its own, each by a named boundary: the framing header by its own field checks, the whole manifest by `ManifestDigestMismatch`, each framed section by its identifier, length, or content digest, and — behind a re-sealed manifest digest, which is what it takes to reach them — the canonical identity and the named canonicity checks.

**The set only the bundle digest catches is non-empty, and every member of it is a whole-run substitution.** Replace the envelope span with a *different valid envelope* and `decode_artifact` accepts it, because it is a valid envelope; the bundle digest is the only thing that refuses. Both the equal-length and the different-length substitution are recorded, and the harness asserts that both substitutes carry a canonical artifact identity that differs from the published one, so what would be returned is a different artifact and not a respelling of the same one.

The neutered run shows the consequence directly: those two rows read `NEITHER`, and the harness's own assertion — that a hit returns the bytes that are in the entry — passes, which is to say **the cache served a validated hit carrying an artifact that was never published under that key**.

## How a pass could have been vacuous, and what prevents it

Each control names the population it covers and is a check that can fail.

**The mode is observed, never assumed.** The neutered column is only evidence about the neutered hit path if the reduction behind it holds. So the harness flips one envelope content byte and reads which boundary refuses it: the bundle digest in a shipped build, the artifact decoder in a neutered one. The refusing boundary therefore *names the build*, and the harness **refuses to record anything** when the observed build disagrees with `--mode`. The neutered result's own control row reads `the bundle envelope-section digest is not live`.

**The neutering was real, and it was not applied to the shipped crate.** `crates/tiler-cache` is unmodified by this spike. The neutered result was taken against a throwaway copy of that crate outside the repository, differing by exactly one hunk — `decode_sections`' digest comparison guarded by `purpose != BundleSection::ArtifactEnvelope` — with the spike's `tiler-cache` path dependency pointed at the copy for the length of one run and then restored. Reproduce it by copying `crates/tiler-cache/src` beside a standalone manifest, applying that guard, and repointing `Cargo.toml`; the observed-mode control tells you whether you succeeded before any row is written.

**The reduction is confirmed row by row rather than argued.** In the neutered result every class's shipped column equals its neutered column, for all 35 classes, and the sweep's sampled public verdicts move from 257/257 refused-by-the-digest to 0/257. That is the reduction "with the digest gone, the next check over these bytes is `decode_artifact`" observed, not assumed.

**Both frame restatements are checked against the bytes.** This harness cannot call `bundle::decode`, the bundle digest, or the envelope's own decoder internals — all crate-private — so it restates their framing constants. Each restatement is then *required to hold*: the derived bundle span must contain the exact published envelope and must end the file; the envelope's magic, declared total length, manifest domain, and manifest digest must all reproduce; the framed section stream must close the run exactly; the manifest's trailing canonical identity must sit where the decoder's own `identity()` says it does, under its own length prefix; and each manifest descriptor must declare the length the stream framed. A framing change in either crate therefore fails this spike loudly instead of aiming a perturbation at the wrong bytes.

**The fixtures are proven to be what the classes need.** Both substitutes decode on their own, both carry an identity that differs from the published one, and the equal-length substitute is asserted equal in length and unequal in bytes. Without those, "the decoder accepted it" would be a claim about malformed bytes rather than about a different artifact.

**The sweep restores what it perturbs**, asserted against the published bytes after the last position, so a leaked perturbation cannot make a later row report a corruption it was not given.

**The run ends on a hit.** After the last corruption the published bundle is restored and `lookup` is required to return the exact published envelope, so a table full of rejections cannot be the product of a cache that had stopped working.

## Measurement boundary

**This is exhaustive over one corruption shape, not over all of them.** The sweep covers every byte position of one envelope under two perturbations. It is not a claim about multi-byte corruption in general, about a different envelope shape, or about a forger who re-seals every digest — the classes cover four re-sealed cases and the substitution cases, and those are enumerated rather than exhaustive.

**One host, one toolchain, one profile.** Apple M4 Max, macOS 27.0 (Darwin 27.0.0), APFS under `$TMPDIR`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` — the `rust-toolchain.toml` pin resolved by directory ancestry with no selector. Release profile. Nothing here is timed and nothing here is a portable performance claim; what is portable is the *reason* each verdict has, which is stated by the named boundary in every row.

**One envelope, one shape, 113,303 bytes.** The published program is the `[4, 3]` serial sum `spikes/cache/hot-path-efficiency` compiles; the substitute is `[2, 5]`, chosen because the governed target profile this harness compiles against admits no feasible plan above four rows — probed over `1..=8` rows by `1..=7` columns, where every row count at or below four compiles at every column count and every row count above four fails `NoFeasiblePlan` at all of them. A larger or structurally different envelope is unmeasured.

**The object bytes are synthetic.** The carried object travels opaquely — artifact identity folds the payload metadata and excludes every object byte — so the artifact layer performs identical work on `n` synthetic bytes and `n` bytes of `metallib`, and synthetic bytes are what let two envelopes reach one exact length. This spike is not evidence about a real Metal compilation and needs no Metal toolchain.

**A retained result is a positive claim that outlives its producer.** Only re-running this spike detects drift from the source beside it; nothing in the gate reaches here.

See [the ticket's outcome](../../../tickets/decide-whether-the-bundle-envelope-section-digest-is-redundant.md) for what was decided on this evidence.
