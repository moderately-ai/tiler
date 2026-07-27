---
id: promote-the-metal-aot-compilation-identity
title: Promote the tiler-metal-aot compilation identity
status: in-progress
priority: p1
dependencies: [bind-recorded-metal-toolchain-to-the-tools-that-execute]
related: [derive-the-pre-compilation-artifact-program-subject, accept-the-tiler-cache-public-boundary, prototype-expansion-content-cache]
scopes: [implementation/metal-aot]
shared_scopes: []
paths: []
tags: [api, review, cache, identity]
---
The expansion cache frames a subject over two facets. One now has a producer and the other is unreachable, which is the whole of what keeps the cache composable rather than usable.

**Fact.** `crates/tiler-metal-aot/src/lib.rs:74` declares `mod identity;` — not `pub mod`. `CompilationIdentity` (`identity.rs:211`) and its `as_bytes` (`identity.rs:236`) are `pub(crate)`. No crate outside `tiler-metal-aot` can obtain those bytes.

**Fact.** `derive-the-pre-compilation-artifact-program-subject` (done) gave the `ArtifactProgram` facet a producer that needs no compiled object, so this is the remaining half.

**This is ADR 0075's always-ask category** — a `pub(crate)`-to-`pub` promotion, and a new publicly reachable namespace if the module is promoted rather than a re-export added. It is Tom's decision and not the worker's to merge. Land the smallest surface that lets a caller obtain the identity bytes and put the exact surface to Tom, following the route other tickets used today: stage privately, append the surface to `accept-the-tiler-cache-public-boundary`, and report.

**The shape is a real choice, so run the elimination before presenting anything.** Promoting the module exposes the whole derivation; adding a re-export or an accessor on an already-public type exposes the capability. The precedent cuts both ways and should be read rather than assumed: the envelope codec's promotion was narrow because its producer entry point was already public, while the proof sidecar's had to be whole because no public producer existed. Determine which case this is by reading, not by analogy.

**A constraint that is not negotiable.** `tiler-metal-aot`'s dependency closure is empty by decision (ADR 0077 item 2). It was pinned as `"tiler-metal-aot": []` in `scripts/check_workspace.py` until `e197176` replaced the Python gate with the `Makefile`; the decision is unchanged and the check that enforced it is gone, so an added edge is now caught by reading the diff rather than by a failing gate. Whatever is promoted must not acquire a dependency, and in particular must not reach for the governed digest in `tiler-artifact`: this crate emits canonical bytes and the caller that owns the algorithm digests them, exactly as `family.rs` already does. A promotion that quietly adds an edge is a different decision than the one being asked for.

## Closes when

A caller outside `tiler-metal-aot` can obtain the compilation identity bytes, the promoted surface is exactly what Tom accepted, the crate's dependency closure is unchanged, and the full gate passes.
