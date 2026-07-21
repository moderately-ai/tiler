---
id: prototype-artifact-family-delivery
title: Implement Apple artifact-family delivery selection
status: todo
priority: p1
dependencies: [prototype-neutral-artifact-codec, prototype-apple-aot-driver]
related: []
scopes: [implementation/frontend, implementation/metal-aot, implementation/artifact]
shared_scopes: []
paths: []
tags: [implementation, apple-targets, inline-dx]
---
Implement explicit family selection and generated routing for supported Apple artifact families, initially macOS, iOS device, iOS simulator, and explicit fallback-only behavior as contracted. Nonmatching targets must not silently select incompatible bytes; target facts belong in identity and diagnostics.
