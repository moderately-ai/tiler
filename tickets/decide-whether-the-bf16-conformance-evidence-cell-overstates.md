---
id: decide-whether-the-bf16-conformance-evidence-cell-overstates
title: Decide whether the BF16 conformance-evidence cell overstates without the end-to-end run
status: todo
priority: p3
dependencies: []
related: [conform-the-bf16-vertical-end-to-end, carry-a-bf16-subnormal-realization-the-reference-can-be-told, re-read-the-bf16-and-elementary-support-rows-against-source]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [navigation, bf16, maturity-claims]
---

## The question (raised by the BF16 vertical's discovery stop, 2026-08-06)

`docs/dtype-support.md`'s BF16 `Conformance evidence` cell reads `[tested guarantee, macOS Apple9 only](#other-ieee-binary-floats-and-bf16)`. The vertical ticket (`conform-the-bf16-vertical-end-to-end`, now `blocked`) establishes that no end-to-end BF16 run exists — the layers are each tested against their neighbour and nothing tests the composition — and separately that the reference cannot yet apply the measured flush (`docs/correctness-and-testing.md` now carries that exception with its reproducing check). The question is whether "tested guarantee" in that cell already claims more than the per-layer evidence supports, or whether the anchor section's own text bounds the claim to what exists.

## The work

Read the cell's column definition and the full anchor section, compare against the vertical ticket's evidence list and the correctness-and-testing exception, and either qualify the cell (e.g. per-layer only, composition untested and blocked behind the flush ticket) or record why the current text is already bounded. Whichever way it resolves, the derivation lands in the section so the next maturity audit does not re-derive it. AGENTS.md's maturity-claim rule governs: a tested guarantee is the strongest of the four claims and must not cover an untested composition.

## Closes when

The cell and its anchor agree with the verified evidence, and the resolution's derivation is recorded at the anchor.
