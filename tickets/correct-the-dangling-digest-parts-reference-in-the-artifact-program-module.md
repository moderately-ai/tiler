---
id: correct-the-dangling-digest-parts-reference-in-the-artifact-program-module
title: Correct the dangling digest parts reference in the artifact program module
status: in-progress
priority: p3
dependencies: []
related: [repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, digest]
claimed_from: todo
assignee: coord
lease_expires_at: 1786175075
---

`crates/tiler-artifact/src/program/mod.rs` refers to a symbol that does not exist, in the crate that owns the re-export it is describing.

## Facts

**Reported by the `tiler-digest` note repair, not coordinator-verified — check it first.** The module is said to state `with \`digest_parts\` private to \`tiler-digest\``. No `digest_parts` exists in `tiler-digest`: the general parts-digest form was **removed**, not made private. The `tiler-digest` header states this correctly — "the general form is gone rather than promoted" — so the two crates disagree, and the one that is wrong is the one describing its own dependency.

## Why p3, and why it is filed rather than ignored

It is a doc comment with no gate behind it and no caller misled at compile time — a wrong `private to` reading costs a reader one failed search. But a dangling symbol reference in the crate that owns the re-export is the kind of thing someone later cites as evidence that a private general form exists, and then designs around it. The cost of leaving it is small and cumulative; the cost of fixing it is one sentence.

## What closes this

The sentence restated to match what `tiler-digest` actually exposes, cited by **searchable anchor** rather than line number. Prefer describing the removal in the terms the owning crate uses over inventing a second phrasing — two crates describing the same absence differently is how this drifted in the first place.

**Check the surrounding paragraph's other claims about `tiler-digest`.** A sentence that survived because it reads plausibly usually has neighbours, and this one describes a boundary the reader cannot see from here. **Name the count you checked**, so a clean result is distinguishable from an unexamined one.

Do not edit `crates/tiler-digest/**` (`implementation/digest`, not this scope). If the correct fix turns out to be a change there instead, report it rather than widening.
