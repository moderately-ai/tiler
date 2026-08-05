---
id: clarify-the-inline-frontend-facades-consumer-scope
title: Clarify the inline frontend facade's consumer scope
status: done
priority: p1
dependencies: [reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission]
related: [accept-the-public-compiler-facade-boundary]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, frontend, consumer-neutral, public-boundary]
---
## User-visible outcome

Rust API documentation distinguishes the accepted inline `tensor!` frontend
facade from the still-separate general semantic-graph and compiler facades. A
reader no longer infers that arbitrary program authors must use the macro or
that every `tiler-*` crate is an implementation detail they may not name.

## Evidence

- **Fact:** `crates/tiler/src/lib.rs` begins "The one crate a Tiler consumer
  depends on" and says every other workspace crate is internal.
- **Fact:** the accepted `tiler`/`tiler-macros` topology is specifically the
  inline Rust frontend and generated-token path.
- **Fact:** general semantic construction and compiler entrypoints currently
  live in `tiler-ir` and `tiler-compiler`; acceptance of a coherent public
  compiler facade remains a separate boundary.
- **Inference:** the unqualified word "consumer" overstates the inline facade's
  scope and conflicts with the consumer-neutral architecture even though the
  dependency direction itself remains correct.

Read the complete crate/module documentation and accepted frontend decisions.
Correct only scope claims: inline-frontend consumers name `tiler`; general
compiler consumers use the separately governed graph/compiler surfaces until a
later facade is accepted. Do not redesign dependencies, accept the pending
compiler facade, or change generated paths.

This ticket was filed deferred on the premise that an architecture-review code
freeze barred editing source files, with "Tom lifting that freeze" as its
activation trigger. The premise is refuted rather than satisfied — see the
trigger check log below, and the independent re-check recorded in the outcome —
so the deferral is spent and the work was done under no freeze.

## Closes when

Every unqualified universal-consumer claim in the two frontend crates is either
narrowed to the inline frontend or supported by an accepted general facade, and
the Markdown packaging contract agrees with the code documentation.

## Outcome

The two frontend crates' documentation now scopes itself the way
[`docs/architecture.md`](../docs/architecture.md) already did, and states the
compiler-facade boundary as undecided rather than as either settled direction.

**Fact — what changed.** `crates/tiler/src/lib.rs` opens "The one crate a
consumer of Tiler's inline Rust frontend depends on", keeps ADR 0088 item 1's
closure claim scoped to *that* contract, and gains a section saying that a
program not written as a `tensor!` region has no entry point here, that the
general graph and compiler surfaces live in `tiler-ir` and `tiler-compiler`
today, and that `"internal" is the wrong reading of the other members` — what is
decided is that the inline frontend routes through none of them, not what a
non-inline consumer may name. `crates/tiler-macros/src/lib.rs` narrows "the
durable facade a consumer imports" to an inline-frontend consumer and adds the
same undecided-boundary sentence. Two smaller scope claims were narrowed: the
`runtime` module's "the only one a consumer declares" and the "internal crates
the consumer did not declare" clause in the crate header.

**Fact — the pending boundary is described, not decided.** Both new sections name
`tiler_compiler::session` as a reviewed experimental draft rather than an
accepted API and point at
[`accept-the-public-compiler-facade-boundary`](accept-the-public-compiler-facade-boundary.md),
which matches `docs/correctness-and-testing.md:117`. The diff adds and removes
only `//!` and `///` lines, so no item's signature, visibility, or path moved and
no interface was accepted here.

**Fact — the Markdown contract already agreed and still does.**
`docs/architecture.md:331` describes `tiler` as "The inline Rust frontend's
import path", and `:424` states that this "does not make `tiler` the accepted
facade for consumers that construct and compile arbitrary semantic programs".
The code documentation now says the same thing; no Markdown edit was required,
and none was in scope.

**Fact — the freeze was re-checked independently before editing.**
`grep -rn "code freeze\|review freeze\|architecture-review" .` (excluding the
vendored LLVM sources) returns only this ticket, the sweep ticket, and the `done`
dependency. No durable contract records a freeze.

**Measurement — verification.** `cargo fmt -p tiler -p tiler-macros -- --check`;
`cargo check --workspace`; `RUSTDOCFLAGS='-D warnings' cargo doc -p tiler
--no-deps`; `cargo test -p tiler -p tiler-macros --doc` (1 passed — the crate
header's example still compiles and runs); `cargo clippy -p tiler -p tiler-macros
--all-targets -- -D warnings`; `cargo nextest run -p tiler -p tiler-macros` (196
passed, 1 skipped).

## Trigger check log

- 2026-08-04 — **FIRED; reactivated to `todo`.** The trigger reads "Tom lifting that freeze", and the freeze is refuted rather than lifted: **no code freeze is recorded in any durable contract.** `grep -rn "freeze" AGENTS.md docs/` returns only registry-freeze prose in `docs/ir.md`, `docs/operation-extensions.md`, and the extension research records — nothing about a review freeze on source edits. The only two statements of it are this ticket and [`reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission`](reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission.md), and `git log -S'code freeze' -- tickets/ docs/` shows both entered the corpus in one commit, `52e088a2` (2026-08-04 14:55). **In the five hours after that commit, four merged commits added 697 lines of Rust doc comment to `crates/`** — `bff546b6` (16:18, 452), `f397c1c2` (16:29, 35), `946abff8` (17:07, 199), `da6f9a42` (20:19, 11) — each a descendant of `52e088a2`; reproduce one with `git show bff546b6 --unified=0 -- crates/ | grep -cE '^\+\s*//[/!]'`. Rustdoc in `crates/` is being edited freely, so the stated obstacle to this ticket's work does not exist. **Stated precisely so it can be refuted in one line:** this sweep asserts that no freeze constrains the edit, **not** that Tom lifted one — an acceptance is a relayed fact and none was relayed. If a freeze is in force, re-park this ticket in one status change; nothing has been released on the reactivation. The dependency [`reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission`](reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission.md) is `done`, and the public-boundary review obligation the body states is untouched: this ticket narrows scope claims in documentation and accepts no interface.
