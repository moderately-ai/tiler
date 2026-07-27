---
id: run-the-suite-through-nextest
title: Run the suite through nextest and drop tool version pinning
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [tooling, testing]
---
Tom's request: run the suite through nextest, report failures only, and drop the tool version pinning.

## What changed

**The gate's test step is now two commands.** `make test` runs `cargo nextest run --workspace --locked` followed by `cargo test --workspace --doc --locked --quiet`. The second is not redundant: **nextest does not run doc-tests at all**. Deleting it would not fail — it would silently stop compiling the four compile-fail doc-tests on `Preflight::commit` that are this repository's evidence for ADR 0051's one-way routing commit, and a doc-test that is never compiled reports nothing. `make full`'s release-profile numerical run moved to nextest for the same reporting benefit.

**`.config/nextest.toml` reports failures only.** `status-level` and `final-status-level` are `fail`, `success-output` is `never`, `failure-output` is `immediate`. A green workspace run is now four lines instead of 962. It is a reporting choice and not a filter: no test is skipped, and the run still fails on failure.

Two settings were chosen against the obvious default, both by measurement rather than taste:

- `failure-output = "immediate"`, not `immediate-final`. The latter reprints every failure's stdout and stderr verbatim beneath the summary, so a run with several failures says everything twice. `final-status-level` already re-lists the failing test *names*, which is the part worth repeating.
- `slow-timeout = { period = "60s" }` with **no** `terminate-after`. An earlier draft terminated after two 30s periods. `single_byte_corruptions_are_rejected` takes ~13s warm and was measured above 30s on a machine also running a build, so any threshold low enough to catch a hang is also low enough to kill a legitimate test under load. A spurious kill is worse than a slow gate.

A `timing` profile is kept beside the default: it lowers `slow-timeout` to 1s and reports every test that crosses it, which is how the quiet default's one blind spot — the test that *is* the suite's critical path — stays visible. `audit-the-suite-s-slowest-tests` cites it.

**Tool version pinning is gone.** `tool-versions.toml` and the ticketsplease revision receipt are deleted rather than left dormant; `deps.sh` installs cargo-nextest and ticketsplease when absent and asserts nothing about the version it finds. `rust-toolchain.toml` remains the sole version authority, covering Rust alone. The pins cost more than they caught — a host binary that auto-updated repeatedly failed the gate over drift that was never a defect, which `resolve-ticketsplease-version-authority-drift` already records.

**Measurement — the residual exposure, stated because it is real.** Nextest treats an unrecognized configuration key as a *warning* and continues (verified by adding `this-key-does-not-exist` to `[profile.default]`: warning, exit 0). So a sufficiently old nextest would run the suite with `.config/nextest.toml` partly ignored. That costs reporting quality, not correctness; no test is skipped by it. This is why the version floor was dropped rather than kept as a special case, and AGENTS.md records it so nobody rediscovers it as a bug.

## Verification

`make full` green: 962 nextest tests, 11 doc-tests across 9 crates, rustdoc, the release numerical tests, `tkt lint`, shellcheck.

The reporting configuration's failure path was exercised rather than assumed, per the repository's own rule that a check is only as good as its ability to say no. A deliberate `assert_eq!` failure and a deliberate `panic!` with stdout both surfaced in full under the quiet profile, and `fail-fast = false` ran both instead of stopping at the first. A deliberately failing doc-test surfaced in full under `--quiet`. All three probes were reverted.

`deps.sh --check` passes, and its failure path was confirmed by pointing it at a version that does not exist before the pinning was removed.
