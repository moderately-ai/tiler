---
id: correct-capability-index-ticket-order
title: Correct capability and index ticket order
status: done
priority: p0
dependencies: [prototype-shared-compiler-ir-ownership]
related: [prototype-canonical-index-region-slice, prototype-physical-implementation-frontier, prototype-operation-capability-registry, implement-opaque-physical-call-providers]
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
physical frontier owns the additive physical-implementation seam; its P0 slice
implements scheduled kernels only. The later reviewed opaque-call ticket owns
opaque registration and contracts.
The dependency direction is therefore durable IR vocabulary first, typed
extension callbacks second, and physical proposal surfaces only when their
verified output types exist.
