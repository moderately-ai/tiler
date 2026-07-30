---
id: separate-metal-launch-index-from-index-and-address-width
title: Separate Metal launch index delivery from index and address width
status: in-progress
priority: p0
dependencies: []
related: [source-or-rephase-first-metal-launch-limits, restore-replayable-apple-compatibility-evidence]
scopes: [implementation/ir, implementation/compiler, implementation/metal, implementation/build, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [metal, indexing, target-profiles, correctness]
claimed_from: todo
assignee: codex-root
lease_expires_at: 1785453661
---
## User-visible outcome

A Metal profile and emitted artifact state three different facts separately: the `uint` launch builtin delivered by Metal, the widened `ulong` arithmetic used by structured kernel IR, and the address width of the device memory model. Feasibility, identity, diagnostics, and runtime validation never treat one as proof of either neighbor.

## Facts and measurement boundary

**Fact:** the current compiler axis is named `IndexWidthBits` and documented as index/address width. Every scheduled region requires 64. The Metal emitter maps KIR `Index` to MSL `ulong`, declares `thread_position_in_grid` as MSL `uint`, and widens that value explicitly.

**Fact:** the MSL specification defines `uint` and `ulong` widths, while Apple feature tables make 64-bit integer math family-dependent and do not turn launch-index type width into device-address width. A Mac artifact family alone does not imply the Apple-family support needed for 64-bit arithmetic.

**Inference:** one exact relation over a conflated index/address axis cannot validate all three meanings. Declaring 64 because `ulong` exists can overclaim arithmetic or address capability; declaring 32 from the launch builtin can wrongly reject widened arithmetic; deriving grid extent from either is a separate launch-limit error.

**Measurement boundary:** source-language widths are normative compile-profile facts. GPU-family arithmetic support and address-model guarantees need their own exact authority and applicability. The concrete invocation extent remains owned by the launch-limit ticket.

## Implementation keys

Replace or refine the conflated capability vocabulary with strongly typed launch-delivery, arithmetic-index-width, and device-address-width properties. Update requirement derivation, canonical tags and schema versions, profile builders, explain output, structured kernel/emitter checks, artifact identity, fixtures, and the bound Metal declaration. Preserve exhaustive matches so a future width or delivery form is a build error. Tom must review any public type or profile-construction boundary before acceptance.

## Required evidence

Tests must independently mutate launch delivery, arithmetic width, and address width and show distinct descriptor/artifact identities and diagnostics. Negative tests must reject a 64-bit arithmetic requirement on a family whose authority supports only storage, reject an address-width mismatch, and reject treating `uint::MAX` as a grid extent. The existing emitted widening must remain explicit and source snapshots must name both source and destination widths.

## Closes when

The three facts have separate typed semantics and authorities, every construction and consumption site is migrated, no compatibility alias preserves the conflation, all identity fixtures are deliberately rebaselined, focused tests and `make check` pass, and Tom has reviewed consequential public changes.

## Graph maintenance

This ticket blocks `construct-and-bind-the-first-authoritative-metal-compile-profile` and is related to `source-or-rephase-first-metal-launch-limits`; neither substitutes for the other. Keep `restore-replayable-apple-compatibility-evidence` related because successful compilation is evidence about a target tuple, not proof that the three semantic properties coincide.
