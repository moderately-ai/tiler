---
id: honour-require-metal-toolchain-in-the-aot-driver-tests
title: Make the metal-aot driver tests able to say no on a toolchain-bearing host
status: todo
priority: p2
dependencies: []
related: [remove-the-fast-honor-pragmas-variant]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [defect, metal-aot, test-validity]
---
## User-visible outcome

An absent Apple toolchain can be turned into a failure for `tiler-metal-aot`'s driver tests, the way it already can for `tiler-metal`'s golden compilation, so a run that exercised none of the toolchain-dependent half is distinguishable from one that passed it.

## Why this exists

**Fact.** `crates/tiler-metal/src/golden_compilation.rs:203` defines `REQUIRE_TOOLCHAIN = "TILER_REQUIRE_METAL_TOOLCHAIN"` and documents it as "the one supported ambient input here", turning a skip into a failure. **Fact.** `crates/tiler-metal-aot/src/driver.rs` has the same self-skip shape in every toolchain-dependent test — `if toolchain.resolve(AppleSdk::MacOs).is_err() { return; }` — and honours no such variable; `rg -c 'TILER_REQUIRE_METAL_TOOLCHAIN' crates/` returns `tiler-metal/src/golden_compilation.rs` only.

**Why that is a defect rather than a style difference.** Five driver tests reach the real toolchain and skip when `resolve` fails, returning green having asserted nothing — `rg -n 'toolchain.resolve\(AppleSdk::MacOs\)' crates/tiler-metal-aot/src/driver.rs` returns six lines, of which `:576` is `resolve_fails_closed_when_launcher_is_absent` deliberately asserting the failure and `:605`, `:634`, `:674`, `:703`, and `:788` are the skip sites. That is the shape `AGENTS.md` names directly: silence reads as success to anything that does not independently know how many answers to expect. It has already misled once — the dispatch brief for `remove-the-fast-honor-pragmas-variant` instructed the worker to run `cargo nextest run -p tiler-metal-aot` "with `TILER_REQUIRE_METAL_TOOLCHAIN=1` for the driver tests", which this package ignores. That run's green was not evidence the driver half executed; only a deliberate perturbation was.

**Not urgent, and why.** Tiler develops on macOS only and the toolchain is normally present, so this costs evidence quality rather than correctness. It is filed because the next reader of a green `-p tiler-metal-aot` run cannot tell which half ran.

## The work

- Honour `TILER_REQUIRE_METAL_TOOLCHAIN` in `crates/tiler-metal-aot/src/driver.rs`'s skip sites: when it is set, an unresolvable toolchain fails with the resolution error rather than returning. Follow `golden_compilation`'s convention — the variable may only make the tests stricter, never weaken a check.
- Announce the taken branch on standard error at each skip site, as `golden_compilation` does, so `--nocapture` distinguishes "the toolchain resolved" from "the test did not run".
- Prove the new refusal can fire: run once with the variable set against a deliberately unresolvable launcher and watch it fail.

## Closes when

Setting the variable turns an absent toolchain into a typed failure for every self-skipping test in `crates/tiler-metal-aot/src/driver.rs`, and that failure was watched happening.
