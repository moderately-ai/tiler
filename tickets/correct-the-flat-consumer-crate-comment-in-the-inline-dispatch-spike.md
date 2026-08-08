---
id: correct-the-flat-consumer-crate-comment-in-the-inline-dispatch-spike
title: Correct the flat consumer-crate comment in the inline dispatch spike
status: in-progress
priority: p3
dependencies: []
related: [correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand, accept-the-public-route-requirement-answer-boundary]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, spikes, contracts]
claimed_from: todo
assignee: coord
lease_expires_at: 1786174094
---

The last surviving site of the retired flat claim. Reported by two workers independently, neither able to edit it in scope.

## Facts

**Reported, not coordinator-verified — check it before editing.** `spikes/runtime/inline-dispatch/Cargo.toml` carries a comment reading "The one crate a Tiler consumer declares". `docs/architecture.md` says *"the one crate an **inline-frontend** consumer names"*, and the same paragraph explicitly carves out consumers that construct and compile arbitrary semantic programs. The flat form states a monopoly the contract refuses.

**Reported: the correct form already exists nearby.** `crates/tiler/src/lib.rs` is said to read "the only one an inline-frontend consumer declares" — so the repair has a house model to follow rather than needing invention. Verify that before copying it.

**Fact — this spike is the reason the distinction matters.** `spikes/runtime/inline-dispatch` delivers through `tiler::tensor!` and then drives Metal by hand. It is therefore an inline-frontend consumer that also dispatches, which places it inside the sentence's population and squarely on the open boundary question. A comment here asserting the flat monopoly is wrong in the one file best positioned to show why.

## Why p3

It is a comment in a spike manifest, reachable by no gate and read by few. It is filed rather than fixed because a correct claim about a contract belongs in the corpus even where the cost of being wrong is low — and because leaving one known-false copy behind is how a retired claim gets re-adopted later by someone grepping for prior art.

## What closes this

The comment restated to match `docs/architecture.md`, cited by **searchable anchor** rather than line number. Check the rest of the spike for the same flat form before closing, and **name the count either way** so a clean result is distinguishable from an unexamined one.

Do not expand this into a review of what the spike does; the manifest comment is the whole scope. If reading it surfaces a substantive claim about dispatch that is also wrong, file that separately rather than folding it in.
