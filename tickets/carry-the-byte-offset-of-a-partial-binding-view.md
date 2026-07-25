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
claimed_from: todo
assignee: agent-carry-the-byte-offset-of-a-partial-binding-view
lease_expires_at: 1785045285
---
Split from `expose-the-dispatch-record-on-a-decoded-artifact`, which encoded *which* buffer each ABI binding slot addresses and deliberately did not encode *where in it*.

**Fact — what the record carries today.** A binding row carries `BindingTarget` (a program input's `InputKey`, the `OutputKey` set publishing an output value, or `Internal`) and `accessible_bytes`, an ABI expression the builder proves equals `access.view().window().length`. The window's `offset` is not carried anywhere.

**Fact — an arbitrary window is representable one layer down.** `tiler_ir::program::KernelProgramBuilder::push_view` takes a `ByteWindow { offset, length }` and admits any window inside the value (`crates/tiler-ir/src/program/builder.rs:224-247`). `push_whole_view` is a convenience over it, not the only constructor.

**Fact — the artifact layer refuses the gap rather than approximating it.** `ArtifactBuildError::PartialBindingView` rejects a binding whose access does not address the whole of its value. That was the fail-closed choice: with a target and an extent but no offset, a loader binds the right buffer at the wrong place, which is a silently wrong result rather than a refusal.

## The work

Carry the offset as a second ABI expression on the binding row, with its own `AbiExprUse`, checked against `access.view().window().offset` exactly as `accessible_bytes` is checked against `.length`, folded into artifact identity, and validated on decode through the same use-site machinery (value type, availability phase, interface-only). Then remove `PartialBindingView` and its refusal.

An expression rather than a constant, for the same reason the extent is one: both are concrete at build time and both generalize over bound extents at run time, and a constant offset beside a formula extent would be two spellings of one contract.

This is a manifest schema step. `MANIFEST_SCHEMA` is at `3.0` and `ARTIFACT_DOMAIN` at `v2` after the binding target landed; both move again, for the reason recorded at those sites.

## Also unblocks the two untested refusals

`PartialBindingView` and `AliasedInternalBinding` are implemented and **not covered by a test**, and the reason is exact rather than an oversight: only a `ValueRole::Temporary` value can be larger than what one stage addresses or be addressed by two slots of one entry, and binding a temporary needs a kernel declaring a `TensorRole::Intermediate` buffer. `grep -rn "TensorRole::Intermediate" crates/tiler-artifact` is empty — every fixture in that crate binds exactly one program input and one program output. `tiler-compiler`'s multi-stage plans do produce intermediate roles (`crates/tiler-compiler/src/physical.rs:232-315`), so the case is real rather than hypothetical.

Whoever takes this should build the intermediate-role fixture the artifact crate lacks; it makes `AliasedInternalBinding` testable in the same pass even though this ticket removes the other refusal rather than testing it.

## Closes when

A binding may address a partial window, its offset is carried and proven against the packaged program, `PartialBindingView` is gone, `AliasedInternalBinding` has a regression test, and `uv run --locked python scripts/check_repository.py` passes.
