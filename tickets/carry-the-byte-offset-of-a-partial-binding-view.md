---
id: carry-the-byte-offset-of-a-partial-binding-view
title: Carry a binding's byte offset so a partial view is packageable
status: todo
priority: p2
dependencies: []
related: [expose-the-dispatch-record-on-a-decoded-artifact, carry-reconstructable-kernel-programs-in-the-neutral-envelope]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, abi]
---
## User-visible outcome

A frontend can package a kernel whose binding addresses a *window* of a buffer — a slice, a sub-tensor, an offset view — and a loader binds it at the right byte. Today the artifact layer refuses any partial view outright (`PartialBindingView`), so every binding must cover its whole value; that refusal is the fail-closed placeholder this ticket replaces with a carried, proven offset.

Split from `expose-the-dispatch-record-on-a-decoded-artifact`, which encoded *which* buffer each ABI binding slot addresses and deliberately did not encode *where in it*.

**Fact — what the record carries today.** A binding row carries `BindingTarget` (a program input's `InputKey`, the `OutputKey` set publishing an output value, or `Internal`) and `accessible_bytes`, an ABI expression the builder proves equals `access.view().window().length`. The window's `offset` is not carried anywhere.

**Fact — an arbitrary window is representable one layer down.** `tiler_ir::program::KernelProgramBuilder::push_view` takes a `ByteWindow { offset, length }` and admits any window inside the value (`crates/tiler-ir/src/program/builder.rs:410`; find it with `grep -n "fn push_view" crates/tiler-ir/src/program/builder.rs`). `push_whole_view` (`:458`) is a convenience over it, not the only constructor.

**Fact — the artifact layer refuses the gap rather than approximating it.** `ArtifactBuildError::PartialBindingView` rejects a binding whose access does not address the whole of its value. That was the fail-closed choice: with a target and an extent but no offset, a loader binds the right buffer at the wrong place, which is a silently wrong result rather than a refusal.

## The work

Carry the offset as a second ABI expression on the binding row, with its own `AbiExprUse`, checked against `access.view().window().offset` exactly as `accessible_bytes` is checked against `.length`, folded into artifact identity, and validated on decode through the same use-site machinery (value type, availability phase, interface-only). Then remove `PartialBindingView` and its refusal.

An expression rather than a constant, for the same reason the extent is one: both are concrete at build time and both generalize over bound extents at run time, and a constant offset beside a formula extent would be two spellings of one contract.

This changes an encoded binding row. Advance the then-current manifest major
and artifact identity domain with the reason recorded at both sites; do not pin
the ticket to historical version numbers.

## Separate neighboring refusal

`AliasedInternalBinding` is an existing, separate refusal. An intermediate-role
fixture may make it convenient to test, but that regression is not required to
make partial offsets packageable and should not expand this ticket's outcome.

## Closes when

A binding may address a partial window, its offset is carried and proven against
the packaged program, `PartialBindingView` is gone, identity/schema versions
state the change, and `make full` passes.

## Work in flight — recorded 2026-07-28

**Fact — the work exists and has not landed.** It is uncommitted in the harness worktree `.claude/worktrees/agent-a0eb91e9c8f485c11`, on branch `tkt/carry-the-byte-offset-of-a-partial-binding-view` at HEAD `06af0c6`. That HEAD is **319 commits behind `main`** as of this record (`git rev-list --count HEAD..main`), so the branch base predates the audit-fix and Makefile-gate commits and the diff will not apply cleanly as-is. `git status --short` reports **17 modified files, 913 insertions / 111 deletions**, plus one untracked ticket.

**Fact — what the diff touches.** Six codec files, not four: `codec/decode.rs`, `codec/encode.rs`, `codec/model.rs`, `codec/tests.rs`, `codec/validate.rs`, `codec/view.rs`. Alongside them, `program/builder.rs`, `program/error.rs`, `program/mod.rs`, `program/model.rs`, `program/tests.rs`, `program/verify.rs`, `proof/mod.rs`, `docs/artifact-abi.md`, `prototypes/serial-sum-compile/src/bundle.rs`, `prototypes/serial-sum-run/src/main.rs`, and this ticket's own copy.

**Fact — the worktree carries a stale copy of this ticket.** Its version of `tickets/carry-the-byte-offset-of-a-partial-binding-view.md` was edited against that 319-commit-stale base and states an Outcome citing `MANIFEST_SCHEMA` at `3.0`/`v2`. The current ticket text contains no such citation, so **the worker's ticket edit will not merge cleanly onto this file** and must be reconciled by hand rather than taken from either side — the version numbers in particular have to be recomputed on the merged tree rather than picked from a branch.

**Fact — a well-specified follow-up exists and is untracked.** `tickets/carry-the-binding-offset-through-the-runtime-route.md` in the same worktree. Its substantive finding, which is the part worth keeping whatever happens to the rest: `RoutedBinding` (`crates/tiler-runtime/src/load/route.rs:102-107` on `main` today — the follow-up cites `:100-107`, which is off by two) publishes `binding`, `transport`, and `accessible_bytes` and no offset, and `place_bindings` evaluates only the extent, so `DecodedBinding::accessible_offset` exists and nothing reads it. A host given a `RoutedBinding` binds storage at byte zero whatever the artifact says. That is unreachable today only because `decode_artifact` refuses a multi-stage envelope through `tiler.artifact.feature.multi-stage-program` — a refusal owned by another layer — so on the day `carry-the-stage-execution-order-in-the-envelope` lifts it, this loader becomes silently wrong. The follow-up's `## Closes when` cites the retired Python gate (`uv run --locked python scripts/check_repository.py`) and needs updating to `make full` before it is filed.

**Status.** Frontmatter is not this record's to change; the ticket remains `in-progress` and the request to move it to `done` is left for the coordinator, which is correct in any case while the diff is unlanded and 319 commits stale.

## Graph maintenance

- **Start from the worktree diff, not from scratch** — see Work in flight above. Reconcile its stale ticket copy by hand; recompute any manifest/identity version numbers on the *merged* tree, never by picking a branch's number.
- **When this lands**: file the follow-up `carry-the-binding-offset-through-the-runtime-route` from the worktree's untracked draft (fixing its stale gate citation to `make full` and its `route.rs` line numbers, which are off by two). That follow-up is load-bearing: `RoutedBinding` publishes no offset and `place_bindings` evaluates only the extent, so the day `carry-the-stage-execution-order-in-the-envelope` lifts the multi-stage refusal, the loader becomes silently wrong without it. State that dependency on the follow-up explicitly.
- **Do not fold `AliasedInternalBinding` work in** — the body marks it a separate refusal; if you find yourself touching it, stop and file a separate ticket instead.
- **When the schema version advances**: record the reason at both the manifest-major site and the identity-domain site, and expect the producer determinism test and serial-sum artifact identity to move exactly once — a second movement means something else changed.
