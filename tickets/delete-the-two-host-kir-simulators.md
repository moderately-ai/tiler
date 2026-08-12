---
id: delete-the-two-host-kir-simulators
title: Delete the two host KIR simulators
status: todo
priority: p1
dependencies: [replace-host-kir-simulator-claims-with-authoritative-evidence]
related: [share-one-structured-kernel-interpreter]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [cleanup, testing, correctness, cpu]
---
## User-visible outcome

There is no test-only implementation that can be mistaken for the CPU backend or for authoritative KIR execution. The repository has one production scalar CPU execution path, independent reference semantics, and layer-local structural verification.

## Work

- Physically delete both `KirMachine` definitions.
- Delete `interpret_fused`, `interpret_fused_inputs`, `interpret_bf16`, `cooperative_reference`, fake canonicalization switches, flattening/step machinery, and every simulator-only value/buffer type.
- Delete—not deprecate—any compatibility import, wrapper, feature, dead module, or documentation that points at the simulators.
- Preserve only generic fixtures that still serve an authoritative structural, reference, or real-backend test; rename them if their old name implies simulated execution.
- Add a source census proving the retired symbols and equivalent shared-test-interpreter surface are absent. Size positive test populations separately so an empty grep cannot make the correctness suite look green.

## Acceptance

- `rg` over tracked source finds no retired symbol or simulator implementation outside the closed historical tickets that explain their removal.
- All replacement-evidence census rows still run and their subject perturbations still fail as recorded.
- No crate named `tiler-kernel-test-support`, no doc-hidden KIR executor, and no second scalar CPU image executor exists.
- Workspace checks and the exact merged-tree gate pass.
