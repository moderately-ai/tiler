---
id: pair-verified-buffer-handles-with-signature-ordinals
title: Let a KIR consumer map a VerifiedBufferId to its signature ordinal
status: review
priority: p1
dependencies: []
related: [prototype-structured-kir-slice, prototype-metal-kir-lowering]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, backend-contract]
claimed_from: todo
assignee: agent-ir2
lease_expires_at: 1784998648
---
A real gap in `tiler_ir::kernel`, found by the first backend actually consuming
it rather than by inspection.

`OperationView::Load` and `OperationView::Store` name the buffer they act on with
a `VerifiedBufferId`. `VerifiedKernel::buffers()` yields `BufferParameter` values.
**Nothing pairs the two.** The handle exposes no ordinal, and the parameter
carries no handle, so a backend that must emit an argument-table index for each
load and store cannot ask the IR which parameter a given handle denotes.

Neither workaround is sound. Matching by parameter value fails because two
parameters can be value-identical at different indices. Relying on the derived
`Ord` on the handle means depending on an undocumented `(owner, index)` field
layout — a private representation detail that is free to change.

`tiler-metal` therefore assigns `[[buffer(N)]]` in **body first-use order**,
reports exactly the table it emitted so subscripts and table agree by
construction, and rejects a signature carrying a parameter the body never
references (`MalformedKernel { rule: "unreferenced-buffer-parameter" }`). That is
sound but it is a backend inventing a binding order the IR should have stated,
and every future backend would have to invent the same one — and could invent a
different one, which is worse.

This matters more than an ergonomic wart because the whole point of the kernel
layer, stated in its own module documentation, is that a backend never
reconstructs facts the IR should carry. A buffer's position in the signature is
exactly such a fact.

**Fix:** have `buffers()` yield `(VerifiedBufferId, BufferParameter)`, or add a
`buffer_id(position) -> Option<VerifiedBufferId>` accessor — whichever composes
better with the existing read views. Then `tiler-metal`'s first-use assignment
and its unreferenced-parameter rejection both disappear, and the binding order
becomes the IR's stated contract rather than a backend convention.

Update `tiler-metal` in the same change or immediately after, so the workaround
does not outlive the gap. Note this touches a public surface Tom already
accepted, so the change goes to him under the approval policy.

## Outcome

**Status `review`, not `done`: every way to close this adds a public item, which is owner-reserved.** The gap is confirmed by reading the source, the fact the fix would publish is now pinned by a test, and the API choice is reduced to one decision for Tom. No public surface was changed.

**Confirmed (inspected source, base `f286289`).** `VerifiedKernel::buffers()` in `crates/tiler-ir/src/kernel/model.rs` yields `BufferParameter` by value through an `ExactSizeIterator` and exposes no ordinal. `OperationView::Load`/`Store` carry a `VerifiedBufferId`. `VerifiedKernel::buffer(id)` resolves a handle to a parameter but not to a position. The pairing therefore genuinely cannot be obtained from the public API.

**Correction — the ticket understates the hazard, and the correction matters for which fix is right.** The ticket says relying on the derived `Ord` "means depending on an undocumented `(owner, index)` field layout". That is true, but the consequence is sharper than "unsound workaround": the workaround is *publicly reachable and silently correct today*. `crates/tiler-ir/src/kernel/handles.rs` derives `Ord` and `PartialOrd` on `VerifiedBufferId` via the `verified_handle!` macro, over fields `owner: VerifiedKernelOwner` then `index: u32`. Both fields are `pub(super)`, so an out-of-crate consumer cannot read `index` — but it can *compare* handles, and every handle of one kernel shares an owner, so comparison falls through to `index`. Sorting a kernel's buffer handles yields signature order.

So the IR does not merely withhold the pairing; it publishes a derived ordering that recovers the pairing correctly, with no documentation saying so and no test holding it. That is the shape a consumer will actually reach for, and it will keep working until someone reorders the two fields in a macro that looks purely internal.

`crates/tiler-ir/src/kernel/tests.rs::referenced_buffer_handles_recover_the_signature_in_handle_order` now pins the fact: the buffer handles the lowered pointwise body references, sorted and deduplicated, resolve through `buffer()` to exactly the sequence `buffers()` yields, and a handle from another kernel is rejected with `ForeignKernel` rather than resolving to the same ordinal. The test deliberately asserts no *position*, because no public accessor yields one. Its value is that it establishes the fix publishes an invariant that already holds rather than computing a new one — and it fails if the handle field order is changed while the decision is pending.

**Recommendation for the owner decision — add `buffer_id`, do not change `buffers()`.** The ticket offers both shapes; they are not equivalent.

- `buffers()` yielding `(VerifiedBufferId, BufferParameter)` is a breaking change to an accepted public API with eight call sites across `tiler-ir`, `tiler-artifact`, `tiler-compiler`, `tiler-metal`, and both prototypes, most of which want only the parameter. It also makes `buffers().len()` — used by `tiler-metal`'s arity check and by the crate-level doctest — read less clearly.
- An additive accessor breaks nothing. The private `fn buffer_id(&self, index: u32) -> VerifiedBufferId` already exists at `model.rs:635` and is already used internally to mint the handles the `Load`/`Store` views carry, so the public form is a visibility change plus a bounds-checked return, not new logic. It composes with the existing read views, which resolve a handle to a thing rather than enumerating pairs.

The one substantive argument for the pairing iterator is that it makes the correspondence unmissable rather than something a consumer has to know to ask for. That is real but is better served by documenting the accessor on `buffers()` than by breaking it.

**Split out.** `retire-the-metal-first-use-buffer-binding-workaround` carries the `tiler-metal` half — the first-use `[[buffer(N)]]` assignment and the `unreferenced-buffer-parameter` rejection that exist only because the IR withholds the ordinal. It is a different scope (`implementation/metal`) and is blocked on this ticket's public-surface decision, so it could not land here regardless.
