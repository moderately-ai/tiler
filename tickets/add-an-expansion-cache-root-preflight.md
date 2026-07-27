---
id: add-an-expansion-cache-root-preflight
title: Add an expansion cache root preflight
status: done
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

## Outcome (2026-07-27)

`ExpansionCache::preflight` runs the five locally decidable checks and returns a `PreflightReport`. Shaped like `account`: explicit, decides nothing, and reachable from no expansion path — `lookup`, `get_or_publish`, and `resolve` are untouched.

**Three verdicts rather than two, and this is the shape that mattered most.** `NotRun` is distinct from `Refuted` because the remedies differ: a refuted property says this root is unsuitable, while a probe that could not run says nothing was learned — most often that the root is not writable, which is a fact about the root rather than a filesystem verdict. Reporting the second as the first would send a caller to replace a filesystem when the answer is a permission.

**`all_probed_properties_hold` does not count `NotRun` as holding.** A report where nothing ran would otherwise read as a clean bill of health, which is exactly the vacuous pass the spike's own design avoids. The read-only test asserts that it returns false when every row is `NotRun`.

**The cross-host caveat is text on the report, not a `bool`.** It began as `cross_host_exclusion_is_unchecked() -> bool` returning a constant `true`; clippy's `unused_self` was right that it ignored the report, and the fix improved it. It is now an associated function returning the sentence a caller renders — a property of the probe, not of any one report. That is what stops a passing lock row from implying the stronger claim, since Darwin's `locallocks` and Linux's `local_lock=` both make a lock succeed while excluding only the local client.

**No `statfs` and no new `unsafe`**, as required: `MetadataExt::dev` is the only identity these checks need. No dependency added.

### The probes are non-vacuous by construction

The lock probe does not merely take a lock. It takes one, asserts a second `try_acquire` is **refused** while held, drops it, and asserts a third succeeds. A probe that only acquired would pass on a primitive that reports success while excluding nothing — the silent failure the supported-filesystem contract names. The rename probe likewise checks that the destination's *contents* changed and the source is gone, not merely that `rename` returned `Ok`.

### Tests

Three: every property holds on an ordinary root; a preflight leaves the root as it found it — asserted against a cache holding a real published entry, and checking that the entry still resolves as a **hit** rather than a republication, since an accounting count alone would be restored by a rebuild; and an unwritable root reports `NotRun` throughout with `all_probed_properties_hold` false.

The surface is appended to `accept-the-tiler-cache-public-boundary` rather than promoted.
