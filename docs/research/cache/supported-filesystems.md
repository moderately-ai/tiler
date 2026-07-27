---
schema: "tiler-doc/v1"
id: "tiler.research.cache.supported-filesystems"
kind: "research"
title: "Supported expansion cache filesystems"
topics: ["cache", "artifacts", "concurrency", "durability"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.artifact-abi", "tiler.contract.metal-backend"]
depends_on: ["tiler.research.cache.crash-race-protocol"]
ticket: "define-supported-expansion-cache-filesystems"
---

# Supported expansion cache filesystems

Status: the fifth follow-up gate of the [crash and race protocol note](crash-and-race-protocol.md). That note asks to "define supported local filesystems and add platform-specific Windows and network-filesystem feasibility gates before claiming portability", and records of its own sourced facts that "These facts do not establish equivalent behavior for every network filesystem".

This note states the properties as a contract, measures them with a checked-in probe, decides what an unsupported filesystem does, and answers the access-time question [the collection note](bounded-collection.md) deferred here.

## The headline, before the evidence

**Inference.** The gate is written as though a filesystem could make the cache wrong. It cannot. Section 2 enumerates every way each property can fail and shows that each one reduces to an absence, an I/O error, an unpublished result, or duplicated compiler work — never to a caller receiving an artifact that is not the one its subject names. Correctness does not rest on the filesystem; **compile-once suppression and cache retention do**.

That changes what this gate can honestly deliver. A supported set is a statement about how well the cache *works*, not about whether it is *safe*, and the enforcement mechanism has to match: an explicit, loud refusal for every locally decidable failure, and a stated, probeable contract for the one that is not locally decidable at all.

## 1. The six properties, and what each costs when it fails

**Fact.** Six filesystem properties carry application names in ADR 0050 and in `crates/tiler-cache/src/expansion/`. Two things the protocol also relies on are *not* on this list, deliberately, and Section 2 turns on the difference: **immutable final entries** hold because no code path in the crate ever opens a content path for writing — the only operations performed on one are `File::open`, `fs::rename`, and `fs::remove_file` — and **validation on every hit** is arithmetic over bytes already in memory. Neither is a filesystem assumption wearing an application name. They are what the filesystem cannot take away.

| # | Property | What the filesystem must guarantee | What happens when it does not |
| --- | --- | --- | --- |
| P1 | Same-device temporary | `tmp/` and `entries/` under one root resolve to one filesystem | `rename` fails `EXDEV`; `PublicationRefusal::CrossesFilesystems`; validated result returned uncached |
| P2 | `rename` replaces atomically | the destination name never resolves to nothing, and never to a blend | reader sees `MissReason::Absent`, or bytes the bundle frame rejects; rebuild |
| P3 | `create_new` excludes | `O_CREAT|O_EXCL` refuses an existing path | two writers may share one temporary; the re-read validation refuses it, or a corrupt entry is published and refused on the next read |
| P4 | Open-unlinked reader | a descriptor opened before `unlink` keeps reading | read error; `MissReason::Unavailable`; rebuild |
| P5 | Advisory lock excludes | an exclusive lock on the per-key file is seen by every contender | duplicate compilation; no correctness consequence |
| P6 | Reportable modification time | `st_mtime` exists and separates publications | collection order degrades toward arbitrary; the collector's re-`stat` window widens |

**Fact — P1.** Darwin [`rename(2)`][darwin-rename] requires that old and new "must reside on the same file system"; POSIX specifies `[EXDEV]` for the cross-filesystem case; Linux [`rename(2)`][linux-rename] adds that "**rename**() does not work across different mount points, even if the same filesystem is mounted on both". `Layout` puts the temporary under the same cache root as the entry, which makes the failure unreachable in ordinary operation, and `store::crosses_filesystems` still classifies it rather than assuming it away.

**Fact — P2.** POSIX [`rename`][posix-rename] states that "a directory entry named _new_ shall remain visible to other threads throughout the renaming operation and refer either to the file referred to by _new_ or _old_ before the operation began". Linux states the same as "it will be atomically replaced, so that there is no point at which another process attempting to access _newpath_ will find it missing". Darwin adds a crash clause the others do not: "The rename() system call guarantees that an instance of new will always exist, even if the system should crash in the middle of the operation."

**Inference — P2 is about visibility, not persistence.** POSIX specifies nothing normative about durability across a system crash, and the Linux page discusses none. Darwin's crash clause is about *which name exists*, not about the renamed file's data blocks having reached the medium. The `ProcessCrash` durability default claims exactly what these three sources support and no more, so the default is correctly stated rather than optimistic. Whether `Fsync` should become the default is not decided here and remains `measure-expansion-cache-durability-policies`'s.

**Fact — P3.** Linux [`open(2)`][linux-open] documents that "On NFS, O_EXCL is supported only when using NFSv3 or later on kernel 2.6 or later", and that "In NFS environments where O_EXCL support is not provided, programs that rely on it for performing locking tasks will contain a race condition". Darwin `mount_nfs(8)` documents that the client's "default is to try version 3 first, and fall back to version 2 if the mount fails", so NFSv2 is reachable by negotiation rather than only by explicit request.

**Fact — P4.** POSIX [`unlink`][posix-unlink] states that "If one or more processes have such a reference to the file when the last link is removed, the link shall be removed before unlink() returns, but the removal of the file contents shall be postponed until there are no such references to the file." Darwin `unlink(2)` says the same in its own words. This is what lets readers take no lock.

**Fact — P5.** Rust's `File::lock` "currently corresponds to the `flock` function on Unix with the `LOCK_EX` flag", documents that this "may change in the future", and says "This lock may be advisory or mandatory" ([`std::fs::File`][rust-file]). Darwin `flock(2)` describes advisory locking, notes that "processes may still access files without using advisory locks possibly resulting in inconsistencies", and lists `[ENOTSUP]` among its errors. Linux [`flock(2)`][linux-flock] states that "flock() places advisory locks only" and that locks "are associated with an open file description".

**Fact — P6.** `collect::select` orders by `metadata.modified()` and breaks ties on the key, and `remove_if_unchanged` compares length and modification time against what the scan saw before removing an entry.

## 2. Why no property failure can produce a wrong artifact

**Inference.** A caller receives artifact bytes by exactly one route: `read_entry` opens a content path, reads at most `max_bundle_bytes`, checks the path itself through `layout::key_of_entry_path`, runs `bundle::decode` against the requested key, and runs the pinned `decode_artifact` validator. `bundle::decode` checks magic, schema, digest-algorithm tag, reserved zeroes, declared total length against the bytes actually present, the embedded key against the requested key, every section's bound, contiguity and digest, the absence of trailing bytes — and finally that the carried subject *re-derives* the embedded key. There is no partial mode and no order in which a caller can skip a step.

**Inference.** So the question "can a bad filesystem produce a wrong artifact" reduces to "can it place, at the content path for key `K`, a complete byte run that passes all of that and is not the artifact `K`'s subject names". Enumerating the failures of Section 1 against that:

- **Truncation, interleaving, or a partially visible replacement** changes the byte run, so the declared total length, a section digest, or the subject-to-key re-derivation refuses it. A blend of two *identical* bundles is that bundle.
- **A stale or blended page cache** is the same case: refused unless the bytes are exactly one published bundle.
- **A valid bundle for another key** is refused twice over — by the embedded-key check and by the shard component of the entry-path parser.
- **An older generation of the same key's entry** is a valid bundle for `K`, produced by a writer whose subject derived `K`. Accepting it is correct, not wrong; complete identity is what makes one entry for `K` interchangeable with another.
- **A lock that does not exclude** lets two writers compile the same subject and each publish a validated bundle for `K`. Whichever rename lands last wins, and both were valid.
- **A lock that does not exclude, plus a shared temporary** (P3 failing at the same time) lets one writer validate a temporary another then rewrites, so a *corrupt* bundle can reach the content path. The next reader refuses it, quarantines it, and rebuilds.
- **An eviction racing a publication** removes an entry, which produces an absence.

**Inference.** The enumeration is total over the six properties and every branch terminates in a miss, an I/O error, an unpublished result, or duplicated work. This is not luck; it is ADR 0050's stated architecture doing what it says: "The lock suppresses duplicate work; validation, immutability, complete identity, and atomic publication provide correctness." A filesystem can take away the lock and the atomicity. It cannot take away the other two, because they do not live on it.

**Measurement boundary.** The derivation assumes the governed digest is not forged. A same-user writable cache is not an adversarial boundary — the crash/race note already records that an attacker able to replace cache files can construct new internally consistent bytes — and nothing here changes that.

## 3. The supported set

**Proposal.** A filesystem is *supported* when P1 to P6 hold on it. The set is stated by property, and membership is decided by evidence rather than by name:

| Platform | Filesystem | Status | Evidence |
| --- | --- | --- | --- |
| macOS | APFS (local volume) | supported | measured, Section 6 |
| macOS | exFAT (local volume) | supported for correctness; P6 is coarse | measured, Section 6 |
| Debian-family Linux | ext4, btrfs, xfs (local) | **inactive** — not a supported row; derivation retained as research | Section 1's POSIX and Linux man-page facts, **unmeasured** |
| any | NFS, SMB, and other network filesystems | **not supported** | Section 4 |
| any | a root whose `tmp/` and `entries/` differ in `st_dev` | **not supported**, and refused loudly | P1 |

**Why the Linux row is inactive rather than merely unmeasured (2026-07-27).** `AGENTS.md` states that Tiler develops on macOS only and that other platforms are unsupported rather than maintained as untested branches. A row calling a Linux filesystem supported would therefore name a platform the product does not support, on evidence that was never gathered — two separate overstatements in one cell. The derivation below is sound and is kept as evidence; it is not a support claim. **The exact trigger for reopening it is the admission of Linux as a supported development or product platform**, and measurement is required before the row becomes active even then. `docs/artifact-abi.md` states the supported set as macOS APFS and exFAT alone.

**Measurement boundary — the exact unmeasured case.** No Linux host was available to this ticket. The probe of Section 6 has never executed on ext4, btrfs, or xfs, and the Linux rows above rest on POSIX and the Linux man-pages alone. The check that would close it is one command on a Debian-family host: `rustc --edition 2021 spikes/cache/filesystem_probe.rs -o /tmp/tiler-fs-probe && /tmp/tiler-fs-probe ~/.cache`. Running it is `probe-the-expansion-cache-filesystem-properties-on-linux`.

**Inference — why the network row is a whole class rather than a list.** The property that fails there is P5, and it fails for a reason that has nothing to do with which protocol is in use: exclusion has to be arbitrated somewhere other than the local kernel, and both supported platforms document a mode in which it is not. Darwin `mount_nfs(8)` documents `locallocks`, under which the client performs "all file locking operations locally on the NFS client (in the VFS layer) instead of on the NFS server", so that "the NFS server and other NFS clients will have no knowledge of the locks" — and it is "enabled by default" for mounts that are both soft and read-only. Linux [`nfs(5)`][linux-nfs] documents `local_lock=` and `nolock` with the same consequence: "applications can lock files, but such locks provide exclusion only against other applications running on the same client. Remote applications are not affected by these locks."

**Fact.** Apple's `mount_smbfs(8)` says nothing about file locking at all. The exact check: `man 8 mount_smbfs | col -b | grep -i 'lock\|advisory\|byte range'` returns no lines on macOS 27.0 build 26A5388g. That is an absence of documentation, not evidence of absent locking, and it is why SMB is in the unsupported class rather than given its own verdict.

## 4. What an unsupported filesystem does, and what can be detected

**Fact.** The failures split cleanly into two groups, and the split is the finding.

**Locally decidable, and already loud.** Each of these already produces a typed, explainable outcome with no new code:

| Condition | Where it surfaces |
| --- | --- |
| `tmp/` and `entries/` on different filesystems | `PublicationRefusal::CrossesFilesystems { temporary, entry }` |
| `flock` unsupported — Darwin `nolocks` returns `ENOTSUP` | `CacheUnavailable { operation: AcquireLock, .. }`, then `Resolution::Uncached` |
| root absent, unwritable, or unreadable | `CacheUnavailable` naming the operation and path |
| a published entry that does not validate | `MissReason::Rejected(..)` naming the exact boundary |

**Not locally decidable, at all.** Whether an advisory lock excludes a process on *another* host is not a question a single host can ask. A lock taken under `locallocks` or `local_lock=all` succeeds, reports success, and is indistinguishable from one that excludes; the only experiment that separates them needs a second machine mounting the same export and contending for the same key. **This is a limitation of the question, not of the implementation, and no amount of probing removes it.** The consequence, from Section 2, is bounded to duplicated compilation.

**Proposal.** Therefore the contract is *stated and probeable*, not *enforced by refusal*, and `spikes/cache/filesystem_probe.rs` is the stated way to decide a candidate root.

## 5. Why detect-and-refuse does not survive

**Inference.** Refusing an unrecognized filesystem was the obvious candidate and it fails three separate tests, so it is not offered as an option.

- **It contradicts the record it would be protecting.** ADR 0050 rejects "treating cache failure as compilation failure" because it "would make an optional accelerator a correctness dependency". A refusal keyed on filesystem identity is that, with an extra step.
- **Its false-negative cost exceeds the harm it prevents.** The harm prevented is duplicated compiler work (Section 2). The harm caused is that every filesystem nobody enumerated — tmpfs, an overlay layer inside a container, ZFS, a future Apple volume format — fails closed on a component that is optional by design. An allowlist over an open set fails in the wrong direction.
- **It does not catch the case that motivated it.** A shared NFS root mounted `locallocks` presents as an ordinary directory on an ordinary path. Identifying the filesystem would not reveal the mount option, and on Darwin the option is not exposed through `statfs`'s `f_flags` at all.

**Inference.** There is also a cost the alternative pays and the chosen design does not: identifying a filesystem needs `statfs`, which is a foreign API with no safe route, so it would require a new admitted `unsafe` site under [ADR 0079](../../decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md). Paying that to build a mechanism whose three failures are listed above is the wrong trade twice over.

**Proposal.** What is worth adding instead is an explicit, caller-invoked preflight that runs the locally decidable subset once against a configured root and returns a report — the same shape as `ExpansionCache::account`, which scans and changes nothing. It is not on the expansion path, it refuses nothing, and it turns "your cache root is on a network share" from something a user discovers as unexplained slowness into something they can ask. That is `add-an-expansion-cache-root-preflight`.

## 6. Access time, and the deferred use-recency question

**The answer is no.** No supported filesystem maintains access time usefully enough to order a working set, and the reason is stronger than the `noatime` mount option that motivated the deferral.

**Measurement.** On macOS 27.0 build 26A5388g, Darwin 27.0.0 arm64, on an APFS user-cache directory of `/System/Volumes/Data` — a volume whose mount options do **not** include `noatime` — `spikes/cache/filesystem_probe.rs` classifies the access-time behaviour as `relatime`:

```text
created=1785020947584495843  read1=1785020949589657921  read2=1785020949589657921
after-mtime-bump=1785020953598050695
```

A read two seconds after creation advanced the access time. A second read two seconds later did **not**. Pushing the modification time past the access time re-armed exactly one further advance. The probe distinguishes these three states rather than observing one, so `strict`, `relatime`, and `none` are separated by the experiment and not by assumption.

**Inference.** A published cache entry's modification time never changes — entries are immutable and publication is the only thing that sets it. Under that predicate, **an entry's access time advances at most once in its entire life, at its first read after publication, and never again.** It therefore records "has this been read at least once since it was published", which is a boolean, and cannot distinguish an entry hit on every build from one hit once and abandoned. That is precisely the distinction least-recently-used ordering exists to make.

**Fact.** Linux is the same by default and says so: [`mount(8)`][linux-mount] documents that "Since Linux 2.6.30, the kernel defaults to the behavior provided by this option" — `relatime` — "unless **noatime** was specified", that "the **strictatime** option is required to obtain traditional semantics", and that "since Linux 2.6.30, the file's last access time is always updated if it is more than 1 day old". The one-day clause makes the Linux boolean refresh daily rather than never, which is 24-hour granularity and still cannot order anything within a build session.

**Measurement.** The second filesystem measured is worse and is the case the deferral anticipated: a macOS exFAT volume mounts `noatime` by default, and the probe classifies it `none` — all four sampled access times were the identical whole second `1785020954000000000`.

**Inference — three further reasons, each independently sufficient.** Even granting a `strictatime` mount, the design does not survive:

1. **The degenerate case is silent.** On a `noatime` filesystem every entry has `atime == mtime`, which is exactly what "never read since publication" looks like. An access-time ordering would therefore *degenerate into publication ordering while believing it was doing use-recency* — a mechanism claiming a property it does not have, which is the failure mode the repository rule names. It is detectable only by an active probe, whose answer is per-mount and can change under a running cache.
2. **Access time is not evidence about this cache.** Any process that reads the file advances it: a backup pass, an indexer, an antivirus scan, a `grep -r` over the cache root. The signal is writable by everything on the machine and attributable to nothing.
3. **The payoff is bounded by one recompilation.** [The collection note](bounded-collection.md) already establishes that a wrong eviction costs at most a rebuild. Trading a stated, host-independent order for one that varies with mount options nobody can see is a bad trade for a bounded gain.

**Proposal.** `CollectionOrder::OldestPublicationFirst` stands as the only order, and its documented pathology stands with it. The deferral in the collection note is **closed with a negative answer**, and its trigger — "that ticket naming a supported filesystem set on which `atime` is maintained with useful granularity" — is not met on any measured or derived member of Section 3's set. Reopening it needs a new trigger, stated in Section 9.

## 7. Windows is dissolved, not deferred

**Fact.** The gate and the ticket both ask for a Windows spike, and the crash/race note records that "Windows cannot inherit the open-unlinked-reader conclusion." `AGENTS.md` stated, when this record was written, that Tiler "supports this bootstrap path on macOS and Debian-family Linux only; Windows and other Linux distributions are explicitly unsupported rather than maintained as untested branches", and `scripts/check_rust.py` "accepts only the CI-proven macOS arm64 and GNU Linux x86-64 profiles". **Both quotations are stale as of 2026-07-26 and the conclusion is unaffected**: `AGENTS.md` now says Tiler "develops on macOS only; other platforms are unsupported rather than maintained as untested branches", which narrows the supported set rather than widening it, and `e197176` deleted `scripts/check_rust.py` along with the rest of the Python gate, so no check enforces a host profile at all. Windows is further from support than when this was written, not closer.

**Inference.** The note's sentence is conditional — Windows needs a spike *before the cache core claims Windows support* — and the repository does not claim it. The condition is unmet, so there is no open gate, and there is no Windows obligation to defer. Should that support decision ever change, the spike becomes owed again by the same sentence; it is not a question this note answers or leaves open.

## 8. The probe

**Fact.** `spikes/cache/filesystem_probe.rs` is a dependency-free `std`-only program that measures P1 to P6 plus the access-time class against any directory, prints one tab-separated row per property, and exits non-zero when a required property is refuted. Every check is a real filesystem operation: the lock checks re-execute the program so the two contenders are separate processes and handshake over a pipe rather than sleeping; the replacement check runs a reader thread against 400 real publications; the cross-device check needs a second filesystem and **reports `skipped` rather than passing** when the caller's `--across` turns out to share the root's device.

**Measurement.** Recorded in [`spikes/cache/results/filesystem-probe-macos-27.0-2026-07-25.tsv`](../../../spikes/cache/results/filesystem-probe-macos-27.0-2026-07-25.tsv). Both roots — APFS and a formatted exFAT RAM disk — hold every required property, including cross-process lock exclusion, lock release on kill, and an unlinked file still readable through a descriptor opened before the unlink. The cross-device rename is refused with `ErrorKind::CrossesDevices`, which is the exact kind `store::crosses_filesystems` matches, so that classification is measured rather than assumed.

**Measurement.** Modification-time granularity differs by two orders of magnitude between them — about 48 microseconds on APFS against 10 milliseconds on exFAT. Both are far finer than a compilation, so P6 holds on both; the figure is recorded because it is the quantity the collector's re-`stat` accuracy depends on.

**Measurement boundary — the refutation branch is unexercised.** Every required property held on both filesystems reachable here, so no `REFUTED` row was produced by a real run. A uniform pass over a population is the signature `AGENTS.md` says to distrust, and the honest statement is that the `require` path's negative branch has been read and not observed. Two things bound the concern: the access-time classifier *did* return different answers for the two roots (`relatime` against `none`), and the cross-device check *did* return `skipped` when given a same-device argument, so neither of those is a constant function. The construction that would exercise a `REFUTED` row is a root whose `tmp/` and `entries/` are on different mounts, which needs a mount inside the cache root; `diskutil mount -mountPoint` at a non-standard path failed on this host without elevated privileges, and mounting is a host mutation this ticket had no authorization to perform.

## 9. What this does not settle

- **The Linux measurement.** Section 3's ext4/btrfs/xfs rows are derived, not measured. `probe-the-expansion-cache-filesystem-properties-on-linux` runs the probe on a Debian-family host and either confirms the row or narrows it.
- **The preflight.** Section 5 proposes it and this note does not implement it; every type it would touch is in `crates/tiler-cache`, outside this ticket's scope. `add-an-expansion-cache-root-preflight` owns it.
- **The durability default** is unchanged and still `measure-expansion-cache-durability-policies`'s. Section 1 establishes only that the *current* default claims no more than its sources support.
- **Deterministic I/O fault injection** remains `inject-deterministic-expansion-cache-io-failures`. Section 2 argues that each filesystem failure lands in a reported branch; injecting them is how that argument is tested rather than reasoned.
- **Framing fuzzing** remains `fuzz-the-expansion-cache-framing-paths`. Section 2's derivation leans on `bundle::decode` refusing every byte run that is not a published bundle, which is exactly the surface that ticket fuzzes.
- **A reported publication failure that actually published.** Linux `rename(2)` records under BUGS that "On NFS filesystems, you can not assume that if the operation failed, the file was not renamed." On a network root this makes `PublicationRefusal::Unavailable` over-report: the entry may be published and valid while the caller is told it was not. The caller still holds a validated artifact, so this is report accuracy and not correctness — the same class as the collector's re-`stat` boundary. It is not tracked separately because network filesystems are unsupported; it would become a real obligation if that ever changed.
- **Access-time ordering.** Closed negatively in Section 6. **Trigger for reconsideration:** a supported filesystem measured by the probe as `strict` *together with* a way for the collector to distinguish "this mount does not maintain access time" from "this entry was never read" without an active probe. Both halves are required — the first alone is what made the original deferral look closable.

## Traceability

Closes gate 5 of the [crash and race protocol note](crash-and-race-protocol.md) and the access-time deferral in [the collection note](bounded-collection.md). Preserves ADR 0050 unchanged: this note adds the supported-filesystem contract that record's protocol presumed, and contradicts none of its decisions. The normative statement lives in [`docs/artifact-abi.md`](../../artifact-abi.md); [`docs/backends/metal.md`](../../backends/metal.md) carries the same sentence for the Metal expansion cache.

[posix-rename]: https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html
[posix-unlink]: https://pubs.opengroup.org/onlinepubs/9799919799/functions/unlink.html
[darwin-rename]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/rename.2.html
[linux-rename]: https://man7.org/linux/man-pages/man2/rename.2.html
[linux-open]: https://man7.org/linux/man-pages/man2/open.2.html
[linux-flock]: https://man7.org/linux/man-pages/man2/flock.2.html
[linux-nfs]: https://man7.org/linux/man-pages/man5/nfs.5.html
[linux-mount]: https://man7.org/linux/man-pages/man8/mount.8.html
[rust-file]: https://doc.rust-lang.org/stable/std/fs/struct.File.html
