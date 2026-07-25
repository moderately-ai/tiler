---
id: define-supported-expansion-cache-filesystems
title: Define the supported expansion cache filesystems
status: in-progress
priority: p2
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [research/cache, contracts/artifacts]
shared_scopes: []
paths: []
tags: [cache, durability, portability]
claimed_from: todo
assignee: agent-cache-filesystems
lease_expires_at: 1785041370
---
The research note's fifth follow-up gate: define supported local filesystems and add platform-specific Windows and network-filesystem feasibility gates before claiming portability.

`tiler-cache` rests on three filesystem facts the research note sources for Unix and Darwin: `rename` is atomic and replaces an existing target, `flock` is advisory and associated with the open file, and a file unlinked after a reader opened it stays readable through that descriptor. **None of the three is established for Windows, and none for a network filesystem.** The crate does not currently refuse either.

## What this ticket owes

- State the supported set as a contract rather than as an assumption, with the evidence for each fact on each member.
- Decide what an unsupported filesystem does. Silently degrading is the wrong answer for a component whose whole argument rests on those three facts; a detected-and-refused root, or a documented and detectable narrowing, are both candidates.
- Windows needs its own spike: its sharing flags, replacement API, and deletion semantics do not inherit the open-unlinked-reader conclusion.
- A network filesystem's `flock` may silently not exclude, which is the failure that costs compile-once suppression without costing correctness — quantify which.
