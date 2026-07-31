---
id: close-or-retype-the-operand-permutation-inference
title: Close or retype the operand-permutation inference in the first Metal profile
status: todo
priority: p2
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, measure-macos-apple9-f32-under-unified-msl4-profile, admit-measured-compile-profile-sources-across-fact-families]
scopes: [research/target-profiles, research/apple-targets, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, metal, target-profile, provenance]
---
## User-visible outcome

The first authoritative macOS Metal profile's operand-permutation row is either an isolated **Measurement** like its four neighbours, or it is typed in a way that stops a reader mistaking it for one.

## Why this is a remainder rather than a defect

**Fact.** The [compile-profile authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md) labels operand permutation an `Inference` and states why: contraction, reassociation, signed zero, and the NaN/infinity assumptions are each isolated by an emitted fast-math attribute or by a result lane that separates the math modes, and operand permutation has neither. It is delivered by the same `safe` compilation, whose attribute strings carry no relaxation at all, so a permutation relaxation would have to be one the front end applied without recording it.

**Fact.** `tiler_build::BoundMetalCompileDeclaration` declares the row through `declare_measured_permutation`, so it carries `FactAuthority::MeasuredProfile` in the descriptor exactly like its four neighbours, and only a code comment and the ledger distinguish it. That was the brief's instruction for the parent ticket — the label lives in the source documentation — and it is a real asymmetry between what the descriptor says and what the evidence supports.

**Inference.** Nothing downstream is wrong today: the row's *value* (permutation forbidden) is what the `safe` realization delivers, and the compiler consults it identically either way. What is unproven is the strength of the evidence, and the profile descriptor cannot currently express the difference.

## Work

1. Attempt the cheaper close first: cite MSL 4.0's normative statement of what `-fmetal-math-mode=safe` guarantees about operand order, if one exists. A citation retypes the row as an external normative guarantee and closes this outright.
2. Otherwise retain one kernel under the exact offline compiler whose result distinguishes an operand order — the ledger's own stated closing condition — and promote the row to an isolated measurement beside its neighbours.
3. If neither is reachable, decide whether the compiler's fact-source vocabulary should carry a distinct authority class for a fact *inferred from* a measurement, and record the elimination. Adding an authority class is a public compiler boundary and is Tom's.

Do not close this by deleting the row: a profile with no permutation row resolves `Unknown`, which refuses the governed numerical contract the bounded serial sum compiles under.

## Closes when

The row is either an isolated measurement, an external normative guarantee, or a typed evidence class that a descriptor reader can tell apart from its neighbours — and the ledger's fourth outcome is updated to match.
