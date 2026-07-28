---
id: promote-the-metal-aot-compilation-identity
title: Promote the tiler-metal-aot compilation identity
status: todo
priority: p1
dependencies: [bind-recorded-metal-toolchain-to-the-tools-that-execute]
related: [derive-the-pre-compilation-artifact-program-subject, accept-the-tiler-cache-public-boundary, prototype-expansion-content-cache]
scopes: [implementation/metal-aot]
shared_scopes: []
paths: []
tags: [api, review, cache, identity]
---
## User-visible outcome

The expansion cache's second subject facet becomes producible: a caller outside `tiler-metal-aot` can obtain a compilation identity's canonical bytes, which is the difference between a cache that is *composable* in principle and one a frontend can actually use. A real consumer now exists to shape the promotion against (`tiler-metal` depends on the crate).

The expansion cache frames a subject over two facets. One now has a producer and the other is unreachable, which is the whole of what keeps the cache composable rather than usable.

**Fact — refreshed at `01264be`.** `crates/tiler-metal-aot/src/lib.rs:91` declares `mod identity;` — not `pub mod`. `CompilationIdentity` (`identity.rs:220`) and its `as_bytes` (`identity.rs:245`) are still `pub(crate)`. No crate outside `tiler-metal-aot` can obtain those bytes. The earlier citations (`lib.rs:74`, `identity.rs:211`, `:236`) named the same constructs before surrounding edits moved them.

**Fact — ADR 0077 item 2 still holds.** `crates/tiler-metal-aot/Cargo.toml` has **no `[dependencies]` section at all**, so the closure is empty rather than merely small: `grep -n 'dependencies' crates/tiler-metal-aot/Cargo.toml` returns nothing.

**Fact — a real out-of-crate caller now exists, and did not when this ticket was written.** `tiler-metal` depends on `tiler-metal-aot` (`crates/tiler-metal/Cargo.toml:20`), added so the gate can compile the golden MSL through the driver. The promotion's shape should be read against that caller — a concrete consumer whose needs can be checked — rather than against a hypothetical one.

**Fact.** `derive-the-pre-compilation-artifact-program-subject` (done) gave the `ArtifactProgram` facet a producer that needs no compiled object, so this is the remaining half.

**This is ADR 0075's always-ask category** — a `pub(crate)`-to-`pub` promotion, and a new publicly reachable namespace if the module is promoted rather than a re-export added. It is Tom's decision and not the worker's to merge. Land the smallest surface that lets a caller obtain the identity bytes and put the exact surface to Tom, following the route other tickets used today: stage privately, append the surface to `accept-the-tiler-cache-public-boundary`, and report.

**The shape is a real choice, so run the elimination before presenting anything.** Promoting the module exposes the whole derivation; adding a re-export or an accessor on an already-public type exposes the capability. The precedent cuts both ways and should be read rather than assumed: the envelope codec's promotion was narrow because its producer entry point was already public, while the proof sidecar's had to be whole because no public producer existed. Determine which case this is by reading, not by analogy.

**A constraint that is not negotiable.** `tiler-metal-aot`'s dependency closure is empty by decision (ADR 0077 item 2). It was pinned as `"tiler-metal-aot": []` in `scripts/check_workspace.py` until `e197176` replaced the Python gate with the `Makefile`; the decision is unchanged and the check that enforced it is gone, so an added edge is now caught by reading the diff rather than by a failing gate. Whatever is promoted must not acquire a dependency, and in particular must not reach for the governed digest in `tiler-artifact`: this crate emits canonical bytes and the caller that owns the algorithm digests them, exactly as `family.rs` already does. A promotion that quietly adds an edge is a different decision than the one being asked for.

## Work in flight — recorded 2026-07-28

- **The branch holds one commit and it is not on `main`.** `tkt/promote-the-metal-aot-compilation-identity` is at `4f8ce90` ("Stage the metal-aot compilation identity promotion -- DO NOT MERGE UNAPPROVED", 2026-07-25). `git merge-base --is-ancestor 4f8ce90 main` exits non-zero, so nothing of it has landed. **No worktree is registered** for it — `git worktree list` does not name one — so the branch exists without a checkout to continue in.
- **The staging half of the route was done; the reporting half did not reach a reader.** `4f8ce90` does append the surface to `tickets/accept-the-tiler-cache-public-boundary.md` (`git show --stat 4f8ce90` shows 19 added lines there), which is the route this ticket named. But the append lives only on the unmerged branch: `grep -n 'metal-aot\|CompilationIdentity' tickets/accept-the-tiler-cache-public-boundary.md` on `main` returns nothing. So a reader of the acceptance ticket has never seen this surface, and Tom has nothing to accept from the place he would look.
- **Status note.** The ticket is `status: in-progress` against a branch with one unmerged commit and no worktree. Whether that should return to `todo` — releasing the claim — or stay `in-progress` pending a decision on `4f8ce90` is a frontmatter change this pass did not make, and it is recorded here instead.

## Closes when

A caller outside `tiler-metal-aot` can obtain the compilation identity bytes, the promoted surface is exactly what Tom accepted **and is visible on `main` where he was asked to accept it**, the crate's dependency closure is unchanged, and the full gate passes.

## Graph maintenance

- **This is an ADR 0075 owner-reserved promotion**: stage the surface privately, append it to the acceptance ticket for Tom's ratification, and do not publish `pub` ahead of his record — `pair-verified-buffer-handles-with-signature-ordinals` documents exactly what it looks like when a promotion ships un-ratified, and it is now an awkward ratify-after-the-fact item on his queue. Do not create a second one.
- **Work in flight exists on an unmerged branch** (`4f8ce90` — see the body): read it before re-deriving; it appended the surface to the acceptance ticket on the branch only, so `main` has none of it.
- **When the facet becomes producible**: update `accept-the-tiler-cache-public-boundary`'s ratification checklist (the composed-subject item assumes both facets can be filled) and tell `prototype-inline-aot-integration-proof`, whose cache-sharing criterion needs this facet.
