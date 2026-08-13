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

## Worker report — 2026-08-12, base `61246804`

**Fact audit.** At this base `the_standard_metal_path_publishes_its_recorded_identities` pins artifact `39e765637a7e014adac2b8a30788798758ca46584b558732c2bda41b7639ddda`, cache `7e00d9fa0ce90749e6f7d3d42e0f2aaabe5670e0359a0c20d1580a09bb967130`, `FIXED_CONTENT_BYTES = 65_313`. The ledger paragraph anchored `What those pins are today` still named `7a2bfe51…` / `8bdcde64…` / 65,294. Descriptor 2,099 remains current.

**Change.** The live pin paragraph now mirrors those three values and the v16/v7 five-byte accounting. `the_authority_ledger_mirrors_the_live_standard_metal_pins` reads that paragraph via `include_str!`.

**Perturbation.** Replacing `fixed content is 65,313 bytes` with `00,000` failed with `the live pin paragraph does not name FIXED_CONTENT_BYTES`. Restored.
