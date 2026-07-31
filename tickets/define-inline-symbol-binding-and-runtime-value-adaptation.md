---
id: define-inline-symbol-binding-and-runtime-value-adaptation
title: Define inline symbol binding and runtime value adaptation
status: todo
priority: p1
dependencies: [promote-the-symbolic-index-profile-to-a-public-boundary]
related: [prototype-inline-proc-macro-frontend, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/frontend, implementation/ir, implementation/runtime, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The approved `sym n; in a: f32[n], ...; out ...` region binds every symbolic extent from actual operand metadata through one checked ShapeEnv environment, validates repeated uses consistently, and returns a consumer-neutral result through an explicit adapter rather than assuming a concrete tensor library.

## Correctness-derived binding

`sym n;` declares one logical extent variable. Its runtime value is unified from every operand dimension that names `n`; at least one occurrence must source it, and every additional occurrence must equal the first checked value. The macro does not inspect values outside its invocation, infer dtype/shape at expansion, or choose one operand occurrence as a semantic identity authority. Generated binding facts name exact input keys and axes through the promoted ShapeEnv vocabulary, so declaration order does not change graph identity.

**Ratified by Tom on 2026-07-30.** Operand unification is the default meaning of the approved `sym n;` form. Expansion selects a canonical input-dimension source independent of declaration order, emits equality obligations for every other occurrence, and reports unbound or inconsistent symbols with typed span-local errors. Future explicit source syntax remains reserved for interface parameters or target properties; it does not replace the ergonomic operand-derived form.

The frontend lowers operations through the public logical operation registry. It emits one adapter-neutral invocation over traits owned by the `tiler` facade: read-only dtype/shape/storage metadata, checked runtime input binding, and construction of the result value.

**Ratified by Tom on 2026-07-30.** The public runtime-value boundary is a facade-owned opaque wrapper parameterized by a consumer-supplied adapter. The facade contract describes only the capabilities Tiler needs and exposes no Candle, Metal, or other consumer-specific type, lifetime, storage layout, allocation policy, or device object. An integration owns its adapter and the conversion into and out of the wrapper; the wrapper may carry the adapter's value and context without making either part of graph semantics or artifact identity. Raw foreign values plus an adapter argument at every invocation remain unnecessary surface area, and a global adapter registry is forbidden. The bounded proof uses an independent test adapter. Candle is neither an implementation target nor a design authority for this ticket.

## Required evidence

Compile-pass fixtures bind one symbol from one and multiple operands and return the declared output. Typed span errors cover unbound symbols, inconsistent repeated extents, rank/dtype mismatch, unsupported adapter capability, and multiple outputs beyond the bounded profile. Generated tokens contain no source scan, runtime JIT, external file reference, or dependency on a consumer's undeclared internal crate. Each negative check is perturbed once and observed failing.

## Closes when

The exact ShapeEnv-to-runtime binding and minimal opaque wrapper and adapter traits are compile-checked, the public facade boundary is reviewed by Tom, the proof demonstrates that an arbitrary external consumer can supply the adapter without a facade change or global registration, and `prototype-inline-proc-macro-frontend` can consume the boundary without inventing what `sym n` or `let d` means.
