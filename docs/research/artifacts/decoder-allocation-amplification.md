---
schema: "tiler-doc/v1"
id: "tiler.research.artifacts.decoder-allocation-amplification"
kind: "research"
title: "Artifact decoder allocation amplification"
topics: ["artifacts", "codec", "performance", "measurement"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "partially-adopted"
implementation_status: "partial"
evidence_classes: ["bounded-measurement"]
informs: ["tiler.contract.artifact-abi"]
depends_on: ["tiler.research.artifacts.target-neutral-envelope"]
ticket: "measure-artifact-decoder-allocation-amplification"
---

# Artifact decoder allocation amplification

**Status:** measured; all three amplifications removed — the decoder's, the arena's by a schema step, and the encoder's by the projection change in Section 9
**Ticket:** `measure-artifact-decoder-allocation-amplification`, then `replace-the-codec-arena-content-key-with-the-existing-comparator`, then `stop-copying-the-carried-payload-through-the-envelope-projection`

Everything below comes from [`spikes/artifacts/decoder-allocation/`](../../../spikes/artifacts/decoder-allocation/README.md), which counts allocations through a counting `GlobalAlloc` while driving the **public** [`decode_artifact`](../../../crates/tiler-artifact/src/program/codec/decode.rs) over real envelopes. Allocation counts are properties of the program rather than of the host, and the harness asserts that two measured repetitions of every call agree exactly, so there is no variance to report: every figure below is the reading, not an estimate of one.

## The headline, before the evidence

**Measurement.** An envelope's carried sections were copied **twice** by a decode and are now copied **once**. Peak live bytes for a decode of a 64 MiB-object envelope fell from 134,614,747 to 67,391,869 — from 2.00× the envelope to 1.00×.

**Measurement, and it is the exact statement of the boundary.** After the change, peak live during a decode is `retained + 220,531` bytes at 1 MiB, 16 MiB, and 64 MiB of carried object — **the same constant to the byte across a 64-fold change in section size**. Before it, the same difference was 1,383,121, 17,111,761, and 67,443,409: proportional to what the envelope carried.

**Measurement, and it was three orders of magnitude larger than the one that was fixed.** A **226,214-byte** envelope carrying a 4,000-node ABI expression chain made a decode allocate a peak of **1,569,620,906 bytes** — 6,939× the envelope it read. The growth was quadratic in chain depth, measured at five points spanning a 31-fold range with the exponent pinned at each doubling. A forged envelope that is *rejected* allocated the identical peak on its way to the rejection.

**Measurement, and it is what closed that.** [`replace-the-codec-arena-content-key-with-the-existing-comparator`](../../../tickets/replace-the-codec-arena-content-key-with-the-existing-comparator.md) took the schema step Section 5 said the artifact layer could not take alone. The same 226,214-byte envelope now peaks at **670,658 bytes**, 2.96× the envelope and 2,340× less than before, and the quadratic term is gone: peak live runs between 2.48× and 3.23× the envelope across the whole 31-fold arena range. Section 5 carries the retained rows beside the new ones and Section 8 records the forger-reach question this note left open, now answered.

**Measurement, and it was the worst amplifier remaining inside one crate's reach.** `VerifiedArtifactProgram::encode` peaked at **4.99×** the envelope for a 64 MiB object, because the projection from artifact data to envelope held four copies of each carried object before the encoder wrote it a fifth. [`stop-copying-the-carried-payload-through-the-envelope-projection`](../../../tickets/stop-copying-the-carried-payload-through-the-envelope-projection.md) took it to **2.00×**, which is the floor the signature admits. Section 9 has the census and the byte-identity evidence.

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
| `parse_expressions` | *was* the arena's canonical content-key table | *quadratic* in arena depth |
| `canonical_identity` | one identity buffer, derived once and reused | manifest bytes |
| the canonicity backstop | *was* a second complete encoding | envelope bytes |

The first two are the ownership the returned value requires and were left alone; taking a borrowed view instead would put the input buffer's lifetime into the public `DecodedArtifact`, which is a different boundary and not one a measurement can decide. The fifth was pure amplification. The third was the finding, and Section 5 records what removed it: the arena is now ordered and deduplicated without any table at all, so that row allocates nothing proportional to arena depth.

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

## 5. The arena content-key table, and the schema step that removed it

At manifest schema `13.0`, `parse_expressions` derived a canonical content key per arena node with `tiler_ir::program::abi::expr_key`, which frames each operand's **whole key** inside its node's key. A chain of depth `d` therefore carried a key linear in `d`, and an arena of `d` such nodes carried key bytes quadratic in `d`.

At `14.0` the codec orders and deduplicates the arena through `tiler_ir::program::abi::compare_expr_nodes` instead, and `expression_keys` is gone from the crate. That comparator is public, is a total content-derived order needing no numbering, is exactly injective, and its own documentation gives this exact reason for existing: "Materializing a key per node embeds that node's whole subtree, which is quadratic on a chain… A comparison walks both subtrees and stops at the first difference, so it never materializes one." The identity encoder already used it, so the crate had been carrying **two definitions of canonical arena order** that only happened to agree.

**Measurement.** Peak live during `decode_artifact`, no object carried, both schemas on one host and one toolchain:

| Chain nodes | Envelope | `13.0` peak | × envelope | `14.0` peak | × envelope | Reduction |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 114,083 | 283,005 | 2.48 | 283,005 | 2.48 | — |
| 128 | 117,798 | 1,770,867 | 15.0 | 299,746 | 2.54 | 5.9× |
| 512 | 128,550 | 25,997,811 | 202.2 | 324,036 | 2.52 | 80.2× |
| 1,024 | 142,886 | 103,258,099 | 722.7 | 429,858 | 3.01 | 240.2× |
| 2,048 | 171,558 | 411,919,347 | 2,401.1 | 554,786 | 3.23 | 742.5× |
| 4,000 | 226,214 | 1,569,620,906 | 6,938.7 | 670,658 | 2.96 | 2,340.4× |

At `13.0` each doubling of the chain multiplied the peak by four. At `14.0` the ratio to the envelope stays between 2.48 and 3.23 across the whole 31-fold range, and the arena-dependent term — peak less the decoded artifact's own retained footprint — grows 220,531 → 480,017 rather than 220,531 → 1,569,430,265. The last row is at the governed arena bound, measured rather than extrapolated to.

**Measurement, and it is what made this a defect rather than a cost.** The `forged/identity` row at every arena size — an envelope whose trailing identity byte is flipped and whose manifest digest is repaired, so it passes framing, integrity, canonical form and the whole of `validate` — allocated **the identical peak**: 1,569,620,906 bytes at 4,000 nodes, on its way to `ArtifactIdentityMismatch`. Roughly 226 KB of attacker-chosen bytes made a consumer that validates untrusted artifacts allocate 1.5 GB before refusing them, the expansion cache validating a stored bundle among them. That row now peaks at 556,247 bytes, 2,821.8× less.

**Measurement, and the producer paid the same cost.** The builder derived the same key table at every `push_node`, so `VerifiedArtifactProgram::encode` of the 4,000-node artifact peaked at 1,569,451,274 bytes and now peaks at 768,193, 2,043.0× less. Removing the codec's last reader of that table is what made the builder's copy dead, so the two moved in one change rather than in two.

**Measurement, and it bounds what the schema step cost.** Every one of the 93 measured rows reports the **same envelope byte length** at both schemas, and 78 of the 93 report the same peak to the byte. The 15 that moved are exactly the arena-bearing encode, decode, and `forged/identity` rows. So the two orders agree on every arena this sweep builds: the wire is *permitted* to move at this step and did not move for these fixtures, which is why no pinned identity or golden in the workspace was rebaselined.

**Why the step was major, and what did not move.** Switching to the comparator changes which byte string is *the* canonical encoding of a given artifact — the two are different relations, since `expr_key` compares an operand's length before its content through its eight-byte frame while the comparator compares structure directly — so `MANIFEST_SCHEMA` took a major step to `14.0`. Artifact identity did **not** move: `encode_identity` numbers the arena through `canonical_arena_traversal`, which is invariant to arena permutation, and already ordered both expression-bearing sets with the comparator. Confirmed on the merged tree rather than argued — `tiler-build`'s `the_standard_metal_path_publishes_its_recorded_identities` holds its pinned artifact identity and expansion-cache subject unchanged, and `tiler-artifact`'s `permuting_the_arena_moves_the_envelope_bytes_and_not_its_identity` permutes a fixture's arena and asserts the bytes move while the identity does not.

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

The first five are refused before anything is read into memory, which is the property `budget.rs` claims and this is the observation of it: a forged length reports truncation or an exhausted budget in under a hundred bytes. The last two walk the whole section stream and, for the identity forgery, the whole of `validate` — and both land at or below the valid decode's peak at every section size. The arena rows in Section 5 were the exception that mattered: the bound held in the sense that a forgery allocated no *more* than a valid decode, and what was wrong was that the valid decode allocated 6,939× the envelope. At `14.0` it allocates 2.96×, so the exception is closed rather than tolerated.

## 7. What this note does not establish

**One artifact shape.** One variant, one entry, one payload, one provider. Nothing here bounds an envelope large in variants, entries, or bindings, and each of those tables has its own governed budget that this sweep never approached.

**Synthetic object bytes.** The carried object travels opaquely and artifact identity excludes every byte of it, so the artifact layer does identical work on `n` synthetic bytes and `n` bytes of `metallib`. This is not evidence about a real Metal compilation.

**No timing.** Wall clock is deliberately absent. This host is shared, and the ticket's question is memory; a time beside these numbers would be a measurement of the machine.

**The arena rows measure a producer's artifact, and the harness still builds no independently forged manifest.** Every arena row is a legitimately built artifact, and the `forged/identity` variant of it shows the same cost on a path that ends in rejection. Section 8 answers the forger-reach question the harness cannot, from a checked-in case rather than from a swept row.

## 8. The forger reach, confirmed rather than inferred

**Fact, and it is the severity this note previously left open.** `parse_expressions` runs inside `parse_manifest`, which `decode` calls at its third statement. Everything before it is the fixed framing header and one digest comparison over the manifest bytes — both of which a forger holding only bytes recomputes. Nothing between those bytes and the arena parser depends on the artifact's identity, on a section digest, or on any obligation `validate` proves; `validate` and the identity comparison both run after `parse_manifest` has returned.

**Confirmed with one hand-built manifest.** `tiler-artifact`'s `a_forged_manifest_reaches_the_arena_parser_before_any_identity_check` takes the ordinary fixture's encoded bytes, splices a 512-node chain into the manifest in place of the arena run the fixture wrote, repairs the manifest length, the total length, and the manifest digest, and changes nothing else. The decode is refused — by `ArtifactDiagnostic::UnusedExpression`, raised in `validate`, which is what proves the whole forged chain was parsed, type-checked, proven distinct, and proven canonically ordered first. Watched failing under one perturbation: omitting the digest repair reports `ManifestDigestMismatch`, the shallow refusal the case exists to get past.

**So the `13.0` figure was an attacker-reachable cost rather than a producer-imposed one.** Roughly 226 KB of bytes a consumer never produced could make it allocate 1.5 GB before refusing them, and every consumer that decodes bytes it did not write was exposed — the expansion cache validating a stored bundle among them. That is the severity the `14.0` step was sized against, and the test is retained so the reachability claim stays checked rather than argued.

## 9. The envelope projection, which copied what only the encoder needed to own

Sections 3 to 8 are all about the *reading* side. This one is the publication side, and it was the larger multiple: every artifact `tiler-macros` embeds and every bundle the expansion cache publishes pays it, and it scales with the compiled object, which for a real `metallib` is the whole point of the envelope.

`ArtifactEnvelope::project` reads `&ArtifactProgramData` and returns an envelope whose `Section` values own their bytes, so **one** copy of each distinct carried object is forced and no more. It made five, four of them live at once.

**Fact, from the source at the measured commit.** The census, and each entry is one live copy of the object unless named otherwise:

| Site | What it copied | Now |
| --- | --- | --- |
| `project_payloads` | cloned each `PayloadContent` to reorder the payload table | borrows it; reordering is not a reason to copy a library |
| `project_sections` | cloned `content.code` into an `encoded` staging table | the staging table holds compilation subjects only |
| `project_sections` | cloned it again pushing it into `contents` | pushes a `Cow::Borrowed` |
| `project_sections` | cloned **every** `contents` entry into an owned `BTreeMap` key | `binary_search_by` over the sorted, deduplicated table |
| `project_sections` | cloned it once more per `index[&(tag, code.clone())]` lookup — transient, so requested rather than live | the search key is borrowed |
| `Section::bytes` | — | the one copy, `Cow::into_owned` on the distinct survivors |

**Measurement.** Peak live during `VerifiedArtifactProgram::encode`, which is `project` followed by `encode`:

| Object bytes | Envelope | Before | ×env | After | ×env | Requested, before → after |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 114,083 | 340,479 | 2.98 | 340,479 | 2.98 | 599,665 → 543,644 |
| 1 MiB | 1,162,659 | 5,308,322 | 4.57 | 2,437,631 | 2.10 | 6,891,121 → 2,640,796 |
| 16 MiB | 16,891,299 | 83,951,522 | 4.97 | 33,894,911 | 2.01 | 101,262,961 → 34,098,076 |
| 64 MiB | 67,222,947 | 335,609,762 | 4.99 | 134,558,207 | 2.00 | 403,252,849 → 134,761,372 |

Allocator calls for the 64 MiB encode fell 237 → 213 with it. The totals are what count the copies: peak was the envelope plus four live object-sized blocks and requested was the envelope plus five, against one of each afterwards. The `largest_blocks` column is the attribution — a block the size of the carried object has one possible origin — and it read `67222947 67108864 67108864 67108864`, the envelope and as many object copies as its four-slot array had room for, against `67222947 67108864 111020 56320` now.

**The no-object row does not move, and that is the boundary of this change.** Its 2.98× is the identity derivation and the manifest, not a section; what the change removed there is one owned key per table entry, which shows up as 56,021 fewer bytes requested and 20 fewer allocator calls and not in the peak at all. The same holds for every arena-bearing encode row: identical peak, 56,021 fewer bytes requested, 20 fewer calls.

**Measurement, and it is what makes this a pure allocation change.** The envelope bytes did not move. Thirteen artifact fixtures — the default, guarded, two-variant in both declaration orders, partial-window, BF16 and F32 pointwise, strict-affine, route-requiring, two carried, and two delivering one object from two payloads — encode to digests identical before and after, at 39,812 to 186,642 bytes each. Independently, all 93 spike rows report the same `envelope_bytes` as the previous run and 84 of the 93 are byte-identical in every column; the 9 that moved are exactly the encode rows above.

**The content-addressed section table is unchanged, and is now asserted rather than assumed.** Two payloads that carry equal objects share one section — that is what makes a section's address its content — and no test covered it. `two_payloads_carrying_equal_objects_share_one_section` builds an artifact delivering two payloads with different compilation subjects and one object, and requires two subject sections, one object section, and a clean round trip. Watched failing: dropping the projection's `dedup` makes it report two object sections.

**What this does not reach.** `ArtifactProgramBuilder::build` copies the same objects once more, into the `ArtifactProgramData` the artifact owns, because `build` promises the intact builder back on failure and therefore cannot move out of it before the diagnostics are known. That copy is outside the harness's measured window — the fixture is built before `measure_twice` opens — so its size here is a reading of the source rather than a measurement. Filed as [`stop-copying-the-carried-payload-through-the-builder-assemble`](../../../tickets/stop-copying-the-carried-payload-through-the-builder-assemble.md), which has to add a `build` phase to the harness before it can claim anything.
