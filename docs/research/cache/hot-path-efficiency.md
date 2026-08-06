---
schema: "tiler-doc/v1"
id: "tiler.research.cache.hot-path-efficiency"
kind: "research"
title: "Expansion cache hot-path efficiency"
topics: ["cache", "performance", "measurement"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "implemented"
evidence_classes: ["bounded-measurement"]
informs: ["tiler.contract.frontend-integration", "tiler.contract.artifact-abi"]
depends_on: ["tiler.research.cache.bounded-collection"]
ticket: "measure-the-expansion-cache-hot-path-efficiency"
---

# Expansion cache hot-path efficiency

Status: Tom asked on 2026-08-04, alongside the eviction decision, that "the cache's efficiency is to be verified separately" ([the collection design](bounded-collection.md) records the relay). This note is that verification. Everything below comes from [`spikes/cache/hot-path-efficiency/`](../../../spikes/cache/hot-path-efficiency/README.md), which drives the **public** [`ExpansionCache`](../../../crates/tiler-cache/src/expansion/store.rs) with every hit validated by the real [`decode_artifact`](../../../crates/tiler-artifact/src/program/codec/decode.rs). Quoted figures are from the first of two retained runs at this commit; Section 1 states what the second reproduced and what it did not.

**Sections 2 to 8 quote the 2026-08-04 runs at 32,136 and 47,803 bytes, and those are not the lengths this producer emits any more.** The envelope band was re-derived on 2026-08-06 and the sweep re-run at its new endpoints, 141,532 and 159,037 bytes; **[Section 9](#9-the-re-run-at-the-re-derived-band-2026-08-06) is that run, and states figure by figure what reproduced and what moved**. Everything before it is retained unedited as the evidence of its own commit — the 2026-08-04 numbers are not wrong, they are measurements of a smaller envelope.

## The headline, before the evidence

**Measurement.** A validated cache hit costs **55.5 µs** at a 32,136-byte envelope and **67.2 µs** at a 47,803-byte one, and that cost is **flat across a thousandfold change in cache population** — over 10, 100, 1,000, and 10,000 stored entries the four minima span 0.6% and 0.3% respectively, and the second run agrees with the first to within 0.6%. The build closure the hit spares, measured in the same process at the start of each run, is 3.6–5.4 ms, so the cache returns a validated artifact **65–97× cheaper** than producing one, before any external backend compiler is involved at all.

**Measurement.** Fail-closed integrity is **73.5–79.3% of a hit**. The cache is validation-bound, not I/O-bound, not lock-bound, and not copy-bound: `decode_artifact` is 54.0–55.3% and the bundle's two section digests are 19.4–24.0%, while reading the file is 18.0–23.1%, key derivation is 0.2%, and path work is 0.4–0.6%.

**Measurement.** Three costs a reader might expect to find on the hit path are not on it, and each was measured rather than read off the source. **A hit takes no lock**: with a *separate process* holding the probe key's lock — proven held by a non-blocking acquisition being refused — a hit costs 55.7 µs against 55.8 µs uncontended, a difference of 0.2%. **A hit copies nothing to the caller**: a stored entry keeps the buffer the read allocated and hands out spans of it, and the copy a caller adds when it wants an owned `Vec` is 0.2–0.8 µs, under 1.5% of a hit. **A hit touches one key path**: the population columns above are flat, which is the structural claim `layout.rs` makes, observed.

**Measurement, and the answer to a question the corpus had left explicitly open.** [The collection design](bounded-collection.md) says "whether the per-eviction scan is cheap enough to run on a long-lived `rust-analyzer` server that expands continuously is `measure-the-expansion-cache-hot-path-efficiency`'s, and it is a measurement this note does not have". It has it now, and the answer is **no**: a full-namespace scan of a 10,000-entry cache costs **29.1–32.6 ms**, which is **460–590× one hit**. An eviction triggered once per expansion would turn a 56 µs hit into a 33 ms one. The trigger has to amortize, which is exactly what [`wire-the-env-configured-eviction-policy-through-the-deliver-path`](../../../tickets/wire-the-env-configured-eviction-policy-through-the-deliver-path.md) already requires of itself; this note supplies the number that requirement has to be sized against.

**Inference.** "Properly efficient at the measured scales" is therefore **supported**, with the boundary in Section 8. No inefficiency inside `tiler-cache` was located. One narrow question the measurement raises — whether the bundle's own envelope-section digest is redundant with what `decode_artifact` already proves about the same bytes, worth 19.4–24.0% of a hit — was filed as [`decide-whether-the-bundle-envelope-section-digest-is-redundant`](../../../tickets/decide-whether-the-bundle-envelope-section-digest-is-redundant.md) rather than acted on, because it is a validation contract and the shortcut is exactly the shape this repository refuses to take on argument. **It has since been answered, and the digest is retained on evidence** — see Section 10's third outcome.

## 1. Environment and procedure

**Measurement.** Apple M4 Max, 14 logical cores, macOS 27.0 (Darwin 27.0.0), APFS under `$TMPDIR`. `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, the `rust-toolchain.toml` pin `nightly-2026-07-19` resolved by directory ancestry. Release profile. No Metal toolchain is used or needed. The note that the Apple toolchain moved to Xcode 27.0 beta / Metal Toolchain 27A5228f on 2026-08-04 is recorded because the ticket asked for it and because earlier retained cache measurements cite the prior environment; nothing measured here touches it.

```sh
cd spikes/cache/hot-path-efficiency
cargo build --release
./target/release/cache-hot-path-efficiency \
  --repeats 8000 --scan-repeats 15 --publish-rounds 128 --cold-children 48 \
  --record macos-27.0-2026-08-04
```

Two runs of that command, back to back, are retained: [`…-2026-08-04.tsv`](../../../spikes/cache/hot-path-efficiency/results/hot-path-efficiency-macos-27.0-2026-08-04.tsv), which this note quotes, and [`…-2026-08-04-reproduction.tsv`](../../../spikes/cache/hot-path-efficiency/results/hot-path-efficiency-macos-27.0-2026-08-04-reproduction.tsv). Each takes 22–23 s.

**Measurement.** Warm-up is 64 hits before any hit sample; 8,000 samples per warm-hit and per decomposition row, 48 child processes per cold-hit row, 128 rounds per publication row, 15 per scan row, and one destructive collection per root. Each reported figure is the **minimum**, because every perturbation a host applies makes an operation slower and none makes it faster, so the distribution has a hard floor at the true cost and an unbounded tail of noise. The median, ninetieth percentile, and maximum are in the retained results beside every minimum.

**Measurement boundary — the host is shared, and the sample counts are the response.** This machine runs several agents' `cargo` builds concurrently; load average was `3.99 10.15 14.42` at the start of the first run and `4.30 9.35 13.90` at the end of the second. Averaging does not remove that interference — it absorbs it — so the estimator is the minimum and the sample counts are well above the harness defaults, which is the only lever that improves a floor estimate under load. It works: the maxima in these runs reach 4–6× their minima on the hit rows while the minima agree between runs to within 0.6%. Isolation is a dedicated scratch root named by process identifier and nanosecond under `$TMPDIR`, so no concurrent process shares a mutable path with the run.

**Measurement — what the second run reproduced.** Every warm-hit minimum within 0.6%; every decomposition component within 0.1–3%; the validation share within 1.4 points; default publication within 5%, the `Fsync` ratio within 11%, lock acquisition within 10%; the scan ladder within 9% at every population; and every oracle and lock control identically. The two figures that agree least are the ones with the fewest samples, and they are the two this note publishes as ranges rather than as values: the cold-process hit (48 samples) and the destructive collection (one sample per root).

**Measurement — one defect found in this harness by these runs, and fixed before they were taken.** The contended-hit row read 5–10% above the uncontended one until the lock-holding child stopped busy-waiting on its release marker. The child is alive for the whole of that measurement, so a spin loop was a second process competing for a core and for the filesystem with the thing being timed. It now sleeps between polls; ordering still comes from the marker's existence and from nothing else. The row it produces is the one quoted in Section 4, and the 5–10% it used to carry was the harness measuring itself.

**Measurement boundary — envelope sizes and populations.** Envelopes are 32,136 and 47,803 bytes, the exact endpoints of the band [the self-contained embedding note](../embedding/self-contained-embedding.md) measured on 2026-07-31 and [`MaxEntryAge::DEFAULT`](../../../crates/tiler-cache/src/expansion/collect.rs)'s ground cites; [Section 9](#9-the-re-run-at-the-re-derived-band-2026-08-06) re-derives that band and re-runs this sweep at 141,532 and 159,037. Populations are 10, 100, 1,000, and 10,000, filled cumulatively into one root per size. The remaining boundaries — synthetic object bytes, the stand-in artifact-program subject facet, "cold" meaning cold in process state rather than in storage, and one shared envelope across a population — are stated exactly in [the spike's own record](../../../spikes/cache/hot-path-efficiency/README.md#measurement-boundary) and are not repeated here.

## 2. What a hit costs

**Measurement.** Minimum nanoseconds, 8,000 samples per warm cell and 48 per cold cell. The reproduction run's warm minima are in brackets.

| Population | 32,136 B warm | 32,136 B cold process | 47,803 B warm | 47,803 B cold process |
| --- | --- | --- | --- | --- |
| 10 | 55,458 [55,333] | 105,791 | 67,167 [67,000] | 120,916 |
| 100 | 55,458 [55,458] | 104,042 | 67,125 [67,084] | 120,167 |
| 1,000 | 55,500 [55,833] | 122,708 | 67,208 [67,000] | 138,417 |
| 10,000 | 55,792 [55,291] | 136,667 | 67,334 [67,167] | 121,583 |

**Inference.** The warm columns are flat. Across a thousandfold population change the four minima span 0.6% at 32,136 bytes and 0.3% at 47,803, and the reproduction run's own four span 1.0% and 0.2% — with its largest value at a *different* population, which a population effect cannot do. A hit therefore **touches one key path**, which is what the two-character shard layout is for, measured rather than assumed.

**Measurement boundary — the cold column is noisier and says so.** It varies 104–137 µs and 120–138 µs with no monotone trend, and the reproduction run puts its own extremes at different populations again. That is 48 samples showing through, not a population term; what the column supports is the level, roughly 105–140 µs, and not a difference between its cells.

**Inference.** The two process patterns [the build-tool exercise](build-tool-exercise.md) measured differ by roughly a factor of two, consistently. One `rustc` per crate pays 105–140 µs for its first and only hit; a long-lived `rust-analyzer` proc-macro server pays 55–67 µs for every hit after the first. Both are small beside the closure they replace, and neither degrades with cache size.

## 3. What dominates a hit

**Measurement.** One hit at population 10,000, attributed. Minimum nanoseconds, 8,000 samples per component.

| Component | 32,136 B | share | 47,803 B | share |
| --- | --- | --- | --- | --- |
| `decode_artifact` | 30,334 | 54.3% | 36,458 | 54.0% |
| Bundle section digests | 10,875 | 19.5% | 16,125 | 23.9% |
| Open and read the entry | 12,875 | 23.1% | 12,166 | 18.0% |
| Key derivations (two per hit) | 125 | 0.2% | 166 | 0.2% |
| Path formation and parse | 333 | 0.6% | 292 | 0.4% |
| Residual | 1,291 | 2.3% | 2,251 | 3.3% |
| **Whole hit** | **55,833** | | **67,458** | |
| *of which fail-closed integrity* | *41,209* | *73.8%* | *52,583* | *77.9%* |

**Measurement.** The residual is 2.3–3.3% here and 0.3–3.1% in the reproduction, which is what makes the attribution usable: the components are reimplementations of crate-private steps, and a step this harness had mis-modelled would appear as a large residual instead of disappearing. The reimplementation and the assertion that keeps it honest — the restated bundle frame must actually delimit the published subject and envelope — are stated in the spike's record.

**Inference.** The cache is **validation-bound**. Three quarters to four fifths of a hit is proving that the stored bytes are what they claim to be, and that is the property [ADR 0050](../../decisions/0050-use-immutable-self-validating-expansion-cache-entries.md) exists to guarantee. Nothing in the remaining fifth is worth attacking: the file read is a fixed ~12 µs of syscall regardless of size, and keying and path work together are under 1%.

**Inference — the shape of each component, from the two sizes.** The bundle section digests scale purely per byte: 0.338 ns/B at 32,136 and 0.337 ns/B at 47,803, which is 2.96 GB/s and is SHA-256 over the whole bundle and nothing else. `decode_artifact` is affine — a two-point fit gives ~17.8 µs fixed plus ~0.39 ns/B — because the envelope's fixed content is 28,527 bytes in this fixture and only the carried object grows. The file read does *not* scale with size (12.9 µs for the smaller envelope, 12.2 µs for the larger), so it is syscall-bound rather than byte-bound at these lengths, and a whole hit fits ~32.0 µs plus ~0.74 ns/B. **These are fits through two points, used only to separate a per-byte cost from a per-read one; they are not a basis for extrapolating to a much larger envelope.**

**Fact, for a reader who asks why `decode_artifact` costs what it does.** It verifies the manifest digest, every section's content digest, and the artifact's canonical identity, and then re-encodes the whole envelope and byte-compares it against the input. That last step is the canonicity backstop, and it is **retained on evidence**: [`decide-whether-the-canonicity-re-encode-is-redundant`](../../../tickets/decide-whether-the-canonicity-re-encode-is-redundant.md) neutered each named canonicity check in turn, found the set of forms only the backstop covers to be non-empty, and closed by keeping it. This note does not reopen that.

**Fact, and a discrepancy left standing rather than reconciled.** That same ticket records "the re-encode at `decode.rs:113` is 50% of decode time — 274 µs of 548 µs on a 26 KB envelope". This note measures 30.3 µs of decode on a 32 KB envelope. The two are not comparable as they stand: they are different envelopes, taken at different commits, and several optimization phases ran between them. Reconciling them would need that harness re-run at this commit, which is not this note's claim and is not asserted here.

## 4. What a hit does not cost

**Measurement — the lock.** Taking and releasing one uncontended key lock costs 13.9–15.3 µs, which is 21–27% of a whole hit. It is **not on the hit path**. With a separate process holding the probe key's lock, a hit costs 55,708 ns against 55,792 uncontended at 32,136 bytes and 67,333 against 67,334 at 47,803 — the same number to within 0.2%, and within 1.4% in the reproduction run.

**Measurement — the control that makes that mean something.** "A hit was served while a lock was held" proves nothing if no lock was held. The child creates its ready marker only *after* it holds the lock; the parent then attempts a non-blocking acquisition on the same lock file and **requires it to be refused** before timing anything. The refusal is a row in the result. Ordering rests on the marker's existence, never on a wall-clock margin.

**Inference.** Had the read path taken the lock, the run would not have produced a slower row — it would have **deadlocked**, because the child releases only once the parent writes the release marker and the parent writes it only after the timed hits return. That is the strongest possible form of the observation: the failure mode is a hang, not a number to squint at. `store.rs` documents the read as lock-free; this is that documentation observed across a real process boundary.

**Measurement — bytes to the caller.** A stored entry keeps the buffer the read allocated and returns spans of it, so `CachedEntry::envelope_bytes` is a borrow and the hit path performs no copy **beyond the single read into that buffer** — the row above accounts for that read, and there is no second traversal handing bytes out. The copy a caller adds when it wants an owned `Vec` is 208–833 ns at 32,136 bytes and 583–708 ns at 47,803 across the two runs, under 1.5% of a hit either way.

**Fact, and a wording sharpened in a sibling note rather than a claim overturned.** [The collection design](bounded-collection.md)'s racing-a-reader enumeration said "`lookup` copies the validated envelope into the `CachedEntry` it returns". That is true of the read as a whole — the entry does hold an in-memory copy of the file's bytes — and it reads as a copy taken *out of* the read's buffer, which `EntryBytes::Stored` in [`store.rs`](../../../crates/tiler-cache/src/expansion/store.rs) deliberately does not take. Sharpened there on 2026-08-04, with the position's conclusion untouched because it was never wrong. The same wording appears in [`collect.rs`](../../../crates/tiler-cache/src/expansion/collect.rs)'s module documentation and is left alone: it is imprecise rather than false, correcting it needs the `implementation/cache` scope this branch does not hold, and a ticket whose whole content is one adverb costs more than it saves.

## 5. What a publication costs

**Measurement.** Minimum nanoseconds over 128 rounds, each on a fresh cache root so the round genuinely publishes rather than hits — asserted, not assumed. Reproduction values in brackets.

| Envelope | `ProcessCrash` | `Fsync` | ratio | bare `rename` of the same length |
| --- | --- | --- | --- | --- |
| 32,136 B | 542,833 [544,459] | 8,172,750 [8,159,166] | 15.1× [15.0×] | 91,917 [67,458] |
| 47,803 B | 571,584 [600,833] | 8,356,667 [8,041,083] | 14.6× [13.4×] | 92,792 [64,792] |

**Inference.** The `Fsync` multiplier sits inside the 6.5×–18.7× band [ADR 0083](../../decisions/0083-keep-process-crash-as-the-default-cache-durability.md) measured and keeps that decision's shape: the cost is flat in payload — 8.17 ms against 8.36 ms across a 49% size increase — so what it buys is a fixed number of synchronization round-trips rather than durable bytes proportional to the entry. Nothing here is evidence about power loss.

**Inference.** Publication is roughly **10× a hit** under the default policy and **150× a hit** under `Fsync`, and the atomic rename itself is 11–17% of a default publication. That asymmetry is the right one for this cache: publication happens once per key and a hit happens on every subsequent expansion of it.

## 6. What an eviction pass costs

**Measurement.** Minimum nanoseconds over 15 scans. `account` is the walk; `collect` under `UNBOUNDED` is the walk plus the selector's early return; `collect` under an age no entry has reached is the walk plus the clone and sort the age predicate forces.

| Population | `account` (32,136 B) | `collect` unbounded | `collect` age, retaining all | `account` (47,803 B) |
| --- | --- | --- | --- | --- |
| 10 | 150,084 | 158,458 | 161,125 | 161,250 |
| 100 | 1,503,042 | 1,507,750 | 1,504,500 | 1,497,250 |
| 1,000 | 7,061,917 | 7,030,417 | 6,417,917 | 6,499,583 |
| 10,000 | 32,600,625 | 31,670,833 | 31,552,958 | 30,965,666 |

**Inference.** The three variants agree to within 10% at every population and to within 3% at the two smallest, so **the scan is essentially the entire cost of a non-removing collection**: cloning and sorting 10,000 `EntryFact`s is inside the run-to-run spread of walking them. The selector is not what to look at.

**Inference.** Per entry, the scan is sub-linear in population — 15.0 µs at 10 entries and 3.3 µs at 10,000 — because the namespace shards on two hexadecimal characters, so the 256 shard directories saturate and their `read_dir` cost amortizes. The **marginal** cost, taken between 1,000 and 10,000 where the shard count is saturated, is **2.5–2.8 µs per entry** across the four size-by-run combinations, which is one `stat` and one path parse each.

**Measurement.** A collection that actually removes everything it scans, at 10,000 entries, one observation per size per run: 812.5 ms and 864.2 ms in the first run, 729.7 ms and 774.1 ms in the second, reclaiming 324.7 MB and 481.3 MB. That is **73–86 µs per removed entry** — a non-blocking lock acquisition, a re-`stat`, an unlink, and a close, per key. It is quoted as a range because it is the one figure here taken from a single sample rather than a minimum over many; what the four observations agree on is that removing an entry costs roughly 1.3 to 1.5 times what reading one costs. The removal report accounted for its whole selection and the removals equalled the counted population in every run, both asserted.

**Inference — the answer to the question left open.** [The collection design](bounded-collection.md) states plainly that whether the per-eviction scan can run on a continuously expanding `rust-analyzer` server "is a measurement this note does not have". The measurement says it cannot run per expansion: at 10,000 entries the scan alone is 460–590× a hit, so an eviction on every expansion would turn a 56 µs cache hit into a 33 ms one and make the accelerator the dominant cost of the thing it accelerates. At a plausible steady state the absolute numbers stay modest — the 30-day window's own ground projects 200–400 MB, which at these envelope sizes is roughly 5,000–10,000 entries, so a full pass is 15–33 ms of scan plus, when a toolchain update orphans a whole generation, 0.4–0.9 s of removals — and that is a perfectly ordinary cost **once**, and an absurd one per invocation.

**Inference.** Two properties of the numbers bound what an amortization has to achieve rather than merely saying one is needed. The scan cost is a function of population and not of how often it runs, so any trigger that fires on a bounded fraction of expansions divides it directly; and the removal cost is per removed entry and independent of the trigger, so the once-per-toolchain-update spike is paid whenever the eviction next runs and cannot be amortized away by triggering less often. `wire-the-env-configured-eviction-policy-through-the-deliver-path` already carries the requirement in its own constraints ("the trigger must amortize, not walk every shard per expansion"); what it lacked was the size of the thing being amortized.

## 7. How this could have been vacuous

Each control names the population it covers, because a uniform pass over a heterogeneous population is the signature to distrust.

**Fact.** Every measured configuration compares the returned envelope against the exact published bytes — 8,000 comparisons per warm-hit row, one per publication round, one per cold-process child (as a governed digest the child computes and the parent compares), and one per decomposition sample. On its own that is consistent with a comparison that never fails, so the run **flips a single byte of a stored entry and observes the refusal**: `cache bundle section 'artifact-envelope' does not match its declared digest`, at both sizes. It then restores the byte and observes the hit return the exact published bytes. All three are rows in the retained result. This is the check that makes every equality in this note evidence rather than decoration.

**Fact.** The contended-lock row is preceded by a non-blocking acquisition that **must be refused**; the run panics if the lock turns out not to be held.

**Fact.** Every row's population is read back from the namespace with `account` and asserted against the fill, so a row cannot be labelled with a population the fill never reached. The destructive collection asserts that its removals equal that counted population and that its report accounts for every selected entry.

**Fact.** The decomposition's reimplemented steps are checked against the bytes: the spans derived from the restated bundle frame constants must actually delimit the published subject and envelope, so a framing change in `tiler-cache` fails the spike rather than silently redirecting its digests. The residual row is the second half of that control and reads 0.3–3.3% across the two runs.

**Fact.** No assertion in the harness is about a time, and no ordering rests on a wall-clock margin. Where a process waits for another's marker file it sleeps between polls and ends on the marker's existence; Section 1 records what happened when it spun instead.

**What this does not establish.** One host, one filesystem, one toolchain, one release profile, one envelope shape, two lengths, four populations. It is not evidence for a network filesystem, a Linux host, a cold storage cache, an envelope an order of magnitude larger, a concurrent multi-process read load, or a real Metal compilation.

## 8. The verdict, and its boundary

**Inference.** *"Efficient at the measured scales"* is supported. The load-bearing parts of the claim, each with the row behind it:

1. A hit is 55.5–67.5 µs and does not degrade with cache size across a thousandfold population range — the four population minima span 0.6% at one size and 0.3% at the other, reproduced.
2. The closure a hit spares is 3.6–5.4 ms — one observation per run, and a floor, since it involves no external backend compiler — so the accelerator is 65–97× cheaper than the work it replaces at its cheapest.
3. Three quarters to four fifths of a hit is contract-required integrity, and every remaining component is already at or near its floor: the read is syscall-bound at ~12 µs, keying and path work are under 1%, no lock is taken, and nothing is copied.
4. Publication is ~10× a hit and is paid once per key.
5. Nothing on the hit path scales with the namespace; everything that does — the scan and the removals — is off it by construction and stays off it by contract.

**Inference — where the cost that matters actually is.** Not in the cache's hit path but in *when* something else calls its collector. The only number in this note large enough to matter to a developer's build is the 29–33 ms full scan, and it is entirely a scheduling question that a live ticket already owns.

**Measurement boundary.** Every figure qualifies this host, this filesystem, this toolchain, and this procedure. None of it is a portable guarantee, and none of it is evidence about an unmeasured platform. Nothing here licenses a universal claim that the expansion cache is efficient; what it licenses is that on the supported development profile, at the entry sizes and populations the corpus already calls realistic, it is — and that the dominant cost is the validation the contract requires rather than anything incidental.

## 9. The re-run at the re-derived band, 2026-08-06

Sections 2 to 8 measure envelopes of 32,136 and 47,803 bytes. Those endpoints were the band [the self-contained embedding note](../embedding/self-contained-embedding.md) measured on 2026-07-31, and by 2026-08-05 the spike could not reach either of them: one envelope of its own fixture carried more fixed content than the whole of the smaller one, so `EnvelopeFactory::exactly` refused rather than reporting a length it had not produced. [`re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps`](../../../tickets/re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps.md) re-derived the band by re-running the producer that set it, and this section is the sweep re-run at the new endpoints. **The 2026-08-04 figures above are retained unedited**: they are measurements of a smaller envelope at their own commit, not errors.

**Measurement — procedure, identical but for the two lengths.** Base commit `8bd720b8`, `MANIFEST_SCHEMA` `14.0`. Same host, same filesystem, same toolchain as Section 1: Apple M4 Max, 14 logical cores, macOS 27.0 (Darwin 27.0.0), APFS under `$TMPDIR`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` from the `nightly-2026-07-19` pin, release profile. Two runs back to back, 35.5 s each, retained as [`…-2026-08-06.tsv`](../../../spikes/cache/hot-path-efficiency/results/hot-path-efficiency-macos-27.0-2026-08-06.tsv) — quoted here — and [`…-2026-08-06-reproduction.tsv`](../../../spikes/cache/hot-path-efficiency/results/hot-path-efficiency-macos-27.0-2026-08-06-reproduction.tsv), in brackets. Load average was `6.71 6.09 8.22` at the start of the first and `5.59 5.93 7.99` at the end of the second: the host was carrying other agents' builds throughout, exactly as in Section 1, and the response is the same one — the minimum is the estimator and the sample counts are the raised ones.

```sh
cd spikes/cache/hot-path-efficiency
cargo build --release
./target/release/cache-hot-path-efficiency \
  --repeats 8000 --scan-repeats 15 --publish-rounds 128 --cold-children 48 \
  --record macos-27.0-2026-08-06
```

### 9.1 Why the envelope moved, measured at both ends

**Measurement.** The same fixture's fixed overhead — one envelope of this harness's program carrying zero object bytes — is **28,527 bytes at `194744e6`** (the commit the 2026-08-04 results were taken and retained at) and **114,043 bytes at `8bd720b8`**. Both were produced by building that commit's own harness and reading the `envelope fixed overhead` line it prints. Parsing the framing header and section table of each attributes every byte of the difference:

| Part of the envelope | 2026-08-04 (`194744e6`) | 2026-08-06 (`8bd720b8`) | Change |
| --- | ---: | ---: | ---: |
| Framing header | 69 | 69 | — |
| Canonical manifest | 22,698 | 88,061 | **+65,363** |
| `KernelProgramSubject` section | 2,750 | 22,903 | **+20,153** |
| `BackendPayloadMetadata` section | 2,974 | 2,974 | — |
| Section framing (3 × 12) | 36 | 36 | — |
| **Fixed overhead** | **28,527** | **114,043** | **+85,516** |

**Inference.** Three quarters of the growth is canonical manifest and the remaining quarter is the packaged kernel program's canonical identity, which grew 8.3×. The payload metadata section is **byte-identical** across the two commits, which is the control that makes the other two rows readable: the parts of the encoding that did not change did not change. The delivered-realization record `MANIFEST_SCHEMA` `13.0` requires is inside the manifest row and is a small part of it — roughly 4.9 KB of the 65,363, on the 2,453-byte canonical measurement `wire-the-delivered-realization-record-into-the-artifact` recorded, carried once inside the folded identity and once as its own framed run. The schema itself only moved `12.0` → `14.0` in this interval, and the `14.0` step re-orders without changing any length, so **the growth is in what the manifest describes rather than in how the codec frames it**. Attributing it to individual changes is not done here and is not this note's question.

**Measurement — the band the sweep now uses.** 141,532 and 159,037 bytes, the minimum and maximum envelope length of the six reduction-class members `prototypes/serial-sum-compile` publishes at this commit — the same population and the same derivation the 32,136–47,803 band came from. That producer's compiled objects are byte-identical to the ones recorded on 2026-07-31, so none of this is backend output. The two contraction members it has since gained, at 89,250 and 90,737 bytes, are **below this harness's own 114,043-byte fixed overhead** and are therefore not lengths it can synthesize; [the embedding note's re-derivation](../embedding/self-contained-embedding.md#the-band-re-derived-2026-08-06--the-envelope-column-moved-and-the-metallib-column-did-not) carries the full eight-member table.

### 9.2 What a hit costs — reproduced in shape, moved in level

**Measurement.** Minimum nanoseconds, 8,000 samples per warm cell and 48 per cold cell. Reproduction run in brackets.

| Population | 141,532 B warm | 141,532 B cold process | 159,037 B warm | 159,037 B cold process |
| --- | --- | --- | --- | --- |
| 10 | 159,500 [159,292] | 239,333 [256,625] | 172,000 [172,083] | 266,459 [259,958] |
| 100 | 159,459 [159,334] | 242,083 [235,083] | 171,875 [171,833] | 263,500 [262,542] |
| 1,000 | 159,417 [159,125] | 246,333 [247,416] | 171,875 [171,875] | 263,417 [269,333] |
| 10,000 | 159,250 [159,375] | 248,792 [243,875] | 171,916 [172,041] | 265,958 [255,792] |

**Reproduced — flatness, and more sharply than before.** Across a thousandfold population change the four minima span **0.16%** at 141,532 bytes and **0.07%** at 159,037, against 0.6% and 0.3% at the old lengths, and the two runs agree to within 0.16%. The first run's largest 141,532-byte value is at population 10 and the second run's is at population 10,000, which a population effect cannot do. A hit still touches one key path.

**Moved — the level, by 2.9×.** A validated hit is **159.3 µs** at 141,532 bytes and **172.0 µs** at 159,037, against 55.5 and 67.2 µs at the old band. The cold-process column is 235–270 µs against 105–140 µs, and it is still the noisier row with 48 samples and no monotone trend. The cold-to-warm ratio narrowed from roughly 2× to roughly **1.5×**: the per-process cost that column adds is fixed while the hit it wraps got larger.

**Moved — the margin over the work a hit replaces.** The build closure measured in the same process is 4.71 ms [4.47 ms], inside the 3.6–5.4 ms Section 8 quotes, so the closure did **not** change. The hit did, which makes the cache **26–30× cheaper** than producing an artifact rather than 65–97×. It is still an accelerator and the comparison is still a floor — no external backend compiler is in it — but the ratio is the number in this note that the encoding growth cost the most.

### 9.3 What dominates a hit — the same components, a more validation-bound hit

**Measurement.** One hit at population 10,000, attributed. Minimum nanoseconds, 2,000 samples per component. Reproduction in brackets.

| Component | 141,532 B | share | 159,037 B | share |
| --- | ---: | ---: | ---: | ---: |
| `decode_artifact` | 94,959 [94,750] | 59.6% | 101,333 [101,750] | 58.9% |
| Bundle section digests | 47,708 [47,666] | 30.0% | 53,583 [53,583] | 31.2% |
| Open and read the entry | 14,334 [14,500] | 9.0% | 14,709 [14,625] | 8.6% |
| Key derivations (two per hit) | 166 [125] | 0.1% | 125 [125] | 0.1% |
| Path formation and parse | 291 [250] | 0.2% | 208 [250] | 0.1% |
| Residual | 1,834 [2,043] | 1.2% | 2,042 [1,667] | 1.2% |
| **Whole hit** | **159,292 [159,334]** | | **172,000 [172,000]** | |
| *of which fail-closed integrity* | *142,667 [142,416]* | *89.6%* | *154,916 [155,333]* | *90.1%* |

**Reproduced — the digest rate, to three digits.** The bundle section digests are **0.337 ns/B** at both lengths, against 0.338 and 0.337 on 2026-08-04. That is the same 2.97 GB/s governed SHA-256, measured over envelopes 4.4× larger, and it is the strongest single reproduction in this re-run.

**Reproduced — the read is syscall-bound.** Opening and reading the entry is 14.3–14.7 µs against 12.2–12.9 µs, a 15% rise for **4.4× the bytes**. Section 3's claim that this component does not scale with envelope size at these lengths survives a much harder test than the one that produced it.

**Reproduced — keying, path work, and the residual.** Key derivation is 125–166 ns, identical to 2026-08-04's 125–166 ns; path work is 208–333 ns against 292–333 ns; the residual is **1.0–1.3%** against 0.3–3.3%. The residual matters beyond its size: the decomposition restates `tiler-cache`'s bundle framing constants, and a framing change under `MANIFEST_SCHEMA` `14.0` would have shown up here as a large residual or as the `bundle_spans` assertion failing. Neither happened.

**Moved — the validation share, upward.** Fail-closed integrity is **89.4–90.3%** of a hit against 73.5–79.3%. The cache is not merely validation-bound now; validation is nearly all of it. The reason is structural rather than a regression: the envelope's fixed content grew from 28,527 to 114,043 bytes and every one of those bytes is digested and decoded, while the file read stayed a fixed ~14 µs of syscall.

**Moved — the fits, and what they are fits of.** `decode_artifact`'s marginal cost is **0.364 [0.400] ns per object byte** against 0.391, and the whole hit's is **0.726 [0.723] ns/B** against 0.74 — both reproduced. The intercepts moved, from ~17.8 µs to 38–43 µs for decode and from ~32.0 µs to ~57 µs for the whole hit, which is what an 85,516-byte increase in fixed content has to do. **The slope is a per-*object*-byte slope**, and always was: both endpoints of a sweep share one fixed content and differ only in carried object, which is exactly what makes the length exact. Dividing decode's intercept by the fixed content gives 0.33–0.38 ns per fixed byte, within the marginal rate — so at this band decode is close to purely per-byte with no significant constant term. As before, these are fits through two points and are not a basis for extrapolating to an envelope an order of magnitude larger.

### 9.4 What a hit does not cost — every control reproduced

**Reproduced — the lock is not on the hit path.** An uncontended acquire-and-release is **13.7–15.2 µs**, against 13.9–15.3 µs: unchanged, and now 8–9% of a hit rather than 21–27%. With a separate process holding the probe key's lock — a non-blocking acquisition **required to be refused** before anything is timed, in all four size-by-run combinations — a hit costs 159,292 against 159,250 uncontended at 141,532 bytes and 172,084 against 171,916 at 159,037, agreeing to within 0.5% in every combination.

**Reproduced — the oracle refuses.** Every configuration returned the exact published bytes; a single flipped byte was refused with `cache bundle section 'artifact-envelope' does not match its declared digest` at both sizes in both runs; restoring the byte restored the hit. Every population was counted back from the namespace with `account` and asserted against the fill, and the destructive collection accounted for its whole selection.

**Moved, slightly — the caller's copy.** An owned `Vec` costs 1,250–1,333 ns at 141,532 bytes and 1,625–1,666 at 159,037, against 208–833 and 583–708. It grew with the buffer, as a copy must, and is **0.8–1.0%** of a hit — still under the 1.5% Section 4 bounds it at.

### 9.5 What a publication costs — the durability ratio holds

**Measurement.** Minimum nanoseconds over 128 rounds on fresh cache roots. Reproduction in brackets.

| Envelope | `ProcessCrash` | `Fsync` | ratio | bare `rename` of the same length |
| --- | --- | --- | --- | --- |
| 141,532 B | 709,542 [762,000] | 8,242,833 [8,244,000] | 11.6× [10.8×] | 52,167 [61,125] |
| 159,037 B | 744,958 [756,792] | 8,276,708 [8,184,916] | 11.1× [10.8×] | 58,750 [65,708] |

**Reproduced, and this is the sharpest test the claim has had.** `Fsync` is **8.18–8.28 ms** here and was 8.17–8.36 ms at the old band. Section 5 inferred from a 49% size increase that what `Fsync` buys is a fixed number of synchronization round-trips rather than durable bytes proportional to the entry; the same figure across envelopes **4.4× larger** is that inference tested at four times the range and passing. The multiplier is **10.8–11.6×**, still inside the 6.5×–18.7× band [ADR 0083](../../decisions/0083-keep-process-crash-as-the-default-cache-durability.md) measured, so that decision's shape is unchanged.

**Moved — the asymmetry between publication and a hit narrowed.** A default publication is **~4.5× a hit** and an `Fsync` publication **~52×**, against ~10× and ~150×. Publication itself grew only 31–37% for 4.4× the bytes; the hit grew 2.9×. The conclusion is unaffected — publication happens once per key and a hit on every subsequent expansion — but the gap it rests on is smaller.

### 9.6 What an eviction pass costs — unchanged, which is the point

**Measurement.** Minimum nanoseconds over 15 scans, `account`. Reproduction in brackets.

| Population | 141,532 B | 159,037 B | 2026-08-04, both sizes |
| --- | --- | --- | --- |
| 10 | 155,166 [159,542] | 156,042 [167,917] | 150,084 / 161,250 |
| 100 | 1,519,084 [1,519,875] | 1,508,833 [1,519,917] | 1,503,042 / 1,497,250 |
| 1,000 | 6,036,291 [6,600,458] | 6,453,458 [6,660,833] | 7,061,917 / 6,499,583 |
| 10,000 | 33,916,959 [32,693,166] | 32,262,000 [30,878,959] | 32,600,625 / 30,965,666 |

**Reproduced — the scan does not read bytes.** Every population is within roughly 7% of its 2026-08-04 value across a 4.4× increase in stored bytes, which is the observation behind "the scan reads metadata and never bytes" measured rather than restated. The marginal cost between 1,000 and 10,000 is **2.7–3.1 µs per entry** against 2.5–2.8 µs. The three collect variants agree within 5% at the small populations and within 15% at 10,000, so the selector is still not what to look at.

**Reproduced — removal is per key, not per byte.** A destructive collection of 10,000 entries costs 718.2–801.3 ms, reclaiming **1,418,628,890** and **1,593,678,890** bytes against 324.7 MB and 481.3 MB, at **71.8–80.1 µs per removed entry** against 73–86 µs. Three to five times the bytes at the same per-entry cost is the clearest possible statement that a removal is a non-blocking lock acquisition, a re-`stat`, an unlink, and a close.

**Moved — the ratio the eviction schedule is sized against.** A full-namespace scan of a 10,000-entry cache is **178–213× one hit**, against 460–590×. The scan did not get cheaper; the hit got more expensive. **Section 6's conclusion is unchanged and its number is not**: an eviction on every expansion would still turn a 159 µs hit into a 33 ms one, which is still absurd, and `wire-the-env-configured-eviction-policy-through-the-deliver-path` still has to amortize. What a re-reader must not do is carry the 460–590× figure forward.

**Inference — a consequence for a sibling record, not resolved here.** [The collection design](bounded-collection.md)'s 30-day default cites 32,136–47,803 bytes per envelope to project a 200–400 MB steady state. At 141,532–159,037 bytes the same entry count is roughly **0.9–1.6 GB**, and the same 200–400 MB is roughly 1,300–2,800 entries. The 30-day figure is a product choice under Tom's decision and is not re-decided from here; the arithmetic under it has moved, and [`re-price-the-envelope-band-consumers-against-the-re-derived-band`](../../../tickets/re-price-the-envelope-band-consumers-against-the-re-derived-band.md) owns the sites that quote the old band, including [`MaxEntryAge::DEFAULT`](../../../crates/tiler-cache/src/expansion/collect.rs)'s own ground.

### 9.7 The verdict, re-checked

**Inference.** *"Efficient at the measured scales"* remains **supported**, and every load-bearing part of Section 8 survives with one number replaced:

1. A hit is 159.3–172.0 µs and still does not degrade with cache size — the population minima span 0.16% and 0.07%, tighter than before, reproduced.
2. The closure a hit spares is unchanged at 4.5–4.7 ms, so the accelerator is **26–30×** cheaper than the work it replaces rather than 65–97×.
3. Nine tenths of a hit is contract-required integrity, and every remaining component is still at its floor: the read is syscall-bound at ~14 µs across 4.4× the bytes, keying and path work are under 0.3%, no lock is taken, and nothing is copied.
4. Publication is ~4.5× a hit and is paid once per key; the `Fsync` multiplier is unchanged.
5. Nothing on the hit path scales with the namespace.

**Inference — what the re-run actually found, which is not about the cache.** No inefficiency inside `tiler-cache` was located this time either, and none of its components regressed: the digest rate, the read cost, the key derivations, the lock, and the scan all reproduced. What moved is the size of the object the cache validates, and with it the cache's margin over the work it replaces. **The cost of a cache hit is now set almost entirely by how large an artifact envelope is**, and Section 9.1 says that three quarters of the recent growth is canonical manifest. If the hit path is ever worth attacking again, the lever is the encoding, not this crate.

**Measurement boundary.** Unchanged from Section 8, with the band restated: one host, one filesystem, one toolchain, one release profile, one envelope shape, two lengths — now 141,532 and 159,037 bytes — and four populations.

## 10. Outcomes

1. **Bounded experiment, preserved.** [`spikes/cache/hot-path-efficiency/`](../../../spikes/cache/hot-path-efficiency/README.md), with its controls, its stated boundary, and now **four** retained runs: the [2026-08-04 result](../../../spikes/cache/hot-path-efficiency/results/hot-path-efficiency-macos-27.0-2026-08-04.tsv) and [its reproduction](../../../spikes/cache/hot-path-efficiency/results/hot-path-efficiency-macos-27.0-2026-08-04-reproduction.tsv) at 32,136 and 47,803 bytes, and the [2026-08-06 result](../../../spikes/cache/hot-path-efficiency/results/hot-path-efficiency-macos-27.0-2026-08-06.tsv) and [its reproduction](../../../spikes/cache/hot-path-efficiency/results/hot-path-efficiency-macos-27.0-2026-08-06-reproduction.tsv) at the re-derived 141,532 and 159,037. The first pair is kept rather than replaced: a run is evidence at the commit it was taken at, and the 2026-08-04 pair is the only measurement of this cache at a 32 KB envelope.
2. **A fact a live ticket needs.** The scan and removal costs above size the amortization `wire-the-env-configured-eviction-policy-through-the-deliver-path` requires of itself. That ticket is in progress and its body is not edited from here; Section 6 is where the number lives, and **Section 9.6 is the one to read** — the scan cost is unchanged, and the scan-to-hit ratio it has to beat fell from 460–590× to 178–213×.
3. **One narrow question filed rather than acted on, and since answered.** [`decide-whether-the-bundle-envelope-section-digest-is-redundant`](../../../tickets/decide-whether-the-bundle-envelope-section-digest-is-redundant.md) carried the 19.4–24.0% measurement and the reasons the answer is not obvious — a digest and a re-encode comparison refuse *different* corruptions with *different* typed reasons, and the cache's rejection and quarantine classification is built on which one fired.

   **Answered 2026-08-05: the digest is retained on evidence.** [`spikes/cache/envelope-digest-coverage/`](../../../spikes/cache/envelope-digest-coverage/README.md) drove thirty-five named corruption classes and every byte position of a real 113,303-byte envelope under two perturbations — 226,606 decodes — through both the public hit path and `decode_artifact`, and re-ran the whole table against a build with the comparison genuinely removed. Every single-byte corruption is caught by the artifact decoder too. What only the bundle digest catches is whole-run substitution: replace the envelope span with a *different valid envelope* and `decode_artifact` accepts it, because the cache's key is a function of the compilation subject alone and the artifact decoder validates an envelope against itself — so with the digest gone the cache serves an artifact that was never published under that key, as a validated hit. **This note's cost figures are unchanged**; what moved is that the 19.4–24.0% is now a priced guarantee rather than an open question. The per-byte cost it buys is the same governed SHA-256 that [`decide-whether-the-canonicity-re-encode-is-redundant`](../../../tickets/decide-whether-the-canonicity-re-encode-is-redundant.md) found running at roughly a quarter of achievable speed, which is where the saving actually is.
4. **One defect found in a retained experiment.** [`spikes/cache/build-tool-exercise/envelope`](../../../spikes/cache/build-tool-exercise/) no longer compiles at this note's base commit; [`restore-the-cache-build-tool-exercise-against-the-current-artifact-api`](../../../tickets/restore-the-cache-build-tool-exercise-against-the-current-artifact-api.md) carries the two drifts and the re-run its README's evidence claim needs.
5. **One wording sharpened in place.** Section 4 records what [the collection design](bounded-collection.md)'s racing-a-reader position said about a copy, what the code actually does, why the position's conclusion is unaffected, and why the identical wording in `collect.rs` is deliberately left alone.
6. **Two catalog entries this branch's scopes cannot reach.** This record and the spike's record each need a line in a `contracts/navigation` catalog; [`catalog-the-cache-hot-path-efficiency-records`](../../../tickets/catalog-the-cache-hot-path-efficiency-records.md) carries the exact text of both.
7. **The band re-derived, and the sites that quote the old one filed.** Section 9 replaces the sweep's endpoints with a band re-derived from the producer that set the original, and records the fixed-overhead growth attributed at both ends. The corpus sites that quote 32,136–47,803 as a live figure and sit outside this branch's scopes — [`MaxEntryAge::DEFAULT`](../../../crates/tiler-cache/src/expansion/collect.rs)'s ground and `docs/open-questions.md`'s Q-ART-003 headroom, which now reads 15.17% of the per-invocation ceiling rather than 4.56% — are carried by [`re-price-the-envelope-band-consumers-against-the-re-derived-band`](../../../tickets/re-price-the-envelope-band-consumers-against-the-re-derived-band.md).

## Traceability

Answers the efficiency verification Tom asked for on 2026-08-04, recorded in [`decide-the-expansion-cache-collection-schedule`](../../../tickets/decide-the-expansion-cache-collection-schedule.md) and relayed in [the collection design](bounded-collection.md). Supplies the scan measurement that design's final section defers to this ticket. Confirms, on the public path and with the real validator, the lock-free read [ADR 0050](../../decisions/0050-use-immutable-self-validating-expansion-cache-entries.md) specifies and the durability ratio [ADR 0083](../../decisions/0083-keep-process-crash-as-the-default-cache-durability.md) measured. Lifts the stand-in validator boundary the in-crate harness at `crates/tiler-cache/src/expansion/hot_path.rs` states for its own numbers.
