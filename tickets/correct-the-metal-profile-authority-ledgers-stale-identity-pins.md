---
id: correct-the-metal-profile-authority-ledgers-stale-identity-pins
title: Correct the Metal profile authority ledger's stale identity pins
status: in-progress
priority: p1
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, declare-metal-subgroup-realization-facts-in-the-target-profile]
scopes: [research/target-profiles, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, identity, target-profiles, correction]
claimed_from: todo
assignee: worker-metal-ledger-pins
lease_expires_at: 1786585710
---
## User-visible outcome

The authoritative Metal profile ledger reports the identities and fixed encoded size the current source actually pins, so a future identity migration starts from a true baseline.

## Fact — 2026-08-11

The ledger section anchored `What those pins are today` reports artifact identity beginning `7a2b`, cache subject beginning `8bdc`, and 65,294 fixed content bytes. The current test `the_standard_metal_path_publishes_its_recorded_identities` in `crates/tiler-build/src/metal_plan.rs` pins artifact identity beginning `39e765`, cache subject beginning `7e00d9`, and 65,313 bytes. The 2,099-byte profile descriptor remains current.

## Required delivery

- Recompute every named value from the exact source tree; do not copy the shortened values above as authority.
- Read the complete ledger section and every source test it claims to mirror, then repair all stale values and any causal prose that no longer holds.
- Add or strengthen a source-to-ledger check only if it reaches the prose subject; a resolving link or grep is not semantic validation.
- Perturb one source pin and show the chosen check fail, or record explicitly why the ledger remains manually audited.

## Closes when

Every present-tense identity/size Fact in the ledger matches the exact current test population and its provenance is reproducible.
