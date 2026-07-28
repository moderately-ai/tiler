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
Demonstrate one ordinary inline Rust invocation constructing and optimizing a program, sharing external compilation through the validated cache, embedding manifest/metallib bytes directly, and emitting guarded runtime selection with fallback authority before commit. Require no build script, registry, scan, prepare command, or runtime source compilation.

## Closes when

- One inline Rust invocation in an ordinary crate constructs a program, optimizes it, and produces a running kernel, with **no** `build.rs`, no duplicated registry, no source scan, no Cargo subcommand, no prepare step, and no runtime source JIT — the accepted inline developer experience `AGENTS.md` names, each absence checkable by reading the consumer crate.
- The external compilation is shared through the validated expansion cache: a second build of the same subject is a hit, the hit is validated on every read rather than trusted, and a mismatch is a typed refusal rather than a silent rebuild.
- Manifest and metallib bytes are embedded directly in the produced binary, with the identity that names them derivable before the compilation it describes has run.
- Runtime selection is guarded, and the fallback authority is exercised **before** the routing commit and nowhere after it, per ADR 0051's one-way commit.
- `make full` passes, and the proof runs end to end on a qualified Apple toolchain.

## Dependency reality (verified 2026-07-28)

This ticket's four dependencies are not uniformly ready, and the gap is not the one the frontmatter suggests. `grep -m1 '^status:' tickets/<id>.md` for each:

- `prototype-metal-runtime-proof` — `done`.
- `make-runtime-routing-commit-authority-one-shot` — `done`.
- `prototype-macro-embedding-and-cargo-behavior` — `todo`, so dispatchable but unclaimed.
- `promote-the-metal-aot-compilation-identity` — `in-progress`, and its work is a single unmerged commit `4f8ce90` that was never put to Tom on `main`. **The consequence for this ticket is specific: the cache-sharing half has no reachable identity producer today.** `CompilationIdentity` and its `as_bytes` are `pub(crate)` in `tiler-metal-aot`, so nothing outside that crate can obtain the bytes one of the cache subject's two facets needs. The other facet has a producer (`derive-the-pre-compilation-artifact-program-subject`, `done`), so it is precisely one of the two that blocks.

Read that as: the embedding half is waiting on ordinary unclaimed work, while the cache-sharing half is waiting on a decision.
