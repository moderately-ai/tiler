---
id: stop-the-identity-join-producer-race
title: Stop the identity-join producer racing itself on the shared target directory
status: todo
priority: p2
dependencies: []
related: [lower-a-loop-carried-cooperative-body]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [research]
---
## User-visible outcome

`cargo nextest run --workspace` is green or red because of the code, not because of what was in `target/` when it started. Until it is, `tiler-runtime::identity_join` fails with a `SIGKILL` on a cold example binary and passes on the next run — which is a dice roll wearing a gate's clothes.

## The mechanism, derived from the harness

**Fact — every case shells out to Cargo.** `produce()` in `crates/tiler-runtime/tests/identity_join/producer.rs` runs `cargo run --quiet --locked --offline --package tiler-build --example identity_join_producer -- <root>` from the workspace root, and it is called once per test rather than once per binary.

**Fact — nextest runs those tests concurrently, each in its own process.** So several `cargo run` invocations contend for one `target/` at the same time.

**Inference — the concurrent invocations relink the same example binary, and one that is already executing dies when its inode is replaced.** That is exactly the hazard [AGENTS.md](../AGENTS.md) records: a process that re-executes a binary owns a private copy of it, because the shared Cargo hardlink under `target/` is unlinked and relinked by sibling invocations. It is invisible once the example is warm, which is why it fires on the first workspace run after a package-scoped build and not on the second.

**Measurement — concurrency is the whole cause, and the experiment separates it in one line.** In the `lower-a-loop-carried-cooperative-body` worktree, macOS, nightly-2026-07-19, with `crates/tiler-runtime` unmodified:

| Invocation | Runs | Runs with a failure |
| --- | --- | --- |
| `cargo nextest run -p tiler-runtime --locked --test identity_join` | 5 | **3** |
| the same with `--test-threads 1` | 3 | **0** |

Every failure is `the build-time producer failed (signal: 9 (SIGKILL))` with empty stdout and empty stderr — a process-level kill of the subprocess, never a nonzero exit or a failed assertion, which is what places the defect in the harness rather than in anything the tests are about. `cargo nextest run --workspace --locked` was green three times before this surfaced and red on a single `identity_join` case twice afterwards, always a different case, which is the rate the table predicts. The same workspace run excluding this one binary is green (2,268 passed, 7 skipped) and the binary serialized is green (13 passed).

## What this owns

Making the producer a shared, once-per-binary artifact rather than a per-test Cargo invocation. Candidates worth eliminating explicitly rather than picking: producing once behind a `OnceLock` guarded by a file lock so the thirteen cases share one tree; running it once from a nextest setup script; or copying the built example to a per-process private path before executing it, which is the shape AGENTS' own sentence points at. Note that `produce()` already keys its output tree by `std::process::id()`, so the *outputs* do not collide — it is the shared `cargo run` and the binary it relinks that do, and a fix that only separates the trees will not move the table above.

## Closes when

`cargo nextest run -p tiler-runtime --locked --test identity_join` is green over at least ten consecutive runs at default concurrency — the measured failure rate above makes ten a meaningful population and three not — the fix names which candidate it took and why the others were eliminated, and the harness no longer starts a Cargo invocation per test case.
