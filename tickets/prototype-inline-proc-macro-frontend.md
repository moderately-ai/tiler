---
id: prototype-inline-proc-macro-frontend
title: Implement the inline proc-macro frontend proof
status: in-progress
priority: p1
dependencies: [prototype-public-compiler-api, prototype-neutral-artifact-codec, admit-the-tiler-facade-and-proc-macro-crate-boundary, define-inline-symbol-binding-and-runtime-value-adaptation, promote-artifact-family-selection-for-the-frontend]
related: []
scopes: [implementation/frontend, implementation/compiler, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, frontend, proc-macro, inline-dx]
claimed_from: todo
assignee: worker-inline-frontend
lease_expires_at: 1785536988
---
## User-visible outcome

An external Rust consumer imports only `tiler`, writes the accepted inline tensor region, receives span-local typed diagnostics, and executes a self-contained AOT result without a build script, source scan, runtime JIT, or consumer-specific compiler dependency.

## Implementation keys

Consume the admitted facade/macro crates, reviewed public compiler and neutral artifact codec, exact ShapeEnv/runtime-value adapter boundary, and canonical artifact-family request. Parse the approved declaration-block grammar into the public logical program, preserve token spans through typed failures, embed the selected artifact family, and generate only paths reachable through the consumer's declared `tiler` dependency.

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

## Graph maintenance

- Keep all five declared prerequisites complete before claiming this implementation; syntax approval alone does not create its public compiler, value, or artifact authorities.
- File broader fusion as a larger explicit inline-region ticket rather than inspecting surrounding Rust or creating an ambient registry.
- Keep consumer-specific adapters and model integration downstream of this consumer-neutral frontend proof.

## Outcome (2026-07-31)

`tiler::tensor!` parses the approved candidate-B grammar, resolves it against the governed semantic operation registry, derives and binds every symbolic extent through `ShapeEnv`, states and validates its artifact-family delivery, and expands to code that binds an out-of-tree consumer's own tensors and returns one. The approved example runs end to end from a crate that depends on `tiler` alone.

### What expands

```
region    := statement* body
statement := "sym" ident ("," ident)* ";" | "in" operand ("," operand)* ";"
body      := "out" expression
operand   := ident ":" element-type "[" axis? ("," axis)* ","? "]"
axis      := ident | integer-literal
expression:= operand-reference | "(" expression ")" | expression ("*"|"+") expression
```

`sym` and `in` repeat; `out` is terminal and takes the rest of the invocation. `*` binds tighter than `+`, as in Rust. An invocation evaluates to `Result<A::Value, tiler::value::BindError<A::Error>>` — the consumer's own tensor type, or a typed refusal naming the operand and axis that failed. It is a `Result` because the operand-count, rank, stored-scalar and symbol-equality checks a region owes are decidable only against the values it is handed, and a region that cannot honour its declared interface must refuse rather than return a value derived from a shape it never verified.

### Two findings that shaped the scope, both measured

**The semantic layer is fixed-extent, so a symbolic region has no expansion-time program.** `tiler_ir::shape::Shape` is "Target-independent **fixed** shape vocabulary" and `Extent` wraps a `u64`; a region's `sym n` binds at `LiveDevicePreflight` from operand metadata. A region whose every extent is a literal is therefore constructed and verified as a real `SemanticProgram` through `SemanticProgramBuilder` and the governed `F32Multiply`/`F32Add` facades, and the shape its registry *infers* for the result is required to equal the shape this frontend *derived*. A region carrying a symbolic extent is recorded as `ProgramEvidence::DeferredSymbolicExtent` rather than built over invented extents, because a program over substituted extents is a different program and its identity would name something no consumer wrote. Filed as `carry-symbolic-extents-into-the-semantic-program`.

**The compiler is not invoked, and that is measurement rather than descoping.** This ticket's settled text says the frontend "invoke[s] the ordinary compiler boundary". A temporary `tiler-compiler` integration test at base `b623670` built the approved region as a semantic program over three `f32[4]` inputs and called `compile_governed` under all four `NumericalContract` values and `compile` under the governed target profile. All five returned `UnsupportedCapability { rule: "signature" }` before any target-qualified trace: `normalize_pointwise` and `normalize_serial_sum` both open with `program.input_count() != 1`, so a three-input region matches neither recognizer. Since `docs/integration/frontends.md` requires target-neutral optimizer and verifier failures to be *unconditional* `compile_error!` diagnostics, wiring the compiler in would have made the region Tom approved a compile error at every call site. No `tiler-macros` → `tiler-compiler` edge was added. Filed as `admit-multi-input-elementwise-programs-at-the-compiler-boundary`.

### Where the code went

- `crates/tiler-macros/src/tokens.rs` — a span-generic copy of the invocation's tokens. It exists because `proc_macro::Span` cannot be constructed and `TokenStream::from_str` panics outside an expanding macro, so a parser written against those types would have diagnostics no test could observe. The conversion is the only untestable part and decides nothing.
- `crates/tiler-macros/src/grammar.rs` — the shape of the region text, and nothing about meaning. Fifteen refusal variants, each carrying the span of the token that caused it.
- `crates/tiler-macros/src/region.rs` — meaning: element types, operand references, the registry's own elementwise rule, `RegionDeclarations` → `BoundRegion`, the specialized `SemanticProgram` where representable, and the emitted facts.
- `crates/tiler-macros/src/lib.rs` — emission. Operand identifiers carry the spans the region's `in` list wrote them at; everything else carries the call site.
- `crates/tiler/src/expansion.rs` — `bind_and_build`, the one item generated code calls.

`binding`'s `RegionDeclarations` is now populated from real tokens, so its crate-wide `#![allow(dead_code)]` is gone, replaced by three narrow item-level allows that name what actually reserves each. `cache_root`'s allow was corrected: it named this ticket as its consumer, and this ticket does not consume it — every region states `FallbackOnly`, which invokes no backend compiler, so there is nothing to cache, and resolving a root anyway would let an unset `HOME` refuse an expansion that opens no cache. `generate-cfg-gated-artifact-family-delivery` is the slice that consumes it. `delivery::stated_policy`'s "while `tensor!` has no grammar" reasoning was replaced with the real one: the approved grammar admits no family statement.

The inert `ExpansionAnchor` and `expansion_anchor` are removed rather than kept beside the real expansion — their own documentation said the grammar tickets replace them.

### Public surface added

**Accepted surface** — nothing. The `tiler::tensor!` path, `tiler::value`, and the `__private` region-facts items were accepted on 2026-07-31 and are used unchanged.

**Needs acceptance** (ADR 0075), one item and one contract:

| Item | Form |
| --- | --- |
| `tiler::__private::bind_and_build` | `pub fn bind_and_build<A: TensorAdapter>(facts: &RegionFacts, operands: &[&Tensor<A>]) -> Result<A::Value, BindError<A::Error>>` |

It is `#[doc(hidden)]` and carries no compatibility claim, but it is publicly reachable and the accepted packet enumerated the `__private` items by name, so adding one is not self-accepted. It exists because generated code cannot name the adapter: `build_result`'s `A` appears only in `&A::Context` and in its return type, and an associated type is not injective, so `A` is inferable only from a `&[&Tensor<A>]`. The accepted pair stays public and unchanged, and `runtime_value_adapter.rs` exercises both directly and asserts the composition agrees with them.

The second is not an item but is consequential: **an invocation evaluates to a `Result`**. Tom approved the syntax, not the evaluated type, and `let d = tiler::tensor! { … }` binding a `Result` is what a consumer sees.

**Removed:** `tiler::__private::ExpansionAnchor` and `tiler::__private::expansion_anchor`.

**Not added:** no new public item on `tiler-macros` (every module is crate-private), and no new public hook on `tiler-compiler` — the frontend needed none, because it does not reach it.

### Grammar implemented vs deferred

Implemented: `sym`, `in` with element type and mixed symbolic/literal axes including rank 0, repeated declaration statements, trailing commas, `out` with `*`, `+`, parentheses, and Rust precedence.

Deferred, each failing closed with a span-typed error rather than an invented meaning: named operation calls (`relu(a)`) — the approved syntax reserves the form and this profile registers no such operation; every operator but `*` and `+`; every element type but `f32`, because `F32` is the only value-type marker the governed profile registers for plain tensor arithmetic; raw identifiers, which `Ident::new` panics on; invisible groups, which is how another macro hands over an already-parsed expression a region cannot see the operand names of; and any syntax for stating an artifact family, which is a public-boundary decision owned by `generate-cfg-gated-artifact-family-delivery`.

### Diagnostic evidence

`crates/tiler/tests/facade/fail/` holds three compile-fail fixtures with byte-compared goldens, out of tree. The span targets:

| Refusal | Caret |
| --- | --- |
| unregistered element type | the element type (`f64`), not the operand — the granularity that eliminated candidate C |
| unsupported operator | the operator (`-`), and a compound operator (`+=`) reported whole rather than as its first character |
| named operation call | the name (`relu`) |
| malformed literal extent | the literal (`4usize`) |
| undeclared symbol | the axis that names it (`k`, inside the brackets) |
| unsourced symbol | its `sym` declaration |
| duplicate operand / duplicate symbol | the second declaration |
| unknown operand reference | the reference in the body |
| incompatible operand shapes | the operator that would have combined them |
| trailing tokens, non-keyword statement | the offending token |
| empty region, missing body | the invocation |

`generated_operand_reference_spans.rs` is the emission half and nothing else covers it: an operand with no Rust binding produces `cannot find value 'a' in this scope` with the caret on `a` in the `in` list, because that is the only part of a region that says where its values come from.

### Commands

- `cargo fmt --all --check`
- `cargo clippy -p tiler -p tiler-macros --all-targets --locked -- -D warnings` — clean
- `cargo nextest run -p tiler -p tiler-macros` — 81 tests, all passing (65 in `tiler-macros`, up from 41)
- `cargo test -p tiler -p tiler-macros --doc` — the facade's crate doc-test is now the approved region executing, not an inert anchor
- `tkt lint`, `git diff --check`, `tkt guard --base b623670`
- `make full`

### Perturbation evidence

Eleven perturbations, each applied alone, run, and restored.

| Perturbation | Result |
| --- | --- |
| `STATEMENT_PUNCT` neutralized | FAIL `tokens_after_the_result_are_refused` — `;` misreported as an unsupported operator |
| joint-operator refusal disabled | FAIL `an_unsupported_operator_is_refused_at_the_operator` — `a += b` read as `a + (= b)` |
| `literal_extent` digit check removed | FAIL `a_literal_extent_must_be_a_plain_integer` — `4u64` silently became extent **464** |
| named-call detection removed | FAIL `a_named_operation_call_is_refused_at_its_name` — `relu(a)` parsed as operand `relu` |
| `+`/`*` precedence inverted | FAIL ×5 including `precedence_matches_rust` |
| `ELEMENT_TYPES` widened with `f64` | FAIL `an_unregistered_element_type_is_refused_at_its_own_token` |
| elementwise shape rule disabled | FAIL ×2: `incompatible_operand_shapes_are_refused_at_the_operator` **and** `a_scalar_operand_broadcasts_and_the_result_takes_the_shaped_side` with `ResultShapeDisagreement { derived: "[]", inferred: "[4]" }` |
| early undeclared-symbol check removed | FAIL `an_undeclared_symbol_is_refused_at_the_axis_that_names_it` |
| program construction always deferred | FAIL ×2 including `a_static_region_is_constructed_as_a_public_logical_program` |
| operand identifiers respanned to the call site | FAIL `generated_operand_reference_spans` golden — both carets degraded to the whole invocation |
| `stated_policy` returns a selected macOS family | FAIL: every pass fixture refused with a spanned `compile_error!`, and no region expanded |

The seventh is the strongest and was not anticipated. With the frontend's elementwise rule disabled, the *static* case was caught independently by the semantic registry itself — `provider tiler::standard-semantics@7 rejected operation tiler::multiply-f32@1: binary.shape: operand shapes must match or one operand must be scalar` — which is the exact sentence the frontend rule quotes. That is direct evidence the rule is the registry's rule restated for the symbolic case, not a second authority with its own opinion.

### Unsupported cases, refusing explicitly

- A symbolic region has no expansion-time public logical program (above), and therefore no semantic identity, optimizer pass, or artifact.
- A region's operand names must be bindings of type `Tensor<A>`; the expansion emits `&a`, so a binding already of type `&Tensor<A>` does not compile. Not refused with a typed error — rustc reports the type mismatch at the `in` list.
- `RegionBindError::NoOperands` is unreachable through the grammar: every body atom is an operand reference, so a region with no `in` statement is refused as an unknown operand first. It remains `binding`'s authority for any other caller and its own test exercises it.
- More than one result, operands on different adapter contexts, per-value storage properties, and storage access of any kind remain outside the bounded profile, unchanged from `define-inline-symbol-binding-and-runtime-value-adaptation`.
- Two axes naming *different* symbols are not one shape. Nothing at expansion time proves `n` and `m` take one value, so treating them as compatible would defer a shape error into a wrong result.

### Deliberately not done

No expansion cache is opened, no backend compiler is invoked, no artifact bytes are embedded, and no `#[cfg]`-gated delivery is emitted. Every region states `FallbackOnly`, which ADR 0053 defines as an explicit valid policy invoking no backend compiler; the delivery half is `generate-cfg-gated-artifact-family-delivery`'s and the embedding half is `prototype-macro-embedding-and-cargo-behavior`'s, and both depend on this ticket.

### Graph maintenance performed

- Filed `carry-symbolic-extents-into-the-semantic-program` (p1, research) and `admit-multi-input-elementwise-programs-at-the-compiler-boundary` (p1, implementation), both with the reproducing check in the body.
- Corrected `cache_root`'s and `binding`'s `allow(dead_code)` reasons, which named this ticket as their consumer.
- `generate-cfg-gated-artifact-family-delivery`'s two stated implementation blockers are now cleared: `crates/tiler-macros/**` exists and this ticket, its remaining dependency, is complete.
