---
id: refuse-two-structurally-identical-output-chains-by-name-not-as-compiler-output
title: Refuse two structurally identical output chains by name, not as invalid compiler output
status: todo
priority: p2
dependencies: []
related: [bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, ir, multi-output]
---
## User-visible outcome

A program declaring two ordered named outputs whose producer chains are structurally identical is refused by a named request-boundary or recognition rule, or compiled, rather than reaching the caller as `InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))` — the compiler reporting its own defect for a caller's valid program.

## Why this exists

**Measurement, 2026-08-06, on `tkt/bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals` at base `afdac9c9`.** Two independent epilogue chains over two declared inputs — `sum(x * x, axis 1) * 2.0` published as `sx` and `sum(y * y, axis 1) * 3.0` published as `sy`, both at `[1, 4]` — fail `compile` with:

```
InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))
```

Reproduced with the same fixture differing only in the prologue expression (`x * x` against `y + y`), which also fails, and *not* reproduced when the two chains fold different extents (`[1, 4]` and `[1, 2]`), which compiles and retains six- and seven-dispatch alternatives. So the discriminator is the assembled stages' canonical keys and not the declaration: two chains over different declared inputs at the same shape assemble stages the shared program layer cannot tell apart.

**Inference — the error class is wrong whatever the remedy is.** `AmbiguousCanonicalKey` is a `tiler_ir::program` core-verification refusal, reported through `CompilerOutputError::Program`. Nothing about the submitted program is invalid: both outputs are independently recognized, their walks partition the occurrences, and each publishes its own value. Either the program layer's stage key must distinguish two structurally identical stages (a `tiler-ir` change), or the compiler must refuse the shape by name before assembly, or the plan must reuse one stage for both. Which of the three is right is the research this ticket owns.

## Scope note

`crates/tiler-ir/**` is `implementation/ir` and `crates/tiler-compiler/**` is `implementation/compiler`; the remedy may need one or both, so both are declared. Read from `ticketsplease.toml` rather than asserted.

## Closes when

The measured two-chain program either compiles or is refused under a named rule whose error class attributes the refusal to the request, with a regression test carrying the fixture, and the chosen remedy's derivation recorded.
