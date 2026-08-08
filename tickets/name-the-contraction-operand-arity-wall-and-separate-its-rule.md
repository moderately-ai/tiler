---
id: name-the-contraction-operand-arity-wall-and-separate-its-rule
title: Name the contraction operand arity wall and separate its rule
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The contraction's two-operand refusal carries its own rule code and its true reason, instead of sharing `input-arity` with the program-wide zero-input check under a stated reason the subset-reads landing falsified.

## Why this exists (audited 2026-08-06, coordinator-verified: both mismatch("input-arity") sites at request.rs:3865/:5417; position-derived ordinals at physical.rs:1136)

`normalize_contraction` refuses `input_count() != 2` claiming "a program declaring a third input has no ordinal for this strategy's two reads to occupy" — false since strictly-ascending-with-gaps landed. The actual wall: `contraction_region` and `contraction_accesses_match` derive each operand's `InputOrdinal` from its array position, and `NormalizedContraction::input_keys` means "the two operands" where every sibling's means "the program's declared inputs" — four sites, one unstated assumption. The sibling `sum-contributor-ordinal` wall shows the correct shape: own rule code, real derivation, named owner.

## The work (no behaviour change)

Own rule code `contraction-input-arity` (update `contraction_direct_path.rs:254`); the comment states the real wall and names the widening in the `sum-contributor-ordinal` shape; `NormalizedContraction::input_keys`' doc distinguishes its meaning. File the follow-on at `deferred`: `admit-a-contraction-over-a-subset-of-the-declared-inputs` (recognized read list with declaration ordinals threaded through both physical sites; trigger: a program declaring a contraction beside an independent output). Corpus correction in the same wave: `docs/research/program-planning/complete-model-ingestion-and-execution.md:211`'s "fires only for input_count() == 0" is refuted by the two-line grep — correct it and record that a prior correction asserted it.

## Closes when

The rule is separated, the wall stated with its owner, the deferred follow-on filed, and the corpus line corrected with its provenance.
