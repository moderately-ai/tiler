---
id: admit-the-tiler-facade-and-proc-macro-crate-boundary
title: Admit the tiler facade and proc-macro crate boundary
status: todo
priority: p1
dependencies: []
related: [prototype-inline-proc-macro-frontend]
scopes: [implementation/frontend, implementation/workspace]
shared_scopes: [implementation/cargo-lock, project/tickets]
paths: []
tags: []
---
## User-visible outcome

Consumers import one ordinary `tiler` facade and call `tiler::tensor!`; the procedural implementation lives in a separate `tiler-macros` proc-macro crate, while normal runtime/frontend types remain available from the facade.

## Implementation keys

The approved surface fixes the public path as `tiler::tensor!`. A proc-macro crate cannot be the durable normal-type/runtime facade because Rust restricts what a proc-macro crate exports. A standalone `tiler-macros` crate would either force users to depend on internal crates named by generated tokens or change the approved import path. A normal `tiler` facade re-exporting the macro from `tiler-macros` is the standard dependency direction and keeps generated paths stable.

Admit both workspace members atomically. `tiler-macros` owns token parsing, span mapping, and expansion; `tiler` owns stable re-exports and the consumer-visible frontend/runtime traits selected by their dedicated boundary tickets. Neither crate creates a second semantic operation vocabulary, invokes runtime JIT, scans source, requires `build.rs`, or hides a generated dependency the consumer did not receive.

## Public boundary for Tom

Ratify the two-crate topology and public `tiler::tensor!` path before workspace admission. The exact manifests, dependency direction, re-export, minimal public module tree, and one compile-pass consumer are the review packet. A crate admission does not stabilize the macro grammar or runtime adapter beyond their separately accepted tickets.

## Ratification (2026-07-30)

Tom approved the `tiler` normal facade plus `tiler-macros` proc-macro implementation topology and the public `tiler::tensor!` path. Implementation may proceed with the dependency direction and exclusions above; the exact manifest/re-export diff remains part of acceptance evidence rather than a reopened topology choice.

## Closes when

Tom ratifies the topology; both members, lockfile, and ticketsplease scope ownership land atomically; dependency checks prove compiler/IR remain frontend-independent; a compile-pass fixture imports only the facade; a deliberate missing re-export or wrong generated path fails; and targeted checks plus `make full` pass.

## Graph maintenance

- Add scope mappings for `crates/tiler/**` and `crates/tiler-macros/**` in the same commit that admits the members; paths alone do not make later frontend work schedulable.
- Keep `define-inline-symbol-binding-and-runtime-value-adaptation` and `promote-artifact-family-selection-for-the-frontend` dependent on this admission rather than relying on shared-scope serialization.
- Do not close `prototype-inline-proc-macro-frontend` from this ticket; it consumes the admitted facade after its separate symbol/value and artifact-family prerequisites land.
