---
id: establish-vector-execution-form-numerical-authority
title: Establish numerical authority for vector execution forms
status: todo
priority: p1
dependencies: []
related: [declare-cpu-vector-realization-facts-in-the-target-profile, define-plural-operation-specific-vector-realization-requirements]
scopes: [implementation/ir, implementation/compiler, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, cpu, vector, public-boundary, correctness]
---
## User-visible outcome

A target can attest the numerical behaviour of the exact scalar, fixed-vector, or scalable-vector operation that will execute, and no scalar fact silently licenses a packed path.

## Fact — 2026-08-11

`ScalarArithmeticSubject` is explicitly scalar. Current honourability rows have no execution-form dimension, while ISA behaviour can vary by exact instruction family and scalar epilogues use two execution paths.

## Required delivery

- Decide one typed execution-form/operation subject or another nonduplicating authority that distinguishes scalar, fixed-vector, and scalable-vector execution where behaviour can differ.
- Preserve per-dimension numerical resolution and exact operation attribution. A broad `lane arithmetic` claim is insufficient.
- Admit a scalar epilogue only when one authoritative source covers both paths or separate path facts compose explicitly; otherwise it remains `Unknown`.
- Re-derive target profile, delivered-realization, schedule/KIR, reference, backend, and artifact identity consequences before implementation.
- Perturb execution form and every numerical dimension independently, including known per-instruction exceptions.

## Closes when

The arithmetic-only vector slice has explicit numerical evidence for what it executes, and scalar evidence cannot satisfy a vector obligation by omission.
