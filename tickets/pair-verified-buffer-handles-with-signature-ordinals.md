---
id: pair-verified-buffer-handles-with-signature-ordinals
title: Let a KIR consumer map a VerifiedBufferId to its signature ordinal
status: awaiting-decision
priority: p1
dependencies: []
related: [prototype-structured-kir-slice, prototype-metal-kir-lowering]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, backend-contract]
---
## Decision needed (2026-07-28)

**The question is now narrower than the one this ticket was parked on.** The gap is closed in the tree: `VerifiedKernel::declared_buffers` is public at `crates/tiler-ir/src/kernel/model.rs:581`, yields `(VerifiedBufferId, BufferParameter)` in declaration order, and `tiler-metal` builds its argument table from it at `crates/tiler-metal/src/emit.rs:407`. So the decision is not *whether* to publish a pairing; it is whether to keep the shape that shipped.

**Approve `declared_buffers` as shipped, or reshape it to a `buffer_id(position) -> Option<VerifiedBufferId>` accessor before an external consumer appears?**

| | Approve `declared_buffers` | Reshape to `buffer_id` |
| --- | --- | --- |
| **Enables** | The correspondence is unmissable rather than something a consumer must know to ask for; one call yields exactly what a backend building a signature needs, and the emitter that consumes it needs no arithmetic on a handle. | The published surface matches the 2026-07-25 resolution and composes with the existing read views, which resolve a handle to a thing rather than enumerating pairs. `fn buffer_id(&self, index: u32) -> VerifiedBufferId` already exists **private** at `model.rs:686` and is already used internally to mint the handles the `Load`/`Store` views carry, so the public form is a visibility change plus a bounds-checked return, not new logic. |
| **Prevents** | An index-addressed lookup still has no public spelling; a consumer wanting the handle at position *n* iterates to it. | Costs the one substantive argument for the pairing iterator, and reshaping now means changing a public item `tiler-metal` already consumes. |

Both are additive to `buffers()`, which is the fact that matters for blast radius: **neither shape requires touching it.** `grep -rn '\.buffers()' $(find crates prototypes -name '*.rs') | wc -l` reports 19 call sites across 10 files today, up from the eight this ticket recorded, so the elimination below is stronger than when it was written, not weaker.

**Recommendation: approve as shipped.** `declared_buffers` is in the tree, tested, and consumed; the reasons the resolution preferred `buffer_id` were composition and blast radius, and the shipped shape pays neither cost — it broke nothing and it is the shape the one real consumer wanted. The counterpoint is that this ratifies a surface chosen inside another ticket's slice rather than at this one's review, which is the thing ADR 0075 reserves; if the shape is wrong, the moment to say so is before a second backend consumes it.

## Supersession — what shipped, and how

`emit-an-empty-domain-reduction-to-metal` (commit `97ab545`, status `done`) needed the pairing to place a declared-but-unread parameter at its declaration ordinal, and added `declared_buffers` to get it. That is **not** the accessor the 2026-07-25 resolution below selected, and it is also **not** the breaking change that resolution eliminated: it is a third shape neither considered, an *additive* pairing iterator. The breaking change stayed eliminated; what the resolution pre-empted was the choice between two additive shapes, and that choice was made by the ticket that needed one first. That ticket's own record states the reasoning: "Pairs rather than an index accessor: a consumer needs to relate a parameter to the loads and stores that reference it, and does not need to do arithmetic on a handle" (`tickets/emit-an-empty-domain-reduction-to-metal.md:51`).

It also closed a latent defect rather than only an ergonomic gap: first-use order and declaration order coincide only for a body that touches its buffers in declaration sequence, the artifact's binding table is in declaration order, and nothing checked that the two agreed — so a body that stored before it loaded would have bound the wrong buffer to each slot, silently.

## The gap as originally found

A real gap in `tiler_ir::kernel`, found by the first backend actually consuming it rather than by inspection.

`OperationView::Load` and `OperationView::Store` name the buffer they act on with a `VerifiedBufferId`. `VerifiedKernel::buffers()` yields `BufferParameter` values. **Nothing paired the two.** The handle exposed no ordinal, and the parameter carried no handle, so a backend that must emit an argument-table index for each load and store could not ask the IR which parameter a given handle denotes.

Neither workaround was sound. Matching by parameter value fails because two parameters can be value-identical at different indices. Relying on the derived `Ord` on the handle means depending on an undocumented `(owner, index)` field layout — a private representation detail that is free to change.

`tiler-metal` therefore assigned `[[buffer(N)]]` in **body first-use order**, reported exactly the table it emitted so subscripts and table agreed by construction, and rejected a signature carrying a parameter the body never referenced (`MalformedKernel { rule: "unreferenced-buffer-parameter" }`). That was sound but it was a backend inventing a binding order the IR should have stated, and every future backend would have had to invent the same one — and could have invented a different one, which is worse.

This matters more than an ergonomic wart because the whole point of the kernel layer, stated in its own module documentation, is that a backend never reconstructs facts the IR should carry. A buffer's position in the signature is exactly such a fact.

## Outcome

**Status `awaiting-decision`, not `done`: every way to close this adds a public item, which is owner-reserved.** The gap is confirmed by reading the source, the fact the fix would publish is pinned by a test, and the API choice is reduced to one decision for Tom.

**Confirmed (inspected source, base `f286289`).** `VerifiedKernel::buffers()` in `crates/tiler-ir/src/kernel/model.rs` yields `BufferParameter` by value through an `ExactSizeIterator` and exposes no ordinal. `OperationView::Load`/`Store` carry a `VerifiedBufferId`. `VerifiedKernel::buffer(id)` resolves a handle to a parameter but not to a position. The pairing therefore genuinely could not be obtained from the public API.

**Correction — the ticket understates the hazard, and the correction matters for which fix is right.** The ticket says relying on the derived `Ord` "means depending on an undocumented `(owner, index)` field layout". That is true, but the consequence is sharper than "unsound workaround": the workaround is *publicly reachable and silently correct today*. `crates/tiler-ir/src/kernel/handles.rs` derives `Ord` and `PartialOrd` on `VerifiedBufferId` via the `verified_handle!` macro, over fields `owner: VerifiedKernelOwner` then `index: u32`. Both fields are `pub(super)`, so an out-of-crate consumer cannot read `index` — but it can *compare* handles, and every handle of one kernel shares an owner, so comparison falls through to `index`. Sorting a kernel's buffer handles yields signature order.

So the IR did not merely withhold the pairing; it published a derived ordering that recovers the pairing correctly, with no documentation saying so and no test holding it. That is the shape a consumer will actually reach for, and it will keep working until someone reorders the two fields in a macro that looks purely internal.

`crates/tiler-ir/src/kernel/tests.rs:449::referenced_buffer_handles_recover_the_signature_in_handle_order` pins the fact: the buffer handles the lowered pointwise body references, sorted and deduplicated, resolve through `buffer()` to exactly the sequence `buffers()` yields, and a handle from another kernel is rejected with `ForeignKernel` rather than resolving to the same ordinal. The test deliberately asserts no *position*, because `buffers()` yields none. Its value is that it establishes the fix publishes an invariant that already holds rather than computing a new one — and it fails if the handle field order is changed while the decision is pending.

**Elimination — do not change `buffers()`.** The ticket offered two shapes; they were not equivalent, and the eliminated one stayed eliminated.

- `buffers()` yielding `(VerifiedBufferId, BufferParameter)` is a breaking change to an accepted public API with eight call sites across `tiler-ir`, `tiler-artifact`, `tiler-compiler`, `tiler-metal`, and both prototypes, most of which want only the parameter. (Eight as counted then; 19 across 10 files as counted 2026-07-28 by the grep above.) It also makes `buffers().len()` — used by `tiler-metal`'s arity check and by the crate-level doctest — read less clearly.
- An additive accessor breaks nothing. The private `fn buffer_id(&self, index: u32) -> VerifiedBufferId` already exists and is already used internally to mint the handles the `Load`/`Store` views carry, so the public form is a visibility change plus a bounds-checked return, not new logic. It composes with the existing read views, which resolve a handle to a thing rather than enumerating pairs.

The one substantive argument for the pairing iterator is that it makes the correspondence unmissable rather than something a consumer has to know to ask for. That is real, and it is the argument the shipped shape acted on.

**Split out.** `retire-the-metal-first-use-buffer-binding-workaround` carries the `tiler-metal` half. It is a different scope (`implementation/metal`), and it is **no longer blocked on the IR gap** — the workaround is already gone from production: `unreferenced-buffer-parameter` survives only as a historical mention in a test comment at `crates/tiler-metal/src/tests.rs:1293`, and `grep -rn 'unreferenced-buffer-parameter' crates prototypes` returns that one line. What remains on that ticket is whatever its own text still asks for beyond the removal, not a dependency on this decision.

## Resolved by the coordinator — 2026-07-25

**Take the additive `buffer_id` accessor; do not change `buffers()`.** Auto-resolved on maintainability: changing `buffers()` churns its call sites to deliver what an added accessor delivers at none, and a signature change to a widely-used reader is the kind of edit whose blast radius is discovered rather than planned.

The promotion itself still needs the owner's approval under ADR 0075 and is not covered by the four promotions approved that day. The ticket remains `awaiting-decision` until that approval is recorded — and the decision now in front of the owner is the narrowed one at the top of this file, because the surface shipped before the approval did.
