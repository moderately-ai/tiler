---
id: carry-the-byte-offset-of-a-partial-binding-view
title: Carry a binding's byte offset so a partial view is packageable
status: in-progress
priority: p2
dependencies: []
related: [expose-the-dispatch-record-on-a-decoded-artifact, carry-reconstructable-kernel-programs-in-the-neutral-envelope]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, abi]
---
Split from `expose-the-dispatch-record-on-a-decoded-artifact`, which encoded *which* buffer each ABI binding slot addresses and deliberately did not encode *where in it*.

**Fact — what the record carries today.** A binding row carries `BindingTarget` (a program input's `InputKey`, the `OutputKey` set publishing an output value, or `Internal`) and `accessible_bytes`, an ABI expression the builder proves equals `access.view().window().length`. The window's `offset` is not carried anywhere.

**Fact — an arbitrary window is representable one layer down.** `tiler_ir::program::KernelProgramBuilder::push_view` takes a `ByteWindow { offset, length }` and admits any window inside the value (`crates/tiler-ir/src/program/builder.rs:224-247`). `push_whole_view` is a convenience over it, not the only constructor.

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
