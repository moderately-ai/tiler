---
id: refresh-the-inline-aot-vertical-status-and-remaining-checks
title: Refresh the inline AOT vertical's status line and remaining-checks list
status: done
priority: p2
dependencies: []
related: [avoid-toolchain-resolution-on-a-warm-expansion-cache-hit, correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, macro-aot, status-drift]
---
## User-visible outcome

`docs/integration/frontends.md` stops contradicting itself: its status line and its must-still-demonstrate list agree with the correction and the measurement table the same file carries.

## Why this exists

**Fact — the status line is falsified by the same file.** `docs/integration/frontends.md:15` reads "**Status:** accepted inline AOT contract; rust-analyzer performance remains unmeasured." The measurement table at `:359-369` records `rust-analyzer 1.97.0-nightly (8b03437a 2026-05-12)` on macOS 27.0 arm64 / Apple M4 Max, 2026-08-01, with a live in-region edit `semanticTokens` round trip at 137–217 ms delivering against 10–16 ms fallback-only. It is measured.

**Fact — two clauses in the must-still-demonstrate list are falsified the same way.** `:439-444` reads "The first vertical implementation slice must still demonstrate an actual Tiler macro compiling, embedding, loading, and dispatching a one- and multi-entry bundle; **a production warm cache hit invoking no `xcrun`**; source-spanned retained MSL diagnostics; and the non-Apple semantic fallback path without consumer setup. **rust-analyzer cold/warm performance also remains unmeasured because the component was unavailable.**" The first is unreachable by construction under the correction at `:349-357` — a warm expansion resolves the toolchain, because the compiler fingerprint is an input to compilation identity and `Toolchain::prepare` must observe it before a lookup exists to skip. The second is contradicted by the table twenty lines earlier, whose own preamble records that the analyzer binary and proc-macro server "were both already present" — the unavailability that clause cites is over.

**Inference — sweep the list, do not patch two clauses.** Several other items in that list have landed since it was written, and a repair that fixes exactly the two an audit named leaves the same defect class in the same paragraph. Read every item against the file's own later sections and against `crates/`.

## Boundaries

- Scope is `contracts/integrations` — this file and its siblings. The parallel unreachable requirement at `docs/correctness-and-testing.md:336` belongs to [`correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract`](correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract.md), which holds `contracts/numerics`; do not reach into that file.
- **Coordinate with [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md):28**, which holds `contracts/integrations` and is required to sweep two *other* named sentences in this file — the symbolic-region family-selection refusal and the two sentences a previous ticket flagged without scope to fix. The two sweeps do not overlap; confirm that by reading its bullet before starting, and do not absorb its work or leave it re-fixing yours.
- A measurement is bounded by its environment. Replacing "unmeasured" with a number means carrying the number's host, date, toolchain, and procedure with it, not asserting a portable property.

## Closes when

`docs/integration/frontends.md:15` states what is actually unmeasured; no item in the must-still-demonstrate list is unreachable by construction or already discharged elsewhere in the same file; every remaining item was checked rather than the two an audit named; and each replacement measurement carries its exact environment and procedure.

## Outcome

**Swept, not patched.** The closing paragraph of "Feasibility evidence and remaining vertical checks" became a four-way disposition — landed with citation, withdrawn as unreachable, outstanding with an owner, parked with a trigger — under a new `### What the first vertical slice has and has not demonstrated` heading. Every clause of the original list was checked against `crates/` and `spikes/` rather than against the audit's two names.

| original clause | disposition | evidence |
| --- | --- | --- |
| macro compiling, embedding, loading, dispatching a **one**-entry bundle | landed | `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs` (producer half, inside `rustc`); `spikes/runtime/inline-dispatch` (load, lookup, pipeline, dispatch on Apple M4 Max, macOS 27.0 26A5388g, 2026-08-01, bit-exact against the consumer's own `f32`) |
| …and a **multi**-entry bundle | outstanding | today's regions package one entry (`1/1 entry(ies) encoded`); the multi-entry route is exercised only against `adapter_route`'s hand-built fixture. Filed [`package-a-multi-entry-bundle-from-one-expansion`](package-a-multi-entry-bundle-from-one-expansion.md) |
| a production warm cache hit invoking no `xcrun` | withdrawn | unreachable by construction per the file's own `### Why a warm expansion resolves the toolchain`; what survives is `tiler_macros::aot::tests::the_second_expansion_of_one_subject_compiles_nothing` |
| source-spanned retained MSL diagnostics | half landed, half outstanding | `DriverError::ToolFailure` retains bounded tool bytes and `aot::retained` emits them at the invocation span (`family_cfg_matching_family_retains_its_diagnostic` + golden) — but the exercised failure is `ToolchainUnavailable`, no MSL line maps to a region construct, and the cache-retention permission is unimplemented. Filed [`retain-and-attribute-a-real-msl-failure-through-an-expansion`](retain-and-attribute-a-real-msl-failure-through-an-expansion.md) |
| the non-Apple semantic fallback path without consumer setup | landed at check level, with the boundary stated | `every_emitted_shape_compiles_as_the_five_target_matrix_says` compiles the emitter's own output for `x86_64-unknown-linux-gnu` and three Apple cross-targets under `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, 2026-08-01, `--emit=metadata` only — no link, no SDK. "Without consumer setup" is checkable by reading the fixture crate |
| rust-analyzer cold/warm performance unmeasured because the component was unavailable | warm landed, cold parked | the table twenty lines above measures it (`rust-analyzer 1.97.0-nightly (8b03437a 2026-05-12)`, 137–217 ms in-region vs 10–16 ms fallback-only); the analyzer and proc-macro server "were both already present", so the unavailability clause is void. No cold-cache IDE wall-clock exists; parked against this section's own analysis-stub trigger |

**Items added that the original list never carried**, because they are the same slice's evidence and were landing while it went stale: the consumer storage seam (`route-an-embedded-artifact-through-a-consumer-storage-seam`, 2026-08-01), and the multi-payload envelope (`carry-one-payload-per-artifact-family-in-one-envelope`, `tiler.artifact-program.v13`, `one_envelope_carries_one_payload_per_artifact_family` — machinery landed, second *measurement* parked).

**Status line.** `**Status:** accepted inline AOT contract; rust-analyzer performance remains unmeasured` → `**Status:** accepted inline AOT contract; one macOS family delivers end to end on one measured host, multi-family delivery is parked on a second measured Apple family, and no non-macOS host has been measured`. The three clauses are the three things actually open; the falsified one is gone.

**Coordination boundary held.** [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md):28 owns the symbolic-region family-selection refusal and the two sentences `prototype-inline-aot-integration-proof`:164 flagged ("a statement selecting a family is refused", "the cache-root resolver is uncalled"). Those live at `:295` and in the status paragraph at `:17`; neither was touched, and neither appears in the remaining-checks list this ticket swept — the sweeps do not overlap.

**Out-of-scope defect filed.** `crates/tiler-macros/src/aot.rs:41-43,52` still says a resolution runs "five `xcrun` queries — two `--find` and three `--show-sdk-*`"; `driver.rs:86-97` makes four (two `--find`, `--show-sdk-version`, `--show-sdk-build-version`), and the contract itself already records "A resolution now makes four." Scope is `implementation/frontend`, so it is [`correct-the-stale-xcrun-count-in-the-macro-aot-module-docs`](correct-the-stale-xcrun-count-in-the-macro-aot-module-docs.md) rather than an edit here.

**Links.** All 24 links in the file counted and resolved: 19 local targets all exist, 1 in-document anchor resolves against the file's 18 headings, 4 external.
