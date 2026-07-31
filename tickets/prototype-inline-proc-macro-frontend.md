---
id: prototype-inline-proc-macro-frontend
title: Implement the inline proc-macro frontend proof
status: todo
priority: p1
dependencies: [prototype-public-compiler-api, prototype-neutral-artifact-codec, admit-the-tiler-facade-and-proc-macro-crate-boundary, define-inline-symbol-binding-and-runtime-value-adaptation, promote-artifact-family-selection-for-the-frontend]
related: []
scopes: [implementation/frontend, implementation/compiler, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, frontend, proc-macro, inline-dx]
---
## Outcome (2026-07-30)

Tom approved candidate B: an expression-position `tiler::tensor!` macro with a leading declaration block, explicit symbolic extents and typed operands, ordinary Rust operators where they carry the intended logical operation, named calls for operations without an operator spelling, and `out` bindings returned to the surrounding Rust expression.

The declaration block is the surviving long-term surface because it states a shared symbol once, scales without repeating attribute runs across many operands, retains token-level spans for typed diagnostics, and can grow from operators into named operation calls without changing the region model. Candidate C is eliminated because a string literal cannot reliably preserve operand-level spans through indirection. Candidate A remains implementable but is rejected because its repeated attributes and method-only body become progressively noisier as regions, dtype declarations, and output cardinality grow.

This decision releases implementation. It does not authorize a runtime JIT, source scan, consumer build step, implicit inspection outside the macro invocation, or a second operation vocabulary disconnected from the public logical program.

## Decision record (2026-07-28; accepted 2026-07-30)

**The question, atomic:** what does the inline tensor region *look like* at a call site?

The ticket reserves it: "Tom reviews public syntax and ergonomics." That is not a boundary that can be drafted and swapped later in the usual way — the syntax **is** the product surface here, every example in the documentation corpus would be written against it, and a later change rewrites all of them rather than just a signature.

**A decision needs to cover four things at once**, so the answer is usable in one pass rather than three: the region delimiter and whether it is expression- or item-position; how a tensor operand names its shape and dtype; whether operations are written as method chains, operators, or an einsum-like string; and how an output is bound back into surrounding Rust. Each candidate below answers all four, over the same program, so the comparison is like for like.

### The program, three ways

One elementwise program: `d = (a * b) + c`, over `f32` tensors of one symbolic extent `n`, with the result bound back into a Rust `let`.

**Candidate A — expression-position macro, attribute-style declarations, method chains.**

```rust
let d: Tensor<f32> = tiler::tensor! {
    #[dtype(f32)] #[shape(n)] a,
    #[dtype(f32)] #[shape(n)] b,
    #[dtype(f32)] #[shape(n)] c,
    => a.mul(b).add(c)
};
```

Delimiter and position: brace-delimited, expression position, so it composes anywhere a value does. Operand spelling: one attribute per property, per operand, which reads like the rest of Rust and gives every property its own span. Body: method chains, so operation names are the compiler's own vocabulary and a misspelling is a name error at the method's span. Result: ordinary `let`, type annotated by the caller.

**Candidate B — expression-position macro, leading declaration block, operator overloading.**

```rust
let d = tiler::tensor! {
    sym n;
    in a: f32[n], b: f32[n], c: f32[n];
    out (a * b) + c
};
```

Delimiter and position: identical to A. Operand spelling: a declaration block in a small DSL, with the symbolic extent declared once by name and each operand given `dtype[shape]` in one token run. Body: Rust operator syntax, so the program reads as the arithmetic it is. Result: `out` names the single result expression; the `let` takes it without annotation because the macro knows the dtype.

**Candidate C — einsum-like string with a separate binding list.**

```rust
let d = tiler::tensor!("i -> i: (a[i] * b[i]) + c[i]", n = n, a = a, b = b, c = c);
```

Delimiter and position: parenthesized call, expression position, body inside a string literal. Operand spelling: index letters in the string; dtype and extent come from the bound Rust values rather than from the region text. Body: an index-notation string, which is the notation the einops-style frontend already trades in. Result: ordinary `let`.

### Options

| | A — attributes + method chains | B — declaration block + operators | C — indexed string |
| --- | --- | --- | --- |
| **Enables** | Every operand property has its own span, so a typed error points at the *dtype* rather than at the operand. Extending the operand vocabulary is adding an attribute, which needs no grammar change. | The shortest declaration of the three, and the body is Rust arithmetic a reader already knows. One symbol declaration serves every operand, which is what a real multi-tensor region needs. | Contraction and reduction are expressible in the notation itself, so the surface does not grow a new form when the program stops being elementwise. Closest to the `candle-einops` frontend this project names as an initial use case. |
| **Prevents** | Verbose at three operands and worse at ten; the attribute run repeats what one `sym n;` line says once. Operator syntax is unavailable, so the arithmetic reads as calls. | Two grammars in one file — Rust and the block's DSL — and the block's grammar has to be documented and versioned like a language. Operator overloading means an operation set bounded by what Rust operators spell; anything else falls back to calls anyway. | **A typed error can name the region but not the operand.** The body is one string literal, so every span inside it is the literal's span unless the macro reconstructs sub-spans, which is possible only for a literal written in place and not for one produced by any indirection. That trades away the frontend's stated obligation to "report span-aware typed errors" at the granularity that makes them useful. |

### What the constraints already eliminate

The accepted inline developer experience removes a whole class of answers before taste is consulted: no consumer `build.rs`, no duplicated registry, no source scan, no Cargo subcommand, no prepare step, no runtime source JIT, and each invocation a self-contained AOT and embedding unit. Any syntax that needs the macro to see *outside* its own invocation — a region referring to tensors declared in an earlier macro call, or a program assembled across statements — is eliminated by that rule rather than by preference; broader fusion requires a larger explicit inline region.

The original analysis left the choice between A, B, and C to product taste. The accepted outcome above resolves that choice in favor of B and strengthens the span consequence against C into an elimination under the retained operand-level diagnostic requirement.

## What is settled

Parse one visible region, construct the public logical program, invoke the ordinary compiler boundary, report span-aware typed errors, emit generated Rust, and preserve the accepted inline developer experience—no consumer `build.rs`, no registry, no source scan, no prepare step, no runtime JIT, each invocation a self-contained AOT and embedding unit.

The 2026-07-30 readiness audit found that implementation was not yet reachable from syntax approval alone. `admit-the-tiler-facade-and-proc-macro-crate-boundary` owns the `tiler` + `tiler-macros` workspace/public path; `define-inline-symbol-binding-and-runtime-value-adaptation` gives `sym n` and the returned `let d` executable meanings over the promoted ShapeEnv surface; and `promote-artifact-family-selection-for-the-frontend` exposes the delivery request without copying its canonical authority. This ticket consumes those reviewed boundaries rather than inventing them during parsing.

## Activated 2026-07-30

Tom's approval of candidate B released the syntax question. The ticket becomes ready for implementation after the three exact dependencies above deliver; until then it must not scaffold a macro whose symbols, result value, or delivery policy have no public owner.
