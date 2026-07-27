---
id: review-the-tiler-ir-identity-namespace
title: Review or narrow the public tiler_ir::identity namespace
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/ir, implementation/artifact, implementation/cache, implementation/compiler, implementation/metal-aot, implementation/reference]
shared_scopes: []
paths: []
tags: [implementation, ir, decisions, identity]
---
`relocate-abi-expressions-into-tiler-ir` added `pub mod identity` to `tiler-ir` (commit `d1a95e1`). ADR 0075 makes a new publicly reachable namespace an always-ask category, and unlike the `abi` module beside it — which accepted ADR 0068 explicitly places in this crate — **no accepted decision covers this one**. It is a draft by default and is recorded here rather than left as an assertion in a conversation.

**What it is.** Two functions, `push_len` and `push_slice`, writing the canonical fixed-width big-endian length prefix every identity digest in the workspace is framed with.

**Current use.** The namespace is no longer an artifact-only convenience.
`tiler-artifact`, `tiler-cache`, `tiler-compiler`, `tiler-metal-aot`, and
`tiler-reference` all use the framing helpers, often at several identity
construction sites. Narrowing it now would recreate a workspace-wide
duplication rather than one local copy.

**User-visible outcome.** Every canonical identity must use one governed,
fallible length-framing rule, so artifacts, caches, compiler products, backend
provenance, and reference authorities cannot silently disagree. Consumers
should encounter the identity contract, not a pair of unexplained byte-pushing
utilities.

**The question review must actually settle.** Publishing the helpers makes
canonical length framing part of `tiler-ir`'s public contract. Determine whether
that ownership is correct and whether the public surface should remain the two
functions or become a nominal encoder that carries the invariant. A private
copy per consumer no longer survives the single-authority requirement and is
not a live alternative.

**Inference, not measurement:** a nominal encoder looks preferable because the
framing rule is already load-bearing across crates and a type can carry the
invariant its doc comment currently only asserts. Test that shape against the
actual consumers before proposing a public change.

## Closes when

The framing has one accepted owner and a reviewed public contract, every current
consumer uses that authority, and `make full` passes.
