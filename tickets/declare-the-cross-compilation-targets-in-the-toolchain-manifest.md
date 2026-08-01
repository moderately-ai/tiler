---
id: declare-the-cross-compilation-targets-in-the-toolchain-manifest
title: Declare the cross-compilation targets in the toolchain manifest
status: todo
priority: p3
dependencies: []
related: [strengthen-the-family-cfg-evidence-with-the-installed-cross-targets]
scopes: [implementation/workspace, implementation/frontend]
shared_scopes: []
paths: []
tags: []
---
## User-visible outcome

The five-target cross-compilation evidence for the family-`cfg` delivery matrix runs in `make check` instead of by hand, because the targets it needs are part of what `./deps.sh` bootstraps and verifies.

## Why

**Fact.** `strengthen-the-family-cfg-evidence-with-the-installed-cross-targets` landed `every_emitted_shape_compiles_as_the_five_target_matrix_says` in `crates/tiler-macros/src/delivery/tests.rs`. It compiles the delivery emitter's three shapes for each of the five targets `docs/correctness-and-testing.md` names, and it is `#[ignore]`d.

**Measurement, 2026-08-01, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, M-series macOS host.** The whole test — five installed-target probes plus fifteen shape compilations — reported 0.88 s to 1.42 s over seven `cargo nextest run` invocations against a warm build. Cost is not why it sits outside the gate; the suite already carries a 13 s test.

**Fact.** `rust-toolchain.toml` declares `channel`, `profile = "minimal"`, and `components`; `deps.sh` reads that `components` array and verifies each entry. Neither declares a *target*. `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-apple-ios-macabi`, and `x86_64-unknown-linux-gnu` exist on this host only because Tom authorized adding them on 2026-07-31. A gate-resident test needing them would therefore fail `make check` on a host bootstrapped exactly as this repository documents, and the alternative — a test that skips the targets it cannot find — reports a clean pass over a population it never counted, which `AGENTS.md` names as the failure mode to distrust.

**Decision boundary.** `rust-toolchain.toml`'s `targets = [...]` key makes rustup install those standard libraries for every host that resolves the toolchain. That is a host-toolchain policy change and is Tom's, not a worker's: `AGENTS.md` reserves installing or mutating toolchain components to him, and the cost is one every checkout would pay whether or not it runs this test. **Measurement, 2026-08-01** (`du -sh ~/.rustup/toolchains/nightly-2026-07-19-aarch64-apple-darwin/lib/rustlib/<triple>`): 133 MB for `aarch64-apple-ios`, 136 MB for `aarch64-apple-ios-sim`, 130 MB for `aarch64-apple-ios-macabi`, and 156 MB for `x86_64-unknown-linux-gnu` — 555 MB of installed standard libraries.

## Work

If Tom accepts the policy: add `targets = ["aarch64-apple-ios", "aarch64-apple-ios-sim", "aarch64-apple-ios-macabi", "x86_64-unknown-linux-gnu"]` to `rust-toolchain.toml` with the reasoning the `components` comment models; extend `deps.sh` to read and verify that array the way it already reads `components`, so a missing target is a bootstrap diagnostic rather than a test failure; remove the `#[ignore]` and rewrite the paragraph in the test's doc comment that explains it; and drop `require_installed_target`'s "install it with `rustup target add`" advice down to an assertion, since the bootstrap now guarantees what it probes for.

If Tom declines: close this, and record the decline in the test's doc comment so the next reader does not re-derive the question.

## Closes when

Either the gate runs the five-target matrix on a host that `./deps.sh` alone prepared, or the decision to keep it hand-run is recorded where the test explains itself.
