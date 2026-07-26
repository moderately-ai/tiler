---
id: review-the-tiler-ir-identity-namespace
title: Review or narrow the public tiler_ir::identity namespace
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, ir, decisions, identity]
---
`relocate-abi-expressions-into-tiler-ir` added `pub mod identity` to `tiler-ir` (commit `d1a95e1`). ADR 0075 makes a new publicly reachable namespace an always-ask category, and unlike the `abi` module beside it — which accepted ADR 0068 explicitly places in this crate — **no accepted decision covers this one**. It is a draft by default and is recorded here rather than left as an assertion in a conversation.

**What it is.** Two functions, `push_len` and `push_slice`, writing the canonical fixed-width big-endian length prefix every identity digest in the workspace is framed with.

**Why it was made public rather than kept private.** It had four definitions — `tiler-ir/src/program/model.rs`, `tiler-ir/src/kernel/model.rs`, the relocated ABI module, and `tiler-artifact/src/program/model.rs` — and `tiler-artifact`'s codec imported a fifth path to one of them. They had already drifted in form: the kernel copy narrowed with `len as u64` where the others used a checked `u64::try_from`. Because `tiler-artifact` is a separate crate, one definition serving all five callers has to be `pub`.

**The question review must actually settle.** Publishing this says the canonical length framing is part of `tiler-ir`'s public contract, which is a real commitment: a consumer could depend on the exact byte framing of identities. The alternatives are (a) keep it public and state the framing as a governed contract, (b) make it `pub(crate)` and give `tiler-artifact` its own copy, restoring the duplication this closed, or (c) give the framing a named domain-separated encoder type rather than two loose functions, so the contract is nominal instead of structural.

**Inference, not measurement:** (c) looks right because the framing rule is already load-bearing across crates and a type can carry the invariant its doc comment currently only asserts. That is an argument, not evidence, and it should be tested against what an out-of-crate identity consumer actually needs before being adopted.

## Closes when

Either an accepted ADR places the canonical identity framing where it now lives with its public contract stated, or the namespace is narrowed and the decision recorded. `make full` passes.
