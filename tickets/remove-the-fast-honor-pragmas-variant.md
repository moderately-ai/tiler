---
id: remove-the-fast-honor-pragmas-variant
title: Remove the fast-honor-pragmas variant
status: in-progress
priority: p3
dependencies: []
related: [decide-whether-fpcontract-retains-the-driver-rejected-variant, record-or-validate-the-fast-honor-pragmas-selection]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal-aot, surface-removal]
claimed_from: todo
assignee: agent-remove-fhp
lease_expires_at: 1786039487
---

## The decision this executes

**Tom decided on 2026-08-06 (provenance on [`decide-whether-fpcontract-retains-the-driver-rejected-variant`](decide-whether-fpcontract-retains-the-driver-rejected-variant.md)):** `FpContract::FastHonorPragmas` is removed. The grounds are on the decision node; the removal commit cites it.

## The work

- Delete the variant and every arm matching it (`crates/tiler-metal-aot/src/input.rs`: the enum, `flag_value`, `contracts_across_statements`; any other site `rg -n 'FastHonorPragmas' crates/` finds — read each).
- Retarget the watcher: `fast_honor_pragmas_is_rejected_by_the_metal_driver` becomes a test asserting the driver's admitted `-ffp-contract` set is exactly `{off, on, fast}` — probing a fourth value (`fast-honor-pragmas` as a raw string, since the variant no longer spells it) still fails typed at the metal stage, so a future toolchain accepting it still fires loudly. Watch the retargeted test fail under a deliberate perturbation before trusting it.
- Preserve the measurement: the `Fast` variant's pragma-honouring measurement paragraph stays (it documents a live variant); the dead variant's doc moves to git history with the removal commit citing the decision node — do not paste the whole measurement into the commit message, cite the node.
- The enum is `pub` without `#[non_exhaustive]`: confirm no out-of-crate matches exist (`rg -n 'FpContract::' crates/ prototypes/` outside the crate) and report the check.

## Closes when

The variant is gone, every site compiles, the retargeted watcher was watched failing, and the removal commit cites the decision node.
