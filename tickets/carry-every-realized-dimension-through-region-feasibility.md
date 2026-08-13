---
id: carry-every-realized-dimension-through-region-feasibility
title: Carry every realized dimension through region feasibility
status: in-progress
priority: p1
dependencies: []
related: [carry-the-elementary-numerical-dimensions-in-the-region-realization, wire-the-delivered-realization-record-into-the-artifact]
scopes: [implementation/compiler, contracts/optimizer, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, feasibility, correctness, fail-closed]
claimed_from: todo
assignee: worker-carry-every-dimension
lease_expires_at: 1786633073
---

# Carry every realized dimension through region feasibility

## User-visible outcome

A selected region carries target evidence for every numerical behaviour its verified `ResourceRequirements` states, rather than asking feasibility about only a prefix and later treating a consumed dimension as not required.

## Facts to re-audit before editing

**Fact at filing base `b48e7719ccd6e9b4919d40bde998beb122bdb9b0`.** `tiler_ir::schedule::ResourceRequirements` carries eight numerical fields: input and result subnormals, contraction, reassociation, permutation, signed zero, NaN assumptions, and infinity assumptions.

**Fact at that base.** `tiler_compiler::physical::region_proposal` constructs numerical requirements for only the first four: input and result subnormals, contraction, and reassociation. Its surrounding contract says the region's declared realization is carried forward per dimension, so the implementation and claim disagree.

**Fact at that base.** Whole-program contract resolution asks every consumable dimension before planning, but that does not replace the selected region's evidence projection. The delivered-realization producer records per-occurrence obligations from the selected plan and must not derive `NotRequired` merely because the physical proposal never asked for a field the verified region already stated.

## Required delivery

- Re-read the complete request-resolution, region-feasibility, selected-plan evidence, and delivered-realization construction paths and repair any stale Fact above before editing.
- Make the region proposal project all eight currently realized dimensions with their exact typed behaviour spaces and canonical order. Do not infer a strict value, omit a field because the current strategy does not transform it, or recover a value from the contract key.
- Prove the projection population from the owning type rather than a hand-written count that can remain green after widening.
- Add positive evidence showing all eight dimensions reach target assessment and selected delivered evidence, plus one-at-a-time subject perturbations for permutation, signed zero, NaN assumptions, and infinity assumptions with unchanged assertions and quoted failures.
- Align stale comments/contracts that claim either four or eight, without pre-implementing the two elementary dimensions owned by the dependent ticket.

## Boundaries

This heals the existing eight-dimension projection only. It does not add reciprocal transformation or approximate intrinsics, change a public vocabulary, choose target behaviour, alter a numerical contract, weaken feasibility, or provide any fallback when a profile is silent. The dependent elementary carrier re-audits this path and grows the complete population from eight to ten after this ticket is done.

## Closes when

Region feasibility and selected delivered evidence carry exactly every dimension in the current `NumericalRealization`/`ResourceRequirements` overlap; removing any one production projection fails its unchanged targeted test; target silence remains `Unknown`; and package plus repository gates are green.
