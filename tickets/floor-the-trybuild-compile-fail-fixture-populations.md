---
id: floor-the-trybuild-compile-fail-fixture-populations
title: Floor the trybuild compile-fail fixture populations
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-floor-the
lease_expires_at: 1786162140
---
## Four compile-fail globs can silently match nothing

Coordinator-verified. `Makefile` floors the *pass* glob — `test $(ls crates/tiler/tests/facade/pass/*.rs | wc -l) -eq 10` — with the reasoning written beside it: *"a glob that has stopped matching produces no complaints, which is indistinguishable from a population that is clean."*

**That reasoning was never applied to the fail globs.** Four harnesses register `compile_fail` with no floor:

- `crates/tiler/tests/facade.rs` — `cases.compile_fail("tests/facade/fail/*.rs")`
- `crates/tiler-ir/tests/index_region_ui.rs` — `tests/index-region/fail/*.rs`
- `crates/tiler-ir/tests/shape_evidence_ui.rs` — `tests/shape-evidence/fail/*.rs`
- `crates/tiler-ir/tests/typed_handles.rs` — `tests/typed-handles/fail/*.rs`

**26 fixtures are unfloored**: 9 under `crates/tiler/tests/facade/fail/` and 17 across the three `tiler-ir` directories.

**The auditor executed the failure**, rather than reasoning about it: a scratch crate with a nonexistent `fail/` directory and a populated `pass/` reports `test result: ok. 1 passed` with **no diagnostic of any kind**. In trybuild 1.0.118, `expand_globs` pushes nothing for a zero-match glob, and `Runner::run` only panics when `failures > 0`. Because each harness registers a `pass` glob *and* a `fail` glob on one `TestCases`, even trybuild's `no_tests_enabled()` notice — itself only a `println!` — cannot fire when one half collapses.

**Why this matters more than an ordinary missing floor:** `AGENTS.md` keeps `cargo test --workspace --doc` in the gate specifically because it *"preserves ADR 0051 compile-fail evidence"*. That evidence is exactly what these globs carry. Only 2 of the 9 facade fixtures are pinned by name anywhere; the other 7 and all 17 `tiler-ir` fixtures are pinned by nothing.

## What to build

Floor each population, in the same shape and place the `pass` floor already uses so the two read as one discipline rather than two mechanisms. Rename a directory or typo a glob and the gate must **fail by name**, saying which population starved.

**Watch each floor fail separately** — one glob at a time, subject perturbed rather than assertion — and quote the message. A floor that only fires when all four collapse is not four floors.

## A methodological note worth carrying

A subagent on this audit cleared these globs as "all non-empty". **Present non-emptiness is not the property**; the absence of a floor is. That distinction is the whole finding, and it is the same one that made the citation checker's anchor path silently unexercised.

## Scope note

`Makefile` is `implementation/workspace`. If the floors live there, the delta **cannot carry the gate** — run `make full`.
