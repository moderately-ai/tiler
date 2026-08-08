---
id: date-adr-0081-s-neither-closure-is-checked-correction-against-the-runtime-test
title: Date ADR 0081 s neither-closure-is-checked correction against the runtime test
status: todo
priority: p3
dependencies: []
related: [correct-the-artifact-abis-claim-that-nothing-asserts-the-kernel-identity-crossing]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, documentation]
---

ADR 0081 item 2's `Correction — 2026-07-26` says "neither closure is checked now. The decision stands; its enforcement is review." A mechanical check has since landed for one of the two closures.

## Facts

**Reported by the worker that found it, not coordinator-verified — check each before editing.** `crates/tiler-runtime/tests/identity_join/main.rs`'s `the_consumer_links_no_compiler_emitter_or_build_provider` parses `Cargo.lock` and mechanically refuses `tiler-build`, `tiler-compiler`, `tiler-cache`, `tiler-metal`, and `tiler-metal-aot` from the transitive closure, with three positive controls.

**Reported: the correction is only half stale, and the surviving half matters.** Still unchecked are the **positive** direction — that the closure is exactly `[tiler-artifact]` — and `tiler-metal-aot`'s empty closure. So ADR 0077's parallel correction reportedly remains accurate, and a repair that reads "this is now checked" without qualification would overstate.

## What closes this

The correction dated beside, naming the test and stating precisely which half it discharges and which it does not. **Do not write that the closure is enforced** — the refusing direction is, the asserting direction is not, and the distinction is the whole content.

It was **true when written**, so date beside rather than substitute. Verify with `git show <commit>:<file>` rather than assuming; this is repository practice — several ADRs state it while applying it and none decides it, so cite the practice, not an authority. A retired sentence quoted verbatim stays greppable; say inline that a later hit lands inside your note.

**Do not change what ADR 0081 decides**, and do not touch ADR 0077 — check whether its parallel correction is genuinely still accurate and report, rather than editing a second record on inference.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`.** Anchors fail as absence three ways: a line break inside them, an emphasis marker the source lacks, and unescaped brackets read as a character class.

**Check this ADR's other claims about the tree and name the count.** Sweeps of two sibling ADRs this week found 9 of 17 and 11 of 18 tree-claim clusters false, most predating the landing that prompted the review — so assume the neighbours are unexamined rather than clean.
