---
id: promote-the-metal-aot-compilation-identity
title: Promote the tiler-metal-aot compilation identity
status: done
priority: p1
dependencies: [bind-recorded-metal-toolchain-to-the-tools-that-execute]
related: [derive-the-pre-compilation-artifact-program-subject, accept-the-tiler-cache-public-boundary, prototype-expansion-content-cache]
scopes: [implementation/metal-aot, project/tickets, implementation/cache]
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

## Decision applied — 2026-07-28

Tom selected the opaque prepared-compilation boundary. `Toolchain::prepare(&CompileRequest)` returns `PreparedCompilation<'_>`, which immutably borrows the request, privately owns the resolved toolchain and derived `CompilationIdentity`, exposes `identity()` for cache lookup, and consumes itself through `compile()` using those resolved paths. `Toolchain::compile` delegates through the same path, so the ordinary entry point does not maintain a second compilation implementation.

The older `4f8ce90` draft is deliberately not merged. Its public `CompilationIdentity::new(&CompileRequest, &ResolvedToolchain)` failed ADR 0074 convention 2 because the derived identity had a public constructor over caller-constructible toolchain facts. It also left cache lookup and compilation as separate resolutions, so a miss could execute a different toolchain from the one its key named. The accepted form removes that constructor, removes the second resolution, and turns request/toolchain agreement into a borrow and private token invariant rather than a caller obligation.

The promoted public surface is `tiler_metal_aot::identity::{CompilationIdentity, ToolchainEvidence, IdentityReuseScope, IdentityError}` plus `tiler_metal_aot::driver::PreparedCompilation` and `Toolchain::prepare`. Encoding helpers, the domain tag, and `CompilationIdentity::new` remain below `pub`. A cross-crate compile-fail doctest pins the constructor boundary, and a deterministic fake-toolchain test deletes the launcher after preparation and still compiles through the already-resolved tools.

## Closes when

A caller outside `tiler-metal-aot` can obtain the compilation identity bytes, the promoted surface is exactly what Tom accepted **and is visible on `main` where he was asked to accept it**, the crate's dependency closure is unchanged, and the full gate passes.

## Graph maintenance

- **The ADR 0075 owner-reserved promotion is ratified.** Tom accepted the prepared-compilation form above, not the public constructor staged by `4f8ce90`.
- **The cache acceptance record is updated.** `accept-the-tiler-cache-public-boundary` now records the exact ratified Metal AOT surface and why the older draft was rejected.
- **The downstream integration owner must consume the new producer.** Tell `prototype-inline-aot-integration-proof` that both cache subject facets are now producible and its remaining cache-sharing work is orchestration rather than identity reachability.
