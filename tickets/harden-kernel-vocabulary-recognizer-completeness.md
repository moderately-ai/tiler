---
id: harden-kernel-vocabulary-recognizer-completeness
title: Keep the kernel-IR vocabulary recognizable by its backends
status: todo
priority: p2
dependencies: [resolve-non-exhaustive-recognizer-hole]
related: [harden-public-enums-non-exhaustive, resolve-non-exhaustive-recognizer-hole, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, api-hardening, correctness]
---
ADR 0074's convention 5, amended on 2026-07-24 by
`resolve-non-exhaustive-recognizer-hole`, records the twelve `#[non_exhaustive]`
vocabulary enums of `crates/tiler-ir/src/kernel/model.rs` as knowingly
non-conforming under clause 5c: `tiler-metal` is an out-of-crate recognizer that
matches them to decide what it can emit, so the attribute converts every future
IR capability into a silent `UnsupportedOperation` rejection at that backend
rather than a compile error naming the site that must decide.

The enums are `KernelType`, `AddressSpace`, `BufferAccess`, `Builtin`,
`KernelConstant`, `BinaryOp`, `CompareOp`, `ConvertOp`, `ExecutionScope`,
`MemoryScope`, `BarrierOrdering`, and `OperationView`. Remove `#[non_exhaustive]`
from each and make every out-of-crate match over them explicit.

**The retrofit is a net deletion, not a cost.** Read against `crates/tiler-metal/src/emit.rs`
at commit `37f1350`, ten of the emitter's wildcard arms are already unreachable —
`msl_type`, `builtin_parameter`, `emit_operation`, `emit_constant`, `emit_binary`,
`emit_compare`, `emit_convert`, `barrier_call`'s execution-scope and ordering
matches, and the `BufferAccess` match under `AddressSpace::Device` each list every
variant explicitly and then carry a `_` arm that exists only because the attribute
forces it. Three arms are live and each already names in a comment exactly which
known variants it rejects: `address_space_declaration` rejects `Workgroup` and
`InvocationPrivate`, its `Constant` branch rejects `BufferAccess::Write`, and
`fence_flag` rejects `Constant` and `InvocationPrivate`. Transcribe those comments
into explicit patterns so the rejection is a stated capability decision rather
than a fallthrough. `barrier_call`'s match on the `(ExecutionScope, MemoryScope)`
pair keeps its catch-all: a product match needs one whatever the attributes say,
and ADR 0074's convention 5 says so explicitly.

Expect `unreachable_patterns` to fire on any wildcard left behind once the
attribute is gone; the workspace gate denies warnings, so the compiler enumerates
the remaining work. Confirm no consumer outside `tiler-metal` recognizes these
enums before finishing — `tiler-compiler`'s uses of `OperationView` are
`if let`-shaped partial reads (ADR 0074 convention 5a) and must stay that way or
be converted deliberately.

This ticket changes no semantics and no canonical identity: the `tag()` encoders
for these enums live in `tiler-ir` alongside the definitions and already match
exhaustively, which the amendment's measurement confirms is unaffected by the
attribute either way. If a per-variant behaviour genuinely changes, say so
explicitly in the Outcome rather than folding it in.
