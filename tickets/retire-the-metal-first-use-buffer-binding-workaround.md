---
id: retire-the-metal-first-use-buffer-binding-workaround
title: Retire the Metal first-use buffer binding workaround
status: todo
priority: p2
dependencies: [pair-verified-buffer-handles-with-signature-ordinals]
related: [pair-verified-buffer-handles-with-signature-ordinals]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, backend-contract]
---
`tiler-metal` invents a buffer binding order because the structured kernel IR withholds one. When `pair-verified-buffer-handles-with-signature-ordinals` publishes the pairing, this workaround must be retired rather than left beside it.

**Why it exists.** `VerifiedKernel::buffers()` yields `BufferParameter` values with no ordinal, and `OperationView::Load`/`Store` name a `VerifiedBufferId` that exposes none, so a backend emitting `[[buffer(N)]]` cannot ask the IR which signature position a handle denotes. `tiler-metal` therefore assigns indices in **body first-use order**, reports exactly the table it emitted so subscripts and table agree by construction, and rejects a signature carrying a parameter the body never references (`MalformedKernel { rule: "unreferenced-buffer-parameter" }`).

That is sound, and it is a backend inventing a binding order the IR should have stated. Every future backend would have to invent the same one, and could invent a different one.

**What closes this, once the IR states the pairing.**

1. Assign `[[buffer(N)]]` from the IR's signature ordinal instead of body first-use order.
2. Remove the `unreferenced-buffer-parameter` rejection. It exists only because first-use order is undefined for a parameter the body never touches; with a signature ordinal, an unreferenced parameter has a well-defined index and is no longer malformed. Check whether any other rule or test depends on that rejection before deleting it, and supersede its test explicitly rather than dropping it.
3. Confirm the emitted table still agrees with the emitted subscripts. The current design guarantees that by construction; ordinal-driven assignment must keep the guarantee rather than assume it.

**Blocked on the public-surface decision** in `pair-verified-buffer-handles-with-signature-ordinals`, which is at `review` awaiting the owner. Do not implement a second private workaround here in the meantime — the point of this ticket is that the existing one stops outliving the gap.
