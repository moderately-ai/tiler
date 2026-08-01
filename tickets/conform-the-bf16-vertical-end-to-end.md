---
id: conform-the-bf16-vertical-end-to-end
title: Conform the BF16 vertical end to end against the exact reference corpus
status: todo
priority: p2
dependencies: [validate-bf16-at-the-runtime-routing-boundary]
related: [spike-bf16-through-the-second-dtype-seams, evaluate-bf16-reference-semantics, own-the-dtype-support-maturity-matrix]
scopes: [implementation/reference, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, conformance, testing]
---
## User-visible outcome

One checked conformance run carries a pure-BF16 program from semantic construction to a dispatched device result and compares it against the exact-rational oracle, so a regression anywhere in the vertical is a red test rather than a wrong tensor. Until this exists the layers are each tested against their neighbour and nothing tests the composition.

## Why a per-layer test is not this

**Fact.** Each BF16 child closes on evidence about its own layer. That is correct and it is not sufficient: the U4/F32 vertical is the standing example of a family whose physical carrier, kernel, and lowering are each tested while the composition is not, and `docs/dtype-support.md` records that non-monotone row deliberately.

**Inference.** The composition is where a dtype's *width* assumptions fail — a two-byte element counted as four survives every single-layer test that uses consistent counts on both sides, and only an end-to-end run with a hand-derived expected result catches it.

## Required evidence

- One program — BF16 constant, multiply, add — carried from semantic construction through compile, artifact, runtime routing, and device dispatch, with the exact expected result bits stated in the test rather than read back from the run.
- The corpus covers, in the end-to-end run and not only in the unit oracle: both zeros with their signs, the least positive and least negative subnormals, the greatest subnormal, the least normal, a tie resolved to even, an ordinary rounding, an overflow to infinity, both infinities, and a non-canonical NaN that canonicalizes.
- The **declared flush is applied to the reference before comparison**, and the elements it moves are named. Finding 24 measures BF16 arithmetic flushing subnormals on the macOS row, so bit equality on the subnormal cases would mean the device did not do what was measured — a passing test there is a signal to distrust, not a success.
- An execution witness on a non-subnormal operand, for the reason finding 24 gives: `flushed` and `the arithmetic was optimized away` produce the same observation without one.
- At least one perturbation of the composition itself, observed failing — for instance an element count derived from the wrong width, which every layer-local test would still pass.
- The measurement boundary stated: host, OS build, Metal version, GPU, and family, with no generalization beyond the row that ran.

## Closes when

The end-to-end run passes on the measured macOS row with hand-derived expected bits, the flush is applied and its affected elements named, the execution witness is present, the composition perturbation is observed failing, the boundary is recorded, and the BF16 `Conformance evidence` cell states what this run actually covers rather than the whole family.

## Graph maintenance

- The last of the BF16 implementation children; depends on the runtime boundary, which transitively depends on the rest.
- A host without the measured environment must still run the deterministic reference and structural half and report the unavailable measurement boundary, rather than skipping silently or claiming a pass.
- This closes the BF16 vertical for the **three operations and one target family** it names. It does not promote BF16 generally: contraction, reduction, conversion, mixed precision, and every other family and target stay where the ledger puts them.
