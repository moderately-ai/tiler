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

Windows cannot inherit the open-unlinked-reader conclusion. Its sharing flags,
replacement API, and deletion semantics need their own spike before the cache
core claims Windows support.

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
   cache frame, then fuzz every framing and bounded-allocation path.
3. Add deterministic injected errors for disk full, rename failure, directory
   sync failure, compiler failure, and retry exhaustion.
4. Measure cache latency and survival for `process-crash` versus `fsync`; only
   then decide the default in an ADR.
5. Define supported local filesystems and add platform-specific Windows and
   network-filesystem feasibility gates before claiming portability.
6. Design bounded GC/accounting separately and stress eviction with active
   writers/readers at 1, 8, and 32 processes.
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
