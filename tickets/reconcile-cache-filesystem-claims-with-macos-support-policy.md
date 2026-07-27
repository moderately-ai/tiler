---
id: reconcile-cache-filesystem-claims-with-macos-support-policy
title: Reconcile cache filesystem claims with the macOS support policy
status: todo
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
