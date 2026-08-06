---
schema: "tiler-doc/v1"
id: "tiler.research.artifacts.decoder-allocation-amplification"
kind: "research"
title: "Artifact decoder allocation amplification"
topics: ["artifacts", "codec", "performance", "measurement"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "partial"
evidence_classes: ["bounded-measurement"]
informs: ["tiler.contract.artifact-abi"]
depends_on: ["tiler.research.artifacts.target-neutral-envelope"]
ticket: "measure-artifact-decoder-allocation-amplification"
---

# Artifact decoder allocation amplification

**Status:** measured; one amplification removed, one filed as a decision the artifact layer cannot take alone
**Ticket:** `measure-artifact-decoder-allocation-amplification`

Everything below comes from [`spikes/artifacts/decoder-allocation/`](../../../spikes/artifacts/decoder-allocation/README.md), which counts allocations through a counting `GlobalAlloc` while driving the **public** [`decode_artifact`](../../../crates/tiler-artifact/src/program/codec/decode.rs) over real envelopes. Allocation counts are properties of the program rather than of the host, and the harness asserts that two measured repetitions of every call agree exactly, so there is no variance to report: every figure below is the reading, not an estimate of one.

## The headline, before the evidence

**Measurement.** An envelope's carried sections were copied **twice** by a decode and are now copied **once**. Peak live bytes for a decode of a 64 MiB-object envelope fell from 134,614,747 to 67,391,869 — from 2.00× the envelope to 1.00×.

**Measurement, and it is the exact statement of the boundary.** After the change, peak live during a decode is `retained + 220,531` bytes at 1 MiB, 16 MiB, and 64 MiB of carried object — **the same constant to the byte across a 64-fold change in section size**. Before it, the same difference was 1,383,121, 17,111,761, and 67,443,409: proportional to what the envelope carried.

**Measurement, and it is three orders of magnitude larger than the one that was fixed.** A **226,214-byte** envelope carrying a 4,000-node ABI expression chain makes a decode allocate a peak of **1,569,620,906 bytes** — 6,939× the envelope it read. The growth is quadratic in chain depth, measured at five points spanning a 31-fold range with the exponent pinned at each doubling. A forged envelope that is *rejected* allocates the identical peak on its way to the rejection. This is not fixable inside `tiler-artifact` and is filed as [`replace-the-codec-arena-content-key-with-the-existing-comparator`](../../../tickets/replace-the-codec-arena-content-key-with-the-existing-comparator.md).

**Measurement.** The encoder, not the decoder, is the worst amplifier that remains inside one crate's reach: `VerifiedArtifactProgram::encode` peaks at **4.99×** the envelope for a 64 MiB object, because the projection from artifact data to envelope copies each carried object four times before the encoder writes it a fifth. Filed as [`stop-copying-the-carried-payload-through-the-envelope-projection`](../../../tickets/stop-copying-the-carried-payload-through-the-envelope-projection.md).

## 1. Procedure and measurement validity

**The metric.** Peak live bytes — the high-water mark of bytes the program owns above the live total when the measured call began. Beside it: retained bytes still live when the call returns, total bytes requested (every `alloc` size plus every `realloc`'s new size, so vector doubling shows as churn rather than as footprint), allocator call count, and the sizes of the largest individual blocks. The last is the allocation-site evidence: a block the size of a whole envelope has exactly one possible origin.

**The oracle.** Every malformed input is decoded once *before* it is measured, and the run asserts that decode failed; the reported verdict is that failure's own text. A forgery that quietly decoded would otherwise be measured and reported as if it were a rejection path.

**Peak live is an accounting model rather than resident memory.** `realloc` is forwarded to the system allocator and charged as `new - old`, so a growth the allocator satisfies by moving a block is not charged for holding both copies. Real RSS can exceed these figures transiently. The direction matters: a reduction reported here is a floor on what a consumer sees, never a ceiling.

**Environment.** Apple M4 Max (`Mac16,6`), 14 logical cores, macOS 27.0 (Darwin 27.0.0, build `26A5388g`), `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, release profile. Allocation counts do not move with host load; they do move with the optimizer, which is why every figure is from a release build.

**Input shapes.** One variant of one compiled governed serial-sum program, swept along the envelope's two independent size dimensions. Sections: 0, 1 MiB, 16 MiB, and 64 MiB of carried object, the last being `MAX_SECTION_BYTES`. Manifest: chains of 128, 512, 1,024, 2,048, and 4,000 ABI expression nodes, the last being `MAX_ABI_EXPRESSIONS` less what the compiled program and the chain's literals occupy. Every other table — variants, entries, bindings, providers, payloads — is at its smallest, so nothing below bounds an envelope that is large in those.

## 2. What a decode allocates, and where

A decode reads bytes it does not own and returns a `DecodedArtifact` that outlives them. Some ownership is therefore not amplification but the boundary itself. The five sites, named from the source and confirmed by block size:

| Site | What it allocates | Proportional to |
| --- | --- | --- |
| `read_sections` | one owned `Section` per framed section | section bytes — **retained** |
| `parse_manifest` | the decoded model: keys, shapes, tables, subjects | manifest bytes — **retained** |
| `parse_expressions` | the arena's canonical content-key table | *quadratic* in arena depth |
| `canonical_identity` | one identity buffer, derived once and reused | manifest bytes |
| the canonicity backstop | *was* a second complete encoding | envelope bytes |

The first two are the ownership the returned value requires and were left alone; taking a borrowed view instead would put the input buffer's lifetime into the public `DecodedArtifact`, which is a different boundary and not one a measurement can decide. The fifth was pure amplification. The third is the finding.

## 3. The canonicity backstop, removed as an amplifier

`decode` re-derives the canonical encoding of what it just parsed and requires it to reproduce the bytes on the wire. That check is load-bearing — it is what makes one artifact have one byte identity — and none of it is weakened here. What changed is that the derivation is now compared against the bytes **run by run** instead of accumulated into a second buffer and compared whole.

The two are equivalent by construction: `encode_with_identity` and `matches_canonical_encoding` both drive one `encode_head` and one `push_section_framing`, and the section stream is the same slices in the same order. The equivalence is not left as an argument — `the_canonicity_backstop_refuses_every_run_it_walks` perturbs one byte in each of the four runs the walk visits (the fixed header, the header's derived manifest digest, the manifest body, a section's framing, a section's content) plus both length disagreements, and each of the four comparisons was watched failing at its own offset — 0, 69, 74343, 97077 — before the check was trusted.

**Measurement.** Peak live during `decode_artifact`:

| Object bytes | Envelope | Before | ×env | After | ×env | Retained | After − retained |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 114,083 | 459,256 | 4.03 | 283,005 | 2.48 | 62,474 | 220,531 |
| 1 MiB | 1,162,659 | 2,494,171 | 2.15 | 1,331,581 | 1.15 | 1,111,050 | 220,531 |
| 16 MiB | 16,891,299 | 33,951,451 | 2.01 | 17,060,221 | 1.01 | 16,839,690 | 220,531 |
| 64 MiB | 67,222,947 | 134,614,747 | 2.00 | 67,391,869 | 1.00 | 67,171,338 | 220,531 |

**The exact allocation boundary, stated as the target this meets.** *Peak live during a decode is the decoded artifact's own footprint plus a term that does not depend on how many section bytes the envelope carries.* The last column is that term, and it is identical to the byte across a 64-fold change in carried object. Before the change the same column read 1,383,121 / 17,111,761 / 67,443,409 — one whole extra envelope, every section byte of it, on every decode and therefore on every cache hit.

Total bytes requested by a 64 MiB decode fell with it, 135,091,020 to 67,582,130, which is the same second copy seen as churn rather than as footprint.

## 4. Two ordering checks that copied what they compared

`validate` proved a variant's entries were in canonical stage-key order by materializing every entry's stage subject into a fresh `Vec<u8>`, and a launch precondition's order by cloning every precondition's content key. Canonical order is decided by adjacent pairs, so both now borrow.

**Measurement.** The removed allocations are exactly two per variant and two per precondition-bearing entry, confirmed by the call-count deltas rather than inferred: 263 → 261 for the fixture with one entry and no precondition, 1,843 → 1,839 for the one-precondition fixture. The **bytes** removed are the stage subjects and precondition keys themselves, which this fixture keeps small; the bound they remove is not small, because a stage subject is opaque to this layer with its own 16 MiB budget and the copy was one per entry.

**Inference.** This is a reduction in the bound rather than a large measured saving at any shape this harness builds, and it is recorded as such. It was made because it is the same defect as Section 3 at a different table and costs nothing to hold.

## 5. The arena content-key table, which the artifact layer cannot fix

`parse_expressions` derives a canonical content key per arena node with `tiler_ir::program::abi::expr_key`, which frames each operand's **whole key** inside its node's key. A chain of depth `d` therefore carries a key linear in `d`, and an arena of `d` such nodes carries key bytes quadratic in `d`.

**Measurement.** Peak live during `decode_artifact`, no object carried:

| Chain nodes | Envelope | Peak | × envelope | Peak ÷ previous peak |
| --- | --- | --- | --- | --- |
| 0 | 114,083 | 283,005 | 2.48 | — |
| 128 | 117,798 | 1,770,867 | 15.0 | — |
| 512 | 128,550 | 25,997,811 | 202.2 | 14.7 (×4 nodes) |
| 1,024 | 142,886 | 103,258,099 | 722.7 | 3.97 (×2 nodes) |
| 2,048 | 171,558 | 411,919,347 | 2,401.1 | 3.99 (×2 nodes) |
| 4,000 | 226,214 | 1,569,620,906 | 6,938.7 | 3.81 (×1.95 nodes) |

Each doubling of the chain multiplies the peak by four. The last row is at the governed arena bound, measured rather than extrapolated to.

**Measurement, and it is what makes this a defect rather than a cost.** The `forged/identity` row at every arena size — an envelope whose trailing identity byte is flipped and whose manifest digest is repaired, so it passes framing, integrity, canonical form and the whole of `validate` — allocates **the identical peak**: 1,569,620,906 bytes at 4,000 nodes, on its way to `ArtifactIdentityMismatch`. Roughly 226 KB of attacker-chosen bytes make a consumer that validates untrusted artifacts allocate 1.5 GB before refusing them. Every consumer that decodes bytes it did not produce is exposed, the expansion cache validating a stored bundle among them.

**Proposal, and the fix already exists.** `tiler_ir::program::abi::compare_expr_nodes` is public, is a total content-derived order needing no numbering, is exactly injective, and its own documentation gives this exact reason for existing: "Materializing a key per node embeds that node's whole subtree, which is quadratic on a chain… A comparison walks both subtrees and stops at the first difference, so it never materializes one." The identity encoder already uses it. The codec does not, and the two are therefore two definitions of canonical arena order that only happen to agree.

**Why this note stops here.** Switching the codec to the comparator changes which byte string is *the* canonical encoding of a given artifact, so it forces a `MANIFEST_SCHEMA` major step and rebaselines every pinned envelope byte in the workspace. Artifact identity does not move — `encode_identity` numbers the arena through `canonical_arena_traversal`, which is invariant to arena permutation — but the wire does, and that is a decision with a blast radius outside this ticket's scopes. [`replace-the-codec-arena-content-key-with-the-existing-comparator`](../../../tickets/replace-the-codec-arena-content-key-with-the-existing-comparator.md) owns it.

## 6. Adversarial inputs stay bounded

Seven malformed inputs per shape. The claim is not that each is refused — other tests own that — but that **no malformed input allocates more than a valid decode of the same bytes**, and that a forged length reserves nothing for content that is not there.

**Measurement.** At every shape swept, peak live for a forged input:

| Forgery | Reaches | Peak live |
| --- | --- | --- |
| truncated to half | the header's total-length field | 76 bytes |
| flipped magic | the framing magic | 8 bytes |
| `u64::MAX` total length | the header's total-length field | 76 bytes |
| declared manifest length raised to 63 MiB | the manifest read | 72 bytes |
| declared section count `u32::MAX` | the section budget | 64 bytes |
| flipped object byte | the last section's digest | 1.00× envelope |
| flipped identity byte, digest repaired | the identity comparison | 1.00–1.56× envelope |

The first five are refused before anything is read into memory, which is the property `budget.rs` claims and this is the observation of it: a forged length reports truncation or an exhausted budget in under a hundred bytes. The last two walk the whole section stream and, for the identity forgery, the whole of `validate` — and both land at or below the valid decode's peak at every section size. The arena rows in Section 5 are the exception that matters, and there the bound is met in the sense that a forgery allocates no *more* than a valid decode; what is wrong is that the valid decode allocates 6,939×.

## 7. What this note does not establish

**One artifact shape.** One variant, one entry, one payload, one provider. Nothing here bounds an envelope large in variants, entries, or bindings, and each of those tables has its own governed budget that this sweep never approached.

**Synthetic object bytes.** The carried object travels opaquely and artifact identity excludes every byte of it, so the artifact layer does identical work on `n` synthetic bytes and `n` bytes of `metallib`. This is not evidence about a real Metal compilation.

**No timing.** Wall clock is deliberately absent. This host is shared, and the ticket's question is memory; a time beside these numbers would be a measurement of the machine.

**The arena rows are producer-reachable, and only inferred to be forger-reachable from bytes alone.** Every arena row is a legitimately built artifact, and the `forged/identity` variant of it proves the same cost is paid on a path that ends in rejection. Constructing the chain directly in a forged manifest — which `parse_expressions` would reach before any identity check — was not done here; the follow-up ticket owns the confirmation.
