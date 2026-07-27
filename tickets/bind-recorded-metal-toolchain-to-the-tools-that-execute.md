---
id: bind-recorded-metal-toolchain-to-the-tools-that-execute
title: Bind Metal provenance to the tools that produce the artifact
status: todo
priority: p1
dependencies: []
related: [promote-the-metal-aot-compilation-identity, prototype-metal-aot-slice]
scopes: [implementation/metal-aot]
shared_scopes: []
paths: []
tags: [metal, aot, provenance, correctness]
---
The compiler must not record toolchain A as artifact provenance while toolchain
B actually produces the AIR or metallib bytes.

## Fact

Preflight resolves and records absolute `metal` and `metallib` paths and their
versions. Compilation later asks `xcrun` to select bare tool names again. Tool
selection can therefore change between the recorded observation and execution.

## Outcome

The canonical compilation identity describes the exact SDK and executable
tools used for compilation and linking. Execute the resolved tools directly or
make one recorded launcher resolution authoritative for both observation and
execution. A changing-selection test must fail closed rather than misattribute
the artifact.

## Closes when

Every tool identity construction site is paired with the command that uses that
tool, and tests prove a selector change cannot produce bytes under stale
provenance.
