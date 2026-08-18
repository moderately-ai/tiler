---
id: match-the-declared-input-contributor-in-the-fused-proof-exemption
title: Match the declared-input contributor explicitly in the fused proof exemption
status: todo
priority: p2
dependencies: [admit-a-materialized-producer-in-a-serial-reduction-contributor]
related: [admit-a-materialized-producer-in-a-serial-reduction-contributor]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, numerics, compiler]
---
## User-visible outcome

The portfolio verifier's no-fused-proof exemption states the fact it means — the fold's contributor is a declared input, so its whole-program region merges nothing — instead of inferring it from the absence of a prologue field or the absence of a serial-sum projection. A serial sum whose contributor is a materialized producer takes the ordinary `portfolio-equivalence` proof path.

## Facts at filing — 2026-08-18, base `1957227cc710a7d7f78b8febacc2d6ccb997448e`

**Fact — the exemption is spelled as two absences.** `verify_equivalence` in `crates/tiler-compiler/src/pipeline/verify.rs` accepts a `(ProgramAlternativeKind::Fused, None)` alternative when every output satisfies `output.try_serial_sum().is_none_or(|serial| serial.prologue.is_none())` (anchor: `serial.prologue.is_none()`). Both halves are absences: a non-serial-sum output is exempt through `is_none_or(None) == true`, and a serial sum is exempt whenever its `prologue` field is `None` — which, under any materialized-producer carrier, is also the state of a produced sum. The arm's own comment says `The condition is the prologue, not the family`; after the carrier lands that spelling stops implying the fact it was written for.

**Fact — reachability today is the forged-portfolio direction, not a live wrong compile.** `ProgramAlternativeKind::of` classifies `Fused` only for a one-region whole-program cover, and no genuine produced-sum plan has one — no scheduled region computes producer, continuation, and fold. The defect is in the verifier's independence contract (`fails closed instead of being carried into a compilation product`): a forged `Fused` receipt over a produced-sum program would be exempted from the numerical-proof replay by the two absences above. Evidence that a genuine `Fused` produced-sum alternative is constructible would upgrade this to a live defect and raise the priority.

**Fact — carrier choice decides whether this site is compile-forced.** Measured on 2026-08-18 in a detached worktree at the same base (census in [`admit-a-materialized-producer-in-a-serial-reduction-contributor`](admit-a-materialized-producer-in-a-serial-reduction-contributor.md)): replacing the three contributor fields with an exhaustive contributor source makes this site one of the 31 compile errors, so the repair rides that migration under review; a boxed top-level produced-sum arm or an additive producer field leaves this site compiling with unchanged, wrong semantics. This ticket exists so the repair is a named, reviewed obligation under every carrier rather than a migration side effect under one.

## Required work

Re-state the exemption over the explicit declared-input fact — under the recommended carrier, the `DeclaredInput` arm of the serial-sum contributor source — so that a `Materialized` (or any future) contributor source falls through to the proving arm, and a non-serial-sum output is exempted only by a stated per-family rule rather than by `is_none_or`'s vacuous truth. No fallback: an alternative the exemption no longer covers must fail under `portfolio-equivalence`, not be re-derived a second way.

## Evidence and negative controls

- **Perturb the subject, not the assertion:** perturb only the contributor source of a recognized serial-sum subject to `Materialized` and require the ordinary `portfolio-equivalence` proof path to run, quoting the failure text.
- Control: `sum(x)` (declared-input contributor) keeps its exemption and compiles; the `sum(x)` two-dispatch split coverage case stays green.
- Control: a fold with a prologue still falls through to the proving arm, as the site's comment already states.

## Non-goals

The carrier itself, its recognizer, subject tag, or spelling — owned by the carrier implementation ticket. Any change to which alternatives are classified `Fused`.

## Coordination

A sibling lane holds `implementation/compiler`; this ticket is filed, not claimed. The dependency edge points at the decision ticket because the carrier implementation ticket does not exist yet; the coordinator retargets the edge to that ticket when filing it, since the perturbation needs a `Materialized` source to exist.
