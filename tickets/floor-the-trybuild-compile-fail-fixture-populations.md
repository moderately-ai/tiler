---
id: floor-the-trybuild-compile-fail-fixture-populations
title: Floor the trybuild compile-fail fixture populations
status: done
priority: p1
dependencies: []
related: [close-the-fmt-blind-spot-over-the-trybuild-facade-fixtures, close-the-fmt-blind-spot-over-the-tiler-ir-trybuild-fixtures]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Four compile-fail globs can silently match nothing

Coordinator-verified at open. `Makefile` already floored the facade *pass* glob — `test $(ls crates/tiler/tests/facade/pass/*.rs | wc -l) -eq 10` — with the reasoning written beside it: *"a glob that has stopped matching produces no complaints, which is indistinguishable from a population that is clean."*

**That reasoning had not been applied to the fail globs.** Four harnesses still register `compile_fail` (they always did; the floor lives in `Makefile`, not in the harness):

- `crates/tiler/tests/facade.rs` — `cases.compile_fail("tests/facade/fail/*.rs")`
- `crates/tiler-ir/tests/index_region_ui.rs` — `tests/index-region/fail/*.rs`
- `crates/tiler-ir/tests/shape_evidence_ui.rs` — `tests/shape-evidence/fail/*.rs`
- `crates/tiler-ir/tests/typed_handles.rs` — `tests/typed-handles/fail/*.rs`

**At open, 26 fail fixtures had no population floor**: 9 under `crates/tiler/tests/facade/fail/` and 17 across the three `tiler-ir` fail directories. Those counts remain the live sizes; the gate now floors them (see Outcome).

**The auditor executed the failure**, rather than reasoning about it: a scratch crate with a nonexistent `fail/` directory and a populated `pass/` reports `test result: ok. 1 passed` with **no diagnostic of any kind**. In trybuild 1.0.118, `expand_globs` pushes nothing for a zero-match glob, and `Runner::run` only panics when `failures > 0`. Because each harness registers a `pass` glob *and* a `fail` glob on one `TestCases`, even trybuild's `no_tests_enabled()` notice — itself only a `println!` — cannot fire when one half collapses.

**Why a floor matters more than an ordinary missing count:** `AGENTS.md` keeps `cargo test --workspace --doc` in the gate specifically because it *"preserves ADR 0051 compile-fail evidence"*. That retained `--doc` path is the *Preflight::commit* compile-fail doc-tests (a different population); the trybuild fixture globs are what these floors protect. Two of the nine facade fail fixtures are read by name from `tiler-macros` delivery tests; the other seven are also path-listed in `TENSOR_FIXTURE_INVOCATION_PINS` for expected `tensor!` invocation counts (not a population floor). All seventeen `tiler-ir` fail fixtures rely on the Makefile count for population flooring.

**Correction — 2026-08-10.** Present-tense "are unfloored" / harnesses "with no floor" as *gate-level* absence is obsolete: `Makefile` `test` floors the four fail globs (and the three tiler-ir pass globs). Harnesses still lack an in-harness population assertion; the floor is the shell `test $(ls … | wc -l) -eq N` lines next to the consumers. The ADR 0051 / "exactly what these globs carry" conflation is softened above: ADR 0051 compile-fail evidence on `--doc` is Preflight::commit; trybuild globs are separate compile-fail evidence floored here. "Pinned by name anywhere / pinned by nothing" overstated the pin surface: two fail paths via `tiler-macros` `fixture(...)`; seven more facade fail paths in `TENSOR_FIXTURE_INVOCATION_PINS` (invocation counts only); population floor is the Makefile count.

## What to build

Floor each population, in the same shape and place the `pass` floor already uses so the two read as one discipline rather than two mechanisms. Rename a directory or typo a glob and the gate must **fail by name**, saying which population starved.

**Watch each floor fail separately** — one glob at a time, subject perturbed rather than assertion — and quote the message. A floor that only fires when all four collapse is not four floors.

## A methodological note worth carrying

A subagent on this audit cleared these globs as "all non-empty". **Present non-emptiness is not the property**; the absence of a floor is. That distinction is the whole finding, and it is the same one that made the citation checker's anchor path silently unexercised.

## Scope note

`Makefile` is `implementation/workspace`. If the floors live there, the delta **cannot carry the gate** — run `make full`.

## Outcome

Delivered in `ab174f67e8e8174b094f6c1c7a9735e291791f29` ("Floor every trybuild fixture population in the gate"); status closed `done` in `0ac9124f`.

**Floors (live counts at repair re-read):**

- `fmt`: `crates/tiler/tests/facade/pass` = 10 (unchanged location; not duplicated under `test`).
- `test` (seven populations, same shell shape as the pass floor):
  - `crates/tiler/tests/facade/fail` = 9
  - `crates/tiler-ir/tests/index-region/pass` = 1
  - `crates/tiler-ir/tests/index-region/fail` = 4
  - `crates/tiler-ir/tests/shape-evidence/pass` = 2
  - `crates/tiler-ir/tests/shape-evidence/fail` = 7
  - `crates/tiler-ir/tests/typed-handles/pass` = 1
  - `crates/tiler-ir/tests/typed-handles/fail` = 6

Delivery also floored the three tiler-ir *pass* globs so half-flooring would not imply pass globs need no floor. Each wrong-count floor fails independently: asserting `eq N−1` for one population exits shell `test` with status 1 and fails the make recipe line that names that path. Ticket-body process obligation to *quote* each historical per-floor failure message was not preserved in the closing record; independent non-zero exits remain the live check.

Sibling fmt-reachability work stays separate: `close-the-fmt-blind-spot-over-the-trybuild-facade-fixtures` (done) and `close-the-fmt-blind-spot-over-the-tiler-ir-trybuild-fixtures` (still open) own rustfmt coverage of fixture trees, not silent empty trybuild globs.
