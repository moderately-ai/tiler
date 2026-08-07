---
id: admit-the-conformance-crate-to-the-workspace
title: Admit the conformance crate to the workspace
status: done
priority: p1
dependencies: []
related: [decide-where-a-device-reaching-conformance-test-may-live, conform-the-bf16-vertical-end-to-end]
scopes: [implementation/frontend, implementation/conformance, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [architecture, workspace, conformance]
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

## Outcome — delivered 2026-08-07 at `5d31fd03`

`crates/tiler-conformance` is a workspace member. It holds **no items at all** — only a module header carrying the crate's boundary — which is the smallest useful slice this ticket was scoped for.

**The population check was watched failing before it was updated**, as required: the test reported 16 derived members against 15 expected and named `tiler-conformance` in the derived set. `EXPECTED_MEMBERS` is now `[&str; 16]`, and its doc comment counts the crate **apart from** the production twelve rather than folding it in — because no dependency edge points at it, and calling it production would claim a thirteenth layer that does not exist.

**Dependencies are normal rather than development**, which inverts what every other member does and is decided rather than accidental: nothing depends on this crate and nothing may, so its closure reaches no consumer, and a normal edge states what the crate is for where a dev edge would record only what its tests happen to reach. The set is the vertical a run crosses — artifact, build, compiler, IR, metal, metal-aot, reference, runtime — with `metal` behind `cfg(target_os = "macos")` so `cargo check --workspace` stays possible off Apple, which is what makes "runs the deterministic half and reports the measurement boundary unavailable" constructible rather than aspirational. `tiler-cache` and `tiler-digest` are reached transitively and deliberately not named.

**The frontend exclusion was verified by perturbation, not assumed.** Appending `[dev-dependencies] tiler = …` to the crate manifest made `no_package_depends_on_the_frontend` report `tiler-conformance -> tiler`, confirming the *dev* spelling trips it too, which is precisely the claim the crate header makes. Reverted and re-verified clean.

**One tooling behaviour isolated rather than assumed.** `tkt guard` warned that two changed files were covered by no scope. The worker probed it — a branch off the admission commit guarded against the *new* base reports `implementation/conformance`, verdict ok, tested twice to rule out added-versus-modified as the cause. **Guard reads the scope table from the base ref, so a scope a branch introduces is invisible to a guard run against a base predating it.** Self-referential and it disappears once landed; worth knowing before someone reads it as an under-declaration.

**Scopes added beyond the brief's two**, because the admission necessarily touches `Cargo.toml` and `Cargo.lock`: `implementation/conformance` and `implementation/workspace` exclusive, `implementation/cargo-lock` shared — matching the closest precedent, `admit-the-tiler-facade-and-proc-macro-crate-boundary`.

`make full` exit 0 on the branch and again on the merged tree: 2,963 workspace tests, 1,033 release numerical.

### Released, and one of them is a blocker

- **[`decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`](decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access.md)** — the inherited `unsafe_code = "forbid"` makes the device half unwritable: `MTLBuffer` storage is reachable only through the raw pointer `metal::Buffer::contents` returns, and `forbid` cannot be relaxed by an inner attribute at any scope. The worker inherited the table whole and **declined to pre-authorize a weaker level**, correctly: a lint relaxation is Tom's under ADR 0079. `conform-the-bf16-vertical-end-to-end` now depends on it.
- [`record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr`](record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr.md) — the component table is stale and there is no admission ADR, where every prior crate admission has one.
