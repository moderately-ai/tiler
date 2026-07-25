---
id: add-an-expansion-cache-root-preflight
title: Add an expansion cache root preflight
status: todo
priority: p2
dependencies: []
related: [define-supported-expansion-cache-filesystems, accept-the-tiler-cache-public-boundary]
scopes: [implementation/cache]
shared_scopes: [contracts/navigation]
paths: []
tags: [cache, portability]
---
The supported-filesystem contract states six properties a cache root must provide and decides **not** to refuse an unrecognized filesystem. That decision is derived, not a shortcut: no filesystem failure can produce a wrong artifact, so a refusal would make an optional accelerator a correctness dependency, would fail closed on every filesystem nobody enumerated, and would still not detect the case that motivates it.

What it leaves is a user with no way to ask. A cache root on a network share, or one whose `tmp/` and `entries/` straddle a mount, presents as unexplained slowness rather than as a fact.

## What this ticket adds

`ExpansionCache::preflight`, shaped like the existing `account`: an explicit call that scans, changes nothing, and returns a report. It runs only the locally decidable subset:

- `entries/`, `tmp/`, and `locks/` under the root share one device;
- a `create_new` on a probe path refuses an existing path;
- an exclusive lock on a probe lock file is acquirable and releasable;
- a `rename` from `tmp/` to `entries/` succeeds;
- the modification time of a written file is reportable.

## Boundaries it must respect

- **Nothing on the expansion path.** Not called from `lookup`, `get_or_publish`, or `resolve`. The collection design eliminated an automatic trigger for the same reason and the argument transfers unchanged.
- **It refuses nothing.** The report is the deliverable; a caller decides.
- **It must not claim what it cannot test.** Whether an advisory lock excludes a process on *another host* is not answerable from one host — a lock taken under Darwin's `locallocks` or Linux's `local_lock=all` succeeds and reports success. The report has to say that this is unchecked rather than let a passing lock row imply it.
- **No `statfs`, and therefore no new `unsafe`.** Identifying the filesystem by type is exactly what the contract eliminated; `std::os::unix::fs::MetadataExt::dev` gives the only identity this needs and is safe.
- Staged `pub(crate)` under ADR 0074 convention 7, with the surface appended to `accept-the-tiler-cache-public-boundary` rather than promoted.

`spikes/cache/filesystem_probe.rs` already implements every check above as a standalone program and is the reference for what each one measures and how it avoids a vacuous pass.
