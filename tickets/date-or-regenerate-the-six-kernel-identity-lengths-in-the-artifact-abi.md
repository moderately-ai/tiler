---
id: date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi
title: Date or regenerate the six kernel identity lengths in the artifact ABI
status: in-progress
priority: p2
dependencies: []
related: [replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, abi, identity, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786176329
---

The last unowned quantity in `docs/artifact-abi.md` after its byte figures were retired. Same defect class, one paragraph over.

## Facts

**Reported by the worker that retired the neighbouring figures; NOT coordinator-verified — re-measure before relying on any of it.** The "Governed budgets" paragraph, anchored at *"the three identities never shared a subject"*, states six kernel-identity lengths — 736, 1,483, 1,845, 1,700, 2,279, and the pinned 1,121 — in the present tense, bounded to a host but to **no commit and no date**. Five are said to carry no pin.

**Reported: the one pinned value cannot catch drift in the others.** Its assertion is only that the value exceeds `MAX_OPAQUE_IDENTITY_BYTES`, so it is a floor rather than an equality and would stay green across a change that moved every length.

**Reported: the kernel identity has stepped since.** `tiler.kernel.v7` and `tiler.kernel-program.v11` are the current domains, so the figures are plausibly stale — but plausibly is not measured, and the sibling found figures that moved **downward** by tens of thousands where everyone assumed growth.

**Reported: regenerating needs** the serial-`f32`-sum kernel at one contributor and ranks 3–8, via `crates/tiler-conformance/src/serial_sum.rs`.

## What closes this

Either the six regenerated from their construction and restated **with the commit and date they were measured at**, or the paragraph rewritten to state the property without the figures — whichever the paragraph's argument actually needs. Read the sibling ticket's route first: it retired rather than refreshed, because every property the figures supported was already pinned, and it kept the structural account that does not decay.

**Do not derive any figure arithmetically.** The sibling measured four offsets that moved in opposite directions across a single identity step; a uniform correction would have been wrong at all of them.

**You cannot add a pin yourself** — assertions live in `crates/**`, outside this scope. If pinning is the right answer, name which construction should assert which value and file it rather than widening.

If you regenerate, say which host and which commit, and keep the measurement bounded to them. `AGENTS.md` is explicit that measurements bound claims but do not prove unmeasured universals; six lengths from one kernel shape are evidence about that shape.

This is reported to be the **last** unowned quantity in the document — its census found 21 sites carrying a quantity, of which 2 were unowned and these are they. Confirm that census rather than trusting it, and report the count either way.
