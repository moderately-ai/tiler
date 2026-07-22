---
id: prototype-artifact-program-model
title: Implement the artifact-facing program model
status: todo
priority: p0
dependencies: [prototype-kernel-program-ir]
related: [prototype-neutral-artifact-codec]
scopes: [implementation/artifact, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, manifest]
---
Project verified KernelProgram content into a bounded versioned artifact model:
entry points, ABI and launch expressions, portfolios/routing predicates,
target requirements, reached admission and selected-provider provenance, and
backend payload descriptors. Runtime and codecs consume this model without
optimizer internals; unused compilation-environment providers do not become
packaged artifact identity.
