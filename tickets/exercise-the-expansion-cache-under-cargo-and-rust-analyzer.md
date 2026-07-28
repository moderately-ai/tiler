---
id: exercise-the-expansion-cache-under-cargo-and-rust-analyzer
title: Exercise the expansion cache under Cargo and rust-analyzer
status: blocked
priority: p2
dependencies: [port-the-cache-harness-to-the-production-bundle, prototype-inline-proc-macro-frontend]
related: [implement-the-expansion-cache-protocol]
scopes: [research/cache]
shared_scopes: []
paths: []
tags: [cache, concurrency, frontend]
---
The research note's seventh follow-up gate: run the harness under Cargo and rust-analyzer process patterns once the proc-macro spike exists.

ADR 0050's context is that "Cargo and rust-analyzer may run equivalent proc-macro expansions concurrently", and that is the workload the whole protocol was designed against. Everything measured so far uses a harness that spawns its own workers, which is a model of that workload rather than the workload.

## What this ticket owes

- The real process pattern: how many expansions run at once, whether they share a working directory, and whether rust-analyzer's and Cargo's expansions overlap in time on one key.
- Whether the per-key lock behaves as measured when the holder is a proc-macro server that may be killed and restarted by its editor.
- Whether the default cache root is reachable and private in both contexts, and what a sandboxed or CI environment overrides it to.

Blocked until there is a proc-macro layer to run under. The prerequisite named here was `port-the-cache-harness-to-the-production-bundle`, and that is stale — it is `status: done`. The live blocker is `prototype-inline-proc-macro-frontend` (`status: awaiting-decision`, pending Tom's acceptance of the public compiler boundary), added as a dependency by `fdf68a2` ("Audit production crates and reconcile open tickets").

## Work in flight — recorded 2026-07-28

**Fact — the work exists and has not landed.** It is uncommitted in the harness worktree `.claude/worktrees/agent-a9c032913d1ed60e0`, on branch `tkt/exercise-the-expansion-cache-under-cargo-and-rust-analyzer` at HEAD `0bcaa2a`, **318 commits behind `main`** as of this record. Modified there: `docs/open-questions.md`, `docs/research/README.md`, `spikes/README.md`, `spikes/cache/README.md`, and this ticket. Untracked: `docs/research/cache/build-tool-exercise.md`, `spikes/cache/build-tool-exercise/`, `spikes/cache/build_tool_exercise.py`, `spikes/cache/results/build-tool-exercise-macos-27.0-2026-07-25.tsv` (12 lines), and a second ticket `tickets/correct-adr-0050-end-to-end-hit-status.md`.

**What the evidence settles, of the three answers this ticket owes.** All three are answered, but two of them by correcting the question rather than by measuring what it asked for.

1. **The process pattern — answered.** `cargo` expands in `rustc`, one short-lived process per crate, so expansions within a crate are sequential and concurrency arrives between crates and between builds. `rust-analyzer` expands in `rust-analyzer-proc-macro-srv`, one long-lived process per editor session, overlapping freely with any Cargo build. Neither coordinates with the other. **They do share a working directory** — `cwds` is `1` in every recorded row, including the row where a `rustc` process and the analyzer's server expanded concurrently — which was the opposite of the going-in guess and matters because a frontend deriving a root from the current directory would otherwise have split its cache in two. Twelve expansions across three genuinely overlapping Cargo builds produced **four** compilations, one per key; the same race under an unusable cache root produced **twelve**, which is what makes the first number evidence rather than a counter that never moves.
2. **The lock under a killed holder — answered, including the case this ticket names.** Two scenarios `SIGKILL` a lock holder's process group; `analyzer-killed-holding-lock` kills the proc-macro server specifically. In both, a subsequent build took the same key's lock and resolved every key. The inference is sound because the alternative is observable: `resolve` blocks on `acquire`, so a leaked lock would have wedged the survivor until its deadline. Ordering is established by an observed marker file rather than a wall-clock margin, deliberately not repeating the defect `remove-the-wall-clock-race-from-the-cache-kill-harness` is fixing.
3. **The default cache root — answered by refutation; there is no such concept.** `ExpansionCache::open(root)` takes the root from its caller, performs no I/O, and creates no directory, and the crate never consults the environment. The exact check: `grep -rn "env::var\|var_os\|home_dir\|dirs::\|std::env" crates/tiler-cache/src/ --include='*.rs'` matches only `expansion/harness.rs`, `expansion/fault.rs`, and `expansion/tests.rs`, all `#[cfg(test)]`. So "reachable and private in both contexts" has no subject and "what a sandboxed environment overrides it to" has no default to override. The real, unowned question is who chooses a root — the frontend proc-macro layer, which does not exist.

**The measurement boundary, stated because it is easy to overstate this result.** The spike is an orchestrator holding both crates and it measured positive hits through the public `get_or_publish`, validated by the real `decode_artifact`. What is still absent is narrower than ADR 0050's wording implies but is not nothing: **the payload is declared by descriptor rather than carried, so no compiled backend object has yet travelled through a cache entry.** Two facets of the composed subject are also stand-ins, necessarily, because no producer exists for `SubjectFacet::ArtifactProgram` yet. Also not reached: a real LSP session with incremental edits, analyzer-initiated cancellation, server restart and re-expansion after a kill, concurrency above three, and Linux.

**Status.** Frontmatter is not this record's to change; the request to move `in-progress` to `blocked` is left for the coordinator.
