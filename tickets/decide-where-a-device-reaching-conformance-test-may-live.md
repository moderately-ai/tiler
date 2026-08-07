---
id: decide-where-a-device-reaching-conformance-test-may-live
title: Decide where a device-reaching conformance test may live
status: done
priority: p2
dependencies: []
related: [conform-the-bf16-vertical-end-to-end, dispatch-a-multi-entry-bundle-on-hardware]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, architecture]
---
## The decision

**Only Tom closes this.** Where may a test that reaches a real Metal device live, so that an end-to-end conformance run is a red test in `make full` rather than a hand-run spike?

## Why this node exists

**Fact — exactly one workspace member can reach a device, and it is not a crate.** `grep -rn 'metal\.workspace\|^metal = ' --include=Cargo.toml .` returns the workspace pin, `prototypes/serial-sum-run/Cargo.toml`, and two *out-of-workspace* spikes. `prototypes/serial-sum-run` maps to `implementation/runtime`, already depends on `tiler-artifact`, `tiler-build`, `tiler-compiler`, `tiler-ir`, `tiler-metal`, `tiler-metal-aot`, `tiler-reference`, `tiler-runtime` and the macOS-gated `metal`, and its `[[bin]]` carries `test = true` — so a `#[test]` there is reached by `cargo nextest run --workspace` and therefore by `make full`. It is the only place in the repository where "a regression anywhere in the vertical is a red test" is presently constructible.

**Fact — the obvious alternative is an architecture violation.** `crates/tiler-reference` depends on `tiler-ir` alone. Adding a device edge there would put a live backend under the target-independent oracle, which is the dependency inversion that crate exists to prevent. A worker may not make that change, and it is not a scope question but an architecture one.

**Fact — this has now blocked work twice and been relayed rather than decided.** [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) recorded it as "Block 1" on 2026-08-06 and could not proceed; the block was carried in prose rather than as an edge until 2026-08-07, when the coordinator filed this node so the dependency is real and the ticket stops looking dispatchable-but-stuck.

## What each answer enables and prevents

- **A `#[test]` in `prototypes/serial-sum-run`.** Enables the end-to-end run today, inside `make full`, with no dependency change anywhere. Prevents nothing structurally. Strongest counterpoint: it makes a *prototype* the home of a gating correctness test, and `AGENTS.md` treats prototypes as excluded from the crates' style gate — so a load-bearing conformance test would live in the one tree the repository deliberately holds to a lower standard.
- **A new workspace member for device-reaching conformance tests.** Enables a proper home with the crates' own gate applied, and keeps prototypes exploratory. Prevents nothing. Strongest counterpoint: a new crate is a public-boundary and workspace-shape change, and `AGENTS.md` warns that premature crates harden unsupported assumptions — this one would exist to hold tests rather than to express a component.
- **Keep device conformance in `spikes/` and out of the gate.** Enables the evidence without touching workspace shape. Strongest counterpoint: it forfeits the whole user-visible outcome of the blocked ticket — a spike is run by hand, so a regression is not a red test, which is precisely what that ticket exists to fix.

**Recommendation: the `#[test]` in `prototypes/serial-sum-run`**, on the ground that it is the only option that delivers the outcome now and the only one that requires no reserved change. The prototype-standard objection is real but is about where the file sits rather than what it checks, and it is cheaper to move a test later than to leave the composition untested — which is the gap `docs/dtype-support.md` already records for the U4/F32 vertical.

## Closes when

Tom names a home, the blocked ticket's edge on this node is discharged, and any workspace-shape change the answer requires is released to its own ticket rather than landed here.

## Graph maintenance

Filed 2026-08-07 by the coordinator on Tom's instruction that anything remaining blocked carry a proper dependent ticket or trigger rather than prose.

## Decided — a new conformance crate, 2026-08-07

**Tom answered on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator: **a new workspace member, `crates/tiler-conformance`.** The prototype option was rejected on the ground that `prototypes/` is by definition throwaway, short-term code, and **everything long-term holding must live in a proper `tiler` crate.** That reasoning generalizes past this ticket and is the reason the survey below was asked for in the same breath.

### Why the alternatives were eliminated, at source

- **`prototypes/serial-sum-run`** — mechanically capable and rejected on principle. A gating correctness test whose regression is the whole point cannot live in a tree the repository excludes from the crates' style gate and treats as disposable.
- **`crates/tiler`** — ruled out by the code, not by taste. `crates/tiler/tests/dependency_direction.rs` forbids the facade an edge to `tiler-metal-aot` (ADR 0077 item 4, accepted 2026-07-31), and it reads `Cargo.lock` precisely because the lockfile "merges normal, build, and dev dependencies into one edge list" — so even a dev-dependency trips it. Compiling a metallib needs that driver.
- **`crates/tiler-runtime`** — ruled out by its own stated boundary: its dev-dependency comment records that its tests must not reach `tiler-compiler`, because "a loader that could rebuild a plan instead of validating the one it was handed is the boundary the crate split exists to enforce, and that stays true of its tests." An end-to-end run must compile.
- **`crates/tiler-build` with dev-dependencies** — mechanically fine and the cheapest option, rejected because it puts the *consume* half of an end-to-end test inside the crate whose job is offline *production*.

### What the crate is for, beyond this test

A target profile is a set of claims; conformance is what refutes them. Tiler has the declaring half built — typed profiles, measured rows, numerical realizations, feasibility predicates — and no home for the refuting half. The grid-axis row is the worked example: declared `4`, measured `268,435,456`, refuted by hand in a spike and transcribed into the declaration manually, with nothing gating it or re-running it.

The evidence that a component is missing rather than a file being homeless: five open conformance tickets and no two share a scope set — three are `implementation/compiler`, one adds `implementation/reference` + `contracts/numerics` + `implementation/runtime`, one adds `research/scheduling` — with the conformance oracle plumbing living inside the compiler because there is nowhere else to put it.

### Released work

- [`admit-the-conformance-crate-to-the-workspace`](admit-the-conformance-crate-to-the-workspace.md) — the crate itself, smallest useful slice.
- [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md) — the spike Tom asked for in the same answer: what else should move in, filed as future tickets rather than migrated now.

Nothing migrates under this node. [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) is re-pointed at the admission ticket.
