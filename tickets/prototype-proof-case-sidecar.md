---
id: prototype-proof-case-sidecar
title: Implement the proof-case evidence sidecar
status: todo
priority: p0
dependencies: [prototype-neutral-artifact-codec, prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/reference, implementation/artifact, implementation/metal-aot]
shared_scopes: []
paths: []
tags: [implementation, testing, artifact, numerics]
---
Implement a separate versioned proof sidecar containing stable case keys, bit-preserving inputs, normative expected bytes, semantic/numerical/reference identities, digests, and exact envelope association. Validate limits, uniqueness, corruption and mismatch; never make it runtime artifact semantics.
