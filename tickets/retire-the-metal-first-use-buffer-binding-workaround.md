---
id: retire-the-metal-first-use-buffer-binding-workaround
title: Retire the Metal first-use buffer binding workaround
status: done
priority: p2
dependencies: [pair-verified-buffer-handles-with-signature-ordinals]
related: [pair-verified-buffer-handles-with-signature-ordinals]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, backend-contract]
---
`tiler-metal` invents a buffer binding order because the structured kernel IR withholds one. When `pair-verified-buffer-handles-with-signature-ordinals` publishes the pairing, this workaround must be retired rather than left beside it.

**Why it existed.** `VerifiedKernel::buffers()` yields `BufferParameter` values with no ordinal, and `OperationView::Load`/`Store` name a `VerifiedBufferId` that exposes none, so a backend emitting `[[buffer(N)]]` could not ask the IR which signature position a handle denoted. `tiler-metal` therefore assigned indices in **body first-use order**, reported exactly the table it emitted so subscripts and table agreed by construction, and rejected a signature carrying a parameter the body never references (`MalformedKernel { rule: "unreferenced-buffer-parameter" }`). That was sound, and it was a backend inventing a binding order the IR should have stated — every future backend would have had to invent the same one, and could have invented a different one.

## Outcome — superseded by `emit-an-empty-domain-reduction-to-metal` (`97ab545`, 2026-07-27)

The three steps this ticket listed have all been performed, by a different ticket, as a consequence of a different problem. Nothing remains to implement. Five Facts, each checked against HEAD on 2026-07-28.

**Fact 1 — the IR states the pairing, and says so in its own documentation.** `VerifiedKernel::declared_buffers` is `pub fn` at `crates/tiler-ir/src/kernel/model.rs:581`, yielding `(VerifiedBufferId, BufferParameter)` pairs. Its doc at `:570-571` states the property this ticket was waiting for: "Declaration order, which *is* the signature order: a parameter's position here is its argument-table ordinal." Pairs rather than an index accessor, because a consumer needs to relate a parameter to the loads and stores referencing it and does not need to do arithmetic on a handle.

**Fact 2 — step 1 is done; ordinals come from the declaration.** `crates/tiler-metal/src/emit.rs:405-414` builds the argument table with `for (ordinal, (handle, parameter)) in kernel.declared_buffers().enumerate()`. The reasoning is recorded in the comment at `:390-404`, which names both things first-use order got wrong: a parameter the body never references still occupies its position, and the ordinals no longer depend on the order the body happens to touch its buffers in — under first-use order "a body that stored before it loaded produced a table whose positions disagreed with the declaration the artifact records, and nothing compared the two."

**Fact 3 — step 2 is done, and the rule was removed rather than weakened.** `grep -rn "unreferenced-buffer-parameter" crates/` returns exactly one hit, and it is a comment: `crates/tiler-metal/src/tests.rs:1293`, in the doc of the empty-domain regression test, explaining what emission *used to* refuse. No rule, no error construction, no live rejection. The distinction is on the record at `tickets/emit-an-empty-domain-reduction-to-metal.md:49`: emitting only the referenced buffers was eliminated because the artifact's binding table is in declaration order and the runtime pairs artifact slot *i* with the emitted table's *i*-th transport, so dropping a parameter shifts every later ordinal and mis-binds. "The rule was not weakened — it was removed because what it guarded against became unrepresentable." Step 2's instruction to supersede the test explicitly was also met: the test survives as the empty-domain case, with its history in its doc.

**Fact 4 — step 3 is done, and the guarantee is preserved by construction rather than re-established.** `emit.rs:743-753` documents `buffer_binding` as "a lookup rather than an assignment": the table is built from `declared_buffers` before the body is walked, so "every emitted subscript and the reported binding table agree because they are the same table". That is the same by-construction guarantee first-use order had, carried across intact rather than assumed.

**Fact 5, and the one caution to carry forward — `unresolvable-buffer-parameter` is a different, narrower rule and must not be mistaken for the retired one.** It is live at `emit.rs:760`, and it fires when a handle resolves through the IR's own check but is absent from the table — a handle belonging to another kernel, or naming no retained parameter. It is not the count check this ticket asked to remove, and removing it would delete a real refusal. The names are one word apart; read the site before touching either.

## Graph observations for the coordinator (2026-07-28)

**Status.** Frontmatter is not this record's to change. This ticket's outcome is fully supported with nothing left to implement, so the request to close it as `done` is left for the coordinator.

**And a second observation this ticket's own text now contradicts.** Line 25 said this was "blocked on the public-surface decision in `pair-verified-buffer-handles-with-signature-ordinals`, which is at `review`". That dependency is now `status: awaiting-decision` — yet `declared_buffers` is already `pub`, so the public surface it gates on has landed and shipped. A ticket awaiting a decision about whether to publish an interface that is published is worth looking at on its own account, independently of what happens to this one.
