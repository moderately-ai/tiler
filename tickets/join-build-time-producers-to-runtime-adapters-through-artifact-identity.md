---
id: join-build-time-producers-to-runtime-adapters-through-artifact-identity
title: Join build-time producers to runtime adapters through artifact identity
status: todo
priority: p1
dependencies: [produce-a-custom-backend-payload-through-the-build-orchestrator, route-a-custom-backend-through-a-registered-runtime-adapter]
related: [accept-and-publish-validated-artifacts-through-the-expansion-cache, bind-runtime-library-and-pipeline-caches-to-exact-payload-bytes]
scopes: [implementation/build, implementation/runtime, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, identity, artifacts]
---
## User-visible outcome

A payload produced in one process can be loaded and executed in another process that has only the matching runtime adapter, proving that build-time and runtime provider halves join through durable artifact identity rather than shared Rust objects.

## Implementation keys

- Produce retained bytes and a sidecar/fixture with the build-time producer installed.
- Start a separate consumer process without compiler, emitter, AOT driver, or build-provider objects.
- Match only governed backend, representation, target profile, payload/compilation subject, entry mapping, and compatibility identities.
- Prove provider Rust `TypeId`, vtable, function, allocation, and registration addresses never enter durable identity.
- Validate every cache hit and artifact load from bytes; do not trust the producer process's prior validation.
- Mutate each join subject independently and observe a typed refusal before routing commit.
- Confirm identical output-affecting producer behavior yields stable bytes/identity while a revision or emitted-content change moves the correct subject.
- Keep cross-process callbacks and dynamic loading out of scope; only artifact transport crosses the process boundary.

## Closes when

The separate-process fixture succeeds with matching durable identities, fails for every mismatched or missing adapter subject, proves no process-local identity leakage, and targeted tests plus one batch `make full` pass.

## Graph maintenance

- Feed this evidence into the final three-provider portfolio and conformance suite.
- Consume the existing validated cache-hit path; if the fixture exposes a cache defect, file it separately rather than changing cache ownership here.
- Recompute any pinned artifact identities on the merged tree rather than accepting a branch golden.
