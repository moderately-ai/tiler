---
id: split-metal-profile-measurement-sources-by-compilation-selection
title: Split Metal profile measurement sources by compilation selection
status: todo
priority: p1
dependencies: [carry-required-compilation-selection-identity-on-compile-profile-contexts]
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, declare-the-bf16-rows-on-the-authoritative-metal-profile]
scopes: [implementation/build, implementation/compiler, implementation/metal-aot, research/target-profiles, contracts/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [metal, target-profiles, provenance, numerics, identity, correction]
---
## User-visible outcome

Every authoritative Metal profile row cites only the exact compilation selection that produced it. Rows measured under different selections cannot share a source merely because their toolchain and execution environment agree.

## Fact

The current `tiler-build` declaration shares one measured source across grid, cost, dispatchability, and numerical rows. The retained grid invocation selected SDK/standard/target but did not make the same explicit O2/safe/precise/contract-off selection used by the projected cost and numerical evidence. Existing authority-ledger prose treats common compiler builds and execution environment as sufficient for sharing; the accepted compilation-selection decision makes that premise false.

## Required delivery

- Re-read the complete retained records, harnesses, authority ledger, declaration builder, and every row consumer at the implementation base.
- Derive each context's selection through the accepted Metal AOT authority; never hand-copy flags into profile construction.
- Give the grid evidence its own source/context.
- Let cost and numerical evidence share a context only after asserting their complete canonical selection bytes are equal. Otherwise split them too.
- Refuse a facts/selection mismatch with a typed error. Do not silently rewrite, merge, or choose one source.
- Update `nonprojected_metal_facts_do_not_reach_the_compiler_descriptor` so a facts-only mutation remains nonprojected or is rejected as a mismatch, while changing the derived selection moves the descriptor.
- Update the authority ledger, descriptor pins, request/explain pins, standard artifact/envelope/cache pins, source populations, and documentation from exact current values.
- Perturb one selection field while leaving facts unchanged, and one facts field while leaving selection unchanged; quote both failures.

## Non-goals

This ticket does not change target feasibility, numerical answers, runtime routing, or backend fallback behavior.

## Closes when

The authoritative profile's source table is partitioned by exact producing selection, every row's evidence remains truthful, and no current declaration can launder a row across selections.
