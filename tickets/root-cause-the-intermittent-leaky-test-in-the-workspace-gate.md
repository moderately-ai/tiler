---
id: root-cause-the-intermittent-leaky-test-in-the-workspace-gate
title: Root-cause the intermittent leaky test in the workspace gate
status: todo
priority: p2
dependencies: []
related: [package-a-multi-entry-bundle-from-one-expansion, prototype-inline-aot-integration-proof]
scopes: [implementation/workspace, implementation/ir]
shared_scopes: [project/tickets]
tags: [testing, gate, determinism, process-lifetime]
---
## User-visible outcome

The workspace gate either never reports a leaky test, or names the one it means and explains why the leak is benign. Today it reports a count with no name, on some runs and not others, which is a dice roll a reader cannot act on.

## What was observed, and it is a count rather than a name

**Measurement — two occurrences on 2026-08-02, on the same host, across a day of gate runs.** `cargo nextest run --workspace` printed:

```
Summary [  10.282s] 2335 tests run: 2335 passed (1 leaky), 7 skipped
```

once at `c6bdd55` (integration gate) and once during `package-a-multi-entry-bundle-from-one-expansion`'s own run at `52d3835`. Every other gate run that day printed no `leaky` marker at all. The worker who saw it first could not reproduce it: not in a second full-workspace run with `--final-status-level=all`, nor in five consecutive `-p tiler-macros` runs, with no `LEAK` line in any.

**Fact — the gate cannot tell you which test it was, by configuration.** `.config/nextest.toml` reports failures only, so a leaky-but-passing test is counted and never named. That is the right default for a green run printing one line, and it is exactly what makes this observation hard to act on. Reproducing with `--final-status-level=all` or `--status-level=all` is the documented override and does not require changing the profile.

**Inference — the shape points at a child process outliving its test.** A nextest "leaky" verdict means the test exited but left a handle open — commonly a spawned child still holding the pipe. Every AOT test in `crates/tiler-macros` spawns `xcrun`/`metal`, which is the obvious candidate, and both observations came from runs that included those tests. This is a hypothesis, not a finding; it is written down so it can be refuted rather than assumed.

## Why this is worth a ticket rather than a shrug

AGENTS.md is explicit: "An intermittent failure is a defect in the mechanism to be root-caused and fixed; re-running until green, loosening the assertion, or shrugging it off as flaky each converts the gate into a dice roll, which is Tom's stated line." A leaky test is not a failure today — but it is the same class of nondeterminism, and a leaked child that later *does* hold a shared path is how a gate goes red for reasons unrelated to the change under test. The repository has already been bitten by exactly that shape: a producer's own binary path stopped resolving mid-run because sibling invocations unlinked and relinked the shared Cargo hardlink.

## Required work

- **Name the test.** Re-run the workspace gate with `--final-status-level=all` (repeatedly if needed, since it is intermittent) and capture the `LEAK` line. Until the test is named, nothing below is actionable.
- Determine what handle is left open. If it is a spawned `xcrun`/`metal` child, establish whether the test waits on it at all, and whether the wait can be missed on an early return or an error path.
- Fix the mechanism rather than the symptom: a test that spawns a child owns reaping it on every path, including the failing one. Do not silence the report and do not add a retry.
- If the leak turns out to be benign and unavoidable — a platform behaviour the test cannot control — say so with the evidence and record it where the next reader will look, rather than leaving a count nobody can explain.

## Required evidence

- The leaking test named, with the `LEAK` line quoted.
- The handle identified, not inferred from the test's name.
- After the fix, the gate run repeatedly enough to make the absence meaningful, with the number of runs stated — a single green run proves nothing about an intermittent defect, and an uncounted "it stopped happening" is the shrug this ticket exists to refuse.

## Explicit non-goals

Do not change `.config/nextest.toml`'s failure-only reporting to make the leak visible in every run; that trades one line of green output for noise on every gate, and the per-invocation override already exists. Do not add a retry, a sleep, or a `--no-capture` workaround.

## Graph maintenance

Filed 2026-08-02 at integration, on the second observation. The first was recorded by `package-a-multi-entry-bundle-from-one-expansion`'s worker, which correctly declined to chase it as out of scope and said it would be worth a ticket if it recurred. It recurred.

## Named but not yet root-caused, 2026-08-02 (base `3aa94f9`)

**Measurement — the leaking test is now named, and the Metal hypothesis is
refuted.** A warm `cargo nextest run --workspace --status-level=all
--final-status-level=all` produced this terminal line on workspace run 2:

```text
        LEAK [   2.467s] (2350/2367) tiler-ir::typed_handles typed_authoring_contract
```

The summary was `2367 passed (1 leaky), 7 skipped`. Workspace runs 1 and 3--8
on the same checkout reported no leak. Twelve consecutive
`cargo nextest run -p tiler-ir --status-level=all
--final-status-level=all` runs, one isolated `typed_handles` run, and one
package run with a diagnostic-only 1 ms leak timeout also reported no leak.
That is one occurrence in eight full-workspace runs and none in fourteen
narrower runs in this investigation; it is a measurement boundary, not an
absence claim. The environment was Apple M4 Max, arm64 macOS 27.0 build
26A5388g, `nightly-2026-07-19`, cargo-nextest 0.9.133, and trybuild 1.0.118.

**Fact — the named test never reaches the frontend AOT path.**
`crates/tiler-ir/tests/typed_handles.rs` creates one `trybuild::TestCases`, then
registers one compile-pass fixture and six compile-fail fixtures. The pass
fixture at `tests/typed-handles/pass/checked_authoring.rs` performs only local
IR construction and starts no process. A search across all 111 Rust source
files under `crates/tiler-ir` found no `std::process` or `Command::new`; its one
`.spawn` match is a scoped Rust thread. The process owner is trybuild.

**Fact — one reachable trybuild path inherits nextest's capture handles.**
Inspection of trybuild 1.0.118's `src/cargo.rs` found nine process invocation
sites. The metadata, fixture build/check, and fixture run calls use
`Command::output`, so they replace
stdout/stderr with their own pipes and cannot return until those pipes reach
EOF. The clean and `--keep-going` probes send both streams to `Stdio::null`.
Two sites use `Command::status` without redirecting either stream: the
`generate-lockfile` fallback and the initial dependency build. The fallback was
unreachable in every measured run because this checkout had one `Cargo.lock`;
the initial dependency build was reachable, so Cargo and its compiler/linker
descendants inherited the test's nextest capture handles. Cargo itself is
waited.

**Inference — the observed leak came from a descendant of that reachable
initial Cargo build.** Nextest's verdict means a process retained captured
stdout or stderr for more than its default 100 ms after the test exited, and
the audit above leaves the initial dependency build as the only reachable
trybuild call that inherited those handles. This identifies the handle and the
reachable ownership chain, not the exact retaining process: the leaking run's
descriptor/process chain was not captured. It does establish that the leak was
not an unreaped `xcrun`/`metal` child owned by this repository.

**Measurement — process sampling did not name the transient descendant.** One
full-workspace run was sampled every 20 ms and observed the expected
`typed_handles -> cargo -> rustc -> clang` compilation chain, but that run did
not leak. Five further full-workspace runs sampled every 10 ms for an orphaned
process under this worktree's `target/tests/trybuild` tree; none leaked and no
orphan was observed. The exact process retaining the inherited descriptor is
therefore still unknown. Calling the `rustc` or `clang` observed in a non-leaky
run the culprit would turn timing correlation into a false finding.

**Stop — the remaining investigation and any repair collide with a live IR
claim.** The named test and its fixture live in `implementation/ir`, which
`admit-an-additive-extent-relation` held concurrently during this
investigation. This ticket now claims that scope so the board cannot redispatch
the collision. `implementation/frontend` was removed: the named test never
reaches that code, so no surviving repair path can require it.
`implementation/workspace` remains because an exact finding of unavoidable
trybuild/Cargo/compiler teardown latency could justify a narrowly scoped
nextest override in `.config/nextest.toml`; retaining the scope does not presume
that outcome. No implementation file changed. Resume when the exclusive IR
claim is free; reproduce with status reporting set to `all` and capture the
live process/descriptor chain on the leaking run. If the retained process is
controlled by repository code, add an omission perturbation that leaks before
fixing ownership on every path. If it is trybuild/Cargo/compiler teardown
latency outside repository control, establish the exact process and closure
duration before proposing that override; the current evidence does not
authorize changing the failure-only profile, silencing leak reports, adding
sleeps, or marking this done.
