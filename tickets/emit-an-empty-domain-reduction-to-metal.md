---
id: emit-an-empty-domain-reduction-to-metal
title: Emit a reduction over an empty domain to Metal
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [metal, numerics, correctness]
---
Exposed by `prototype-metal-runtime-proof`, whose success condition names an empty-domain reduction among the cases both device programs must agree on. It cannot be produced today, so that member of the proof matrix is absent and the proof is qualified rather than complete.

## The measurement

**Fact.** For a program whose reduced axis has extent 0 — `Shape::from_dims([4, 0])` through the standard scale-then-reduce builder in `prototypes/serial-sum-compile` — `compile_governed` succeeds and retains the usual two alternatives (one fused, one materialized with two kernels), but `emit_translation_unit` refuses **both**:

```text
MalformedKernel { rule: "unreferenced-buffer-parameter" }
```

Extents 1, 2, and 3 all emit and pass `require_declared_realization`. The refusal is specific to the empty domain, not to the shape being unusual.

**Fact — why, read from the source.** `crates/tiler-metal/src/emit.rs:410` derives the binding table from what the kernel body actually reads, then refuses when `bindings.len() != declared`. Its comment states the reason: a declared parameter the body never touches has no argument-table position to occupy, and emitting a signature that silently dropped it would change the ABI. A reduction over zero contributors never reads its input buffer, so the declared input parameter is unreferenced and the count check fires.

**Inference — the refusal is correct and the gap is upstream of it.** Fail-closed is the right behaviour for that check; silently dropping a declared buffer is exactly the ABI-changing move the rule exists to prevent. What is missing is not a relaxation of the check but the ability to express an empty-domain reduction at all: the kernel body should produce the reduction's identity element for every output element without reading its input, and the signature should then honestly declare only what it uses — or declare the input and reference it in a way the ABI still describes.

## Why it matters beyond one prototype

An empty domain is where a reduction's identity element is either right or silently invented, which makes it one of the more valuable numerical cases rather than a curiosity. A stack that cannot emit it also cannot test it, so the identity element for every reduction family is currently unexercised on device.

## Scope

Decide and implement how the Metal emitter expresses a reduction over an empty domain, including what the emitted signature declares and what the ABI therefore says about a buffer the body does not read. The decision is the substance here — the two candidate shapes above have different ABI consequences and should be compared explicitly rather than one being reached for.

Do not resolve it by weakening `unreferenced-buffer-parameter`. That rule is load-bearing, and a check relaxed to admit one legitimate case admits every accidental one with it.

## Closes when

A reduction over an empty domain emits, passes `require_declared_realization`, and dispatches on device with the reference's identity element as its result; the ABI consequence of the chosen signature is recorded; and `prototype-metal-runtime-proof`'s matrix regains its `empty-domain` class — `REDUCTION_CLASSES` in `prototypes/serial-sum-compile/src/main.rs` grows the third entry its comment reserves.
