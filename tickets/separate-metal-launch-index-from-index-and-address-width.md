---
id: separate-metal-launch-index-from-index-and-address-width
title: Separate Metal launch index delivery from index and address width
status: in-progress
priority: p0
dependencies: []
related: [source-or-rephase-first-metal-launch-limits, restore-replayable-apple-compatibility-evidence]
scopes: [implementation/ir, implementation/compiler, implementation/metal, implementation/build, contracts/foundation, contracts/artifacts, contracts/navigation, research/apple-targets, contracts/decisions, implementation/metal-aot, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [metal, indexing, target-profiles, correctness]
claimed_from: todo
assignee: codex-root
lease_expires_at: 1785453661
---
## User-visible outcome

A Metal profile and emitted artifact state three different facts separately: the selected `uint` launch builtin declaration, the widened `uint64_t` arithmetic used by structured kernel IR, and the address width of the device memory model. Feasibility, identity, diagnostics, and runtime validation never treat one as proof of either neighbor.

## Facts and measurement boundary

**Fact:** the current compiler axis is named `IndexWidthBits` and documented as index/address width. Every scheduled region requires 64. The Metal emitter maps KIR `Index` to MSL `ulong`, declares `thread_position_in_grid` as MSL `uint`, and widens that value explicitly.

**Fact:** MSL 4.0 defines `uint32_t` and `uint64_t` widths and separately permits `ushort` or `uint` declarations for `thread_position_in_grid`. Tiler's `uint` declaration is therefore a selected emission realization, not a language-fixed target delivery fact. Apple feature tables make 64-bit integer math family-dependent and do not turn launch-index type width, 64-bit storage availability, `size_t`, or an `air64` spelling into a device-address-width guarantee. A Mac artifact family alone does not imply the Apple-family support needed for 64-bit arithmetic.

**Inference:** one exact relation over a conflated index/address axis cannot validate all three meanings. Declaring 64 because `ulong` exists can overclaim arithmetic or address capability; declaring 32 from the launch builtin can wrongly reject widened arithmetic; deriving grid extent from either is a separate launch-limit error.

**Measurement boundary:** source-language widths are normative compile-profile facts. GPU-family arithmetic support and address-model guarantees need their own exact authority and applicability. The concrete invocation extent remains owned by the launch-limit ticket.

## Implementation keys

Retire `CapabilityAxis::IndexWidthBits`, reserve tag `0x04`, and delete both raw-bit public builder methods without compatibility aliases. Introduce nominal `IndexArithmeticSupport` and `DeviceAddressWidth` facts that cannot be substituted for one another. The implemented arithmetic fact means complete support for the governed unsigned-64 `KernelType::Index` operation family, not merely availability of a 64-bit type or storage slot; derive the proposal requirement from that KIR authority rather than restating `64`. The current KIR performs buffer-relative integer offsets and has no pointer-integer operation, so it consumes no device-address-width requirement. Keep the governed Metal address-width row absent and therefore `Unknown` until a real consumer and authority exist; device-memory-space availability remains its existing separate fact.

Move the selected launch declaration out of `MetalTargetFacts` into a distinct `MetalEmissionRealization` carried by the translation unit. Retain only the proven scalar `uint` realization in the implemented profile, remove the capacity-shaped `maximum_index()` API and emitted pseudo-grid precondition, and keep the explicit widening while spelling KIR's exact unsigned-64 type as normative MSL 4.0 `uint64_t`. A future `ushort` realization requires a separately verified launch-domain fit; neither form may imply grid capacity, arithmetic support, or address width.

Advance the checked target descriptor to v8, the complete declaration to v9, and the governed feasibility vocabulary key to v3 at revision 1. KIR v4, artifact v10, neutral manifest 8.0, fact-source v4, and dtype-dispatch v2 remain unchanged because their grammars do not change. Recompute every descriptor, request, payload, source, and artifact identity pin on the tree this work lands into.

## Required evidence

Tests must prove the arithmetic and address facts have independent nominal types, descriptor rows, identity effects, and diagnostics. A storage-only or missing arithmetic declaration must not satisfy `IndexArithmeticSupport::CompleteU64`; a synthetic address-consuming proposal must reject an absent or mismatched `DeviceAddressWidth`, while the current buffer-relative region remains feasible with address width absent. The Metal translation unit must carry the selected launch realization separately from target facts, and source snapshots must name both `uint` and `uint64_t`.

A dedicated negative check must prove the selected `uint` declaration cannot populate `GridAxisThreads`, arithmetic support, or address width. Remove the old `u32::MAX` launch-precondition prose: maximum coordinate value and maximum thread count are different quantities, and actual grid limits remain owned by `source-or-rephase-first-metal-launch-limits`. Every new check must be perturbed once and observed failing.

## Closes when

The selected launch realization, governed KIR arithmetic support, and optional device-address-width fact have separate typed semantics and authorities; every construction and consumption site is migrated; absent address authority remains unknown; no compatibility alias preserves the conflation; all identity fixtures are deliberately rebaselined; focused tests and `make check` pass; and Tom has approved the consequential public boundary.

## Graph maintenance

This ticket blocks `construct-and-bind-the-first-authoritative-metal-compile-profile` and is related to `source-or-rephase-first-metal-launch-limits`; neither substitutes for the other. Keep `restore-replayable-apple-compatibility-evidence` related because successful compilation is evidence about a target tuple, not proof that the three semantic properties coincide.

Update the parent authority ledger from one index row to an operation-complete unsigned-64 arithmetic row, an explicitly absent device-address-width row, and a separately selected Metal launch realization. Do not add an Apple-family arithmetic guarantee here: the parent must source it from an exact applicable authority. Keep device GPU-family routing with `declare-a-required-gpu-family-in-the-artifact`; do not duplicate live-device predicates in the compiler profile.

## Outcome

The conflated axis and both raw-bit builders are gone. `IndexArithmeticSupport` and `DeviceAddressWidth` are nominal compiler facts with independent descriptor rows, units, diagnostics, and identity effects; governed KIR derives its complete-U64 arithmetic requirement from `KernelType::Index` and leaves device-address width absent. The checked descriptor is v8, the complete declaration is v9, the feasibility vocabulary is v3 revision 1, and explain uses schema v7/renderer v5 with a distinct `bits` quantity. KIR v4, artifact v10, and neutral manifest 8.0 remain unchanged.

Metal source emission now takes and retains a separate `MetalEmissionRealization`; `MetalTargetFacts` contains no launch choice. The implemented realization selects scalar `uint`, emitted KIR index values use MSL 4.0's `uint64_t` spelling with an explicit widening, and the false `u32::MAX` launch precondition is deleted. `MetalPlanBuildPolicy` groups required emission, optimization, and numerical choices without absorbing target authority, and exact emitted source continues to bind payload and compilation identity.

The parent profile ledger, architecture, Metal contract, artifact ledger, status summary, accepted numerical ADR, GPU-family follow-up, and stale historical ticket claims were reconciled. Three independent compile-fail doctests prove the selected launch declaration cannot serve as grid capacity, arithmetic support, or device-address width. Arithmetic/address feasibility, identity, explain-unit, source-retention, widening, and migrated-callsite checks were each perturbed and observed failing before restoration.

Targeted nextest and Clippy runs passed for compiler, Metal, build, Metal AOT, and both prototypes; affected doctests passed; and `make full` passed with 1,362 workspace tests, 547 release numerical tests, rustdoc warnings denied, ticket lint, and shellcheck. Tom approved the public separation and the required policy boundary.
