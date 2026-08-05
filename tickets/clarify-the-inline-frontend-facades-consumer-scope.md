---
id: clarify-the-inline-frontend-facades-consumer-scope
title: Clarify the inline frontend facade's consumer scope
status: in-progress
priority: p1
dependencies: [reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission]
related: [accept-the-public-compiler-facade-boundary]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, frontend, consumer-neutral, public-boundary]
claimed_from: todo
assignee: agent-facade-scope
lease_expires_at: 1785894582
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

This ticket is deferred while the architecture-review code freeze applies,
because Rustdoc text lives in source files. Its activation trigger is Tom
lifting that freeze.

## Closes when

Every unqualified universal-consumer claim in the two frontend crates is either
narrowed to the inline frontend or supported by an accepted general facade, and
the Markdown packaging contract agrees with the code documentation.

## Trigger check log

- 2026-08-04 — **FIRED; reactivated to `todo`.** The trigger reads "Tom lifting that freeze", and the freeze is refuted rather than lifted: **no code freeze is recorded in any durable contract.** `grep -rn "freeze" AGENTS.md docs/` returns only registry-freeze prose in `docs/ir.md`, `docs/operation-extensions.md`, and the extension research records — nothing about a review freeze on source edits. The only two statements of it are this ticket and [`reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission`](reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission.md), and `git log -S'code freeze' -- tickets/ docs/` shows both entered the corpus in one commit, `52e088a2` (2026-08-04 14:55). **In the five hours after that commit, four merged commits added 697 lines of Rust doc comment to `crates/`** — `bff546b6` (16:18, 452), `f397c1c2` (16:29, 35), `946abff8` (17:07, 199), `da6f9a42` (20:19, 11) — each a descendant of `52e088a2`; reproduce one with `git show bff546b6 --unified=0 -- crates/ | grep -cE '^\+\s*//[/!]'`. Rustdoc in `crates/` is being edited freely, so the stated obstacle to this ticket's work does not exist. **Stated precisely so it can be refuted in one line:** this sweep asserts that no freeze constrains the edit, **not** that Tom lifted one — an acceptance is a relayed fact and none was relayed. If a freeze is in force, re-park this ticket in one status change; nothing has been released on the reactivation. The dependency [`reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission`](reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission.md) is `done`, and the public-boundary review obligation the body states is untouched: this ticket narrows scope claims in documentation and accepts no interface.
