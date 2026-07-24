---
id: carry-reconstructable-kernel-programs-in-the-neutral-envelope
title: Decide what a decoded artifact envelope must reconstruct
status: todo
priority: p1
dependencies: []
related: [prototype-neutral-artifact-codec, prototype-metal-bundle-assembly]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization, ir]
---
`prototype-neutral-artifact-codec` framed a neutral program section carrying one packaged variant's *canonical kernel-program identity*, not the program. A decoder therefore proves which program an artifact names and cannot resurrect it.

**Fact — the blocker is structural, not an omission.** `tiler_ir::program::KernelProgramBuilder::new` takes a `&SemanticProgram`, and a `SemanticProgram` requires a frozen `SemanticRegistry` holding `Arc<dyn OperationInferencer>` implementations. Neither is representable as bytes, so no codec can rebuild a `VerifiedKernelProgram` from an envelope alone without a consumer-supplied registry.

**Two consequences the codec states rather than approximates.**

- A consumer that needs the program must hold the one it compiled; the envelope binds them by identity.
- A multi-stage program's *stage execution order* is not recoverable, because the envelope orders entries by canonical stage key (as identity does) and does not carry the dependency graph. The codec emits `tiler.artifact.feature.multi-stage-program` for such an artifact and its own reader refuses to read it, which is the fail-closed form of that gap.

**What closes this ticket.** One of: (a) a decided contract that the envelope never reconstructs a program, with the runtime's program-binding path written down and the multi-stage feature replaced by carrying execution order and dependency obligations explicitly; or (b) a neutral program section that a registry-supplied decoder can drive back through `KernelProgramBuilder`, with the registry dependency stated at the API boundary. Either way `docs/artifact-abi.md`'s "A decoder must reconstruct shared IR through its checked builders" sentence must end up true or amended.

**Not closed by** adding fields speculatively: the codec deliberately does not carry a value only the program can establish, because a carried copy would let a forged envelope assert a range no verifier examined.
