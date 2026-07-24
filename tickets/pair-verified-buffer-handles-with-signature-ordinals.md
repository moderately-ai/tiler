---
id: pair-verified-buffer-handles-with-signature-ordinals
title: Let a KIR consumer map a VerifiedBufferId to its signature ordinal
status: todo
priority: p1
dependencies: []
related: [prototype-structured-kir-slice, prototype-metal-kir-lowering]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, backend-contract]
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
