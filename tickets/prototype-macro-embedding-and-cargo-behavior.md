---
id: prototype-macro-embedding-and-cargo-behavior
title: Measure macro embedding and Cargo behavior
status: in-progress
priority: p1
dependencies: [prototype-inline-proc-macro-frontend, implement-the-expansion-cache-protocol, compose-the-complete-expansion-cache-subject, prototype-artifact-family-delivery, prototype-metal-aot-slice]
related: [repair-macro-and-embedding-harness-integrity]
scopes: [implementation/frontend, research/embedding, research/macro-environment]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, measurement, proc-macro, inline-dx]
claimed_from: todo
assignee: worker-embedding
lease_expires_at: 1785544168
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
