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

## Graph correction — 2026-08-17

Exact-current readiness audit found that the required selection carrier and this source partition cannot land in dependency order as two compiling production revisions: the only production constructor already shares one source across rows whose selections differ, so making the carrier mandatory first would either fail compilation or temporarily misattribute evidence. `carry-required-compilation-selection-identity-on-compile-profile-contexts` now owns the atomic production partition and adds `implementation/build` plus the retained research authority to its scope. This ticket remains the dependent closure/evidence carrier: after that landing, re-audit the exact row population, ledgers, and negative evidence, then close as satisfied or repair any bounded remainder. It must not repeat the production migration independently.

## Fact

The current `tiler-build` declaration shares one measured source across grid, cost, dispatchability, and numerical rows. The retained grid invocation selected SDK/standard/target but did not make the same explicit O2/safe/precise/contract-off selection used by the projected cost and numerical evidence. Existing authority-ledger prose treats common compiler builds and execution environment as sufficient for sharing; the accepted compilation-selection decision makes that premise false.

## Required delivery

- Re-read the complete retained records, harnesses, authority ledger, declaration builder, and every row consumer at the implementation base.
- Derive each context's selection through the accepted Metal AOT authority; never hand-copy flags into profile construction.
- Give the grid evidence its own source/context.
- Let cost and numerical evidence share a context only after asserting their complete canonical selection bytes are equal. Otherwise split them too.
- Refuse a facts/selection mismatch with a typed error. Do not silently rewrite, merge, or choose one source.
- ~~Update `nonprojected_metal_facts_do_not_reach_the_compiler_descriptor` so a facts-only mutation remains nonprojected or is rejected as a mismatch, while changing the derived selection moves the descriptor.~~ **Superseded — see the correction immediately below. The obligation is retained there in the vocabulary that replaced it; do not act on the struck text.**

**Correction — 2026-08-19 (the named test was deleted by this ticket's own dependency, and its premise with it).** `nonprojected_metal_facts_do_not_reach_the_compiler_descriptor` **does not exist**: `grep -rn "nonprojected_metal_facts_do_not_reach_the_compiler_descriptor" crates/` returns 0 lines at this base. Nor does the concept — `grep -rn "nonprojected" crates/` also returns 0 lines, so a worker cannot repair the bullet by renaming the test. It was removed by `1f6ec214` ("Carry required compilation-selection identity on compile-profile contexts"), which is an ancestor of this base and is the landing of this ticket's own dependency [`carry-required-compilation-selection-identity-on-compile-profile-contexts`](carry-required-compilation-selection-identity-on-compile-profile-contexts.md) (`status: done`).

**The removal was by design, not accident, so the bullet's premise is what went stale.** The bullet assumed a facts-only mutation could leave the descriptor byte-identical — that is what "remains nonprojected" meant. Under the required compilation-selection carrier that is no longer reachable: `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md`, anchor `the previous "byte-identical descriptor" claim is superseded by that stronger control`, records that the language standard, artifact family, and deployment minimum now reach the descriptor *through* the required selection, so a coherent language move shifts both the AOT target and the descriptor, and an incoherent facts-only move refuses as `BoundMetalDeclarationError::CompilationSelectionMismatch` (`crates/tiler-build/src/metal_declaration.rs`). The strengthened control is what replaced the weaker assertion.

**What the obligation becomes, stated in the live vocabulary.** Three tests landed with the carrier and are the current subject; all three verified present in `crates/tiler-build/src/metal_declaration.rs` at this base:

- `fn a_language_standard_change_moves_selection_identity_and_the_descriptor` — the descriptor-moves half.
- `fn every_coherent_selection_field_move_reaches_the_descriptor` — the same half, generalized over the selection's fields.
- `fn every_populations_selection_mismatch_is_named_independently` — the refuse-by-population-name half, which is where a facts-only incoherence now lands.

**Not determinable from `tickets/` alone, and left for the coordinator.** Whether these three fully discharge this bullet — in particular whether they cover a facts-only mutation held against an unchanged selection, which is the exact perturbation the next bullet still demands — requires reading `crates/tiler-build/src/metal_declaration.rs` in full. This lane holds neither `implementation/build` nor `implementation/compiler` and did not read those test bodies, so no closure claim is made here. The bullet is repaired rather than deleted precisely because the delivery obligation may still be partly live; the stale *name* is what is withdrawn, not the requirement that both directions be evidenced.
- Update the authority ledger, descriptor pins, request/explain pins, standard artifact/envelope/cache pins, source populations, and documentation from exact current values.
- Perturb one selection field while leaving facts unchanged, and one facts field while leaving selection unchanged; quote both failures.

## Non-goals

This ticket does not change target feasibility, numerical answers, runtime routing, or backend fallback behavior.

## Closes when

The authoritative profile's source table is partitioned by exact producing selection, every row's evidence remains truthful, and no current declaration can launder a row across selections.
