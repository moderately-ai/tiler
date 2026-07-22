---
id: prototype-metal-runtime-execution
title: Implement Metal runtime execution mechanics
status: todo
priority: p0
dependencies: [prototype-metal-runtime-preflight, prototype-runtime-routing-commit]
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, metal, execution]
---
Implement bounded allocation, ABI binding, checked dispatch, asynchronous resource retention through final device use, submission, exact terminal-status validation, and readback. Inject post-commit failures and prove no fallback occurs after commit.
