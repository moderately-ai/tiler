---
id: honour-require-metal-toolchain-in-the-aot-driver-tests
title: Make the metal-aot driver tests able to say no on a toolchain-bearing host
status: in-progress
priority: p2
dependencies: []
related: [remove-the-fast-honor-pragmas-variant]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [defect, metal-aot, test-validity]
claimed_from: todo
assignee: agent-aot-skip
lease_expires_at: 1786041779
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

## Outcome

**Done.** `crates/tiler-metal-aot/src/driver.rs`'s test module gained `REQUIRE_TOOLCHAIN = "TILER_REQUIRE_METAL_TOOLCHAIN"` and one `resolved_system_toolchain() -> Option<(Toolchain, ResolvedToolchain)>`, and all five self-skip sites now route through it. The classification follows `tiler-metal`'s `golden_compilation::resolved_toolchain`: `ToolchainUnavailable` and `SdkUnavailable` are the only absent-toolchain answers, and under the variable each becomes an assertion failure carrying the resolution error; `ToolFailure`, `Host`, and `EmptyArtifact` panic as defects rather than skipping. The match is exhaustive over `DriverError`, so a new variant stops the module compiling instead of defaulting to a skip.

**Why one function rather than five honoured conditions.** The five sites spelled the decision two ways — three `if resolve(..).is_err() { return; }` and two `let Ok(resolved) = .. else { return; }` — and a per-site check would have had to restate both the classification and the ambient read, which is how the two crates diverged in the first place. The returned tuple carries the resolved observation beside the toolchain so the two sites that need it do not resolve a second time; two resolutions can disagree, and this crate's own `prepare` exists to stop exactly that.

**Sites converted** (all in `crates/tiler-metal-aot/src/driver.rs`): `a_real_front_end_warning_survives_a_succeeding_compilation`, `compiles_trivial_kernel_when_toolchain_available`, `the_integer_nan_predicate_compiles_under_every_realization`, `rejects_invalid_source_when_toolchain_available`, `the_metal_driver_admits_exactly_the_three_stated_fp_contract_values`. `resolve_fails_closed_when_launcher_is_absent` and `compile_fails_closed_when_launcher_is_absent` are untouched: they assert the resolution failure rather than skipping on it. The shim-launcher tests supply their own tools and never skip; the standing comment above the helper now says so, because "the following tests exercise the real Apple toolchain" was not true of them.

**Announcement.** Both branches print to standard error. `cargo nextest run -p tiler-metal-aot --no-capture -E 'test(driver::tests)'` on this host prints, from each of the five, `driver: compiling with metal Apple metal version 32023.921 (metalfe-32023.921) / metallib AIR-LLD 32023.921 (metalfe-32023.921) (compatible with legacy metallib linker) (SDK 27.0 build 26A5388f)` — 12 of 62 tests selected, 5 announcements, which is the count that distinguishes a run that executed the toolchain-dependent half from one that did not.

**Watched failing, both directions.** The perturbation was a one-line substitution of `Toolchain::system()` for `Toolchain::with_launcher("/nonexistent/tiler-metal-aot-xcrun")` inside the helper, reverted afterwards. With `TILER_REQUIRE_METAL_TOOLCHAIN=1`: `62 tests run: 57 passed, 5 failed`, each of the five failing at `driver.rs:634` with `TILER_REQUIRE_METAL_TOOLCHAIN is set, but no qualified Apple Metal toolchain resolved: Apple Metal toolchain unavailable (metal, discovery): could not run /nonexistent/tiler-metal-aot-xcrun: No such file or directory (os error 2)`. The failing five are exactly the five converted sites. With the variable unset the same perturbed build passes and prints `driver: skipped, no qualified Apple Metal toolchain resolved: …`, so an absent toolchain still shows as an announced skip rather than a silent pass.

**Test populations.** `cargo nextest run -p tiler-metal-aot` reports `62 tests run: 62 passed, 0 skipped` both with the variable unset (via `env -u`) and with `TILER_REQUIRE_METAL_TOOLCHAIN=1`. The counts are equal by design — the variable removes a permission, it does not add or filter tests — and on this toolchain-bearing host the equality plus the five announcements is what shows the toolchain-dependent half ran under both.

**Determinism.** No wall-clock and no shared mutable path: the helper only *reads* the environment, never sets it, so no test mutates process state another test observes. The skip decision is a typed classification of `DriverError`, not the absence of an assertion.

**Checks.** `cargo fmt --check`; `cargo check -p tiler-metal-aot --all-targets`; `cargo clippy -p tiler-metal-aot --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-metal-aot`; `cargo nextest run -p tiler-metal-aot` (both ways, above); `cargo test -p tiler-metal-aot --doc` (4 + 3 passed); `git diff --check`; `tkt lint`; `tkt guard --base c22d4b24`. All clean.

**Superseded note.** [`remove-the-fast-honor-pragmas-variant`](remove-the-fast-honor-pragmas-variant.md)'s Outcome closes with a note stating that `TILER_REQUIRE_METAL_TOOLCHAIN` is honoured only by `tiler-metal`'s `golden_compilation` and that setting it while running this package changes nothing. That was the observation this ticket was filed from; it no longer holds. The note is left in place as the record of that discovery.
