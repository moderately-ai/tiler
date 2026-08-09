---
id: refuse-two-structurally-identical-output-chains-by-name-not-as-compiler-output
title: Refuse two structurally identical output chains by name, not as invalid compiler output
status: awaiting-decision
priority: p2
dependencies: [reproduce-the-identical-output-chain-stage-key-collision]
related: [bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, ir, multi-output]
---
## User-visible outcome

A program declaring two ordered named outputs whose producer chains are structurally identical is refused by a named request-boundary or recognition rule, or compiled, rather than reaching the caller as `InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))` — the compiler reporting its own defect for a caller's valid program.

## Why this exists

**Historical Measurement, 2026-08-06, on `tkt/bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals` at base `afdac9c9`.** Two independent epilogue chains over two declared inputs — `sum(x * x, axis 1) * 2.0` published as `sx` and `sum(y * y, axis 1) * 3.0` published as `sy`, both at `[1, 4]` — failed `compile` with:

```
InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))
```

It was reproduced with the same fixture differing only in the prologue expression (`x * x` against `y + y`), and was *not* reproduced when the two chains folded different extents (`[1, 4]` and `[1, 2]`). The current compiler test source preserves that distinction under the anchor `two chains of identical shape assemble two stages carrying one canonical key`, but its executable fixture uses different extents and therefore does not re-prove the collision at this base. [`reproduce-the-identical-output-chain-stage-key-collision`](reproduce-the-identical-output-chain-stage-key-collision.md) now owns that current-boundary evidence.

**Inference — the public error class is wrong if the historical failure remains.** `AmbiguousCanonicalKey` is a `tiler_ir::program` core-verification refusal reported through `CompilerOutputError::Program`. The old measurement established that both outputs had passed recognition and assembly far enough to create the collision, but only the prerequisite can establish that current path. If it remains, either the program layer's stage key must distinguish the two stages, the compiler must refuse the request by name before assembly, or an actually equivalent stage must be shared.

## Decision boundary — corrected 2026-08-09

The three remedies are not interchangeable implementation details. `stage_key` currently derives identity from the bound kernel and proof-bound coverage; `verify_unambiguous` makes its pairwise distinctness a program invariant; and program identity orders stages and value definitions through that key. Widening its subject or merging instances changes program identity and cross-reference semantics. A new request refusal changes the caller-visible stable diagnostic boundary. After the prerequisite reproduces or retires the issue, Tom must choose the semantic owner and identity evolution. This ticket therefore belongs in `awaiting-decision`, not the executable ready queue.

## Scope note

`crates/tiler-ir/**` is `implementation/ir` and `crates/tiler-compiler/**` is `implementation/compiler`; the remedy may need one or both, so both are declared. Read from `ticketsplease.toml` rather than asserted.

## Closes when

The prerequisite has established the current behavior, Tom has chosen the stage-identity, sharing, or request-refusal contract, and the chosen remedy is implemented with its identity/domain and diagnostic consequences derived, a regression carrying the same-shaped pair, and the different-extent and one-chain controls preserved.
