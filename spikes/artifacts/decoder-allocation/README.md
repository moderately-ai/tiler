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
ticket: "stop-copying-the-carried-payload-through-the-envelope-projection"
---

# What validating one artifact envelope allocates

This harness counts every byte the **public** [`decode_artifact`](../../../crates/tiler-artifact/src/program/codec/decode.rs) allocates while it validates one real artifact envelope, and does the same for the encoder, the identity derivation, the re-encode, and seven malformed inputs.

```sh
cd spikes/artifacts/decoder-allocation
cargo build --release
./target/release/artifact-decoder-allocation --record macos-27.0-2026-08-06-projection
```

Nothing runs it automatically; no `make` target reaches `spikes/`. Run it from this directory, which is where `--record` resolves `results/` from. `cargo run --release -- --record <name>` is the same binary and works identically. Without `--record` it prints the table and writes nothing. One run takes about two seconds and now peaks near 135 MB, which is the 64 MiB encode row and no longer the arena; the two earliest retained runs peak near 1.6 GB, which was the finding rather than an accident of the harness.

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

**Manifest** — `arena_chain` is a chain of ABI expression nodes minted through the builder's public `push_binary`, reachable from one launch precondition. The sweep runs 128, 512, 1,024, and 2,048, and ends at 4,000, which is `MAX_ABI_EXPRESSIONS` less the nodes the compiled program and the chain's own literals occupy. A chain rather than a wide fan, because the quantity it exercises is arena **depth**: at manifest schema `13.0` the codec derived a canonical content key per node with `tiler_ir::program::abi::expr_key`, which frames each operand's whole key inside its node's key.

## What the retained results say

Four files, each taken at one commit and each answering the row its change moved. [before](results/decoder-allocation-macos-27.0-2026-08-06-before.tsv) and [after](results/decoder-allocation-macos-27.0-2026-08-06-after.tsv) sit on either side of the canonicity-backstop change, which replaced a re-encode into a second buffer with a derivation compared against the bytes run by run. [comparator](results/decoder-allocation-macos-27.0-2026-08-06-comparator.tsv) was taken on the manifest `14.0` tree, where the codec orders and deduplicates the arena through `compare_expr_nodes` and derives no key table at all. [projection](results/decoder-allocation-macos-27.0-2026-08-06-projection.tsv) was taken after `ArtifactEnvelope::project` stopped copying each carried object five times on its way to the encoder.

The **decode** rows, which the first and third runs moved:

| Shape | `decode` peak, before | after | comparator (`14.0`) | projection |
| --- | --- | --- | --- | --- |
| 64 MiB object | 134,614,747 (2.00× envelope) | 67,391,869 (1.00×) | 67,391,869 (1.00×) | 67,391,869 (1.00×) |
| 16 MiB object | 33,951,451 (2.01×) | 17,060,221 (1.01×) | 17,060,221 (1.01×) | 17,060,221 (1.01×) |
| 1 MiB object | 2,494,171 (2.15×) | 1,331,581 (1.15×) | 1,331,581 (1.15×) | 1,331,581 (1.15×) |
| no object | 459,256 (4.03×) | 283,005 (2.48×) | 283,005 (2.48×) | 283,005 (2.48×) |
| 4,000-node chain | 1,569,929,746 (6,940×) | 1,569,620,906 (6,939×) | **670,658 (2.96×)** | 670,658 (2.96×) |

The section rows are the first change, and the third and fourth runs reproduce them to the byte because they touched nothing a decode does. The chain rows are the finding the first change did not touch and the third one closes: a **226,214-byte** envelope made a decode allocate **1.57 GB** and now makes it allocate 670,658 bytes, and the quadratic growth in chain depth is gone.

The **encode** rows, which the fourth run moved:

| Shape | `encode` peak, comparator | projection | `requested_bytes`, comparator → projection |
| --- | --- | --- | --- |
| 64 MiB object | 335,609,762 (4.99× envelope) | **134,558,207 (2.00×)** | 403,252,849 → 134,761,372 |
| 16 MiB object | 83,951,522 (4.97×) | 33,894,911 (2.01×) | 101,262,961 → 34,098,076 |
| 1 MiB object | 5,308,322 (4.57×) | 2,437,631 (2.10×) | 6,891,121 → 2,640,796 |
| no object | 340,479 (2.98×) | 340,479 (2.98×) | 599,665 → 543,644 |

`largest_blocks` records only the four largest requests, and at 64 MiB it read `67222947 67108864 67108864 67108864` — the envelope and as many object-sized blocks as the array had room for. It now reads `67222947 67108864 111020 56320`, so the second object-sized block is gone rather than merely pushed off the end. The peak and requested totals say how many there were: peak was the envelope plus four live copies and requested was the envelope plus five, against one of each now.

Only the encode rows moved: the fourth run is byte-identical to the third in all 84 other rows, and every row of both reports the **same envelope byte length**, which is the evidence that removing the copies did not move the wire. The object rows fall to the floor the projection has — `project` takes `&ArtifactProgramData` and a `Section` owns its bytes, so one copy plus the encoder's output buffer is 2×. The no-object row does not move because nothing it allocates is a section: its peak is the identity derivation, and what the change removed there shows up only as 56,021 fewer bytes requested and 20 fewer allocator calls. All 93 rows report the **same envelope byte length** across the second, third, and fourth runs, so neither the schema step's permission to move the wire nor the projection change used it on these fixtures. See [the research result](../../../docs/research/artifacts/decoder-allocation-amplification.md).

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

**The arena rows measure a legitimately built artifact.** They are produced through the ordinary builder, so they say what a *producer* can make a consumer do. That the same cost is reachable from bytes alone is shown here only by the `forged/identity` row at each arena size, which allocates the identical peak on its way to a rejection. This harness constructs no independently forged manifest; `tiler-artifact`'s `a_forged_manifest_reaches_the_arena_parser_before_any_identity_check` is what answers that question, by splicing a chain into a real manifest and repairing its digest.
