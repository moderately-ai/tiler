---
id: root-cause-the-intermittent-leaky-test-in-the-workspace-gate
title: Root-cause the intermittent leaky test in the workspace gate
status: todo
priority: p2
dependencies: []
related: [package-a-multi-entry-bundle-from-one-expansion, prototype-inline-aot-integration-proof]
scopes: [implementation/frontend, implementation/workspace]
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
