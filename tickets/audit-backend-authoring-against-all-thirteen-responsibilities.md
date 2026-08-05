---
id: audit-backend-authoring-against-all-thirteen-responsibilities
title: Audit backend authoring against all thirteen responsibilities
status: in-progress
priority: p1
dependencies: []
related: [specify-the-consumer-neutral-backend-provider-composition-contract, publish-the-backend-provider-conformance-suite, expose-explicit-backend-provider-and-selection-policy-composition]
scopes: [research/extensions, research/program-planning, research/artifacts, research/runtime, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, backend-providers, audit, conformance]
claimed_from: todo
assignee: agent-backend-audit
lease_expires_at: 1785951341
---
## User-visible outcome

An independent backend author can determine, at compile time and without Metal or
Candle types entering the core, every contract that must be supplied to compile,
package, select, bind, and execute a backend program.

Audit the accepted compositional design across all thirteen responsibilities:
semantic authority; index lowering; scalar lowering; physical implementations;
opaque calls; target profile; emitter; build orchestration; backend family and
representation identity; payload provenance; entry mapping; runtime adapter; and
live context. For each, record the public typed seam, construction/registration path,
validation and identity obligations, explain behavior, conformance evidence, and
current maturity. A monolithic `Backend` trait is not the goal; completeness of the
composed authoring contract is.

Trace the existing provider chain, including the still-private physical provider,
explicit provider/policy composition, multi-family portfolio, build/runtime join,
and conformance suite. File missing design or implementation nodes with correct
dependencies. Define what a backend-author conformance kit must prove, including
deliberate invalid providers and identity/routing failures. Any new public trait or
module boundary remains Tom's to accept.

## Closes when

All thirteen rows have one authoritative owner and maturity state; an external
backend's end-to-end path has no undocumented repository-private hook; the
conformance-suite dependency graph covers negative as well as successful paths; and
remaining unsupported responsibilities reject explicitly.
