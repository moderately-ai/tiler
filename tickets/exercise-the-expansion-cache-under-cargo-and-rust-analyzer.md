---
id: exercise-the-expansion-cache-under-cargo-and-rust-analyzer
title: Exercise the expansion cache under Cargo and rust-analyzer
status: done
priority: p2
dependencies: [port-the-cache-harness-to-the-production-bundle]
related: [implement-the-expansion-cache-protocol, correct-adr-0050-end-to-end-hit-status]
scopes: [research/cache, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [cache, concurrency, frontend]
---
The research note's seventh follow-up gate: run the harness under Cargo and rust-analyzer process patterns once the proc-macro spike exists.

ADR 0050's context is that "Cargo and rust-analyzer may run equivalent proc-macro expansions concurrently", and that is the workload the whole protocol was designed against. Everything measured so far uses a harness that spawns its own workers, which is a model of that workload rather than the workload.

## What this ticket owes

- The real process pattern: how many expansions run at once, whether they share a working directory, and whether rust-analyzer's and Cargo's expansions overlap in time on one key.
- Whether the per-key lock behaves as measured when the holder is a proc-macro server that may be killed and restarted by its editor.
- Whether the default cache root is reachable and private in both contexts, and what a sandboxed or CI environment overrides it to.

## Outcome

All three answers are delivered, two of them by correcting the question rather than by measuring what it asked for. The evidence is `docs/research/cache/build-tool-exercise.md`, the checked-in spike `spikes/cache/build-tool-exercise/` with its driver `spikes/cache/build_tool_exercise.py`, and the recorded run `spikes/cache/results/build-tool-exercise-macos-27.0-2026-07-25.tsv`.

**The blocker this ticket waited on turned out not to be one.** It was held blocked pending a proc-macro layer to run under — latterly `prototype-inline-proc-macro-frontend`, added as a dependency by `fdf68a2`. The worker instead built the spike its own three-crate Cargo workspace — envelope, expansion macro, consumer — whose macro resolves a real `tiler-compiler`-produced, `tiler-artifact`-encoded envelope through the public `ExpansionCache::get_or_publish`, under real `cargo` and the pinned toolchain's own `rust-analyzer-proc-macro-srv`. That dependency is removed with this closure: the production frontend remains unbuilt, and nothing this ticket owed needs it.

1. **The process pattern — answered.** `cargo` expands in `rustc`, one short-lived process per crate, so expansions within a crate are sequential and concurrency arrives between crates and between builds. `rust-analyzer` expands in `rust-analyzer-proc-macro-srv`, one long-lived process per editor session, overlapping freely with any Cargo build. Neither coordinates with the other. **They do share a working directory** — `cwds` is `1` in every recorded row, including the row where a `rustc` process and the analyzer's server expanded concurrently — which was the opposite of the going-in guess and matters because a frontend deriving a root from the current directory would otherwise have split its cache in two. Twelve expansions across three genuinely overlapping Cargo builds produced **four** compilations, one per key; the same race under an unusable cache root produced **twelve**, which is what makes the first number evidence rather than a counter that never moves.
2. **The lock under a killed holder — answered, including the case this ticket names.** Two scenarios `SIGKILL` a lock holder's process group; `analyzer-killed-holding-lock` kills the proc-macro server specifically. In both, a subsequent build took the same key's lock and resolved every key. The inference is sound because the alternative is observable: `resolve` blocks on `acquire`, so a leaked lock would have wedged the survivor until its deadline. Ordering is established by an observed marker file rather than a wall-clock margin, deliberately not repeating the defect `remove-the-wall-clock-race-from-the-cache-kill-harness` is fixing.
3. **The default cache root — answered by refutation; there is no such concept.** `ExpansionCache::open(root)` takes the root from its caller, performs no I/O, and creates no directory, and the crate never consults the environment. The exact check: `grep -rn "env::var\|var_os\|home_dir\|dirs::\|std::env" crates/tiler-cache/src/ --include='*.rs'` matches only `expansion/harness.rs`, `expansion/fault.rs`, and `expansion/tests.rs`, all `#[cfg(test)]`. So "reachable and private in both contexts" has no subject and "what a sandboxed environment overrides it to" has no default to override. The real, unowned question is who chooses a root — the frontend proc-macro layer, which does not exist — and the research doc defers it with exactly that trigger.

**The measurement boundary, stated because it is easy to overstate this result.** The spike is an orchestrator holding both crates and it measured positive hits through the public `get_or_publish`, validated by the real `decode_artifact` — lifting the substituted-validator caveat the crash-and-race spike recorded. What is still absent is narrower than ADR 0050's wording implies but is not nothing: **the payload is declared by descriptor rather than carried, so no compiled backend object has yet travelled through a cache entry.** Two facets of the composed subject are also stand-ins, necessarily, because no producer exists for `SubjectFacet::ArtifactProgram` yet. Also not reached: a real LSP session with incremental edits, analyzer-initiated cancellation, server restart and re-expansion after a kill, concurrency above three, and Linux.

**Split rather than absorbed.** ADR 0050's traceability paragraph still says a positive end-to-end hit "remains the orchestrator's", which this evidence partly supersedes; editing it needs the `contracts/decisions` scope, so [`correct-adr-0050-end-to-end-hit-status`](correct-adr-0050-end-to-end-hit-status.md) carries the correction. Q-ART-006's stale availability blocker is corrected in `docs/open-questions.md` in the same landing.

## Landed 2026-07-28

Rescued from the stranded worktree `agent-a9c032913d1ed60e0` (323 commits behind): preserved verbatim as `80e5ea1` on `tkt/exercise-the-expansion-cache-under-cargo-and-rust-analyzer`, squash-merged without conflicts, and corrected only where the repository moved underneath it — the research doc's gate-collection deferral cited the retired Python gate scripts and now cites the no-`make`-target-touches-`spikes/` contract, with its conclusion intact. The measurement itself is tied to its recorded 2026-07-25 environment and base commit, which is the spike trade `AGENTS.md` states.

## Closes when

The three owed answers are recorded with checked-in evidence, the remaining ADR-0050 correction is tracked by a live ticket, and `make full` passes.
