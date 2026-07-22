---
id: implement-first-quantized-backend-profile
title: Implement the first selected quantized backend profile
status: deferred
priority: p2
dependencies: [prototype-quantized-value-vertical]
related: []
scopes: [implementation/compiler, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, backend, deferred]
---
Activate only after a concrete quantized format, operation set, target backend,
storage layout, numerical contract, and conformance corpus are selected. Then
implement lowering, schedule feasibility, code generation, ABI/runtime binding,
and device comparison without generalizing beyond that measured profile.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
