---
id: split-the-compiler-pipeline-by-orchestration-phase
title: Split the compiler pipeline by orchestration phase
status: todo
priority: p2
dependencies: [prototype-public-compiler-api]
related: [prototype-optimizer-conformance-gate]
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [refactor, compiler, progressive-disclosure]
---
Make the compiler's top-level orchestration read as the compilation story rather
than as one file containing orchestration, mechanisms, conformance, and tests.

## Outcome

Keep `pipeline` as one compiler concept and public/internal path. Let its root
show request verification, transactional planning, alternative construction,
selection, and result formation in order. Move transactional state,
alternative enumeration, trace production, verification, conformance support,
and large tests into shallow phase-owned files.

The public request/provider boundary lands first so this refactor preserves the
reviewed call site rather than organizing around a temporary facade.

## Closes when

The ordinary compilation path can be followed from one module root, mechanisms
are separated by invariant rather than line count, public behavior and explain
identity are unchanged, and the full gate passes.
