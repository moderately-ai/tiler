---
id: prototype-shared-compiler-ir-ownership
title: Establish shared compiler IR ownership
status: todo
priority: p0
dependencies: [reconcile-implementation-delivery-graph]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/workspace]
shared_scopes: []
paths: []
tags: [implementation, compiler-foundation, milestone-0b]
---
Place occurrence references, symbolic access/index IR, schedules, structured KIR, KernelProgram, and neutral ABI-facing types in reviewed owning layers with an acyclic crate graph. Expose only the smallest ordinary non-test compiler path required downstream; keep IR levels distinct and compile-check dependency direction. Tom reviews consequential public crate, module, trait, type, and call-site boundaries before acceptance.
