---
id: prototype-metal-bundle-assembly
title: Assemble the Metal artifact bundle
status: todo
priority: p0
dependencies: [prototype-neutral-artifact-codec, prototype-metal-kir-lowering, prototype-metal-numerical-realization, prototype-apple-aot-driver]
related: []
scopes: [implementation/artifact, implementation/metal, implementation/metal-aot]
shared_scopes: []
paths: []
tags: [implementation, metal, artifact, aot]
---
Assemble deterministic MSL, metallib sections, entry mappings, neutral program metadata, target requirements, provenance and section digests into one bounded self-validating bundle. Validate it without a live device; treat metallib reproducibility as measured evidence, not an assumed guarantee.
