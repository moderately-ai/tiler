---
id: preserve-typed-metal-aot-failure-causes
title: Preserve typed Metal AOT failure causes
status: todo
priority: p2
dependencies: [bind-recorded-metal-toolchain-to-the-tools-that-execute]
related: [promote-the-metal-aot-compilation-identity, prototype-metal-aot-slice]
scopes: [implementation/metal-aot]
shared_scopes: []
paths: []
tags: [metal, aot, diagnostics, api]
---
Let callers retain the tool, phase, executable, exit status, and bounded output
that explain an offline Metal compilation failure.

The driver distinguishes discovery, version probing, source compilation,
linking, and output validation internally, but some public-facing failures
flatten their causal detail into strings. The exact executed-tool authority
must land first so diagnostics name the tool that actually ran.

## Outcome

Carry typed phase/tool/status/output causes through `DriverError` and retained
artifact-family diagnostics. Rendering remains convenient and bounded, but it
is a view of structured evidence rather than the only copy of it. Preserve
non-UTF-8 and truncated output honestly.

Any changed public error shape requires Tom's review before acceptance.

## Closes when

Callers can branch on the failure phase without parsing text, rendered errors
remain actionable, tests cover discovery/compile/link/output-validation
neighbors, and the full gate passes.
