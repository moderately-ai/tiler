---
id: compile-extension-spike-fixtures-in-the-gate
title: Decide whether the gate should compile the extension spike fixtures
status: in-progress
priority: p2
dependencies: []
related: [preserve-non-exhaustive-visibility-probe]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [testing, gate-reliability, harness]
claimed_from: todo
assignee: agent-compile-extension-spike-fixtures-in-the-gate
lease_expires_at: 1784925959
---
`preserve-non-exhaustive-visibility-probe` closed a decay gap, and left one open. The retained `#[non_exhaustive]` diagnostics are now verified by the repository gate — `scripts/tests/test_research_harnesses.py` runs `spikes/extensions/run.py --self-test`, which reads the checked-in `.stderr` files and the measurement beside them without invoking Cargo.

**What that check cannot see.** It verifies that a retained diagnostic still says what the record claims. It does not verify that the fixture `.rs` beside it still *produces* that diagnostic. Someone can edit `consuming/tests/ui/fail/cross_crate_total_map.rs` until it no longer fails for the recorded reason — or no longer fails at all — and every gate check stays green, because only a Cargo run compares the two. The same is true of `operation-api`, `proc-macro-visibility`, and `semantic-foundation-api-v2`: no Cargo fixture under `spikes/extensions/` is compiled by the gate or by CI.

**Fact — the gate deliberately builds no spike Cargo fixture except one.** `scripts/check_rust.py` runs the root workspace plus `spikes/shapes/nightly-dependent-static-shapes` through its `check.sh`, and nothing else. Every pytest module the gate collects (`spikes/apple-targets`, `spikes/embedding`, `spikes/macro-environment`, `spikes/numerics/sound_accuracy`) is a verifier over retained results; not one invokes Cargo. So this is a standing policy, not an oversight in one directory.

**The decision.** Whether that policy should hold for `spikes/extensions`. Compiling its suite costs a small Cargo build on both CI hosts and buys the fixture/diagnostic agreement above. Two routes exist and they differ in who owns them:

- add `spikes/extensions` to `testpaths` in `pyproject.toml` and to `EXPECTED_PYTEST_PATHS` in `scripts/check_repository.py` (both `implementation/workspace`, which is why this ticket declares it), then add a pytest module that shells out to `run.py --suite …`; or
- extend `scripts/tests/test_research_harnesses.py` (`contracts/navigation`) with the Cargo suite alongside the `--self-test` entry it already runs.

Either way the ticket must state the added gate wall-clock cost measured on both supported hosts, and must keep `spikes/shapes/nightly-dependent-static-shapes` as the one Cargo spike the *Rust* gate owns rather than moving it. If the answer is that the policy should hold, record that and close: an explicit "the gate verifies retained evidence, not fixture reproduction" is a better resting state than an unexamined gap.
