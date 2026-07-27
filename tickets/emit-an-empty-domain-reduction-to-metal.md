---
id: emit-an-empty-domain-reduction-to-metal
title: Emit a reduction over an empty domain to Metal
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/metal, implementation/ir, implementation/runtime, implementation/metal-aot]
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

## Outcome

Done. A reduction over an empty domain emits, honours its declared contract, and dispatches on device with the reference's identity element, in both plan roles. `prototype-metal-runtime-proof`'s matrix regains its `empty-domain` class and the proof is no longer qualified.

**The decision: the argument table is the declaration, not the body's reading order.** The ticket offered two candidate signature shapes and asked for the ABI consequence to be compared rather than assumed. The comparison eliminated one of them and neither was the shape the ticket sketched.

Emitting only the referenced buffers was eliminated on the ticket's own ground: the artifact's binding table is in declaration order and the runtime pairs artifact slot *i* with the emitted table's *i*-th transport, so dropping a parameter shifts every later ordinal and mis-binds. That is the ABI change `unreferenced-buffer-parameter` exists to prevent, and the rule was not weakened — it was removed because what it guarded against became unrepresentable.

**A doc claim had to be checked before it could be acted on.** `buffer_binding` stated that "the structured kernel IR does not expose the signature ordinal of a `VerifiedBufferId`", which is why indices were assigned in first-use order. That is **true**: `VerifiedBufferId::index` and `as_usize` are both `pub(super)`, so no external crate can read an ordinal, and the ordinal only *looks* available because `VerifiedKernel::buffer` resolves through it internally. So `tiler-ir` gained `VerifiedKernel::declared_buffers`, yielding each parameter with the handle naming it. Pairs rather than an index accessor: a consumer needs to relate a parameter to the loads and stores that reference it, and does not need to do arithmetic on a handle.

**A latent defect closed with it, and it is the more serious half.** First-use order and declaration order coincide only for a body that touches its buffers in declaration sequence. Nothing checked that they agreed, each side was internally consistent, and the artifact's own binding table is in declaration order — so a body that stored before it loaded would have bound the wrong buffer to each slot, silently. Every current kernel happened to be in order, which is why the goldens are unchanged. It is now correct by construction rather than by coincidence.

**Admitting the empty domain surfaced one consequence in the runner.** A materialized empty-domain route has a first stage that maps zero elements, so it legitimately covers no threads, and `plan_route` refused every zero-thread launch. That refusal was written when a route was a single entry, where no threads did mean no result. The artifact already *states* which of skip-or-encode an empty launch is, so the runner now reads it: a skipped entry is prepared in full — its pipeline is still built, so readiness does not depend on the operands — and simply not encoded, while a route demanding a zero-thread dispatch be encoded is still refused, because `dispatch_threads` has no meaning at zero.

**Evidence.** Two emitter tests, both confirmed to fail against first-use ordering. The empty-domain kernel emits with every declared parameter present at `[[buffer(0)]]` and `[[buffer(1)]]`; under the old scheme it produced one binding instead of two. The declaration-order test is the sharper one — with the input never read, first-use order gave the *output* table position 0, and the neutered run fails with `left: Output, right: Intermediate`, which is the mis-binding demonstrated rather than argued.

**Measurement — Apple M4 Max.** The full proof matrix is now six members: `{empty-domain, singleton, nontrivial}` times `{selected, materialized}`, five operand cases each. **30 cases, all agreeing bit for bit with the published reference.** `empty-domain.materialized` routes 2 dispatches over 1 shared allocation with its first stage skipped; `empty-domain.selected` routes 1.

Gate: `make full` green (971 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck).
