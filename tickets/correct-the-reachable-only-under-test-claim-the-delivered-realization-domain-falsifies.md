---
id: correct-the-reachable-only-under-test-claim-the-delivered-realization-domain-falsifies
title: Correct the reachable-only-under-test claim the delivered realization domain falsifies
status: in-progress
priority: p3
dependencies: []
related: [correct-the-dangling-digest-parts-reference-in-the-artifact-program-module, pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, identity]
claimed_from: todo
assignee: coord
lease_expires_at: 1786176329
---

A domain census in `crates/tiler-artifact/src/program/mod.rs` says its fourteen named domains are "reachable only under test". **Thirteen are. One is public.**

## Facts, coordinator-verified at the merge that found it

**Fact.** The sentence is anchored by `The envelope's seven governed domains and this module's seven` and ends "reachable only under test".

**Fact.** `DELIVERED_REALIZATION_DOMAIN` is declared `pub const` in `crates/tiler-artifact/src/program/realization.rs` and publicly re-exported from `program/mod.rs` alongside `AssessmentDisposition` and `DeliveredRealizationBuilder`. The other thirteen are `#[cfg(test)] pub(crate)`.

**Fact — the two counts the sentence rests on are correct** and should not be touched: `DomainContainer::ENVELOPE = 7` and `DomainContainer::PROGRAM_IDENTITY = 7`, both sized from `GovernedDomain` by `variant_count`. Only the reachability clause is wrong.

## Why it is filed rather than waved through

The same argument the sibling ticket made for `digest_parts` applies: a reader could conclude **no** domain constant is publicly reachable, and then reason about the crate's public surface from a false premise. A domain that is `pub` is contract — its value is observable, and under ADR 0075 its surface is Tom's.

Note the sibling's finding about how this class arises: the false `digest_parts` sentence was **authored by the commit that deleted the symbol**, which rewrote an accurate sentence into an inaccurate one. Check whether this clause was accurate before some later change made `DELIVERED_REALIZATION_DOMAIN` public — `git log -S DELIVERED_REALIZATION_DOMAIN` will say — because that changes the correction from "someone was careless" to "a change moved a symbol and left its description behind", and the note should say which.

## What closes this

The clause restated so a reader can tell which of the fourteen is publicly reachable and which are test-only. Do not restate the counts; they are correct and derived.

**If the correction implies the public domain is an accepted surface, it is not** — say so rather than implying acceptance. Under ADR 0075 a `pub` item is a labelled draft until Tom accepts its exact included and excluded surface; report the surface, do not decide it.

**Check the neighbouring blocks while you are in it.** The sibling worker checked eighteen claims across three comment blocks here and found two false and this one imprecise — a base rate of roughly one in six. **Name the count you checked**, so a clean result is distinguishable from an unexamined one.

Cite by searchable anchor, not line number, and run the anchor's grep before committing to it.
