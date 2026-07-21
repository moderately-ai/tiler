---
id: prototype-neutral-artifact-codec
title: Implement the neutral artifact codec
status: todo
priority: p0
dependencies: [prototype-neutral-program-and-artifact-types]
related: [prototype-artifact-slice]
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [implementation, artifact, serialization]
---
Implement a bounded canonical lockstep envelope/program codec independent of compiler internals. Validate schema/version, canonical encoding/order, limits, references, duplicates, section digests and identity, truncation, trailing bytes, corruption, and unsupported features with typed diagnostics.
