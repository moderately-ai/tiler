---
id: denote-a-reduction-region-in-the-inline-macro-grammar
title: Denote a reduction region in the inline macro grammar
status: in-progress
priority: p2
dependencies: []
related: [package-a-multi-entry-bundle-from-one-expansion, admit-multi-input-elementwise-programs-at-the-compiler-boundary]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, inline-dx]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785612950
---
## User-visible outcome

A `tiler::tensor!` region can denote a reduction, so the inline frontend reaches the one whole-program shape the compiler recognizes besides a pointwise chain.

## Why this exists

**Fact.** `crates/tiler-macros/src/region.rs` resolves exactly two operators — `*` and `+`, to `multiply_f32_op` and `add_f32_op` — over operands declared by `in` statements, and its `ELEMENT_TYPES` holds one row. A region therefore denotes a pointwise chain and nothing else. There is no reduction spelling and no scalar-constant literal.

**Fact.** `crates/tiler-compiler/src/request.rs`'s `select_supported_strategy` tries three whole-program shapes: `normalize_serial_sum`, `normalize_contraction`, and `normalize_pointwise`. Only the third is reachable from a region today. The serial-sum window is narrow and measured rather than guessed: exactly one input, one output, and 4–5 operations in the form `strict_serial_sum(input * c_scale + c_bias, axes)` — so a bare `sum(x, axis)` is *not* recognized, and reaching the window needs a scalar-constant literal in the grammar as well as a reduction.

**Measurement — base `2aa0824`, 2026-08-01.** Compiling grammar-admissible programs against the bound macOS declaration under `FlushSubnormalsToZeroF32`: a pointwise chain retains exactly **one** plan alternative (fused, one kernel), across input counts 2–8, chain depths 1–32, and extents 4–1048576. The recognized serial-sum program retains **two** (fused/1 kernel and materialized/2 kernels). A multi-input reduction is refused before any target-qualified trace with `UnsupportedCapability { rule: "input-arity" }`; softmax, RMS-norm, and SiLU programs are refused with `rule: "operation-set"` or `"operation-family"`.

**Inference — this is why a pointwise region can never be multi-entry.** The governed physical provider's pointwise branch offers an implementation only when a region's members equal the whole recognized pointwise member set, so every multi-region cover has a region with no implementation and contributes no plan at all. A pointwise region has one alternative not because fusion is cheaper but because nothing else is *constructible*; no cost model, present or future, changes that. Widening what a region can denote is the only route to a region with more than one plan.

## Implementation keys

The grammar addition must denote a *computation*, never a plan: a region says it sums an axis, and the optimizer decides whether that is one kernel or several. A spelling that asks for a number of kernels, a materialization, or a pass count is the thing this ticket exists to avoid.

Two things are needed together, and the ticket is not deliverable with one: a reduction spelling over a named axis, and a scalar literal in the expression grammar. Reaching the recognizer's 4–5-operation window with only the first is impossible, and landing only the first would ship a region that parses and then fails to compile with a capability refusal — a worse consumer experience than not admitting it.

`region.rs`'s existing discipline holds unchanged: the operator resolves to the registry's own operation key rather than to a meaning defined here; the derived result shape is checked against the shape the registry infers; and a symbolic extent still defers the program rather than substituting an invented one. A reduction changes the result's rank, so the derived-versus-inferred check becomes load-bearing in a way it is not for shape-preserving operators.

## Required evidence

A region denoting the recognized serial-sum shape parses, lowers to a verified `SemanticProgram`, and compiles against the bound macOS declaration. The registry-inferred result shape and the module-derived one agree, and a deliberately wrong derivation is watched failing as `ResultShapeDisagreement`. An out-of-window region — a bare reduction with no scalar arithmetic, or a reduction over more than one input — is refused with a spanned diagnostic naming what a consumer would change, not with a raw capability refusal leaking through.

## Closes when

A region denotes a reduction, the expansion compiles it, the public grammar surface is accepted by Tom, every new check is perturbation-proved, and targeted tests plus the batch gate pass.

## Graph maintenance

- **Public boundary, not self-accepted.** The region grammar is consumer-visible syntax. The reduction spelling, the axis-naming form, and the scalar-literal form each go to Tom under ADR 0075 before acceptance; a working implementation is a concrete draft of the syntax, not approval of it.
- This ticket does **not** deliver a multi-entry bundle and must not be reported as doing so. It makes a region with more than one plan alternative *expressible*; which alternative is selected is `calibrate-and-activate-parallel-reduction-selection`'s, and the end-to-end packaging is `package-a-multi-entry-bundle-from-one-expansion`.
- Reaching further shapes — softmax, RMS-norm, SiLU, contraction — is a compiler-recognizer question owned by `admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary`, not a grammar one.
