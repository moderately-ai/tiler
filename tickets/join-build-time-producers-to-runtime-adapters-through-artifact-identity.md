---
id: join-build-time-producers-to-runtime-adapters-through-artifact-identity
title: Join build-time producers to runtime adapters through artifact identity
status: in-progress
priority: p1
dependencies: [produce-a-custom-backend-payload-through-the-build-orchestrator, route-a-custom-backend-through-an-independently-selected-adapter]
related: [accept-and-publish-validated-artifacts-through-the-expansion-cache, bind-runtime-library-and-pipeline-caches-to-exact-payload-bytes]
scopes: [implementation/build, implementation/runtime, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, identity, artifacts]
claimed_from: todo
assignee: worker-identity-join
lease_expires_at: 1785571630
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

## Outcome

- The fixture is genuinely two programs rather than one process pretending. The producer is `crates/tiler-build/examples/identity_join_producer.rs`; the consumer is `crates/tiler-runtime/tests/identity_join/`, which spawns the producer through Cargo and then reads back only `artifact.bin` and `sidecar.txt`. The producer re-executes itself for a second run against the same cache root, so three live processes take part: two producers and one consumer.
- "The consumer constructs no compiler, emitter, AOT driver, or build-provider object" is proven by linkage rather than by inspection. `tiler-runtime`'s resolved closure — development dependencies included, walked transitively out of `Cargo.lock` — contains none of `tiler-build`, `tiler-compiler`, `tiler-cache`, `tiler-metal`, or `tiler-metal-aot`, so no target of that package can name one of their types. The check carries three positive controls and was watched failing under a temporary `tiler-build` development dependency.
- Six governed join subjects are moved one at a time by the producer — backend family, executable representation, assessed target profile, payload compatibility profile, compilation subject, and entry mapping in two forms (an unmapped entry key and an absent symbol) — plus the caller's recorded expectation and the emitted object. Every one is refused before the routing commit, asserted through `fallback_permitted` and through the adapter's stage log rather than through the returned error alone. Each perturbation was neutralized in turn and the corresponding case watched failing.
- Stability is a cross-process measurement. Two producer processes with different process identifiers, working directories, and environments write byte-identical envelopes and identical records; the second resolves the first's published subject as a cache **hit** that the promoted seam re-validates from bytes; and the consumer re-derives the same identity from those bytes in a third binary with entirely different type layouts. Injecting the producer's process identifier into the payload provenance moved the subject and was watched failing.
- Two variants deliberately bypass the cache seam and say so in the record. An envelope whose entry mapping reaches no packaged entry cannot be published — the seam's own refusal is recorded beside the bytes, and the consumer refuses the same bytes from its own decode — and an artifact differing only in its emitted object shares the sound artifact's subject, so publishing it would return the sound envelope. Neither is a cache defect; both are the subject correctly declining to distinguish what artifact identity deliberately excludes.
- Nothing in the fixture pins an artifact identity: every identity, digest, and subject is derived at run time from the artifact the producer actually built, so there is no golden to recompute on the merged tree.
- Cross-process callbacks and dynamic loading stayed out of scope. Only an envelope and a text record cross the boundary.
