---
id: prototype-metal-runtime-proof
title: Run and validate the fused Metal value proof
status: todo
priority: p0
dependencies: [prototype-metal-aot-slice]
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets, contracts/integrations, contracts/navigation]
paths: []
tags: [implementation, prototype, metal, runtime]
---
Implement the publish=false prototype-run consumer using only tiler-artifact plus live Metal bindings. Validate/load/preflight before one-way routing commit, dispatch the fused kernel, compare readback with the reference, and record dispatch/intermediate elimination and failure boundaries. No Candle integration or production runtime API.
