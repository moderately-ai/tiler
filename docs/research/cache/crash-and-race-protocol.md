---
schema: "tiler-doc/v1"
id: "tiler.research.cache.crash-race-protocol"
kind: "research"
title: "Expansion cache crash and race protocol"
topics: ["cache", "artifacts", "concurrency", "durability"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model", "bounded-measurement"]
informs: ["tiler.contract.artifact-abi", "tiler.contract.frontend-integration"]
adopted_by: ["ADR-0050"]
ticket: "cache-crash-race-harness"
---

# Expansion cache crash and race protocol

Status: completed research adopted by ADR 0050 and exercised by a process-level spike on 2026-07-20.

This note separates sourced filesystem facts, protocol deductions, product
proposals, and host observations. The cache is an expansion-time accelerator;
it is never a runtime dependency or an authority for tensor semantics.

## Decision summary

Use one immutable, self-validating bundle per complete compilation key. A miss
opens a stable per-key lock file, takes an exclusive advisory lock, rechecks,
builds in a unique same-filesystem temporary file, validates the completed
bytes, and publishes with one atomic rename. Readers are lock-free: they open a
final path and validate the requested key, framing, lengths, schema, manifest,
and every section digest before accepting it.

The lock suppresses duplicate external compiler work. It is not a correctness
boundary. Correctness comes from complete identity, immutable final entries,
validation on every hit, one atomic publication operation, and treating every
cache failure as a miss. A process dying releases its OS lock; there is no PID
file, timestamp lease, or stale-lock deletion algorithm.

The initial durability default should optimize for process-crash safety, not
claim power-loss durability. An opt-in `fsync` policy can synchronize the
temporary file before rename and the containing entry directory afterward.
Even that name is intentionally narrower than “power safe” on Darwin.

## Sourced facts

- Rust `File::lock` was stabilized in Rust 1.89. On Unix it currently maps to
  `flock(LOCK_EX)`; the lock is released after the file and duplicated or
  inherited descriptors close. Rust explicitly says the mapping may change
  and that the lock may be advisory. [Rust `File` documentation][rust-file]
- Darwin describes `flock` as advisory: cooperating processes exclude each
  other, but a process that ignores the protocol may still access the file.
  Locks are associated with the open file and duplicated descriptors refer to
  the same lock. [Darwin `flock(2)`][darwin-flock]
- POSIX requires `rename` to act atomically. Darwin additionally requires old
  and new to be on the same filesystem, replaces an existing `new`, and says
  an instance of `new` always exists if a crash occurs during replacement.
  [POSIX `rename`][posix-rename] [Darwin `rename(2)`][darwin-rename]
- Rust `fs::rename` exposes the platform operation and documents that it does
  not cross mount points. Its exact replacement behavior is platform-specific.
  [Rust `rename` documentation][rust-rename]
- Darwin `unlink` removes the directory entry immediately but defers reclaiming
  the file while a process still has it open. This is the basis for lock-free
  readers racing coordinated eviction on the initial Unix/Darwin host.
  [Darwin `unlink(2)`][darwin-unlink]
- Rust `File::sync_all` asks the OS to synchronize file content and metadata;
  `File::flush` is currently a no-op for unbuffered files on Unix and Windows.
  Closing a Rust `File` also discards close errors, so close alone is not a
  durability primitive. [Rust `File` documentation][rust-file]
- Darwin warns that `fsync` can leave data in a drive's volatile cache and can
  therefore lose or reorder it on power loss. `F_FULLFSYNC` additionally asks
  the device to flush buffered data, may be slow, and can still be unsupported
  or ignored by hardware. [Darwin `fsync(2)`][darwin-fsync]
  [Darwin `fcntl(2)`][darwin-fcntl]

These facts do not establish equivalent behavior for every network filesystem,
Windows filesystem, container mount, or cache directory supplied by a user.
The production cache needs a platform adapter and an explicit supported-
filesystem contract; this spike establishes the first Apple-host protocol.
[The supported-filesystem note](supported-filesystems.md) now states that
contract and measures it; the platform adapter remains `lock.rs` naming the
primitive in one place, because no supported platform has yet needed a second.

## Cache namespace

For a versioned algorithm namespace and lowercase hexadecimal digest `K`:

```text
<root>/v1/entries/<K[0..2]>/<K>.bundle
<root>/v1/locks/<K[0..2]>/<K>.lock
<root>/v1/tmp/<K[0..2]>/<K>.<pid>.<nonce>.tmp
```

The path parser accepts only a fixed-width hexadecimal key. Temporary files use
`create_new`; uniqueness is established by the filesystem operation, not by
trusting the PID or nonce. Putting the temporary file below the same cache root
is a construction rule that makes cross-filesystem rename impossible under
normal operation. Production code should still report `EXDEV` as a cache miss.

Lock files are stable namespace objects and are not cache entries. Internal GC
must not unlink them. Unlinking a locked file allows a new process to create a
different inode at the same path and take an independent lock while an old
process still holds the first inode.

## Required protocol

```text
derive complete compilation key K
open final entry for K
if exact validation succeeds: return hit
if cache namespace cannot be read: compile without cache publication

create/open stable lock file for K
acquire exclusive advisory lock
recheck final entry for K
if exact validation succeeds: return hit

invoke compiler into process-owned working state
encode one complete cache bundle embedding K
create_new a unique temporary file on the final filesystem
write all bytes
open the temporary file separately and validate it exactly
[fsync policy: sync temporary file]
rename temporary file over final entry
[fsync policy: sync containing entry directory]
release lock by closing its descriptor
```

The one-file cache bundle should contain the already specified artifact
envelope plus cache framing sufficient to reject a misplaced entry. Validation
is bounded and includes at least:

1. cache magic, cache schema, algorithm/domain identifier, exact total length,
   and absence of unknown required framing;
2. embedded compilation key equal to requested `K`;
3. canonical artifact manifest and exact envelope identity;
4. every declared section length and cryptographic digest;
5. compiler, target, numerical, ABI, routing, helper, and artifact provenance
   already required by the complete key contract.

A valid bundle at the wrong content path is a miss. A valid envelope with an
invalid cache frame is a miss. No reader consumes a section before the whole
envelope and all required meanings have validated.

If an old final entry is corrupt, the locked writer may atomically replace it
with the newly validated temporary file. A reader that opened the old inode can
only reject it; a later reader sees the new file. Optional quarantine is for
diagnostics, not correctness, and must be bounded so corrupt data cannot grow
the cache without limit.

## Durability policy

Atomic visibility and durable persistence are separate properties.

| Policy | Write sequence | Claim |
| --- | --- | --- |
| `process-crash` (recommended default) | write, separately validate, close, rename | A killed writer cannot expose its partial temporary file at the final path. Abandoned temps are ignored. No OS/power-loss persistence claim. |
| `fsync` | write, validate, `sync_all(temp)`, rename, `sync_all(entry_dir)` | Requests persistence of file bytes/metadata and the directory update through the OS/filesystem APIs. Does not claim Darwin drive-cache flush or universal filesystem behavior. |
| future `full-fsync` | platform-specific full flushes around publication | Potential opt-in for unusually strict cache survival; requires capability detection, error policy, and measurement before adoption. |

The default is appropriate because a cache lost or corrupted by machine failure
is revalidated and rebuilt, while every expanded Rust artifact already embeds
its bytes. `fsync` changes expected cache survival and latency, not generated
program correctness. Production telemetry should measure both modes before the
default is made durable in an ADR.

## Failure outcomes

| Event | Allowed cache result | Required expansion result |
| --- | --- | --- |
| writer dies before rename | no final entry or prior valid final; abandoned temp | next process rebuilds or hits prior valid entry |
| writer dies after rename | new valid final entry | next process validates and hits |
| corrupt/truncated final | miss | locked recheck then rebuild |
| lock holder dies | OS releases lock on last close | waiter rechecks and continues |
| cache root is absent or unwritable | no publication | compile into process-owned temporary state, validate, and embed |
| entry/cache deleted while idle | miss | rebuild |
| externally deleted while active | duplicate work and transient I/O errors are allowed | retry/fail open; never accept unvalidated bytes |
| compiler or artifact validation fails | no publication | fail expansion with compiler/validation diagnostic |

“Fail open” applies only to the cache mechanism. It does not convert an actual
compiler error, invalid generated artifact, unsupported target, or numerical
contract failure into success.

An external recursive deletion can unlink a live lock inode. The harness
demonstrates that this can lose compile-once behavior while preserving output
correctness. Therefore a Tiler-provided whole-cache purge should either require
quiescence or rename the version root out of service and tolerate active users;
it must not promise compile-once during arbitrary external `rm -r`.

Of those two, only the rename is implementable: no code can establish that no
other process is using a configured root, so requiring quiescence would promise
something unverifiable. `ExpansionCache::purge` therefore renames the version
root out of service in one atomic operation and reclaims it afterwards, and
[the collection note](bounded-collection.md) states exactly what it does not
promise — compile-once does not survive the rename, because a lock held in the
retired tree and one taken in the new tree are different inodes.

## Garbage collection

Internal eviction obeys these rules:

1. Final entries are immutable. GC removes or renames them; it never truncates
   or edits them.
2. GC acquires the same per-key exclusive lock before evicting an entry or
   deleting abandoned temporaries for that key. This serializes it with writers.
3. Lock files are retained. Directory-level cleanup must not remove a lock
   shard while it may be in use.
4. A Darwin reader opens the final file before reading and validates through
   that descriptor. If GC unlinks after open, the descriptor remains readable;
   if GC wins before open, the reader observes a miss.
5. Recency and size accounting are separate, disposable metadata. They never
   mutate bundle bytes and are not trusted for hit correctness.
6. GC has bounded work and storage policies: maximum total bytes, maximum entry
   count, maximum diagnostic/quarantine bytes, temporary-file grace period,
   and a best-effort cleanup budget per invocation. Exact defaults require
   workload measurement.

Rules 1 to 4 are implemented and tested. Rules 5 and 6 are now designed,
implemented, and measured, and [the collection note](bounded-collection.md)
records two ways the result departs from rule 6 as written. **There is no
default bound**: the ceilings exist exactly when a caller states one, because
this note's own "exact defaults require workload measurement" makes any value
chosen now a guess, and a default bound would delete entries invisibly on the
strength of it. **There is no per-invocation work budget**, because the
collector takes each key lock with `try_lock` and never waits — a budget's
purpose would have been to cap an unbounded wait, and there is none to cap. A
held key lock is also positive evidence that the entry is live, so skipping a
contended key is better selection as well as bounded latency.

Windows cannot inherit the open-unlinked-reader conclusion. Its sharing flags,
replacement API, and deletion semantics need their own spike before the cache
core claims Windows support. That condition is unmet and no such spike is owed:
`AGENTS.md` states that Tiler develops on macOS only and that other platforms
are unsupported rather than maintained as untested branches — narrower than when
this was written, which named Debian-family Linux too. The Rust sub-gate this
paragraph once cited was deleted by `e197176` with the rest of the Python
tooling, so no check enforces a host profile at all; the policy is the authority
and it is held by review. The sentence stands as the obligation that would
revive if the support decision ever changed, and Windows is further from that
than it was.

## Rust version consequence

Using only `std::fs::File::{lock,try_lock,lock_shared,unlock}` sets the cache
implementation's MSRV to at least 1.89. That is the smallest and clearest
initial choice if Tiler's eventual workspace MSRV permits it. If the product
chooses an older MSRV, use a narrowly audited locking crate or OS adapter with
the same semantics; do not replace advisory locking with create-once PID lock
files. The spike intentionally uses the standard 1.89 API so its exercised
primitive and its documented primitive are identical.

## Security boundary

Integrity validation handles accidents, partial writes, and non-cooperating
cache cleanup. It does not make a same-user writable cache an adversarial code-
signing boundary: an attacker able to replace cache files can construct new
internally consistent bytes. The default root must be private to the user and
must not silently follow attacker-controlled symlinks. A shared or hostile cache
requires a separate authentication/sandbox design. This is independent from
the cross-process race protocol.

## Spike and observations

[`spikes/cache/cache_harness.rs`](../../../spikes/cache/cache_harness.rs) is a
dependency-free parent/worker executable. It uses real processes, Rust's
standard advisory file lock, `create_new`, separate-descriptor validation,
same-root temporary files, atomic rename, optional file/directory `sync_all`,
and SHA-256 framing. The parent stops writers with the OS process-kill API at:

```text
after lock
after locked recheck
after temporary creation
mid-write
after write
after temporary validation
after file sync
after rename
after directory sync
```

On an Apple-silicon host running macOS 27.0 build 26A5388g and
`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, the published entrypoint ran the
full suite ten times with a concurrency setting of 32. The
[compact per-run evidence][cache-evidence] records every repetition and count:

- 32 simultaneous processes for one key produced one compilation record and
  one valid final entry;
- 32 simultaneous distinct keys produced 32 independently valid entries;
- recovery passed at all nine kill points;
- truncated and digest-corrupt final files were rejected and replaced;
- entry deletion and idle whole-cache deletion rebuilt correctly;
- active recursive deletion lost compile-once suppression, as predicted, but
  the surviving final entry validated;
- an unusable cache root returned a validated uncached result;
- a reader that opened an entry before coordinated eviction completed reading
  and validating its open descriptor after unlink;
- an injected permanently blocked child was killed and reaped at its 100 ms
  overall deadline in every repetition;
- file and containing-directory `sync_all` succeeded on the tested APFS volume.

These are observations, not portable guarantees. The harness does not emulate
power failure, storage-controller reordering, NFS lock failure, disk-full
partial writes, checksum collision, malicious same-user replacement, Windows
sharing modes, or a real Metal compiler. Production implementation remains out
of scope for this ticket.

## Follow-up gates

1. Choose and publish Tiler's workspace MSRV; accept Rust 1.89 or select and
   audit an older-compatible lock adapter.
2. Integrate the finalized artifact envelope rather than the spike's miniature
   cache frame, then fuzz every framing and bounded-allocation path. **The
   process half is closed:** `tiler_cache::expansion::harness` runs the same nine
   kill points against the bundle `tiler-cache` publishes, with real
   re-executed processes aborting inside the real publication path, recorded in
   [`spikes/cache/results/`](../../../spikes/cache/results/). It substitutes a
   stand-in payload validator for `decode_artifact`, because a real envelope
   needs `tiler-ir` and ADR 0082 item 2 decides the cache does not depend on it;
   a positive end-to-end hit carrying a real compiled artifact is still owed by
   the orchestrator holding both crates. The fuzzing half remains open under
   `fuzz-the-expansion-cache-framing-paths`.
3. Add deterministic injected errors for disk full, rename failure, directory
   sync failure, compiler failure, and retry exhaustion.
4. Measure cache latency and survival for `process-crash` versus `fsync`; only
   then decide the default in an ADR. **Closed** by
   [ADR 0083](../../decisions/0083-keep-process-crash-as-the-default-cache-durability.md),
   with one half measured and the other bounded by what the platform documents.
   `Fsync` costs 6.5x to 18.7x more per publication on the supported macOS/APFS
   profile across two hosts, and the cost is flat in the payload rather than
   proportional to it, so what it buys is a fixed number of synchronization
   round-trips. Survival is *not* measured and the latency cannot stand in for
   it: Darwin's `fsync(2)` documents that data may remain in a device's volatile
   cache, so establishing power-loss survival would need `F_FULLFSYNC` and a way
   to cut power. `ProcessCrash` stays the default because every cache failure
   resolves to repeated work rather than an incorrect artifact.
5. Define supported local filesystems and add platform-specific Windows and
   network-filesystem feasibility gates before claiming portability. **Closed**
   by [the supported-filesystem note](supported-filesystems.md), with one part
   dissolved rather than answered. It states the six properties the protocol
   rests on, measures them with `spikes/cache/filesystem_probe.rs` on local APFS
   and local exFAT — the supported set — and derives ext4, btrfs, and xfs from
   POSIX and the Linux manual pages without measuring them, retained as inactive
   research rather than as a supported row. It excludes network filesystems
   because macOS and Linux alike document a mount mode under which an advisory
   lock reports success while excluding only the local client, and no single
   host can detect it. That failure costs compile-once suppression and not correctness, which is
   why the note refuses an unrecognized filesystem nowhere and reports every
   locally decidable failure instead. The **Windows** half is dissolved: this
   note's Windows sentence below is conditional on the cache core claiming
   Windows support, and the repository does not claim it.
6. Design bounded GC/accounting separately and stress eviction with active
   writers/readers at 1, 8, and 32 processes. **Closed** by
   [the collection note](bounded-collection.md):
   `tiler_cache::expansion::collect` implements non-durable whole-cache
   accounting, a declared bound that defaults to removing nothing, a
   never-blocking collector that names every entry it removes, and a purge that
   retires the version root in one rename. `expansion::harness` runs the ladder
   at 1, 8, and 32 real writer processes against a real collecting process.
   Every type is staged `pub(crate)` under ADR 0074 convention 7, so no consumer
   can reach it until `accept-the-expansion-cache-maintenance-boundary` promotes
   it, and nothing schedules a collection yet.
7. Run the harness under Cargo and rust-analyzer process patterns once the
   proc-macro spike exists; this ticket only establishes the storage protocol.

## Traceability

ADR 0050 and the artifact/frontend contracts adopt this protocol. The
[cache spike](../../../spikes/cache/README.md) owns the bounded process tests;
power-loss durability, filesystem portability, and production GC remain open.

[rust-file]: https://doc.rust-lang.org/stable/std/fs/struct.File.html
[rust-rename]: https://doc.rust-lang.org/stable/std/fs/fn.rename.html
[posix-rename]: https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html
[darwin-flock]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html
[darwin-rename]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/rename.2.html
[darwin-unlink]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/unlink.2.html
[darwin-fsync]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html
[darwin-fcntl]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html
[cache-evidence]: ../../../spikes/cache/results/macos-27.0-rustc-1.99.0-nightly-2026-07-21.tsv
