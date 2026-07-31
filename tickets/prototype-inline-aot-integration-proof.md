---
id: prototype-inline-aot-integration-proof
title: Prove the complete inline AOT workflow
status: todo
priority: p1
dependencies: [prototype-macro-embedding-and-cargo-behavior, prototype-metal-runtime-proof, promote-the-metal-aot-compilation-identity, make-runtime-routing-commit-authority-one-shot]
related: []
scopes: [implementation/frontend, implementation/cache, implementation/compiler, implementation/artifact, implementation/metal-aot, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, integration, inline-dx, milestone-0b]
---
## User-visible outcome

One ordinary inline Rust invocation — no build script, no registry, no scan, no prepare step, no runtime JIT — constructs a program, shares its external compilation through the validated cache, embeds the bytes, and runs with guarded selection and pre-commit fallback. This is the end-to-end proof of the accepted inline developer experience; every absence in that list is checkable by reading the consumer crate.

Demonstrate one ordinary inline Rust invocation constructing and optimizing a program, sharing external compilation through the validated cache, embedding manifest/metallib bytes directly, and emitting guarded runtime selection with fallback authority before commit. Require no build script, registry, scan, prepare command, or runtime source compilation.

## Closes when

- One inline Rust invocation in an ordinary crate constructs a program, optimizes it, and produces a running kernel, with **no** `build.rs`, no duplicated registry, no source scan, no Cargo subcommand, no prepare step, and no runtime source JIT — the accepted inline developer experience `AGENTS.md` names, each absence checkable by reading the consumer crate.
- The external compilation is shared through the validated expansion cache: a second build of the same subject is a hit, the hit is validated on every read rather than trusted, and a mismatch is a typed refusal rather than a silent rebuild.
- Manifest and metallib bytes are embedded directly in the produced binary, with the identity that names them derivable before the compilation it describes has run.
- Runtime selection is guarded, and the fallback authority is exercised **before** the routing commit and nowhere after it, per ADR 0051's one-way commit.
- `make full` passes, and the proof runs end to end on a qualified Apple toolchain.

## Dependency reality (verified 2026-07-31)

This ticket's four dependencies are not uniformly ready, and the gap is not the one the frontmatter suggests. `grep -m1 '^status:' tickets/<id>.md` for each:

- `prototype-metal-runtime-proof` — `done`.
- `make-runtime-routing-commit-authority-one-shot` — `done`.
- `prototype-macro-embedding-and-cargo-behavior` — `todo`, so dispatchable but unclaimed.
- `promote-the-metal-aot-compilation-identity` — `done`. **The consequence for this ticket changed: the cache-sharing half now has a reachable identity producer.** `CompilationIdentity` and its `as_bytes` are `pub` in `tiler-metal-aot`, obtained only from the public `PreparedCompilation::identity` after `Toolchain::prepare`, and `tiler-build` already consumes them at `crates/tiler-build/src/metal_assembly.rs:119`. The other facet's producer is `derive-the-pre-compilation-artifact-program-subject` (`done`), so both facets of the cache subject now have one.

Read that as: neither half is waiting on a decision any more. What remains is ordinary unclaimed work on the embedding half, plus the frontend that has to call both — `tiler` and `tiler-macros` exist as of 2026-07-31 but carry only the `tensor!` re-export and its anchor, so the grammar, expansion, and family delivery this proof needs are still open under `prototype-inline-proc-macro-frontend`, `define-inline-symbol-binding-and-runtime-value-adaptation`, and `promote-artifact-family-selection-for-the-frontend`.

**Superseded 2026-07-28 reading, kept so the correction is legible:** this section previously recorded `promote-the-metal-aot-compilation-identity` as `in-progress` on a single unmerged commit `4f8ce90`, and concluded that the cache-sharing half had no reachable identity producer because `CompilationIdentity` and `as_bytes` were `pub(crate)`. That was true when written and is false now; the promotion merged and the public boundary it gated was accepted.
