---
id: stop-recomputing-pure-derivations-in-the-codec
title: Stop recomputing pure derivations in the artifact codec
status: done
priority: p1
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [performance, artifact]
---
Duplicate work inside `decode`, each item a pure function of the same value computed more than once. No semantic change.

## Facts

**`canonical_identity()` twice per decode.** `decode.rs:102` derives it, then the canonicity re-encode at `decode.rs:113` reaches `encode.rs:194` and derives it again — same pure function, same value. The re-encode is **50% of decode time** (274 µs of 548 µs, measured), and this is a share of it available without touching the canonicity guarantee at all.

**`expression_keys` four times per decode** — `decode.rs:590`, `validate.rs:80`, and twice via `encode_identity` (`program/model.rs:1510`).

**`decode_metadata` `2 + E` times** where `E` is entries realized by the payload — `validate.rs:252`, `validate.rs:342` (per entry, deliberately, with a sound comment), and `view.rs:152`. Each call re-allocates a full `PayloadMetadata` including `source.to_vec()` at `payload.rs:403`, bounded at 16 MB.

**Every hashed byte is hashed twice.** Manifest SHA at `decode.rs:83` and again at `encode.rs:107`; per-section digest at `decode.rs:211` and again at `encode.rs:357`.

**`DecodedExpr::value_type()`** (`view.rs:849`) rebuilds the type vector from node 0 on every call, though `validate.rs:103-117` already computed the full table and discarded it.

## Scope

Derive each once and reuse. Where a re-derivation exists to *verify* rather than to produce, keep the verification and remove only the repetition.

This ticket does **not** decide whether the canonicity re-encode should exist — `decide-whether-the-canonicity-re-encode-is-redundant` owns that, and this work is worth doing whichever way it goes.

## Closes when

One decode derives the canonical identity once and the expression keys once, pinned by work-count guards; decode time is measured before and after; every existing codec test still passes unchanged; `make full` passes.

## Outcome

Partially done, and the part that landed is the largest of the four. **Decode fell from 662 µs to 501 µs — 24% — on a 26,126-byte envelope, paid back on every artifact load and every cache hit.**

**What landed: the identity is derived once per decode.** `decode` derived the canonical identity to compare against the manifest's, then ran its canonicity re-encode, which derived the *same identity from the same value* a second time. `encode` now splits: the public entry derives and delegates to `encode_with_identity`, which takes the identity as a parameter. `decode` passes the one it already has.

The parameter is documented at the site as being there because deriving it is not cheap and the one caller that needs it already holds the value — so a future caller has to pass it deliberately rather than get a second derivation for free.

**Nothing about the canonicity guarantee changed.** The re-encode still runs and still compares byte-for-byte; it simply stops re-deriving one of its inputs. Whether the backstop itself should exist is `decide-whether-the-canonicity-re-encode-is-redundant`, and this reduction stands whichever way that goes.

## Not done, and left explicitly

Three items from this ticket remain, each independent of the one above:

- **`expression_keys` runs four times per decode** — `decode.rs:590`, `validate.rs:80`, and twice through `encode_identity`. Worth attacking together with `encode-abi-expression-identity-in-linear-space`, since that ticket changes what the keys cost in the first place and doing them in the other order means measuring twice.
- **`decode_metadata` runs `2 + E` times**, each re-allocating a `PayloadMetadata` including a `source.to_vec()` bounded at 16 MB. The per-entry call at `validate.rs:342` carries a sound comment about not depending on having been reached, so the fix is to hoist the decode rather than to skip it — a real change to that function's shape and not a one-liner.
- **`DecodedExpr::value_type()`** rebuilds the type vector from node 0 on every call, though `validate.rs:103-117` already computes the full table and discards it.

Reopening rather than closing would misstate the state: the identity half is done and measured, the rest is untouched.

Gate: `make full` green (982 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck). Every existing codec test passes unchanged, which is what shows the encoding is byte-identical.

## Outcome, second pass: the fourth fact was the only one that paid

**Decode fell from 494.9 µs to 419.0 µs — 15.3% — on the same 26,126-byte envelope.** The item that produced it is the one this ticket listed under *Facts* and then dropped from its own "not done" list: **every hashed byte was hashed twice.** The other three were measured and skipped, and the numbers are below.

Measurement protocol: minimum of 400 in-process repeats, and the minimum of four such runs reported. Host noise only ever makes a run slower, so the distribution has a hard floor and an unbounded tail; a mean is a report about the machine. Before and after were taken with the *same* harness (`hot_path_decode_stage_budget`), reverting the source between them.

### What landed: the section digests are derived once

`decode` digests every section to compare against its descriptor, and then re-encoded the envelope as its canonicity backstop — which digested the very same `Section` values a second time. Section content is 8,075 of the 26,045 hashed bytes, 31%, and the governed digest is where a decode spends nearly all of its time, so the repetition was 75.4 µs of a 494.9 µs decode.

`read_sections` now returns the digests it derived alongside the sections, and `encode_with_identity` takes them as a required parameter — required rather than optional, because its one production caller already holds them and a future caller should have to pass them deliberately.

**This does not weaken the backstop, and the symmetric change to the manifest digest would.** A section digest is a pure function of the `Section` the envelope holds, so a caller passing the one it derived from that same value supplies nothing the encoder did not already have; a wrong one would make the re-encoded bytes differ from the wire and fail *closed*. The manifest digest is different in kind — it covers the manifest bytes the encoder is rebuilding, and whether those reproduce the wire's is the very question the backstop asks — so it is still derived on every re-encode. That asymmetry is recorded at the site.

### What was measured and skipped

| Item | Measured | Verdict |
|---|---|---|
| `expression_keys` 4× per decode | 541 ns per call, **0.1%** of decode | skipped; all four are 0.4% |
| `decode_metadata` `2 + E` times | 583 ns per call, **0.11%** of decode | skipped; removing two of three is 0.22% |
| `DecodedExpr::value_type()` | **not on the decode path at all** | skipped |

`decode_metadata` was measured against a carried-payload fixture, since the default fixture carries no payload content and never reaches it. Hoisting it would mean removing the per-entry re-parse whose comment states that the check must not depend on `check_payload_identity` having been reached — trading a stated correctness property for 0.22% is the wrong direction.

`DecodedExpr::value_type()` is a reader accessor on `DecodedArtifact`. `decode` never calls it; the only call sites in the repository are two assertions in this crate's own tests. Its cost is 0% of a decode, and there is no production caller to speed up.

### Where the remaining time actually goes

The profile, not the source, answers this, and it names a function neither this ticket nor its sibling mentions. **~90% of a decode is the governed SHA-256 at ~120 MB/s**; a sampling profile attributes 57% of active self time to `_platform_memmove` and 10% to `<[u32]>::rotate_right`, both reached from `Sha256::compress`, which shifts its `[u32; 8]` working state with the *slice* `rotate_right(1)` — a `memmove` call, 64 times per 64-byte block. A standalone A/B of the same rounds, asserted to produce identical state, measures 280.7 MB/s for that spelling against 407.2 MB/s for named-variable reassignment. Everything else this ticket named sums to under 1%.

Also worth recording: **the first pass's headline is not reproducible.** It reported decode falling 662 µs → 501 µs, 24%, from deriving the canonical identity once instead of twice. Deriving it measures **1.5 µs, 0.3% of a decode** — removing one duplicate cannot save 161 µs. That change is still correct and still worth keeping; its measured effect was host noise read as a mean.

### New measurement harness

`hot_path_decode_stage_budget` reports the four stages of a decode against one another, `hot_path_digest_throughput` reports what the budget actually is, `hot_path_carried_metadata_decode` prices the carried-payload path, and `hot_path_decode_profile_loop` is the `#[ignore]`d loop a sampling profiler attaches to, with the exact recording commands in its doc comment. `min_and_mean` is shared by all of them and prints both numbers.

## Closes when

Done for the items this ticket named. One decode derives the canonical identity once, the section digests once, and the expression keys the same four times it always did — that last deliberately, with the number that says why. Decode is measured before and after, every existing codec test passes unchanged, which is what shows the encoding is byte-identical, and `make full` passes.

`encode-abi-expression-identity-in-linear-space` still owns what the expression keys cost; at 0.1% of a decode it is not a decode-path concern. Making the governed digest fast is the remaining decode work by an order of magnitude and belongs to whoever owns `codec/digest.rs`.
