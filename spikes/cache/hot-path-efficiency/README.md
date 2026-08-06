---
schema: "tiler-doc/v1"
id: "tiler.spike.cache.hot-path-efficiency"
kind: "experiment"
title: "Expansion cache hot-path efficiency probe"
topics: ["cache", "performance", "measurement"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.cache.hot-path-efficiency"]
entrypoints: ["spikes/cache/hot-path-efficiency/harness/src/main.rs", "spikes/cache/hot-path-efficiency/harness/src/envelope.rs"]
last_verified: "2026-08-06"
ticket: "measure-the-expansion-cache-hot-path-efficiency"
---

# Expansion cache hot-path efficiency probe

This harness measures what the expansion cache costs on its hot paths, through the **public** [`ExpansionCache`](../../../crates/tiler-cache/src/expansion/store.rs), with every hit validated by the real [`decode_artifact`](../../../crates/tiler-artifact/src/program/codec/decode.rs).

```sh
cd spikes/cache/hot-path-efficiency
cargo build --release
./target/release/cache-hot-path-efficiency \
  --repeats 8000 --scan-repeats 15 --publish-rounds 128 --cold-children 48 \
  --record macos-27.0-2026-08-06
```

Nothing runs it automatically; no `make` target reaches `spikes/`. Run it from this directory, which is where `--record` resolves `results/` from. `cargo run --release -- <args>` is the same binary with the same arguments and works identically; the two-step form is recorded because it is what produced the retained files, and a recorded invocation is a claim about what produced a file. `--quick` shortens every dimension for development and produces a result nobody should record.

**The four counts are above the defaults deliberately, and the reason is the host.** This machine runs several agents' `cargo` builds concurrently. The estimator is the minimum, so more samples do not average the interference away — they give the minimum more chances to land in a window where the interference was not happening. Raising the counts is the only lever that improves a floor estimate under load, and it costs seconds: the 2026-08-04 runs take 22–23 s each and the 2026-08-06 runs, at envelopes 4.4× larger, take 35.5 s each. The defaults are what a quiet host needs.

**Four results are retained, as two back-to-back pairs at two commits.** At `194744e6` and 32,136 / 47,803 bytes: the [2026-08-04 result](results/hot-path-efficiency-macos-27.0-2026-08-04.tsv) and its [reproduction](results/hot-path-efficiency-macos-27.0-2026-08-04-reproduction.tsv). At `8bd720b8` and the re-derived 141,532 / 159,037 bytes: the [2026-08-06 result](results/hot-path-efficiency-macos-27.0-2026-08-06.tsv) and its [reproduction](results/hot-path-efficiency-macos-27.0-2026-08-06-reproduction.tsv). The 2026-08-04 pair is **kept rather than replaced**, because a run is evidence at the commit it was taken at and that pair is the only measurement of this cache at a 32 KB envelope. One run cannot distinguish a cost from an interruption, and this harness is deliberately run on a shared machine, so the second run of each pair is what says which of the first's figures are measurements. What reproduced and what moved is stated in [the research note's procedure section](../../../docs/research/cache/hot-path-efficiency.md#1-environment-and-procedure) for the first pair and in [its Section 9](../../../docs/research/cache/hot-path-efficiency.md#9-the-re-run-at-the-re-derived-band-2026-08-06) for the second.

## What it lifts, and from what

`crates/tiler-cache/src/expansion/hot_path.rs` measures the same protocol with a payload validator that does nothing, and its module documentation says why: building a real artifact envelope needs a `SemanticProgram`, and [ADR 0082](../../../docs/decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md) item 2 decides `tiler-cache` does not depend on `tiler-ir`. That harness therefore offers every share it reports as an **upper bound** and explicitly declines to say which step dominates a real hit.

This spike is the orchestrator that lifts the substitution. It is a separate workspace holding `tiler-ir`, `tiler-compiler`, `tiler-artifact`, and `tiler-cache` by path, so it can compile a governed program, assemble a real artifact, publish it through `get_or_publish`, and read it back through `lookup` — the same route [`tiler-macros`](../../../crates/tiler-macros/src/aot.rs) takes.

## What is measured

| Section | What the row reports |
| --- | --- |
| `oracle` | The returned bytes equal the published bytes; one flipped byte is refused; restoring it restores the hit |
| `publish` | `get_or_publish` on a fresh key under each durability policy, and a bare `rename` for attribution |
| `hit-warm` | Repeated `lookup` in one long-lived process — the `rust-analyzer` server pattern |
| `hit-cold` | One `lookup` in a freshly executed process — the `cargo`/`rustc` pattern |
| `scan` | `account`, `collect` under `UNBOUNDED`, `collect` under an age nothing reached, and `collect` under an age everything reached |
| `decompose` | One hit attributed to open-and-read, bundle section digests, key derivations, path work, and `decode_artifact`, with the residual |
| `lock` | Uncontended acquire-and-release, and a hit served while another process holds that key's lock |

Envelope lengths are **141,532 and 159,037 bytes**: the endpoints of the band [the self-contained embedding note](../../../docs/research/embedding/self-contained-embedding.md) measured, re-derived at this spike's base commit on 2026-08-06 by re-running the producer that set it and taking the minimum and maximum of the envelope lengths of the same six members. A cost then lands against sizes the corpus already calls realistic rather than against a round number nothing produced. The derivation, and why the producer's two newer contraction members at 89,250 and 90,737 bytes are unreachable from here, is stated at [`SIZES`](harness/src/main.rs). The lengths this spike swept until 2026-08-05 were 32,136 and 47,803, which is what the two 2026-08-04 results measure. Populations are 10, 100, 1,000, and 10,000 entries, filled cumulatively into one root per size — at these lengths that is 1.42 GB and 1.59 GB of scratch under `$TMPDIR` at the largest population, against 325 MB and 481 MB before.

## How a pass could have been vacuous, and what prevents it

Each control below is a check that can fail, and the first two are checks that *were watched failing*.

**The oracle is proven able to refuse.** Every measured configuration compares the returned envelope against the exact published bytes. On its own that is consistent with a comparison that never runs, so the run flips a single byte of the stored entry, observes the read refuse it — `cache bundle section 'artifact-envelope' does not match its declared digest` — restores the byte, and observes the hit return. All three observations are rows in the result.

**The contended-lock control is proven held.** "A hit was served while another process held the lock" would prove nothing if no lock were held. A child process takes the key lock and creates its ready marker *after* holding it; the parent then attempts a non-blocking acquisition and requires it to be **refused** before timing a single hit. Ordering comes from that marker's existence, never from a wall-clock margin.

**The population is counted from the namespace.** Every row's population is read back with `account` and asserted against the fill, so a fill that lost an entry fails instead of labelling a row with a population it never reached. The destructive collection asserts that the removals equal the counted population and that the report accounts for its whole selection.

**The frame restatement is checked against the bytes.** The decomposition cannot call `read_entry`, `bundle::decode`, or the entry-path parser — all crate-private — so it reproduces each from the public digest, the public key derivation, and `std`. `bundle_spans` asserts that the spans it derives from the restated frame constants actually delimit the published subject and envelope, so a framing change in `tiler-cache` fails this spike loudly rather than being digested around. The `residual` row reports what the components do not account for; it is 0.3–3.3% of the whole hit across the 2026-08-04 runs and 1.0–1.3% across the 2026-08-06 ones, which is what makes the attribution believable — and the second pair is the check that the restatement still holds at `MANIFEST_SCHEMA` `14.0`.

**Nothing asserts a time.** A timing assertion passes on a fast host and fails on a loaded one, which makes it a flake rather than a guard. The minimum is the estimator — every host perturbation makes an operation slower and none makes it faster — and the median, ninetieth percentile, and maximum sit beside it so a reader can see the load rather than trust an assertion about it.

**A waiting process sleeps rather than spins, and that is a measurement decision.** The contended-hit row read 5–10% above the uncontended one until the lock-holding child stopped busy-waiting on its release marker: that child is alive for the whole of the measurement, so a spin loop was a second process competing for a core and for the filesystem with the thing being timed. Ordering still comes from the marker's existence and from nothing else; the sleep is only how often the waiter asks.

## Measurement boundary

**One host, one toolchain, one filesystem.** Apple M4 Max, 14 logical cores, macOS 27.0 (Darwin 27.0.0), APFS under `$TMPDIR`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` — the `rust-toolchain.toml` pin `nightly-2026-07-19`, resolved by directory ancestry with no selector. Release profile. Nothing here is a portable guarantee.

**The host is shared, and the rows say so.** Several agents' `cargo` builds run on this machine concurrently; load average was `3.99 10.15 14.42` at the start of the first 2026-08-04 run and `4.30 9.35 13.90` at the end of its reproduction, and `6.71 6.09 8.22` and `5.59 5.93 7.99` at the corresponding points of the 2026-08-06 pair. That is why the minimum is the reported estimator, why the maximum is printed beside it — the hit rows' maxima reach 4–6× their minima in these runs — and why the sample counts are raised. Isolation is by dedicated scratch root, a directory named by process identifier and nanosecond under `$TMPDIR` and removed when the run ends, so no concurrent process shares a mutable path with it.

**"Cold" means cold in process state, not in storage.** A `hit-cold` sample is a freshly executed process with no warmed allocator and no prior lookup, reading an entry the parent published moments earlier on the same host, so the unified buffer cache is warm. Purging it needs privileges this spike does not take; a genuinely cold-storage first hit is unmeasured.

**The object bytes are synthetic and the artifact-program subject facet is a stand-in.** The carried object travels opaquely — artifact identity folds the payload *metadata* and excludes every object byte — so the artifact layer performs identical work on `n` synthetic bytes and `n` bytes of `metallib`, and synthetic bytes are what let an envelope be produced at an exact length. This spike is therefore not evidence about a real Metal compilation, and not evidence about identity completeness; `spikes/embedding/self-contained/` is where envelopes carrying genuine compiled objects travel through `get_or_publish`, and `bind-the-cache-subject-to-the-carried-payload-provenance` owns the subject gap.

**Every population entry carries the same envelope under a different subject.** The cache does not tie a subject to the envelope it files (a documented gap, not one this spike introduces), so one compile fills a 10,000-entry root. Nothing measured depends on the envelopes differing: the scan reads metadata and never bytes, and the probe entry is read on its own key path.

**Two points do not make a curve.** Where this spike reports a per-byte slope, it is a fit through two envelope lengths. It is enough to separate a per-byte cost from a per-read one, which is what it is used for, and it is not a basis for extrapolating to an envelope an order of magnitude larger.

**Two rows have few samples and are published as ranges rather than values.** The cold-process hit takes 48 samples, one per child process, and the destructive collection takes exactly one per root. Both vary run to run by more than the rows with thousands of samples; the research note quotes them as ranges and says so.

## Re-run — the harness was repaired, the sweep stopped starting, and the band was re-derived

**Fact.** `wire-the-delivered-realization-record-into-the-artifact` made every executable artifact carry a delivered-realization record, so `harness/src/envelope.rs` stopped *producing* an artifact while continuing to compile: `ArtifactProgramBuilder::build` refuses a draft that never called `declare_realization`. That was repaired in that ticket's commit, by one `declare_realization` call and one `realization_record` helper deriving the eleven governed resolutions from the packaged program's own scheduled realization.

**Measurement, and the verdict was a refusal rather than a result.** Re-run on that ticket's tree based at `55d1d09f`, `--quick`, same host and toolchain: the harness compiled the program, assembled the artifact, and then aborted on its own precondition — `an envelope cannot be smaller than its 114025 byte fixed overhead; asked for 32136`. `SIZES` was then the envelope band `docs/research/embedding/self-contained-embedding.md` measured, 32,136 to 47,803 bytes, and the fixed overhead of one artifact envelope had risen above both endpoints. No row was produced and no fixture was recorded.

**That precondition was already unsatisfiable before that ticket, and the arithmetic says so.** The retained 2026-08-04 runs record a fixed overhead of **28,527** bytes. The record contributes 2,453 canonical bytes to this envelope — measured directly, by printing `canonical_bytes().len()` from the repaired harness — and the envelope carries it twice, once inside the folded artifact identity the manifest also carries and once as its own framed run, so roughly 4.9 KB of the 114,025. The remainder accumulated between 2026-08-04 and `55d1d09f`, an interval in which `spikes/target-profiles/scalar-cpu-vertical` independently records its own envelope growing from 21,296 to 82,918 bytes with no delivered-realization record anywhere in sight. So the band and the overhead crossed before that ticket touched anything, and its contribution was to make that visible rather than to cause it.

**Resolved 2026-08-06 at `8bd720b8`, by re-deriving rather than by rounding.** [`re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps`](../../../tickets/re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps.md) re-ran `prototypes/serial-sum-compile`, the producer whose members set the original band, and took the minimum and maximum of the same six members' envelope lengths: **141,532 to 159,037 bytes**. Picking larger round numbers here would have replaced evidence with convenience, which is why it was not done. Two properties of that re-derivation are worth carrying:

- The producer's six `metallib` counts are **byte-identical** to the ones recorded on 2026-07-31, so none of the growth is backend output — it is entirely artifact encoding.
- This harness's own fixed overhead is **114,043** bytes at this commit against 28,527 at `194744e6`, and the difference is attributed exactly: +65,363 bytes of canonical manifest, +20,153 bytes of `KernelProgramSubject` section, and a `BackendPayloadMetadata` section that did not move a byte. [The research note's Section 9.1](../../../docs/research/cache/hot-path-efficiency.md#91-why-the-envelope-moved-measured-at-both-ends) carries the table.

The sweep runs end to end again, and the four retained results are two pairs at two commits. What reproduced — the digest rate to three digits, the syscall-bound read across 4.4× the bytes, the flat `Fsync` cost, the per-entry scan and removal costs — and what moved — the hit level, the validation share, and the cache's margin over the build closure — is [Section 9](../../../docs/research/cache/hot-path-efficiency.md#9-the-re-run-at-the-re-derived-band-2026-08-06).

See [the research result](../../../docs/research/cache/hot-path-efficiency.md) for what the numbers mean.
