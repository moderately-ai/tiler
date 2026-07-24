---
id: complete-program-identity-with-abi-guards-and-routing
title: Complete program identity with ABI expressions, guards, and routing
status: todo
priority: p1
dependencies: []
related: [prototype-kernel-program-ir, prototype-artifact-program-model]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, identity, contracts]
---
`tiler_ir::program::CanonicalKernelProgramIdentity` is **not** the complete
program identity ADR 0072 describes, and the divergence is between an accepted
contract and the ticket that was executed — not an oversight in the code.

The tension, recorded by the implementing agent rather than hidden:

- **ADR 0072** states that complete program identity covers buffers, ABI, guards,
  and routing.
- **ADR 0068** places `AbiExpr` and its evaluation in `tiler_ir::program`, and
  **ADR 0070** lists `AbiExpr` under `program`.
- **`prototype-kernel-program-ir`** scoped ABI, guards, and routing to the
  artifact-facing projection instead, and that is what was built.

So today the identity folds the semantic graph identity, each stage's kernel
identity (which already carries its scheduled-region identity), the proven
disjoint occurrence partition, values, views, allocations, dependencies, and
named outputs — while host expressions, the applicability guard, and routing
remain compiler-owned and outside it. The name reads like the complete article
and is not, which is the part most likely to mislead a later reader.

Nothing is presently unsound: the identity is honest about what it covers, and no
consumer treats it as ADR 0072's complete identity. The risk is that one starts
to — for example a cache keyed on program identity that is blind to a differing
ABI expression or routing decision, which is precisely the "complete cache and
artifact identity" hazard `AGENTS.md` singles out.

**Resolve by moving `AbiExpr`, the applicability guard, and routing into
`tiler_ir::program` per ADR 0068/0070, folding them into the identity, and
bumping the canonical domain tag to `v2`** — the tag is versioned exactly so an
identity-semantics change is explicit rather than silent. Rebaseline the affected
fixtures deliberately and state in the Outcome that it is an intended identity
re-baseline.

If instead the *ADRs* are judged wrong — that ABI, guards, and routing genuinely
belong to the artifact projection — then amend ADR 0072 and ADR 0068 explicitly
rather than leaving an accepted contract describing something the code does not
build. Either outcome is acceptable; a silent standing divergence is not.

Also note for whoever takes this: ADR 0071's `VerifiedProgramPortfolio` remains
unimplemented, and the bounded profile restricts the layer in ways that will need
widening — one materialization per value (which blocks recomputation), a required
direct data edge rather than reachability, `MemorySpace::Device` only, and byte
counts derived from static `Shape` (symbolic extents need `ShapeEnv`).
