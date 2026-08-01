---
schema: "tiler-doc/v1"
id: "tiler.research.cache.build-tool-exercise"
kind: "research"
title: "The expansion cache under Cargo and rust-analyzer"
topics: ["cache", "artifacts", "concurrency", "frontend"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "partial"
evidence_classes: ["bounded-measurement"]
informs: ["tiler.contract.artifact-abi", "tiler.contract.frontend-integration"]
depends_on: ["tiler.research.cache.crash-race-protocol"]
ticket: "exercise-the-expansion-cache-under-cargo-and-rust-analyzer"
---

# The expansion cache under Cargo and rust-analyzer

Status: the seventh follow-up gate of the [crash and race protocol note](crash-and-race-protocol.md). [ADR 0050](../../decisions/0050-use-immutable-self-validating-expansion-cache-entries.md)'s context sentence is that "Cargo and rust-analyzer may run equivalent proc-macro expansions concurrently", and that is the workload the whole protocol was designed against. Every measurement before this one used a harness that spawns its own workers, which is a *model* of that workload rather than the workload.

This note runs it. `spikes/cache/build-tool-exercise/` is a Cargo workspace whose proc macro resolves a real artifact envelope through the real public [`ExpansionCache`](../../../crates/tiler-cache/src/expansion/store.rs), and `spikes/cache/build_tool_exercise.py` drives it with real `cargo` processes and a real `rust-analyzer` proc-macro server.

## The headline, before the evidence

**Measurement.** The protocol behaves under both tools as the harness said it would. Across three genuinely overlapping `cargo` builds sharing one cache root, twelve expansions produced **four** compilations — one per key — and eight validated hits. Under an unusable cache root the same race produced **twelve** compilations, which is what makes the first number evidence rather than an artifact of a counter that never moves.

**Fact.** Two of the ticket's three questions had answers different from the shape the question assumed, and both are recorded in full below:

- **There is no default cache root.** The ticket asks "whether the default cache root is reachable and private in both contexts". `tiler-cache` has no such concept: `ExpansionCache::open` takes a root from its caller and the crate never consults the environment. Choosing a root is unowned work belonging to the frontend. **Correction — 2026-07-31:** the frontend crates now exist and the policy is now chosen. `tiler` and `tiler-macros` were admitted under [ADR 0088](../../decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md), and later the same day [the root policy note](root-policy.md) settled the derivation, its precedence, and its refusals, implemented in `crates/tiler-macros/src/cache_root.rs`. What is still true of `tiler-cache` is what this bullet reports: it has no default and deliberately acquires none. The measurement and its conclusion are unaffected.
- **`rust-analyzer` expansion is reachable on the pinned toolchain after all.** The full LSP binary is not a pinned component, but `rust-analyzer-proc-macro-srv` — the process that actually performs expansion for the editor — ships with the pin and was used here. This makes one open question's stated blocker stale; Section 8 names it exactly, and names the document it does *not* invalidate.

**Fact.** No case produced wrong bytes, consistent with the [supported-filesystem note](supported-filesystems.md)'s conclusion that no filesystem failure can make the cache serve a wrong artifact. That note's argument was not tested for the first time here; it simply was not contradicted.

## 1. What the two drivers actually do

**Measurement.** The process pattern the ticket asks for, observed rather than assumed. Each expansion records the executable performing it, its process identifier, and its wall-clock window.

| Driver | Process performing expansion | Invocations per process | Lifetime |
| --- | --- | --- | --- |
| `cargo` | `rustc` | every invocation in one crate | one crate compilation |
| `rust-analyzer` | `rust-analyzer-proc-macro-srv` | every invocation it is asked to expand | the editor session |

**Inference.** These differ in the way that matters to a cache. Cargo gives one short-lived process per crate, so expansions within a crate are sequential and concurrency arrives *between* crates and *between* builds. The analyzer gives one long-lived process holding the loaded proc-macro dylib across edits, so its expansions are sequential within the server but overlap freely with any Cargo build the developer starts. Neither coordinates with the other.

**Measurement, and not the answer that was expected.** They *do* share a working directory. Every expansion in the recorded run reports one — including `cargo-and-analyzer`, where a `rustc` process and the analyzer's server expanded concurrently — so the `cwds` column is `1` in every row. The guess going in was that they would differ, because the analyzer sends a `current_dir` with each expansion request rather than letting the server inherit the editor's, and a divergence would have meant a frontend deriving a root from the current directory silently splitting its cache in two. Measuring it is what settles it; three concurrent Cargo builds with *separate target directories* also share the one workspace root.

**Measurement.** They interoperate through the cache. In the `analyzer` scenario the server resolved all four keys as validated hits against entries a *Cargo* build had published, and in `cargo-and-analyzer` the two tools resolved one key set concurrently with one compilation between them.

### `CARGO_PKG_NAME` does not distinguish them

**Measurement, and a corrected assumption.** The obvious discriminator fails: `rust-analyzer` populates a proc-macro's environment from the crate graph it loaded, so `CARGO_PKG_NAME` is present during analyzer expansion too, and a macro reading it concludes "cargo" under both drivers. The spike's first version did exactly that and reported `driver=cargo` for an expansion running inside the proc-macro server.

`std::env::current_exe()` is the reliable signal, because the two drivers run the expansion in genuinely different programs. The [proc-macro build environment note](../macro-environment/proc-macro-build-environment.md) measured which variables are present during *Cargo* expansion; that its conclusions carry over to the analyzer is now measured for `CARGO_PKG_NAME` and unmeasured for the rest.

## 2. The per-key lock when the holder is killed

**Measurement.** Two scenarios kill a lock holder with `SIGKILL` to its process group, so no destructor runs, no descriptor is closed deliberately, and no buffer is flushed. `killed-writer` kills a Cargo build; `analyzer-killed-holding-lock` kills the proc-macro server, which is the case the ticket names — a server "that may be killed and restarted by its editor".

In both, a subsequent build took the same key's lock and resolved every key. **Inference:** the lock was released, because the alternative is observable — `ExpansionCache::resolve` blocks on `acquire`, so a leaked lock would have wedged the survivor until its deadline rather than letting it publish. That is the whole of ADR 0050's recovery story, which [`lock.rs`](../../../crates/tiler-cache/src/expansion/lock.rs) states has no stale-owner rule by design.

**Ordering is established by observed state, never by a wall-clock margin.** An expansion writes a marker file *before* it waits and removes it after; the driver kills only once that file exists. This deliberately does not repeat the defect `remove-the-wall-clock-race-from-the-cache-kill-harness` is fixing in the in-crate harness, where a 50 ms delay keeps an armed child alive and fails under load.

**Not reached:** an editor restarting the server and re-expanding, and cancellation through the analyzer's own mechanism rather than a signal. Section 5 states what each would need.

## 3. There is no default cache root

**Fact.** The ticket's third obligation presumes a default root. None exists.

- `ExpansionCache::open(root)` takes the root from its caller, performs no I/O, and creates no directory.
- The crate never consults the environment. The exact check, reproducible in one line: `grep -rn "env::var\|var_os\|home_dir\|dirs::\|std::env" crates/tiler-cache/src/ --include='*.rs'` matches only `expansion/harness.rs`, `expansion/fault.rs`, and `expansion/tests.rs`, all of which are `#[cfg(test)]`.

**Inference.** So "reachable and private in both contexts" has no subject to be asked about, and "what a sandboxed or CI environment overrides it to" has no default to override. What the question was reaching for is real and unowned: *something* must choose a root, and that chooser is the frontend proc-macro layer.

**Correction — 2026-07-31.** When this was written, the layer did not exist and `crates/tiler-macros/**` was a mapped path with no crate behind it. It has one: `tiler` and `tiler-macros` were admitted on 2026-07-31 under [ADR 0088](../../decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md), and `ticketsplease.toml`'s `implementation/frontend` scope now maps to the real package. The premise changed and the conclusion did not: `tiler-macros` opened no cache and named no root, and the chooser was unowned work that had a home rather than work that had been done.

**Second correction — 2026-07-31, later the same day.** The reproducible check this paragraph published — that `grep -rn 'ExpansionCache\|cache' crates/tiler-macros/src/` reports no match — is now false, and it is corrected here rather than deleted, because ADR 0088 was written precisely about a corpus that kept asserting an absence the tree had already falsified. `crates/tiler-macros/src/cache_root.rs` states the root policy. The check that is still true, and is the one worth publishing, is that the frontend *chooses* a root without being able to *open* one: `grep -n 'tiler-cache' crates/tiler-macros/Cargo.toml` reports no match, so the crate holds no edge to the cache at all. (A grep for `ExpansionCache` over the source is not that check — the new module names the type in prose, explaining whose contract its output has to satisfy.) [The root policy note](root-policy.md) records what was decided.

**Proposal.** The choice belongs with that layer, not with `tiler-cache`, and it must be made explicitly rather than defaulted into a home directory. Two constraints are already decided and bound it: `ExpansionCache::open`'s own contract requires a root "private to the user running Tiler", and this spike measured that a root derived from `CARGO_MANIFEST_DIR` is reachable under both drivers — which matters because the analyzer populates a proc-macro's environment from the crate graph, so a variable the editor's own process environment carries does not necessarily arrive. `add-an-expansion-cache-root-preflight` owns the validation of whatever root is chosen and is `done`; `choose-the-expansion-cache-root-policy` owns choosing one and was filed on 2026-07-31 when the layer landed. This note does not decide the policy, and records that it is undecided rather than inventing one.

**Correction — 2026-07-31. The policy is now decided, and this paragraph's second clause was overtaken by an accepted contract it did not cite.** [The root policy note](root-policy.md) settles it: `TILER_EXPANSION_CACHE_DIR` when set, otherwise `$HOME/Library/Caches/ai.moderately.tiler/expansion`, with `off` as an explicit disable and a typed refusal for every unusable or non-private root. Two parts of the paragraph above need reading with that record beside them.

*"Rather than defaulted into a home directory" is not the accepted contract, and was not when this was written.* [`docs/integration/frontends.md`](../../integration/frontends.md) and [`docs/backends/metal.md`](../../backends/metal.md) are both `contract_status: accepted` and both state that the default *is* an OS-appropriate user cache with a CI/sandbox override, rather than consumer `OUT_DIR`. The sentence above is a **Proposal** and does not outrank them; what it protects survives intact, because the objectionable thing is a root that arrives unstated rather than one that arrives from `$HOME`. The chosen policy names every input, documents the derivation, and refuses rather than substituting.

*The `CARGO_MANIFEST_DIR` measurement is correct and was not the binding constraint.* It is eliminated on privacy — a checkout directory is not private to one user — and on sharing: it names the consumer *package* directory, so a root derived from it confines cache sharing to one package when `frontends.md` requires that identical invocations share compiler work across processes. The full elimination, including four further alternatives, is in the root policy note.

## 4. Measurement

**Measurement.** macOS 27.0, arm64, 14 logical cores. `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, the `rust-toolchain.toml` pin `nightly-2026-07-19`. `rust-analyzer 1.97.0-nightly (8b03437a 2026-05-12)` from the rolling `nightly` toolchain, driving the **pin's** `rust-analyzer-proc-macro-srv`. Four `resolve!` invocations per build over four distinct keys; concurrency three.

```sh
python3 spikes/cache/build_tool_exercise.py --concurrency 3 \
  --analyzer "$(rustup which --toolchain nightly rust-analyzer)" \
  --record macos-27.0-2026-07-25
```

Recorded in [`spikes/cache/results/build-tool-exercise-macos-27.0-2026-07-25.tsv`](../../../spikes/cache/results/build-tool-exercise-macos-27.0-2026-07-25.tsv). `builds` counts how many times the expensive closure ran; `overlaps` counts pairs of expansion windows in *different* processes that genuinely intersect; `cwds` counts the distinct working directories the expansions ran in.

It is an observation about one host and one pair of tool versions, not a portable guarantee.

### Why the analyzer version is not the pin, and what that costs

**Fact.** `rust-toolchain.toml` declares `profile = "minimal"` with components `clippy` and `rustfmt`, so the pin carries no `rust-analyzer` binary; `rustup which --toolchain nightly-2026-07-19 rust-analyzer` fails. Installing it would be a toolchain mutation this work is not authorized to make.

**Fact.** The pin *does* ship `libexec/rust-analyzer-proc-macro-srv`, and that is the process that performs expansion. The analyzer's `--proc-macro-srv` flag points it at exactly that binary, so the process loading the proc-macro dylib and running this crate's code is the pinned one; only the LSP front half is off-pin.

**Measurement.** Every installed toolchain from 1.89.0 onward reports proc-macro server API version 6, including the pin, which is why an off-pin analyzer can drive the pinned server at all. The two toolchains that ship a full `rust-analyzer` here are 1.91.0 and 1.94.0 alongside rolling `nightly`.

**Inference.** The residual exposure is the LSP half's behaviour, not the expansion half's: request scheduling and cancellation policy are the analyzer's, while everything this note measures about locking, publication, and validation happens inside the pinned server.

## 5. How this experiment could have failed

A uniform pass over a heterogeneous population is the signature to distrust, so each control is named with the population it covers.

**The population is counted.** Every scenario declares the exact number of expansion events it must observe — `INVOCATIONS` per build, times the number of builds — and fails when the count differs. Events are one file per expansion rather than appended lines, because several uncoordinated processes write at once and an interleaved append can drop a record with no reader noticing. The driver also refuses to run if its `INVOCATIONS` and the consumer crate's declared constant have drifted apart.

**Concurrency is observed, not assumed.** Each event carries its wall-clock window and the concurrent scenarios fail unless windows in different processes intersect. **This check caught a real gap:** the first run reported the correct outcome counts with `overlaps = 0`, because the compile is a few milliseconds wide and three "concurrent" builds had serialized. The scenarios now widen the build window until overlap is a fact; the recorded run shows 21 overlapping pairs in each of the two three-way scenarios and 7 across the two build tools. **It caught a second gap on the analyzer side:** the interleaved scenario released the analyzer's lock as soon as Cargo was *launched*, and Cargo had not yet reached its expansion, so the run failed rather than reporting a concurrency it never achieved.

**The negative control moves the metric.** `negative-control-x3` runs the identical race with the cache root pointed at a *file*, so no namespace can be created. It produces twelve compilations where `cargo-concurrent-x3` produces four, and its own overlap count confirms it raced. Without it, "one compile per key" would be consistent with a counter that never increments.

**Three checks failed during development and were fixed rather than relaxed:** the driver-name heuristic (Section 1), the missing overlap evidence above, and an `unquoted` shell glob that silently matched nothing while appearing to search the crate.

**What this does not establish.** Every scenario passing says the protocol held for these populations on this host. It is not evidence for an unmeasured filesystem, a different host, a real editor session, or a load level above concurrency three.

## 6. What was not reached, and why

| Not reached | Reason | What it would need |
| --- | --- | --- |
| A real LSP session with incremental edits | Driving `rust-analyzer` as a language server needs an LSP client; `analysis-stats` loads the project and expands once | An LSP client harness, or an editor driven by script |
| Analyzer-initiated cancellation | `analysis-stats` exposes no cancellation; the kill scenarios use `SIGKILL` instead | The LSP client above, cancelling a request mid-expansion |
| Server restart and re-expansion after a kill | The scenario measures that the *lock* is released, not that the analyzer restarts its server | The LSP client above |
| Concurrency above three | Bounded deliberately to keep the recorded run short | A larger `--concurrency`; the driver takes it as an argument |
| A carried compiled payload | The envelope declares its payload by descriptor rather than carrying object bytes | **Closed 2026-07-31 by [the self-contained embedding note](../embedding/self-contained-embedding.md)**: envelopes from `prototypes/serial-sum-compile` carrying compiled `metallib` objects of 3,491–7,158 bytes travelled through the public `get_or_publish`, with every hit validated by the real `decode_artifact`. The row is kept because the gap was real when this note ran, and what it protected — that a descriptor-only envelope is not evidence about carried bytes — is exactly what the closing evidence had to supply |
| Linux | One macOS host was available | `probe-the-expansion-cache-filesystem-properties-on-linux` |

**The end-to-end gap ADR 0050 records is now narrower but not closed.** That record says "a positive end-to-end hit carrying a real compiled artifact remains the orchestrator's". This spike *is* an orchestrator holding both crates, and it measured positive hits through the public `get_or_publish`, whose validator is the real `decode_artifact`: the envelope is produced by a genuine `tiler-compiler` session, encoded by `tiler-artifact`, published, re-read, and completely validated. What is still absent is narrower and should be stated as such — the payload is *declared* by descriptor rather than *carried*, so no compiled backend object has travelled through a cache entry. **Correction, 2026-07-31: that last sentence is no longer true of the corpus.** [The self-contained embedding note](../embedding/self-contained-embedding.md) put envelopes carrying genuine compiled `metallib` objects through `get_or_publish` and validated every hit with the real `decode_artifact`; the carried-payload half of the gap is closed there, on that note's measured host and boundary. The paragraph above is retained as this note's own measurement boundary rather than rewritten, because its rows never carried object bytes and still do not.

**Two facets of the composed subject are stand-ins**, and necessarily so: no producer exists for `SubjectFacet::ArtifactProgram` until `derive-the-pre-compilation-artifact-program-subject` lands, which `subject.rs` records as a deliberate loud stop. This spike measures process behaviour, for which a subject only has to be a stable function of the invocation; it is not evidence about identity completeness, and `bind-the-cache-subject-to-the-carried-payload-provenance` owns that.

## 7. An input Cargo does not track

**Measurement.** Cargo does not know an expansion consulted a cache, so nothing about a changed cache root invalidates a crate's fingerprint. Forcing re-expansion in these scenarios requires touching a source file; changing `TILER_EXERCISE_ROOT` alone does not.

**Inference.** This is benign for correctness under the current design and worth recording anyway. A resolution is a pure function of the composed subject, so a stale *cargo* fingerprint elides an expansion whose result would have been identical. It stops being benign if a future frontend lets anything outside the subject influence generated code — which is the same rebuild-boundary hazard the [proc-macro build environment note](../macro-environment/proc-macro-build-environment.md) records for a Metal toolchain change, arriving by a second route.

## 8. Outcomes

1. **Contract update.** The spike README now states what is measured under the two build tools and what is not, and replaces its "a positive end-to-end hit carrying a real compiled artifact is still unmeasured" paragraph with the narrower gap that actually remains.

   **ADR 0050 needs the same correction and does not get it here.** Its traceability paragraph still says such a hit "remains the orchestrator's", which this note supersedes in part. Editing it needs the `contracts/decisions` scope, which an unrelated ticket held while this work ran, so `correct-adr-0050-end-to-end-hit-status` carries it rather than this branch taking a scope it would contend for. The claim is superseded by evidence either way; what is deferred is only where the record says so.
2. **Contract update.** [`docs/open-questions.md`](../../open-questions.md)'s Q-ART-006 says it closes "when a real rust-analyzer binary, rather than only the rustup proxy, is available". That blocker is stale twice over: analyzer binaries are installed on this host, and the expansion half is reachable on the pin itself. Corrected there.

   **The macro-environment note is *not* wrong and is left alone.** Its "rust-analyzer component: not installed" line is part of a recorded environment for the 2026-07-20 measurement, not a claim that measurement is impossible, and a recorded environment must keep saying what was true when it was recorded. Reading it before editing is what distinguished the two cases.
3. **Bounded experiment, preserved.** `spikes/cache/build-tool-exercise/` and `spikes/cache/build_tool_exercise.py`, with stop conditions, declared populations, and a negative control, recorded in `spikes/cache/results/`.
4. **Deferred, with triggers.**
   - *Which root a frontend chooses.* **No longer deferred — decided on 2026-07-31, the same day the trigger fired.** The trigger was "the first proc-macro frontend crate", on the ground that nothing could decide it earlier because there was no caller to own the choice; `tiler-macros` is that crate, admitted under [ADR 0088](../../decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md). [The root policy note](root-policy.md) records the decision, its seven eliminated alternatives, its measurement boundary, and its unsupported cases, and the resolver is implemented as a crate-private draft in `crates/tiler-macros/src/cache_root.rs`. The two constraints stated in Section 3 both held and both did work: the privacy requirement is what eliminated the `CARGO_MANIFEST_DIR` derivation this note measured reachable, and reachability under both drivers is what the two-variable policy preserves structurally rather than by convention.

     **What remains open is narrower, and is not this item.** The consumer-visible spellings — the variable name, the disable value, the derived path, the refusal text — are unaccepted and are Tom's under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md); [`record-the-expansion-cache-root-policy-decision`](../../../tickets/record-the-expansion-cache-root-policy-decision.md) carries the ADR and the contract propagation. [Q-ART-004](../../open-questions.md#q-art-004--expansion-cache-root-accounting-and-gc-policy)'s accounting and GC half is untouched and stays with [`decide-the-expansion-cache-collection-schedule`](../../../tickets/decide-the-expansion-cache-collection-schedule.md).
   - *A real LSP session, cancellation, and server restart.* Trigger: an LSP client harness, or a decision that editor-side scheduling is worth its own gate. Nothing measured here depends on the answer.
   - *Collecting this driver into the repository gate.* **Not a gap under the current contract.** No `make` target touches `spikes/` — a spike is a recorded measurement whose value is its record, run from its own directory when someone is working on it — so this driver runs ad hoc as `spikes/README.md` describes. Trigger for revisiting: a decision to give the gate a driven-build scenario, which would be a change to that stated contract rather than an addition under it. (This item originally cited the Python gate scripts; they were retired for the Makefile of cargo commands while this work was stranded, and the conclusion survives the translation.)
