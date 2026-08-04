---
id: clarify-the-inline-frontend-facades-consumer-scope
title: Clarify the inline frontend facade's consumer scope
status: deferred
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

This ticket is deferred while the architecture-review code freeze applies,
because Rustdoc text lives in source files. Its activation trigger is Tom
lifting that freeze.

## Closes when

Every unqualified universal-consumer claim in the two frontend crates is either
narrowed to the inline frontend or supported by an accepted general facade, and
the Markdown packaging contract agrees with the code documentation.
