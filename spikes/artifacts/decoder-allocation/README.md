---
schema: "tiler-doc/v1"
id: "tiler.spike.artifacts.decoder-allocation"
kind: "experiment"
title: "What validating one artifact envelope allocates"
topics: ["artifacts", "codec", "measurement", "performance"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.artifacts.decoder-allocation-amplification"]
entrypoints: ["spikes/artifacts/decoder-allocation/harness/src/main.rs", "spikes/artifacts/decoder-allocation/harness/src/envelope.rs"]
last_verified: "2026-08-06"
ticket: "measure-artifact-decoder-allocation-amplification"
---

# What validating one artifact envelope allocates

This harness counts every byte the **public** [`decode_artifact`](../../../crates/tiler-artifact/src/program/codec/decode.rs) allocates while it validates one real artifact envelope, and does the same for the encoder, the identity derivation, the re-encode, and seven malformed inputs.

```sh
cd spikes/artifacts/decoder-allocation
cargo build --release
./target/release/artifact-decoder-allocation --record macos-27.0-2026-08-06-after
```

Nothing runs it automatically; no `make` target reaches `spikes/`. Run it from this directory, which is where `--record` resolves `results/` from. `cargo run --release -- --record <name>` is the same binary and works identically. Without `--record` it prints the table and writes nothing. One run takes about two seconds and peaks near 1.6 GB, which is the finding rather than an accident of the harness.

## The instrument, and why it is a counting allocator

A `GlobalAlloc` wrapping `System`, not an external profiler. The choice is what makes the numbers usable: they are exact rather than sampled, they are reproducible by one command on any host, and — the property that matters most — they are **deterministic**. An allocation count is a property of the program, not of the machine, so two runs of one call must agree byte for byte. The harness measures every call twice after a warm-up and asserts the two readings are identical, so a lazily initialized static caught inside the window fails the run instead of quietly becoming a variance to explain.

Four quantities per measured call, plus the largest individual blocks it requested:

| Column | What it is |
| --- | --- |
| `peak_bytes` | High-water mark of bytes the program owns, above the live total when the call began |
| `retained_bytes` | Live bytes still held when the call returns — for a decode, the `DecodedArtifact` itself |
| `requested_bytes` | Every `alloc` size plus every `realloc`'s *new* size, so a vector that doubles its way to `n` shows about `2n` |
| `calls` | `alloc`, `alloc_zeroed`, and `realloc` |
| `largest_blocks` | The largest individual requests, descending |

The block sizes are the allocation-site evidence. A block the size of the whole envelope has one possible origin, so naming its size turns a total into an attribution.

## The two size dimensions

An envelope grows in two independent ways and they behave completely differently, so the sweep crosses both.

**Sections** — `object_bytes` is the carried backend object, which travels opaquely in its own framed section. The sweep runs 0, 1 MiB, 16 MiB, and 64 MiB, the last being `MAX_SECTION_BYTES`, so the top row is the largest section this envelope shape may carry rather than a round number.

**Manifest** — `arena_chain` is a chain of ABI expression nodes minted through the builder's public `push_binary`, reachable from one launch precondition. The sweep runs 128, 512, 1,024, and 2,048, and ends at 4,000, which is `MAX_ABI_EXPRESSIONS` less the nodes the compiled program and the chain's own literals occupy. A chain rather than a wide fan, because the quantity it exercises is the **depth** of an arena node's canonical content key: `tiler_ir::program::abi::expr_key` frames each operand's whole key inside its node's key.

## What the retained results say

Two files, taken at the same commit on either side of one change: [before](results/decoder-allocation-macos-27.0-2026-08-06-before.tsv) and [after](results/decoder-allocation-macos-27.0-2026-08-06-after.tsv). The change replaced the decoder's canonicity backstop — which re-encoded the envelope into a second buffer and compared the two — with a derivation compared against the bytes run by run.

| Shape | `decode` peak, before | after |
| --- | --- | --- |
| 64 MiB object | 134,614,747 (2.00× envelope) | 67,391,869 (1.00×) |
| 16 MiB object | 33,951,451 (2.01×) | 17,060,221 (1.01×) |
| 1 MiB object | 2,494,171 (2.15×) | 1,331,581 (1.15×) |
| no object | 459,256 (4.03×) | 283,005 (2.48×) |
| 4,000-node chain | 1,569,929,746 (6,940× ) | 1,569,620,906 (6,939×) |

The section rows are the change. The chain rows are the finding it does not touch, and they are the larger number by three orders of magnitude: a **226,214-byte** envelope makes a decode allocate **1.57 GB**, and a forged envelope that will be rejected makes it allocate exactly the same. See [the research result](../../../docs/research/artifacts/decoder-allocation-amplification.md).

## How a pass could have been vacuous, and what prevents it

**Every refusal is proven to be a refusal.** Each malformed input is decoded once before it is measured and the run asserts the decode *failed*; the printed verdict is that failure's own text. A forgery that silently decoded would otherwise be measured and reported as if it were a rejection path.

**The deepest forgery is proven to reach the deepest path.** `forged/identity` flips the manifest's trailing identity byte and re-derives the manifest digest, so it passes framing, integrity, canonical form, every structural obligation and the whole of `validate`, and is refused only by the identity comparison. Its verdict column says `ArtifactIdentityMismatch`, which is what proves it got there; had the re-digest been wrong it would read `ManifestDigestMismatch` and the row would be measuring a shallow refusal under a deep name.

**The readings are proven identical rather than assumed.** Two measured repetitions per call, asserted equal, after a warm-up.

**The block recorder cannot silently drop evidence.** It writes into a fixed array because it runs inside the allocator and must not allocate; requests past the array are **counted** and printed as `(+n unrecorded)` rather than dropped.

## Measurement boundary

**Peak live is an accounting model, not RSS.** `realloc` forwards to the system allocator and is accounted as `new - old`, so a growth the allocator satisfies by moving a block is not charged for holding both copies. Real resident memory can exceed these figures transiently. The direction is stated because it means a reduction reported here is a floor on what a consumer sees, never a ceiling.

**One process, one thread.** The counters are process-wide and this harness spawns no thread. A harness that did would have to make them thread-local before any reading meant anything.

**One host, one toolchain, one profile.** Apple M4 Max (`Mac16,6`), 14 logical cores, macOS 27.0 (Darwin 27.0.0, build `26A5388g`), `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` — the `rust-toolchain.toml` pin resolved by directory ancestry with no selector. Release profile. Allocation counts do not vary with host load, but they *do* vary with the optimizer, so a debug build would report a different program's allocations than the one a consumer runs.

**The object bytes are synthetic and the artifact is one shape.** The carried object travels opaquely — artifact identity folds the payload *metadata* and excludes every object byte — so the artifact layer performs identical work on `n` synthetic bytes and `n` bytes of `metallib`. This is not evidence about a real Metal compilation. Every row packages one variant of one compiled serial-sum program, so the variant, entry, binding, provider, and payload tables are at their smallest; the sweep varies section bytes and arena size and nothing else.

**The arena rows measure a legitimately built artifact.** They are produced through the ordinary builder, so they say what a *producer* can make a consumer do. That the same cost is reachable from bytes alone is shown by the `forged/identity` row at each arena size, which allocates the identical peak on its way to a rejection — not by an independently forged manifest, which this harness does not construct.
