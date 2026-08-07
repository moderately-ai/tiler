---
id: admit-the-conformance-crate-to-the-workspace
title: Admit the conformance crate to the workspace
status: in-progress
priority: p1
dependencies: []
related: [decide-where-a-device-reaching-conformance-test-may-live, conform-the-bf16-vertical-end-to-end]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, workspace, conformance]
claimed_from: todo
assignee: agent-conformance-crate
lease_expires_at: 1786118584
---
## User-visible outcome

`crates/tiler-conformance` exists as a workspace member, so an end-to-end conformance run that must reach both a real device and the reference oracle has a home that is inside `make full` and inside the crates' style gate.

## The decision this implements

**Tom accepted a new workspace member on 2026-08-07** on [`decide-where-a-device-reaching-conformance-test-may-live`](decide-where-a-device-reaching-conformance-test-may-live.md), rejecting `prototypes/serial-sum-run` on the ground that prototypes are throwaway and **everything long-term holding must live in a proper `tiler` crate.** Read that node's Decided section in full before starting; it records why `crates/tiler` and `crates/tiler-runtime` are ruled out by the code rather than by preference.

## Smallest useful slice — and it is deliberately small

The crate exists, is wired into every authority that must know about it, and **hosts nothing yet.** No test migrates under this ticket. [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) is the first content and has its own ticket; [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md) decides what follows. A crate admitted with a migration attached would be deciding the survey's answer before it ran.

## Four authorities that must be updated in the same commit

`AGENTS.md` states that a crate-admission ticket "must atomically add the real workspace package and add or move its mapping". Each of these fails deliberately when a member is added, which is the intended behaviour rather than an obstacle:

1. **`crates/tiler/tests/workspace_population.rs`** — `EXPECTED_MEMBERS` is a hand-maintained list of 15, derived against `cargo metadata`. It fails on a missing *or* unexpected member. Update the count and the list, and the doc comment's "Twelve production crates plus the three prototype proof executables" phrasing with it.
2. **`ticketsplease.toml`** — add `"implementation/conformance" = ["crates/tiler-conformance/**"]` under `[scopes]`, and the `[scope_crates]` mapping so the Rust backend can expand reverse-dependents. The file's own comment requires the mapping be added atomically with the package.
3. **`crates/tiler/tests/dependency_direction.rs`** — verify rather than assume. The new crate **must not depend on `tiler` or `tiler-macros`**: `no_package_depends_on_the_frontend` asserts no non-frontend package holds such an edge, and the lockfile it reads merges dev-dependencies. So conformance builds its programs through `tiler-ir` and the compiler directly, never through the frontend facade. **State that constraint in the crate's own header**, because it is a real limitation: the inline-macro path is not reachable from here and stays covered by `crates/tiler/tests/facade/`.
4. **Workspace lints** — the crate inherits workspace Rust and Clippy lints via `[lints]`. `AGENTS.md` notes inheritance is not enforced, so this must be inspected rather than assumed.

## What the crate declares

Whatever the BF16 vertical will need, and no more: the compile-and-package stack, the runtime, the reference oracle, and the macOS-gated `metal` crate. Prefer normal dependencies over dev-dependencies here — unlike every other crate in the workspace, **nothing depends on this one**, so its dependency closure costs no consumer anything, and a normal edge states the crate's purpose where a dev edge would hide it.

## Write the crate's boundary into its own header

Three anti-goals, load-bearing rather than aspirational, from the accepted decision:

- **Not a second semantic authority.** `tiler-reference` remains the oracle; conformance *uses* it. The moment conformance computes an expected value itself, that is the authority substitution ADR 0076 forbids.
- **Not a benchmark harness.** Timing has its own discipline — idle host, warm-up, repetitions, noise controls — and mixing gated correctness with measurement would make the gate flaky and the measurements untrustworthy.
- **Not a home for layer-local tests.** It owns *cross-layer executed evidence*. A unit test of one layer stays in that layer's crate. Without this line it becomes the place tests go when nobody wants to decide.

And one hard requirement to state now, because the first content depends on it: **a host without the measured environment runs the deterministic reference and structural half and reports the measurement boundary as unavailable — never a silent skip, never a claimed pass.**

## Required evidence

- `cargo metadata` reports the member; `workspace_population.rs` passes with the updated list and **was observed failing before it was updated**.
- `tkt guard` resolves the new scope — verify by running it against a branch touching a file in the new crate.
- `make full` green, including the doc build for the new crate under `-D warnings`.

## Reserved

The crate's **public surface**, if it grows one, is a separate boundary under ADR 0075. Admitting the member is not accepting an API. Keep everything `pub(crate)` or test-only until there is a reason otherwise, and report any public item so an acceptance node can be filed.

## Graph maintenance

Filed 2026-08-07 by the coordinator at Tom's acceptance. Kept separate from both its first content and its migration survey so that admitting the member is reviewable on its own.
