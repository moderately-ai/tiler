---
id: replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin
title: Replace the stale artifact ABI byte figures with the properties tests pin
status: in-progress
priority: p1
dependencies: []
related: [recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, abi, identity, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786172921
---

`docs/artifact-abi.md` carries byte figures that no test asserts and that the `v15 -> v16` identity step moved. Two of them point **past the end of the structure they index**, so a reader following them lands nowhere. This is the sibling of [the dtype-support repair](recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix.md), which resolved the same defect class in `contracts/navigation`; that ticket's route and its argument should be read before starting here.

## Facts

**Reported by the dtype-support worker, coordinator-verified in part.** Verified at `775e410f`: `36,832` appears in `docs/artifact-abi.md` and in no other document. Verified: `DIFFERING_CARRIER_POSITIONS = 68` is asserted at `crates/tiler-artifact/src/program/codec/tests.rs`, anchored by the constant name.

**Reported and NOT independently verified by the coordinator — re-measure before relying on any of it.** Six figures are said to be stale: identity length `48,584` (measured `40,132`), offsets `3,104`, `3,106`, `47,898`, `47,899`, and the four lengths `90,806` / `45,457` / `73,556` / `36,832`. The claim that **`47,898` and `47,899` fall past the end of the identity** follows from the identity having shrunk to `40,132`, and is the reason this is p1 rather than p2 — a reader is directed to an offset that does not exist.

**Reported: the document's differing-position count and its pinning account are correct.** Do not "fix" those. A repair that overshoots into correct text is the failure this ticket most invites, because the surrounding figures are wrong.

## What closes this

The stale figures replaced the way the sibling ticket replaced its own: **name the property and the constant that pins it, rather than copying a value into prose.** The document already contains the precedent in its own voice — the neighbouring paragraph reading *"Measurement, and it is now pinned by a test rather than carried as prose here."* Follow that sentence's lead; it is the house style for exactly this situation, written before the drift happened.

Do not restate the measured numbers as fresh prose figures. They will decay the same way on the next identity step, which is the whole lesson of the sibling ticket: an unasserted number that looks measured is worse than no number, and the `v15 -> v16` step moved these by tens of thousands of bytes **downward** while every reader's instinct was that an envelope only grows.

**Where a figure genuinely has no pin behind it**, say so in the text and either propose the assertion — naming the construction and the value, for a `crates/**` ticket to carry, since that is out of scope here — or state the property qualitatively. Do not leave a bare number with no owner.

Before closing, enumerate every numeral in the document and classify each as pinned, spec-constant, dated measurement, or unowned. **Report the census with its counts**, so "no others" is distinguishable from "did not look". The sibling ticket did this over its whole file and found zero survivors, which is what made its result trustworthy.
