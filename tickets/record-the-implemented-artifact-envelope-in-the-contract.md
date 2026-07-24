---
id: record-the-implemented-artifact-envelope-in-the-contract
title: Record the implemented artifact envelope in docs/artifact-abi.md
status: todo
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, artifact]
---
`docs/artifact-abi.md` states "artifact codec unimplemented" and "Canonical envelope serialization, backend payloads, integrity validation, and public artifact APIs remain unimplemented." `prototype-neutral-artifact-codec` implemented a bounded canonical lockstep codec, but deliberately did **not** edit that contract: the codec landed as a crate-private draft under ADR 0074 convention 7, its facade is unaccepted, and the ticket held only `implementation/artifact`.

**What this ticket must state, once the facade is accepted.**

- The envelope framing this build writes: fixed 69-byte header (magic `TILERART`, envelope format, canonical encoding version, governed digest algorithm tag, total length, manifest length, section count, manifest digest), one canonical manifest, then length-delimited sections.
- That the manifest is written in canonical content order for every set-meaning collection, so two artifacts with equal identity encode to equal bytes, and that a well-formed but non-canonical encoding is rejected rather than normalized.
- That the governed digest algorithm is named by an explicit tag and never inferred from a digest width.
- That the initial section vocabulary has exactly one governed purpose, the packaged variant's canonical kernel-program identity, and that backend metadata and code sections are `prototype-metal-bundle-assembly`'s versioned extension.
- The required-feature mechanism and the four keys this build derives, including the one it emits and refuses to read.
- What the envelope deliberately excludes: the frozen registry snapshot (ADR 0072), presentation-only declaration order, backend payload bytes, and a reconstructable kernel program.

**Do not** mark the contract implemented while the surface is `pub(crate)`; the accurate statement is that a bounded lockstep codec exists behind an unaccepted facade.
