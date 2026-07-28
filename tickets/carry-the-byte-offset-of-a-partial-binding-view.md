---
id: carry-the-byte-offset-of-a-partial-binding-view
title: Carry a binding's byte offset so a partial view is packageable
status: done
priority: p2
dependencies: []
related: [expose-the-dispatch-record-on-a-decoded-artifact, carry-reconstructable-kernel-programs-in-the-neutral-envelope, carry-the-binding-offset-through-the-runtime-route]
scopes: [implementation/artifact, contracts/artifacts, implementation/runtime, implementation/metal-aot]
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

## Outcome

Done as specified, with the offset's *source* reshaped by a decision that landed on `main` while this work was stranded, and with the corrections below.

**Implemented.** An ABI binding row carries `accessible_offset` beside `accessible_bytes`, with its own `AbiExprUse::AccessibleOffset`: folded into artifact identity beside the extent in `push_entry` and `identity_use_sites`; encoded in the manifest binding row; re-proven at its use site by the decoder's `check_entry`; reached by both reachability closures; and published as `BindingRef::accessible_offset` and `DecodedBinding::accessible_offset`. `PartialBindingView` is gone. `MANIFEST_SCHEMA` moved to `5.0` and `ARTIFACT_DOMAIN` to `tiler.artifact-program.v7`, both recomputed on the merged tree.

**Reshaped at landing — the offset is derived, not declared.** This ticket predates `bind-the-artifact-variant-abi-to-the-program-abi` (the `v6` identity step), which removed every caller-restated ABI field: the guard, launch geometry, and accessible extent are taken from the bound program. The offset follows the same rule. The program states an *expression* for the extent and only a concrete `ByteWindow` for where it starts, so `ArtifactProgramBuilder::adopt_offsets` mints each access's window offset as its canonical literal; `BindingSpec` gained no field, and the originally specified `AccessibleOffsetDisagreement` refusal does not exist because there is no producer statement left to disagree. The row keeps an expression reference rather than a plain number so a program that one day computes its window offset carries that formula without a schema step.

**Correction — `AliasedInternalBinding` cannot be given a regression test, because it is unreachable rather than merely unfixtured.** Two bindings of one entry are two accesses of one stage, and `push_stage` pins each access's mode to its buffer parameter's, so of any two one reads and the other writes. A stage both defining and reading one value needs a data dependency from its defining stage to its reading stage, which `verify_dependencies` requires and `push_data_dependency` refuses to create for a stage naming itself. So the two accesses of one stage always address different values, whatever their tensor roles. The variant is retained for the reason `UnnameableBindingTarget` is — the guarantee is another crate's builder rule — and its doc comment states that derivation.

**Correction — the smallest partial-window plan is two stages, and that is no longer a decode boundary.** A verified kernel refines the canonical lowering of a scheduled region, a region has exactly two accesses, and of the three admitted refinements the only two naming an intermediate role are the pointwise write and the reduction read, which are different regions — so the fixture is two stages. When this work was written, a two-stage envelope was refused through `tiler.artifact.feature.multi-stage-program` and the offset had no positive nonzero end-to-end case. `carry-the-stage-execution-order-in-the-envelope` landed while the work was stranded, so the boundary test this ticket originally pinned flipped into the payoff: `codec::tests::a_partial_binding_window_survives_encode_and_decode` decodes the two-stage plan and reads `SCRATCH_OFFSET` back out of bytes, and `program::tests::a_binding_may_address_part_of_the_value_it_names` proves the offset at build.

**Consequence closed fail-closed at landing, honouring still owned by the follow-up.** With the multi-stage refusal gone, a partial-window artifact decodes and routes, and `tiler-runtime`'s `RoutedBinding` publishes an extent and no offset — the silently-wrong-dispatch hazard this ticket's own text predicted, with no other layer's refusal left standing in front of it. The landing therefore added `LoadRejection::UnpublishedBindingOffset`: `place_bindings` evaluates every binding's offset and refuses a nonzero one by name. [`carry-the-binding-offset-through-the-runtime-route`](carry-the-binding-offset-through-the-runtime-route.md) owns replacing that refusal with a published, honoured offset — and owes the two-stage loader fixture the refusal's unit test records as missing.

## Landed 2026-07-28

Rescued from the stranded worktree `agent-a0eb91e9c8f485c11` (324 commits behind), per `rebase-and-land-the-stranded-numerical-policies-worktree`'s sibling protocol: preserved verbatim as `30595f3` on `tkt/carry-the-byte-offset-of-a-partial-binding-view`, then squash-merged with every conflict resolved toward `main`'s structure — the derived-ABI builder (`v6`), the split `serial-sum-run` prototype, the readable multi-stage envelope — and the branch's intent re-applied in that shape. The manifest schema and identity domain were recomputed on the merged tree (`5.0`, `v7`), not taken from the branch's `5.0`/`v4`.

## Closes when

A binding may address a partial window, its offset is carried and proven against the packaged program, `PartialBindingView` is gone, identity/schema versions state the change, the loader-side consequence is tracked by a live ticket, and `make full` passes.
