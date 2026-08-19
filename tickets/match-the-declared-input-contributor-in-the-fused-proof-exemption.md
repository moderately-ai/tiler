---
id: match-the-declared-input-contributor-in-the-fused-proof-exemption
title: Match the declared-input contributor explicitly in the fused proof exemption
status: done
priority: p2
dependencies: [replace-the-serial-sum-contributor-fields-with-the-exhaustive-source]
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

A sibling lane holds `implementation/compiler`; this ticket is filed, not claimed. The dependency edge originally pointed at the decision ticket; on 2026-08-18, after Tom accepted carrier (4), the coordinator retargeted it onto the filed implementation carrier [`replace-the-serial-sum-contributor-fields-with-the-exhaustive-source`](replace-the-serial-sum-contributor-fields-with-the-exhaustive-source.md), since the perturbation needs a `Materialized` source to exist. Under the accepted carrier this site is one of the 31 compile-forced errors, so the repair rides that migration — but this ticket's statement, perturbation obligation, and reviewer remain its own.

## Outcome — 2026-08-19

Landed with the carrier migration on `tkt/replace-the-serial-sum-contributor-fields-with-the-exhaustive-source`, under this ticket's own statement rather than as a migration side effect.

**Facts re-audited at base `441f3215`.** All three verified. `serial.prologue.is_none()` sat at `crates/tiler-compiler/src/pipeline/verify.rs:324` under `output.try_serial_sum().is_none_or(|serial| serial.prologue.is_none())`; the arm comment `The condition is the prologue, not the family` and the module's `fails closed instead of being carried into a compilation product` were both present; `ProgramAlternativeKind::of` still classifies `Fused` only for a one-region whole-program cover, so the reachability Fact — forged-portfolio rather than live wrong compile — holds. The carrier-decides-forcing Fact reproduced exactly: the re-derived 31-error lib census names this site as its one `pipeline/verify.rs` error.

**The repair.** The guard is now `request.normalized().outputs().iter().all(merges_nothing)`, where `merges_nothing` is an exhaustive match over the output vocabulary *and* over the fold's contributor source within it. `SerialSumContributor::DeclaredInput` answers `true`; `PointwisePrologue` and `Materialized` answer `false`; a chain and a staged family answer `false` by the per-family rule the ticket asked for rather than through `is_none_or`'s vacuous truth; pointwise and contraction answer `true` because each publishes from one region computing one recognized family. No fallback: an alternative the exemption no longer covers fails under `portfolio-equivalence`.

**Perturbation, with the failure text.** `a_produced_folds_fused_receipt_takes_the_ordinary_proof_path` in `crates/tiler-compiler/src/pipeline/tests.rs` compiles `sum(input, [cols])`, takes the genuine `Fused` alternative carrying no numerical proof, and moves *only* the recognized fold's contributor source to `Materialized` over that same fold's own recognized shape — through a narrow `#[cfg(test)] VerifiedTargetRequest::perturb_serial_sum_contributor` seam, because no genuine produced-sum plan classifies `Fused` and the arm is otherwise unreachable with a materialized source. The perturbed request refuses:

```
program.structure.portfolio-equivalence: rejected
```

Both controls the ticket names are asserted beside it: `sum(x)` keeps its exemption and verifies `Ok(())`, and a fold perturbed to carry a pointwise prologue falls through to the proving arm with the same refusal. Reversing the repair — making `merges_nothing` answer `true` for `Materialized`, which is the retired absence form — reddens the check with `a produced fold is not exempt from the numerical replay: ()`.

Severity is unchanged: no genuine produced-sum plan is `Fused`, so this remains the verifier's forged-receipt independence contract rather than a live wrong compile. Evidence that such an alternative is constructible would still raise it.
