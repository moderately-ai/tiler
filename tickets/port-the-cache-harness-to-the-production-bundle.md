---
id: port-the-cache-harness-to-the-production-bundle
title: Port the cache harness to the production bundle
status: done
priority: p1
dependencies: []
related: [implement-the-expansion-cache-protocol, cache-crash-race-harness]
scopes: [research/cache, implementation/cache, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [cache, concurrency, durability, testing]
---
`spikes/cache/cache_harness.rs` kills real processes at nine publication phases and is the only evidence Tiler has for the cross-process crash and race behaviour ADR 0050 decides. It exercises **its own miniature frame**, not the bundle `tiler-cache` publishes.

`tiler-cache`'s in-crate suite is threaded, and it says so rather than implying otherwise: a thread that returns unwinds, closes its own descriptors, and never leaves a half-written file with no owner, so it is not evidence for a killed-process property and is not offered as one. **The production bundle's crash and race behaviour is currently unmeasured.**

## What this ticket owes

- Re-point the harness at `tiler_cache::expansion`, so the nine kill points exercise the real namespace, the real lock adapter, the real `create_new` temporary, the real separate-descriptor validation, and the real rename.
- Keep every case the spike already covers: concurrent identical keys producing one compilation, concurrent distinct keys, recovery at each kill point, truncated and digest-corrupt finals, entry and whole-cache deletion, active recursive deletion, an unusable root, and a reader holding an open descriptor across eviction.
- Record the exact host, toolchain, and repetition counts as a measurement, in the form `spikes/cache/results/` already uses. It is an observation about a host, not a portable guarantee.
- The harness needs a real artifact envelope to publish, which needs a semantic program; decide whether it builds one or whether this waits on a fixture the orchestrator can supply.

This is the process half of the research note's second follow-up gate. The envelope-integration half landed with `implement-the-expansion-cache-protocol`: the bundle carries a real artifact envelope and validates it through `decode_artifact` on every hit.

## Outcome

The nine kill points now run against the bundle `tiler-cache` publishes. `crates/tiler-cache/src/expansion/harness.rs` is the harness and `crates/tiler-cache/src/expansion/fault.rs` is the seam it drives.

### Where the harness lives, and why not beside the spike

Re-pointing the existing standalone binary at `tiler_cache::expansion` was eliminated, not skipped. Reaching a phase inside `ExpansionCache::publish` needs a seam, and the two phases that matter most — a temporary half written, and one written and validated but not yet renamed — are interior states no external observer can schedule. The candidates:

- **A Cargo feature the spike binary enables.** Eliminated on correctness, not cost: features are public surface on a boundary Tom has not accepted, and Cargo unifies them across a build graph, so one unrelated crate enabling it would arm mid-publication aborts inside somebody's production cache. That is a defect with an opt-in spelling.
- **An environment variable the crate reads unconditionally.** Strictly worse — ambient configuration that arms aborts in a shipped build.
- **An external supervisor killing by watching the filesystem.** Cannot deterministically hit `mid-write` or `after-temp-validation`. It would report nine phases having measured fewer, which is the failure mode of a test that looks like evidence.
- **A `cfg(test)` seam, with the harness re-executing the crate's own test binary.** Survives: no public surface, no feature, no dependency, deterministic phases, and the child is a genuine process running the real `ExpansionCache`.

One survivor, so there was no question to escalate. `spikes/cache/cache_harness.rs` is preserved unchanged as the miniature-frame evidence it already was.

### What is exercised

Children are real processes: `Command` re-executes the test binary selecting a child entry point, and the armed child calls `process::abort` — not `exit`, so no destructor runs, no descriptor closes deliberately, and no buffer flushes. The parent then observes only the filesystem, through `ExpansionCache::read_entry`, never anything the dead process reported.

Sixteen cases, covering every case the spike had: recovery at each of the nine phases under both durability policies; that no entry exists at a content path before the rename and does after it; that a killed writer's lock is released with no recovery rule; that nothing extra is left in an entry shard; concurrent identical keys producing exactly one compilation; concurrent distinct keys; a race against a dying writer; truncated and digest-corrupt finals; entry and whole-cache deletion; recursive deletion racing live writers; an unusable root falling open; and a reader holding an open descriptor across eviction.

Three of them exist to stop the others passing for the wrong reason. `an_unarmed_child_completes_and_publishes` proves a dying child means the phase was reached rather than the child being broken. `no_entry_exists_at_a_content_path_before_the_rename` proves the phases are distinct — a fixed abort point would fail it. `every_phase_name_round_trips` proves a phase name that crosses the process boundary as text still arms something.

`a_stuck_child_is_killed_at_its_deadline` drives `wait_bounded` to an actual `Death::TimedOut` with a short deadline, because a timeout path nothing reaches is a guess. That found and fixed a real defect in this work: the case originally asserted a weaker property than its comment claimed.

### Measurement

**Measurement.** macOS 27.0 build 26A5388g, arm64, 14 logical cores; rustc 1.99.0-nightly (eff8269f7 2026-07-18), the `rust-toolchain.toml` pin. `TILER_CACHE_HARNESS_REPETITIONS=10 TILER_CACHE_HARNESS_CONCURRENCY=32 ... cargo nextest run -p tiler-cache -E 'test(expansion::harness)' --test-threads 1`. 16 cases passed, 1720 child processes, 15.5 s. Recorded in `spikes/cache/results/production-bundle-macos-27.0-rustc-1.99.0-nightly-2026-07-19.tsv`, in the schema/columns/run-rows form the existing result uses. It is an observation about one host, not a portable guarantee.

The gate runs the same cases at one repetition and four-way concurrency, in well under a second.

### The envelope question this ticket left open, decided

The ticket asked whether the harness builds a real artifact envelope or waits for a fixture. **Neither.** It drives the crate-private `resolve` with a stand-in payload validator, and the reason is a consequence of ADR 0082 item 2 rather than convenience: a real envelope needs a `SemanticProgram`, which needs `tiler-ir`, which that record decides `tiler-cache` does not depend on. The two ways to get one were a `tiler-ir` dev-dependency — which changes a *decided* property and a pin in `scripts/check_workspace.py`, so it is Tom's — and a checked-in envelope fixture plus a generator spike, which needs a new excluded Cargo workspace and leaves an encoder-produced golden the gate cannot keep honest.

The substitution does not reach these properties, and the argument is stated wherever the claim is: every byte of the bundle frame is real — encoder, header, section digests, embedded key, re-derivation from the carried subject — and every filesystem operation is real. The payload validator sits strictly *inside* an envelope the frame has already delimited, so which validator runs changes how long the pre-rename window is and whether it can fail, and changes nothing about what a killed writer leaves at a content path.

**So a positive end-to-end hit carrying a real compiled artifact is still unmeasured**, and it is recorded as the orchestrator's in ADR 0050, ADR 0082, the research note, and the spike README rather than quietly folded into this result.

### Also

The harness composes a real `ComposedSubject` with stand-in facet bytes, which is all a crash property needs — it is a fact about files, not about identity. ADR 0050's `implementation_status` note, ADR 0082's consequences, the crash/race research note's second follow-up gate, `expansion`'s and `tests`'s module documentation, and the spike README and its frontmatter all now say what is measured and what is not.
