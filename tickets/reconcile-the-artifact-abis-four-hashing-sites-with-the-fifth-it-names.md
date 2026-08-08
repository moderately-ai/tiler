---
id: reconcile-the-artifact-abis-four-hashing-sites-with-the-fifth-it-names
title: Reconcile the artifact ABIs four hashing sites with the fifth it names
status: in-progress
priority: p2
dependencies: []
related: [date-the-two-v4-step-paragraphs-trailing-the-v5-block]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, identity, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786185307
---

`docs/artifact-abi.md` contradicts itself about how many sites hash. One passage says four and enumerates them; another names a fifth.

## Facts, coordinator-verified at the merge that found it

**Fact.** The document contains `Hashing occurs at exactly four sites, all of them envelope framing` and, separately, `A fifth is a digest argument reached through a carried payload` — the latter giving `payload_identity = H(…)` under an envelope domain. Both strings resolve, once each.

**Fact.** This is **substantive, not positional.** A sibling repaired two paragraphs in this file whose *referents* moved when a block was inserted above them; this is a different defect — the two counts disagree on their face, in the same document, about the same subject.

## Why it matters

"Exactly four, all of them envelope framing" is the kind of closed enumeration a reader builds an argument on — that every hash in the crate is accounted for and shares a shape. If a fifth exists and reaches through a carried payload, both the count and the *characterization* are wrong, and any downstream reasoning that leaned on "all of them envelope framing" needs re-examining.

## What closes this

The two passages reconciled — establish from source which is right before choosing, and say which construction you read. Do not assume the larger number wins; the fifth may be a different kind of site that the four-count deliberately excludes, in which case the fix is to say so rather than to renumber.

**Prefer naming the construction over restating a count.** This file has had figures replaced by references to their owners repeatedly this week, on the reasoning that a number in prose rots on a schedule nobody watches. If an enumeration exists in code that owns this, name it.

**Establish the treatment from history** with `git log -S` and `git show <commit>:<file>`: true when written → dated beside; never true → substituted with the retired wording quoted. Repository **practice**, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim stays greppable; say inline that a later hit lands inside your note.

**Two known defects in this file are not yours and must not be folded in:** the premise that every `tiler-ir` domain opens `tiler.ir.` (46 of 60 do not) and the "first differing byte after `tiler.`" variant beside it, both in the same sentence, already reported. Report if you meet them.

**Preserve `git log -S` anchors.** Several tickets locate text in this file by distinctive substring; a sibling deliberately made its edits **prefix-only** so every protected substring stayed byte-identical, then verified each anchor still resolved to its original commit and not to the repair. Meet that standard.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`** — anchors fail as absence four ways: a line break inside them, an emphasis or backtick marker the source lacks, unescaped brackets read as a character class, and a quoted sentence that never appeared contiguously in source.

Check the neighbouring claims and **name the count**; six sweeps of this file this week each found more than they were sent for.
