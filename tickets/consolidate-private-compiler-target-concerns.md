---
id: consolidate-private-compiler-target-concerns
title: Consolidate private compiler target concerns
status: done
priority: p2
dependencies: [express-metal-honourability-in-the-shared-form, source-or-rephase-first-metal-launch-limits]
related: [prototype-public-compiler-api, decide-per-dtype-dispatchability-as-a-target-capability]
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [refactor, compiler, target-profiles, progressive-disclosure]
---
Make the compiler's target-description and assessment boundary visible without
putting hardware facts into the logical graph.

Target requests, profile description, feasibility, honourability, and
assessment are currently spread across several top-level compiler modules. The
shared honourability ownership decision must land first so this refactor does
not assume where the target-neutral vocabulary belongs.

## User-visible outcome

Create one shallow private target cluster whose files own profile description,
hard feasibility, numerical honourability, and assessment. Preserve the
distinction between feasibility and cost and preserve the reviewed public
session facade.

This is organization, not a new target IR, and not permission to make the
compiler depend on Metal-specific types.

## Implementation keys

Move `target.rs` to `target/mod.rs`, private `feasibility.rs` to `target/feasibility.rs`, and private `honourability.rs` to `target/honourability.rs`. Keep the general compilation request in top-level `request.rs`, preserve the public `tiler_compiler::target::*` facade byte-for-byte, and use crate-private re-exports only where they avoid unrelated churn. Do not move cost, physical IR, or target-independent request concepts into this cluster.

This follows `source-or-rephase-first-metal-launch-limits` because that ticket changes the same target and feasibility files. Moving those files first would turn active semantic work into an avoidable rename conflict.

## Closes when

Target-related dependency direction is visible from one private module root, logical IR remains target-independent, feasibility remains distinct from cost, all public imports and rustdoc paths remain unchanged, no identity/domain/schema value moves, and the full gate passes. Audit every `crate::feasibility`/`crate::honourability` link and demonstrate a public-path compile check can fail before restoring it; use rename-aware diff inspection rather than treating delete/add noise as semantic churn.

## Graph maintenance

- Preserve the direct dependency on the completed launch-limit and shared-honourability work because both establish the files and ownership this refactor moves.
- Update later compiler tickets that cite private file paths to the new module paths without changing their semantic dependencies.
- Do not advance identity or schema domains for a private source reorganization whose public paths and encodings remain unchanged.
