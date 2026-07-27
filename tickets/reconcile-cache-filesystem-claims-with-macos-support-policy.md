---
id: reconcile-cache-filesystem-claims-with-macos-support-policy
title: Reconcile cache filesystem claims with the macOS support policy
status: done
priority: p2
dependencies: []
related: [define-supported-expansion-cache-filesystems, probe-the-expansion-cache-filesystem-properties-on-linux]
scopes: [research/cache, contracts/artifacts]
shared_scopes: [contracts/navigation]
paths: []
tags: [cache, portability, contract, correction]
---
The product contract must not call Linux filesystems supported while the
repository support policy says every non-macOS platform is unsupported.

## Outcome

Limit the active supported cache-filesystem table to supported macOS host and
filesystem profiles. Preserve the POSIX and Linux derivation as future
research evidence, explicitly inactive until Linux support is admitted. Correct
every cache record or artifact-contract sentence that still describes
Debian-family Linux as a supported product row or refers to the deleted host
gate.

## Closes when

The host-support policy, artifact/cache contract, research disposition, and
open-work list agree; no unmeasured Linux row is labeled supported; and the
exact trigger for reopening Linux measurement is the admission of Linux as a
supported development or product platform.

## Outcome (2026-07-27)

**The product contract now states the supported set as local APFS and local exFAT on macOS, both measured, and nothing else.** `docs/artifact-abi.md` previously presented ext4, btrfs, and xfs on Debian-family Linux in the same sentence as the measured macOS rows, qualified only as "unmeasured". That was two overstatements in one place: it named a platform the support policy does not admit, on evidence never gathered.

**The derivation is preserved as inactive research, not deleted.** `docs/research/cache/supported-filesystems.md`'s Linux row now reads **inactive — not a supported row; derivation retained as research**, with the reason stated beside it and the reopening trigger named exactly as this ticket required: *the admission of Linux as a supported development or product platform*, with measurement still required before the row becomes active even then. The POSIX and man-page facts are sound and are worth keeping; they are simply not a support claim.

**Every stale host-policy and deleted-gate reference is corrected.** Four sites said `AGENTS.md` supports "macOS and Debian-family Linux", which it no longer does — it says macOS only, a *narrowing*. Two of them also cited a Rust sub-gate accepting "macOS arm64 and GNU Linux x86-64 profiles"; `e197176` deleted that script with the rest of the Python tooling, so no check enforces a host profile at all and the policy is held by review. Corrected in `docs/artifact-abi.md`, `docs/research/cache/crash-and-race-protocol.md` (two passages), `docs/research/extensions/proc-macro-extension-visibility.md`, and `tickets/define-supported-expansion-cache-filesystems.md`.

**The open-work list now agrees.** `probe-the-expansion-cache-filesystem-properties-on-linux` was already `closed`, but its parent still listed it as split-out remaining work. It is now struck with the reason: measuring those filesystems would qualify rows on a platform the policy does not admit, so the probe revives with Linux support rather than before it.

**Note on direction.** Every correction here narrows a claim. Windows was already dissolved rather than deferred, and it is further from support than when that was written — the conclusion is unaffected by the policy change, which is why the Windows reasoning is kept rather than rewritten.
