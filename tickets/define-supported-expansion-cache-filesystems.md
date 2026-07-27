---
id: define-supported-expansion-cache-filesystems
title: Define the supported expansion cache filesystems
status: done
priority: p2
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [research/cache, contracts/artifacts]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [cache, durability, portability]
---
The research note's fifth follow-up gate: define supported local filesystems and add platform-specific Windows and network-filesystem feasibility gates before claiming portability.

`tiler-cache` rests on three filesystem facts the research note sources for Unix and Darwin: `rename` is atomic and replaces an existing target, `flock` is advisory and associated with the open file, and a file unlinked after a reader opened it stays readable through that descriptor. **None of the three is established for Windows, and none for a network filesystem.** The crate does not currently refuse either.

## What this ticket owes

- State the supported set as a contract rather than as an assumption, with the evidence for each fact on each member.
- Decide what an unsupported filesystem does. Silently degrading is the wrong answer for a component whose whole argument rests on those three facts; a detected-and-refused root, or a documented and detectable narrowing, are both candidates.
- Windows needs its own spike: its sharing flags, replacement API, and deletion semantics do not inherit the open-unlinked-reader conclusion.
- A network filesystem's `flock` may silently not exclude, which is the failure that costs compile-once suppression without costing correctness — quantify which.

## Outcome

`docs/research/cache/supported-filesystems.md` states the contract; `docs/artifact-abi.md` and `docs/backends/metal.md` carry its normative form; `spikes/cache/filesystem_probe.rs` measures it and `spikes/cache/results/filesystem-probe-macos-27.0-2026-07-25.tsv` records one run over local APFS and a formatted exFAT RAM disk.

**The premise that a filesystem could make the cache wrong did not survive.** Enumerating every failure of the six properties against the read path shows each reduces to an absence, an I/O error, an unpublished result, or duplicated compiler work. Complete identity, immutable final entries, and validation on every hit are not filesystem assumptions — they are arithmetic over bytes, and the absence of any write-mode open on a content path — so a filesystem can take away the lock and the atomicity and still not take away correctness. This ticket's fourth bullet had it right; what the note adds is the derivation.

**Therefore no detect-and-refuse.** A refusal keyed on filesystem identity contradicts ADR 0050's own reason for falling open, fails closed on every filesystem nobody enumerated, does not detect the case that motivates it — a shared NFS root under `locallocks` presents as an ordinary directory — and would need a new admitted `unsafe` `statfs` site to attempt. The contract is stated and probeable instead, and every locally decidable failure was already loud: `CrossesFilesystems` for a cross-device rename, an `AcquireLock` unavailability for a lock the host refuses.

**The one thing that cannot be detected** is whether an advisory lock excludes a process on another host. Both platforms document a mount mode (Darwin `locallocks`, Linux `local_lock=`) under which it does not while still reporting success, and no single host can ask the question. That limitation is the finding, not a gap in the implementation.

**`atime` answered no**, closing the deferral in the collection note. Measured on macOS 27.0/APFS with no `noatime`: access time follows a `relatime`-like predicate, and a published entry's modification time never changes, so its access time advances at most once ever — at its first read after publication. It is a boolean "read since published", not a recency. A macOS exFAT volume mounts `noatime` and measures `none`; Linux defaults to `relatime` by documentation.

**Windows dissolved rather than deferred.** The crash/race note's Windows sentence is conditional on the cache core claiming Windows support. `AGENTS.md` now states that Tiler develops on **macOS only** — narrower than when this was written, which named Debian-family Linux too — and the Rust sub-gate this once cited was deleted by `e197176` with the rest of the Python tooling. No spike is owed, and Windows is further from support than it was.

## Split out

- ~~`probe-the-expansion-cache-filesystem-properties-on-linux`~~ — **closed without completion (2026-07-27)**, by `reconcile-cache-filesystem-claims-with-macos-support-policy`. Measuring ext4, btrfs, and xfs would qualify rows on a platform the support policy does not admit; the derivation is retained as inactive research and the trigger for reviving the probe is the admission of Linux as a supported platform.
- `add-an-expansion-cache-root-preflight` — an explicit, non-refusing `ExpansionCache::preflight` over the locally decidable subset, in `implementation/cache` scope.
