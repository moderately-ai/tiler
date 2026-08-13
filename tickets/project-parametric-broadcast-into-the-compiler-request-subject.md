---
id: project-parametric-broadcast-into-the-compiler-request-subject
title: Project parametric broadcast into the compiler request subject
status: todo
priority: p1
dependencies: [accept-the-parametric-broadcast-access-surface]
related: [carry-the-parametric-broadcast-relation-through-index-and-schedule-ir]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, identity]
---
## User-visible outcome

A scheduled region whose access map is the accepted parametric broadcast carrier projects into the compiler request subject under its own tag, so two different sourced mappings cannot share explain or plan identity.

## Exact gap

**Fact at `cefa74394ca81468409cdfc123e766227a78f178`.** `crates/tiler-compiler/src/request.rs` `encode_access_relation` matches `ReindexBijection` as `0x01`, `BroadcastReplication` as `0x02`, `LinearIdentity` as `0x03`, and every other `LogicalAccess` as `0x00`. The comment says the arm is a refusal to encode rather than a wildcard tag. [`carry-the-parametric-broadcast-relation-through-index-and-schedule-ir`](carry-the-parametric-broadcast-relation-through-index-and-schedule-ir.md) correctly did not edit this crate.

After Tom accepts the surface, `0x00` is no longer an honest encoding of an admitted carrier.

## Required work

- Project the accepted `ParametricBroadcast` payload into the request subject under a fresh compiler tag that does not collide with `0x01`/`0x02`/`0x03`.
- Keep existing request-subject bytes unchanged for every previously encodable map.
- Unknown future maps must still refuse rather than share a real tag.
- If the accepted spelling differs from `cefa7439`, implement that spelling.

## Required evidence

- Two parametric mappings that differ only in environment identity or one pad symbol produce different request-subject bytes.
- Concrete reindex and broadcast request-subject bytes are byte-identical to the pre-change neighbour.
- Perturb the new tag into an existing one and watch the injectivity check fail.

## Closes when

The accepted carrier has a named request-subject encoding, existing maps do not move, and the `0x00` arm remains a refusal for maps that are still unprojected.
