---
id: refresh-the-stale-identity-ledger-in-status
title: Refresh the stale identity ledger in the status portal
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## User-visible outcome

`docs/status.md`'s identity-and-schema bullet reports the same artifact, manifest, and target-requirement versions the source and the artifact contract report, so a reader orienting from the status portal is not told a superseded ledger.

## Why this slice exists

**Fact — the portal disagrees with both the source and its owning contract.** `docs/status.md`'s "Fact — identity and schema" bullet states "artifact program v11, and neutral manifest schema 9.0" and "the target-requirement component schema is 2.0". `crates/tiler-artifact/src/program/model.rs:168` declares `ARTIFACT_DOMAIN = b"tiler.artifact-program.v12\0"` and `crates/tiler-artifact/src/program/codec/encode.rs:65` declares `MANIFEST_SCHEMA: (u16, u16) = (10, 0)`. `docs/artifact-abi.md:166` — the stated owner of the complete ledger — reports v12, 10.0, and target-requirement component schema 3.0.

The v12 and 10.0 step is the live-device route-requirement change `docs/artifact-abi.md:164` records, so the portal appears to predate that landing rather than to disagree about a subject.

Found while defining backend/device vocabulary under `define-backend-device-and-execution-context-vocabulary`, which read the artifact contract in full. It was left as a ticket rather than absorbed because an identity ledger is a different subject from vocabulary and `contracts/navigation` is a shared scope where an unrelated edit invites a collision.

## Implementation keys

- Take the values from the source constants and from `docs/artifact-abi.md`, which owns the ledger; do not restate a third set.
- Check the remaining versions in the same bullet against the source rather than only the three named above, since one stale bullet is weak evidence that its siblings are current.
- `docs/artifact-abi.md:166` says these are separate subjects and must not be collapsed into one global version; preserve that.

## Closes when

Every version in the status portal's identity bullet agrees with the source constant or the artifact contract that owns it, and the exact check used is recorded.

## Outcome (2026-07-31)

**Fact.** The bullet's three stale values moved to what the source and the owning contract state: artifact program v12 (`crates/tiler-artifact/src/program/model.rs:168`, `ARTIFACT_DOMAIN = b"tiler.artifact-program.v12\0"`), neutral manifest schema 10.0 (`crates/tiler-artifact/src/program/codec/encode.rs:65`, `MANIFEST_SCHEMA: (u16, u16) = (10, 0)`), and target-requirement component schema 3.0 (`docs/artifact-abi.md:164`, the v12 step's own record, which states the component moved because its governed vocabulary changed).

**Fact — the siblings were checked, not assumed, per the ticket's second key.** Every other value in the bullet was verified against a source constant or the ledger at `docs/artifact-abi.md:166`: `tiler.resolved-value-type.v3`, `tiler.schedule.v2`, `tiler.kernel.v4`, `tiler.kernel-program.v6` (all grepped as exact strings in `crates/tiler-ir/src`), descriptor encoding v9 and complete declaration `tiler.target-profile.declaration.v10` (`crates/tiler-compiler/src/target.rs:158,163`), feasibility vocabulary v4 revision 1, and the 1.0 rows. None of them disagreed; the three named above were the whole drift, and it is exactly the v11-to-v12 route-requirement step the portal predated.

**Fact.** The separate-subjects sentence and the artifact-abi ownership pointer were preserved verbatim, as the ticket requires.
