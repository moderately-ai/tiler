---
id: re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps
title: Re-derive the measured envelope band the cache hot-path sweeps
status: done
priority: p2
dependencies: []
related: [wire-the-delivered-realization-record-into-the-artifact, re-price-the-envelope-band-consumers-against-the-re-derived-band]
scopes: [research/cache, research/embedding]
shared_scopes: [project/tickets]
paths: []
tags: [research, cache, measurement]
---
`spikes/cache/hot-path-efficiency` sweeps two envelope lengths, `SIZES = [32_136, 47_803]`, and its own comment says why those and not round numbers: they are the exact endpoints of the envelope band `docs/research/embedding/self-contained-embedding.md` measured, and measuring at a measured band's endpoints is what lets a reader put a cost against sizes the corpus already claims are realistic.

**Fact — the band no longer contains one envelope.** Re-running the harness on 2026-08-05 (`wire-the-delivered-realization-record-into-the-artifact`, based at `55d1d09f`) aborts on the harness's own precondition before any row is produced:

```
# envelope fixed overhead=114025 bytes; compiled in 11.777584ms
an envelope cannot be smaller than its 114025 byte fixed overhead; asked for 32136
```

`EnvelopeFactory::exactly` panics deliberately here rather than rounding, on the reasoning its own documentation states: a spike that silently reported a 33 KB envelope as a 32 KB one would put the wrong number beside every per-byte cost it measured.

**Fact — this predates the ticket that found it, and the arithmetic is on the spike's README.** The retained 2026-08-04 results record a fixed overhead of 28,527 bytes. The delivered-realization record contributes 2,453 canonical bytes to this envelope — measured by printing `canonical_bytes().len()` from the repaired harness — carried twice, once inside the folded artifact identity the manifest also carries and once as its own framed run, so roughly 4.9 KB of the current 114,025. The remaining ≈109 KB accumulated between 2026-08-04 and `55d1d09f`, an interval over which `spikes/target-profiles/scalar-cpu-vertical` independently records its own envelope growing from 21,296 to 82,918 bytes with no delivered-realization record involved.

## What this is not

**Not "raise `SIZES` to something above the overhead".** The endpoints are a *measured* claim borrowed from another document, and replacing them with round numbers above the current overhead would substitute convenience for evidence and leave the research note quoting per-byte costs against a band nothing measures any more.

## Closes when

1. The envelope band `docs/research/embedding/self-contained-embedding.md` states is re-derived against the current artifact encoding, or that document records why the earlier band still stands and what the hot-path sweep should measure instead.
2. `spikes/cache/hot-path-efficiency`'s `SIZES` names endpoints derived from that answer, with the derivation stated where the constant is.
3. The spike runs end to end again and records a result; `docs/research/cache/hot-path-efficiency.md` states which of its numbers the re-run reproduced and which moved, keeping the superseded values rather than overwriting them.
4. The two retained 2026-08-04 results are left in place: they are evidence taken at their own commit, and a re-run under a new label is what replaces them.

## Trigger check log

- 2026-08-05: not a deferral — this is dispatchable work with no gate on it. Filed at `todo`. The condition that produced it reproduces with `cd spikes/cache/hot-path-efficiency && cargo build --release && ./target/release/cache-hot-path-efficiency --quick`.

## Outcome — 2026-08-06, at `8bd720b8`

**The band is re-derived, not rounded, and the derivation is the original's.** `docs/research/embedding/self-contained-embedding.md` derived 32,136–47,803 by running `prototypes/serial-sum-compile` and taking the minimum and maximum of its members' envelope lengths. The same run at this base, with `MANIFEST_SCHEMA` at `14.0`:

| Member | `metallib` | Envelope 2026-07-31 | Envelope 2026-08-06 |
| --- | ---: | ---: | ---: |
| `empty-domain.selected` | 3,491 | 32,136 | 143,106 |
| `empty-domain.materialized` | 6,662 | 45,683 | 158,401 |
| `singleton.selected` | 3,603 | 34,030 | 141,532 |
| `singleton.materialized` | 7,078 | 46,445 | 155,695 |
| `nontrivial.selected` | 3,763 | 36,838 | 146,324 |
| `nontrivial.materialized` | 7,158 | 47,803 | 159,037 |
| `contraction.selected` | 3,891 | — | 90,737 |
| `contraction-w-decode-kv.selected` | 3,891 | — | 89,250 |

**Old band 32,136–47,803. New band 141,532–159,037** over the six members the original was taken over, and 89,250–159,037 over everything this producer publishes today; the two contraction members are new since the record was written. **Every `metallib` count is byte-identical to the 2026-07-31 record**, so no part of the growth is backend output, an optimization-level change, or a Metal toolchain difference — it is entirely artifact encoding, measured rather than inferred. The authoritative profile moved from `tiler.metal.macos-apple9.msl4-0.f32.v1` to `…msl4-0.f32-bf16.v1` with a 1,999-byte descriptor.

**The overhead now, and where it is.** The hot-path harness's own fixed overhead — one envelope of its fixture carrying zero object bytes — is **114,043 bytes** at this base, against **114,025** at `55d1d09f` (+18) and **28,527** at `194744e6`, the commit the retained 2026-08-04 results were taken at. Both ends were measured by building that commit's own harness and reading the `envelope fixed overhead` line it prints; parsing the framing header and section table of each attributes the whole +85,516:

| Part | `194744e6` | `8bd720b8` | Change |
| --- | ---: | ---: | ---: |
| Framing header | 69 | 69 | — |
| Canonical manifest | 22,698 | 88,061 | +65,363 |
| `KernelProgramSubject` section | 2,750 | 22,903 | +20,153 |
| `BackendPayloadMetadata` section | 2,974 | 2,974 | — |
| Section framing (3 × 12) | 36 | 36 | — |
| **Fixed overhead** | **28,527** | **114,043** | **+85,516** |

Three quarters is canonical manifest and the rest is the packaged kernel program's canonical identity, which grew 8.3×; the payload metadata section did not move a byte, which is the control that makes the other rows readable. The delivered-realization record is roughly 4.9 KB of the manifest row, ≈7.5% of it. `MANIFEST_SCHEMA` moved only `12.0` → `14.0` in the interval and `14.0` changes no length, so **the growth is in what the manifest describes, not in how the codec frames it**. Attributing it to individual commits would need the fixture rebuilt at intermediate commits and is deliberately not claimed.

**`SIZES` = `[141_532, 159_037]`**, with that derivation stated at the constant, including why the 89,250-byte contraction member is unreachable: this harness compiles the same scale-then-reduce program and its own fixed content is 114,043 bytes, so that length cannot be synthesized at all.

**The sweep runs end to end and two results are recorded**, back to back, 35.5 s each, same host and toolchain as the retained pair (Apple M4 Max, macOS 27.0 / Darwin 27.0.0, APFS under `$TMPDIR`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` from the `nightly-2026-07-19` pin, release profile; load average `6.71 6.09 8.22` at start and `5.59 5.93 7.99` at end — the host was carrying other agents' builds, and the response is the harness's own, the minimum as estimator at the raised sample counts). The two 2026-08-04 results are untouched.

```sh
cargo run -p tiler-prototype-compile --release -- --out <dir>/serial-sum
cd spikes/cache/hot-path-efficiency && cargo build --release
./target/release/cache-hot-path-efficiency --repeats 8000 --scan-repeats 15 \
  --publish-rounds 128 --cold-children 48 --record macos-27.0-2026-08-06
./target/release/cache-hot-path-efficiency --repeats 8000 --scan-repeats 15 \
  --publish-rounds 128 --cold-children 48 --record macos-27.0-2026-08-06-reproduction
```

**Reproduced.** The bundle section digest rate, 0.337 ns/B at both lengths against 0.338/0.337 — the same 2.97 GB/s over envelopes 4.4× larger. The read is syscall-bound: 14.3–14.7 µs against 12.2–12.9 µs for 4.4× the bytes. `Fsync` publication is 8.18–8.28 ms against 8.17–8.36 ms, so what it buys really is a fixed number of round-trips. Key derivation 125–166 ns, identical. Uncontended lock 13.7–15.2 µs against 13.9–15.3, and still not on the hit path (contended and uncontended hits agree within 0.5%). The scan is within ~7% of its old value at every population and its marginal cost is 2.7–3.1 µs/entry against 2.5–2.8; removal is 71.8–80.1 µs/entry against 73–86, reclaiming 1.42 GB and 1.59 GB against 325 MB and 481 MB. Population flatness is *tighter*: 0.16% and 0.07% across a thousandfold range against 0.6% and 0.3%. The decomposition residual is 1.0–1.3%, so the restated bundle framing still holds at `MANIFEST_SCHEMA` `14.0`. Every oracle, population, and lock-held control fired as before.

**Moved.** A validated hit is **159.3 µs / 172.0 µs** against 55.5 / 67.2. Cold-process 235–270 µs against 105–140, so the cold-to-warm ratio narrowed from ~2× to ~1.5×. Fail-closed integrity is **89.4–90.3%** of a hit against 73.5–79.3%. The build closure did **not** change (4.71 / 4.47 ms, inside the old 3.6–5.4 ms), so the cache is **26–30×** cheaper than producing an artifact rather than 65–97× — the number this growth cost the most. Publication is ~4.5× a hit and `Fsync` ~52×, against ~10× and ~150×; the `Fsync` multiplier is 10.8–11.6× against 14.6–15.1×, still inside ADR 0083's 6.5–18.7× band. The full-namespace scan is **178–213× one hit** against 460–590× — the scan did not get cheaper, the hit got more expensive, and Section 6's conclusion stands with its number replaced. The caller's owned copy grew to 1.25–1.67 µs, still ~1% of a hit.

**Verdict.** "Efficient at the measured scales" remains supported; no component of `tiler-cache` regressed. What moved is the size of the object it validates. **The cost of a hit is now set almost entirely by envelope size**, so if the hit path is worth attacking again the lever is the encoding, not this crate.

**Files changed.** `spikes/cache/hot-path-efficiency/harness/src/main.rs` (`SIZES` and its derivation), `spikes/cache/hot-path-efficiency/README.md`, the two new `results/…-2026-08-06*.tsv`, `docs/research/embedding/self-contained-embedding.md` (re-derivation subsection in §1, the ceiling distance in §5, the §7 row, an Outcome), `docs/research/cache/hot-path-efficiency.md` (new §9, Outcomes renumbered to §10), `docs/research/cache/bounded-collection.md` (dated correction to ground (2), decision untouched), and this ticket. Nothing outside `docs/`, `spikes/`, and `tickets/`.

**Filed rather than absorbed.** [`re-price-the-envelope-band-consumers-against-the-re-derived-band`](re-price-the-envelope-band-consumers-against-the-re-derived-band.md) carries the two out-of-scope citers: `MaxEntryAge::DEFAULT`'s ground in `crates/tiler-cache/src/expansion/collect.rs` (`implementation/cache`), whose 200–400 MB projection reads 0.9–1.6 GB at the new band, and Q-ART-003 in `docs/open-questions.md` (`contracts/navigation`), whose recorded ceiling headroom is 15.17% rather than 4.56%.
