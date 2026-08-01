---
id: prototype-macro-embedding-and-cargo-behavior
title: Measure macro embedding and Cargo behavior
status: in-progress
priority: p1
dependencies: [prototype-inline-proc-macro-frontend, implement-the-expansion-cache-protocol, compose-the-complete-expansion-cache-subject, prototype-artifact-family-delivery, prototype-metal-aot-slice]
related: [repair-macro-and-embedding-harness-integrity]
scopes: [implementation/frontend, research/embedding, research/macro-environment]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [implementation, measurement, proc-macro, inline-dx]
assignee: worker-embedding
lease_expires_at: 1785545860
---
## User-visible outcome

We know — as recorded measurements with exact environments, not as belief — that byte-literal embedding is genuinely self-contained (a consumer builds and runs with every Tiler file deleted), and how Cargo and rust-analyzer behave cold/warm across edits, toolchain changes, and repeated/unique artifacts. This is the evidence the inline-macro frontend decision consumes.

Prove direct byte-literal embedding is self-contained and cache deletion cannot break expanded code. Measure Cargo/rust-analyzer cold and warm behavior, edits and toolchain changes, repeated/unique artifacts across crates, and bounded sizes with exact environments and explicit diagnostic/size gates.

## Closes when (2026-07-28)

1. **Self-contained byte-literal embedding is demonstrated, not argued.** An expanded invocation carries its artifact as a byte literal in the generated tokens, and the resulting crate compiles and runs with no path, no `include_bytes!`, no build script, and no file outside the crate's own sources. Show it by building the consumer with every Tiler-produced file removed from the filesystem.
2. **Cache-root deletion after expansion is shown not to break expanded code.** Expand, delete the entire cache root, then build and run. This is the load-bearing half of "self-contained": if a deleted cache breaks the build, the embedding was a reference wearing a literal's clothes. Confirm the deletion actually happened — a check that passes because the path was wrong proves nothing.
3. **Cargo and rust-analyzer cold and warm behaviour is measured with exact environment, exact commands, and recorded results**, across four axes named separately because they fail differently: a source edit, a toolchain change, **repeated identical artifacts across crates** (does the second crate reuse or recompile), and **unique artifacts across crates** (does concurrency behave as the cache protocol expects). Each axis gets its own recorded row; a single aggregate number hides which one regressed.
4. **The size and diagnostic gates are stated as numbers, not adjectives.** "Bounded sizes" closes on an actual byte figure per embedded artifact and a stated ceiling; the diagnostic gate closes on the exact rendered text a consumer sees for each failure class. "Acceptable", "reasonable", and "small" do not close this ticket.
5. **Every unmeasured case is recorded as an explicit boundary**, in the form the cache exercise already uses: what was not reached, why, and what it would need. A measurement's value is bounded by the honesty of what it excludes, and an unstated gap reads as a covered case.

## Reuse — evidence that already exists (2026-07-28)

**A neighbouring spike answers part of axis 3 and should be read before re-measuring it.** `docs/research/cache/build-tool-exercise.md`, with its recorded results at `spikes/cache/results/build-tool-exercise-macos-27.0-2026-07-25.tsv` (12 rows) and its driver `spikes/cache/build_tool_exercise.py`. Both are currently **uncommitted** in the harness worktree `.claude/worktrees/agent-a9c032913d1ed60e0` and land with `exercise-the-expansion-cache-under-cargo-and-rust-analyzer`. What it already establishes: `cargo` expands in `rustc`, one short-lived process per crate; `rust-analyzer` expands in the long-lived `rust-analyzer-proc-macro-srv`; they share a working directory in every recorded row; and `CARGO_PKG_NAME` does **not** distinguish the two drivers — the analyzer populates it from the crate graph, so a macro reading it reports "cargo" under both. `std::env::current_exe()` is the signal that works. What it does *not* cover, and this ticket must: a carried compiled payload (that spike's envelope declares its payload by descriptor, so no backend object travelled through a cache entry), a real LSP session with incremental edits, and any embedding or size question at all.

**The family-`cfg` probe is checked in and is a golden, with the caveat that implies.** `spikes/macro-environment/run-family-cfg.sh` exists and is executable, and demonstrates on the measured macOS host that a nonmatching iOS family removes its `compile_error!` and executes fallback while the matching macOS family produces the retained diagnostic. **No `make` target reaches `spikes/`** — confirmed by `grep -n "spikes" Makefile`, which returns nothing. So that probe's retained output is a positive claim about what a compiler emitted on the day it was captured, it outlives whatever produced it, and nothing compares it to the source beside it until someone runs the script by hand from its own directory. Re-run it before citing it as current, and treat its result as evidence of what was measured rather than as a live check.

## Graph maintenance

- **The reuse evidence in the body is uncommitted** — it lands with `exercise-the-expansion-cache-under-cargo-and-rust-analyzer` (currently blocked, its results sit in worktree `agent-a9c032913d1ed60e0`). If that has not landed when you start, read the worktree TSV directly and say so in your measurement record; do not re-measure axis 3's already-answered half without noting why.
- **Findings about macro-frontend ergonomics belong on `prototype-inline-proc-macro-frontend`** (awaiting Tom's decision on syntax) — feed measurements there, do not pre-empt the decision here.

## Outcome

All five closes-when items are met by recorded measurement. The record is [`docs/research/embedding/self-contained-embedding.md`](../docs/research/embedding/self-contained-embedding.md); the harness is `spikes/embedding/self-contained/` driven by [`spikes/embedding/self_contained.py`](../spikes/embedding/self_contained.py), with results at [`spikes/embedding/results/self-contained-embedding-macos-27.0-2026-07-31.tsv`](../spikes/embedding/results/self-contained-embedding-macos-27.0-2026-07-31.tsv) (15 rows) and [`spikes/embedding/results/self-contained-diagnostics-macos-27.0-2026-07-31.txt`](../spikes/embedding/results/self-contained-diagnostics-macos-27.0-2026-07-31.txt) (7 failure classes, verbatim).

**1. Self-containment, demonstrated.** `cargo rustc -- -Zunpretty=expanded` renders the tokens rustc received; that source becomes a crate with an empty `[dependencies]` table, sixteen Tiler-produced files are deleted and proved deleted, and the crate builds and runs cold from an empty target directory. It printed `slot=a len=36838 fnv1a=cde14cdbcc31cb32`, equal to what the driver computed independently from the producer's file. The payload is exactly one byte-string literal, 68,076 source bytes for 36,838 payload bytes.

**2. Cache-root deletion, with the deletion proved.** Three deletion rows separate the cases: artifacts deleted with the cache surviving resolves as a validated hit; the cache deleted with the artifacts already gone runs **zero** expansions and leaves build and run untouched; the cache deleted with artifacts restored republishes. Every deletion requires the same path to hold at least one file before removal and none afterwards, so a mistyped path fails before anything is deleted rather than passing an after-check for free — the exact trap this ticket names.

**3. Four axes, nine rows, both drivers.** Source edit (cold 1 expansion / 1 read, warm 1 / 0); toolchain change under Cargo (`nightly-2026-07-19` → `nightly-2026-07-20`, 1 expansion / 0 reads) and under rust-analyzer (a `nightly-2026-07-20` proc-macro server hits entries the pin published); repeated artifacts across crates (2 expansions, **1** read, both drivers); unique artifacts across crates (2 expansions, 2 reads, both drivers). The finding worth carrying: Cargo's fingerprint carries the compiler and the cache subject does not, so a toolchain change re-expands but reads nothing.

**4. Gates as numbers.** Largest real envelope 47,803 bytes; measured member 36,838 bytes; ceiling 1,048,576 bytes per invocation, so the largest real artifact is 4.56% of it. Carried `metallib` objects are 3,491–7,158 bytes. Byte-string source-text ratio 1.848. Seven diagnostic classes rendered verbatim, each reached by a build that had to fail.

**5. Boundaries.** Ten unreached cases with reasons and triggers are in the research note's section 7, including the largest one: the production `tiler::tensor!` states `FallbackOnly`, embeds no bytes and opens no cache, so there is no production embedding to measure until `generate-cfg-gated-artifact-family-delivery`.

**A neighbouring gap is closed by evidence, not edited here.** [The build-tool exercise](../docs/research/cache/build-tool-exercise.md) lists "a carried compiled payload" as not reached, because its envelope declared its payload by descriptor. This spike's envelopes come from `prototypes/serial-sum-compile` and carry compiled `metallib` objects through `get_or_publish`, validated on every hit by the real `decode_artifact`. That note belongs to `exercise-the-expansion-cache-under-cargo-and-rust-analyzer` and its correction is filed as `correct-the-carried-payload-gap-in-the-build-tool-exercise` rather than taken on this branch.

**A contract sentence this evidence supports is not written here.** `docs/integration/frontends.md` states the self-contained-AOT-unit invariant but does not say what "self-contained" was measured to mean. Adding it needs the `contracts/decisions` scope this ticket does not hold; filed as `state-the-measured-meaning-of-self-contained-embedding`.

**Stale ticket text corrected, verified at base `c9c7127`.** The graph-maintenance section said the build-tool-exercise evidence was uncommitted in worktree `agent-a9c032913d1ed60e0`; it is committed — `git log --oneline -1 -- docs/research/cache/build-tool-exercise.md spikes/cache/results/build-tool-exercise-macos-27.0-2026-07-25.tsv` resolves at the base. Its axis-3 half is reused and cited rather than re-measured. The same section said findings belong on `prototype-inline-proc-macro-frontend` as "awaiting Tom's decision on syntax"; that ticket is `done` and the grammar is approved, so the note records spike-side evidence for the delivery tickets instead. No claim for this ticket was live in `tkt claims` when work began, despite the dispatch brief stating one; it was claimed as `worker-embedding` and moved to `in-progress` at that point.

**Not done, deliberately.** No production code changed: `crates/tiler-macros` and `crates/tiler` are untouched, because every measurement was reachable spike-side and adding embedding to the production macro would have pre-empted a public-boundary decision that is Tom's. Release-profile linking, folding, and constant-section size stay with the cost note's matrix; the crate-wide 32-invocation gate stays unowned by any proc macro, as that note already records.
