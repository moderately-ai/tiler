---
id: correct-capability-index-ticket-order
title: Correct capability and index ticket order
status: done
priority: p0
dependencies: [prototype-shared-compiler-ir-ownership]
related: [prototype-operation-compilation-capabilities, prototype-canonical-index-region-slice, prototype-physical-implementation-frontier]
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: []
---
## Outcome

Reorder the implementation DAG so checked canonical index IR precedes executable index/access and scalar lowering capabilities. Record that physical implementation capability registration belongs to the physical-frontier slice. This prevents placeholder traits, duplicate IR, and unchecked opaque payloads.

## Result

The canonical index-region slice now follows shared IR ownership directly.
Operation compilation capabilities follow that checked vocabulary, and the
physical frontier owns physical implementation and opaque-call registration.
The dependency direction is therefore durable IR vocabulary first, typed
extension callbacks second, and physical proposal surfaces only when their
verified output types exist.
