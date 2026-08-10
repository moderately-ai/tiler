---
schema: "tiler-doc/v1"
id: "tiler.research.verification.kani-push-slice-framing"
kind: "research"
title: "Kani bounded verification of push_slice framing"
topics: ["verification", "kani", "identity", "injectivity", "length-framing"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement", "primary-source-synthesis"]
informs: ["tiler.contract.correctness-and-testing"]
ticket: "spike-kani-push-slice-framing-over-a-symbolic-byte-run"
---

# Kani bounded verification of `push_slice` framing

**Status:** complete bounded experiment over a guarded copy

**Reviewed:** 2026-08-10

## Traceability

- **Work record:** [spike-kani-push-slice-framing-over-a-symbolic-byte-run](../../../tickets/spike-kani-push-slice-framing-over-a-symbolic-byte-run.md).
- **Predecessor:** [Kani bounded verification of inexhaustible identity encoders](kani-bounded-encoder-verification.md) isolated symbolic `String` construction as the cost driver and proposed this decomposition.
- **Reproduction:** [Kani bounded verification of `push_slice` framing](../../../spikes/verification/kani-push-slice-framing/README.md).
- **Host:** Apple M4 Max, macOS 27.0 (26A5388g), arm64. Timings are bounded to this host.

## Result

**Measurement.** Kani 0.67.0 and CBMC 6.8.0 prove two properties over every
ordered pair of byte runs of length at most four:

1. equal `push_slice` encodings imply equal slice lengths and equal active bytes;
2. neither complete encoding is a strict prefix of the other.

Across repeated runs on the active coordination host, injectivity took 3.51–8.58 s wall and strict-prefix freedom took 3.36–10.49 s wall. Both use unwind 13 for a maximum twelve-byte encoding, and both report `memcmp.unwind.0: SUCCESS`. The explicit input boundary contains 4,311,810,305 semantic byte runs; length five and above are outside the model check. The runtime spread is host-load noise recorded as a boundary, not a performance conclusion.

**Fact.** The framing construction is not bounded to four bytes. The live
primitive writes an eight-byte big-endian length and then exactly that many
payload bytes. Equal prefixes therefore carry equal lengths, and no shorter
complete encoding can be a prefix of a longer one, for every slice length that
the checked `usize`-to-`u64` conversion represents. The bounded claim is the
Kani result, not this source-level argument.

**Fact.** The subject is a guarded copy rather than the live crate. Re-running
`cargo kani -p tiler-ir --only-codegen` at the dispatched base still produced
the same nine errors: unknown `min_adt_const_params`, four const-generic array
sites, and four `atomic_try_update` sites. No host or toolchain change was made.

## Why this is independent of the predecessor's stale guard

The new spike copies only `push_len` and `push_slice`. Its guard extracts exactly
those two functions from `crates/tiler-ir/src/identity.rs` and asserts a
population of two. It succeeds on the current tree.

The predecessor guard still fails on two of its 28 items because
`ResourceRequirements` and `push_resources` gained `IndexArithmetic`. That is a
real, separately ticketed drift in a resource-encoder proof. It neither enters
this guard's population nor weakens the successful comparison of the two
framing primitives.

## Interpretation against the nine categories

The native sweep's “nine string encoders” are nine named encoder categories,
not nine concrete function instances or nine fields. `push_numerical` alone has
three copies, and `ExecutionEnvironmentIdentity::encode` writes five strings.
Re-reading all nine categories shows their variable byte runs reach the shared
`crate::identity::push_slice` primitive.

**Inference.** This bounded result supplies the shared framing half of the
decomposition without paying for symbolic UTF-8 validation. It does not by
itself prove each complete category injective: the predecessor separately
proved only `push_numerical`'s non-key tail with a fixed key. Other categories'
tails retain their prior evidence and limits.

## Deliberate failures

Deleting the copied payload append made both the independent guard and the
injectivity harness fail; Kani named `equal framing must carry equal active
bytes`. Restoring it and deleting the copied length write made the guard and
prefix-free harness fail; Kani named `one framed byte run is a strict prefix of
another` twice. These perturb independent subject properties: retaining a
payload without a length stays injective as a standalone run but loses strict
prefix freedom, while retaining only a length loses payload injectivity.

## Evidence limit and decision boundary

**Fact.** This is an executable model plus a bounded measurement over a guarded
copy. The guard is token-based, does not tie callers, and is not in any
repository gate. The result is neither a live-crate proof nor an unbounded Kani
proof.

**Proposal, not a decision.** Retain the result with its explicit length and
provenance boundaries. Whether it changes any claims-ledger evidence class
remains Tom's decision; this experiment does not classify itself as
`SoundProof` or create a new evidence class.
