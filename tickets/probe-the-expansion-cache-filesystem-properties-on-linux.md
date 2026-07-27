---
id: probe-the-expansion-cache-filesystem-properties-on-linux
title: Probe the expansion cache filesystem properties on Linux
status: closed
priority: p2
dependencies: []
related: [define-supported-expansion-cache-filesystems]
scopes: [research/cache]
shared_scopes: [contracts/navigation]
paths: []
tags: [cache, portability, durability]
---
The supported-filesystem contract names local ext4, btrfs, and xfs on Debian-family Linux as supported **by derivation only**. No Linux host was available when it was written, so those rows rest on POSIX and the Linux manual pages and have never been executed against.

`spikes/cache/filesystem_probe.rs` already exists and needs no changes to answer this. Run it on a Debian-family host:

```sh
rustc --edition 2021 spikes/cache/filesystem_probe.rs -o /tmp/tiler-fs-probe
/tmp/tiler-fs-probe ~/.cache --across /path/on/another/mount \
  --evidence /tmp/tiler-fs-evidence.tsv
```

## What closes this

- One evidence row per filesystem under `spikes/cache/results/`, with the distribution, kernel, and mount options in the header.
- Each of ext4, btrfs, and xfs either confirmed or moved out of the supported table with the property that refuted it.
- The access-time class recorded for each. The contract's negative answer on use-recency ordering predicts `relatime` — the kernel default since 2.6.30 — and a `strict` result on any of them is *half* of that answer's stated reopening trigger. It is not the whole trigger: the other half is a way to distinguish "this mount does not maintain access time" from "this entry was never read", without an active probe.
- The `mtime` granularity figure for each, because that is what the collector's re-`stat` accuracy depends on.

**One thing worth watching.** Every required property held on both filesystems the original run could reach, so the probe's `REFUTED` branch has been read and never observed. A Linux run is the natural place to see it fire — a `tmpfs` `/tmp` beside an ext4 home makes the cross-device case trivial to construct without root, which is what blocked it on macOS.

## Closed as obsolete under current support policy

Tiler now develops on macOS only; Linux is explicitly unsupported. A Linux
measurement cannot establish a supported product row while that policy holds.
`reconcile-cache-filesystem-claims-with-macos-support-policy` owns removal of
the stale supported-Linux claim while preserving this derivation as inactive
future evidence. Reopen a bounded Linux measurement only after Linux support is
admitted.
