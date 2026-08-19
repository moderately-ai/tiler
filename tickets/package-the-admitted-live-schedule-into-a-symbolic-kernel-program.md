---
id: package-the-admitted-live-schedule-into-a-symbolic-kernel-program
title: Package the admitted live schedule into a symbolic kernel program
status: in-progress
priority: p1
dependencies: [admit-symbolic-extents-through-schedule-formation]
related: [deliver-an-artifact-family-from-a-symbolic-region, carry-live-extent-operands-through-the-artifact-envelope, associate-live-extent-operands-with-symbolic-semantic-interface-axes]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, identity, shapes, public-boundary, needs-tom]
claimed_from: todo
assignee: worker-symbolic-packaging-packet
lease_expires_at: 1787105011
---
## User-visible outcome

`compile()` of the admitted same-shape rank-one symbolic elementwise population packages its verified source-bound live plan into a verified kernel program and neutral artifact construction plan, so the `deliver macos;` chain's next wall moves from `program-assembly.named-output-symbolic` into artifact family delivery itself.

## Why this exists

[`admit-symbolic-extents-through-schedule-formation`](admit-symbolic-extents-through-schedule-formation.md) landed real schedule formation for this population: the fieldless source-bound live schedule is formed, intrinsically verified, request-bound, assessed, selected, and lowerable to a verified kernel. What remains is packaging, and it is refused by two deliberate fail-closed owners this ticket exists to lift coherently rather than as an implementation fallback:

- **Fact — the compiler assembler's own gate.** `crates/tiler-compiler/src/program.rs`, anchor `named-output-symbolic`: a symbolic named output has no static shape for physical assembly to size, so `CoverAssembly::from_plan` declines with the typed program-assembly capability refusal the frontend now renders as the next wall.
- **Fact — the shared kernel-program identity property.** `crates/tiler-ir/src/program/error.rs`, anchor `SymbolicInterfaceExtent`: the shared builder refuses a symbolic interface boundary so that "no symbolic program reaches a packaged artifact" stays a property of the type — "a symbolic program cannot reach one and cannot ship with its shape-environment subject unrepresented in the artifact's three carried subjects". Lifting it is an identity decision about the artifact's carried subjects, not a code path.
- **Fact — the operand row already exists.** [`carry-live-extent-operands-through-the-artifact-envelope`](carry-live-extent-operands-through-the-artifact-envelope.md) is `done`: `AbiRoot::InputExtent` rows construct, encode, decode, validate, and bind at preflight. What that ticket deliberately did not do is represent the *interface subject* — the shape environment whose decoded root the live plan is bound to — in kernel-program and artifact identity.

## Required work

- Decide, as a packet for Tom, how a packaged symbolic program represents its shape-environment subject in kernel-program and artifact identity (a fourth carried subject, a folded environment identity, or a narrower spelling), preserving the stated invariant that the subject is never unrepresented and the live extent value never enters identity.
- Under the accepted spelling, lift `SymbolicInterfaceExtent` for exactly the admitted population: interface values sized by the zero-extent convention plus honest `AbiRoot::InputExtent` byte formulas rooted at the decoded input dimension (the input side of `build_cover_core` already does this; the internal/output side must join it).
- Replace the `named-output-symbolic` decline for the admitted population with real packaging; keep it, or a narrower named rule, for every other symbolic shape.
- One packaged program identity across bound extents stays testable; a bound value folded anywhere into program or artifact identity is a defect.

## Non-goals

Metal emission, `deliver` cache identity across extents, and the frontend contract flips — those remain [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md)'s. Semantic-interface association proof remains [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md)'s.

## Closes when

The admitted population's `compile()` returns a compiled product whose kernel program and artifact plan carry the represented shape-environment subject, or Tom rejects the representation and this records the retained typed decline.
