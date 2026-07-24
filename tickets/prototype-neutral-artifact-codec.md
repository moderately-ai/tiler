---
id: prototype-neutral-artifact-codec
title: Implement the neutral artifact codec
status: in-progress
priority: p0
dependencies: [prototype-artifact-program-model]
related: [prototype-artifact-slice]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, serialization]
claimed_from: todo
assignee: agent-prototype-neutral-artifact-codec
lease_expires_at: 1784923183
---
Implement a bounded canonical lockstep envelope/program codec independent of compiler internals. Validate schema/version, canonical encoding/order, limits, references, duplicates, section digests and identity, truncation, trailing bytes, corruption, and unsupported features with typed diagnostics.
