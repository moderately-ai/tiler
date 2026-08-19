---
id: replace-the-serial-sum-contributor-fields-with-the-exhaustive-source
title: Replace the serial-sum contributor fields with the exhaustive source
status: todo
priority: p3
dependencies: [admit-a-materialized-producer-in-a-serial-reduction-contributor]
related: [match-the-declared-input-contributor-in-the-fused-proof-exemption, admit-a-recognized-chain-more-than-one-materialization-boundary-deep]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, identity, numerics]
---
## User-visible outcome

`sum(sum(x) * 2)`, `sum(contract(a, b) * 2)`, and `sum(rms_norm(x, w))` compile: a strict serial reduction consumes a contributor computed across one materialization boundary, with the producer's numerical identity preserved, while the declared-input and pointwise-prologue neighbours keep their exact `serial-sum-f32.v3` bytes. Subjects one edge deeper report `reduction-contributor-depth` by name.

## Authority

Tom accepted carrier (4) on 2026-08-18 — see `## Accepted decision — 2026-08-18` in [`admit-a-materialized-producer-in-a-serial-reduction-contributor`](admit-a-materialized-producer-in-a-serial-reduction-contributor.md). That ticket's re-gated packet (measurements at `7c90391d`-era base `1957227c`, independently reviewed at `90714048`) is the complete specification; this ticket lands it and must not re-litigate the carrier shape. Its Facts were audited at `1957227c`/`236753a3`; re-verify each at this ticket's actual base before editing, per the stale-Facts rule.

## Required work

- Replace `NormalizedSerialSum`'s `prologue`/`prologue_reads`/`contributor_input` trio with the exhaustive contributor source (`DeclaredInput(ordinal)` | `PointwisePrologue { expr, reads }` | `Materialized(Box<MaterializedContributor>)` where `MaterializedContributor { producer: NormalizedOutput, continuation: Option<ContributorContinuation> }`), and mirror it in `NormalizedSerialSumSubject`. The packet's 31-error lib census is the migration map; the sites it does **not** force are the named hand-work list: the member partition (continuation members are a third part, never `RecognizedSerialSumMembers::pointwise`), `output_region_role` (a continuation region must not answer whole-program), and the `check_output_cover` members extension.
- Recognition per the packet: `plan_elementwise` with `staged: None`; on `Folded` under `ReductionContributorAdmission::OneEdge`, re-plan with the staged leaf, mint the continuation, call `recognize_epilogue_producer`; under `NoEdge`, refuse `reduction-contributor-depth`; never map `Folded` through `From<ElementwiseRefusal>` after a retain is attempted. `recognize_output` passes `OneEdge`; `recognize_epilogue_producer` passes `NoEdge`.
- Subject encoding: the `Materialized` arm writes the new `serial-sum-produced-f32.v1` framed tag, the producer through `encode_output_subject` recursion, and a continuation presence byte then the epilogue arm's `BoundaryRead`-tagged read vocabulary. The `DeclaredInput` and `PointwisePrologue` arms keep writing `serial-sum-f32.v3` byte-for-byte; the `domains.rs` request-subject pin row stays `(1, 0)`.
- Spelling: continuation members → `RegionSpellingKind::Epilogue` (staged-read builder), prologue → `Pointwise`, reduction → `SerialSum`, fused affine candidate unchanged (prologue ∪ fold only); continuation ∪ fold is not a part. `contributor_tensor` stays `Intermediate` when the fold names no declared input.
- The verifier-exemption repair rides this migration: the compile-forced `serial.prologue.is_none()` site is repaired under [`match-the-declared-input-contributor-in-the-fused-proof-exemption`](match-the-declared-input-contributor-in-the-fused-proof-exemption.md)'s statement (match `DeclaredInput` explicitly; no `is_none_or` vacuity), which keeps its own perturbation obligation and reviewer.
- `reduction-contributor-depth` regression population, complete: `contraction_direct_path`'s `sum(sum(contract)*2)`, a triple nested sum `sum(sum(sum(x)*2)*2)`, and an epilogue over a produced sum `(sum(sum(x)*2))*3`; plus the optimizer-contract sentence beside the retiring `reduction-contributor-materialization`, which becomes unreachable for one-edge subjects — a test still expecting it on such a subject is the admission check.
- Repair the two doc sentences the carrier makes false: `StagedOperandAdmission`'s `declared inputs by construction` and `encode_output_subject`'s `admits a folding family as a chain's producer and nothing else`.
- Run the packet's full perturbation list (2026-08-18 section, superseding 2026-08-13 where they overlap), perturbing subjects and quoting failure text: byte-identical `folded_prologue(false)`; producer + one-`BoundaryRead::Staged` continuation on `folded_prologue(true)`; `Staged` producer with no synthesized continuation on `sum(rms_norm(x, w))`; forged-bytes subject under `serial-sum-f32.v3` and omitted producer presence fail binding (forge the bytes, not the marker-run accident); `fused_prologue_constants` stays `None` on `staged * 2 + 1`; width and depth refusals unchanged; `record_numerical_equivalence` resolves a produced sum's fold.
- Regression: existing pinned request qualifiers (`deterministic_trace_is_sealed_and_rendered_separately`, the tiler-build Metal goldens) recompute unchanged — new programs mint new content identities within existing domains; no domain steps, no pin movement.

## Non-goals

The worklist producer walk (recorded reversal path; arbitrary chain depth stays refused), lifting `StagedOperandAdmission::NoEdge` or two-edge regions, backend emission, performance selection, producer-family-specific diagnostic keys.

## Coordination

Exclusive `implementation/compiler`; queues behind whatever lane holds that scope (the contraction-key replacement at dispatch time). Not a pin-moving identity migration — it does not enter the solo migration slot, but the pinned-qualifier recompute check above is mandatory evidence at review.

## Closes when

The three admitted subjects compile through the shared carrier with the packet's perturbations shown failing on perturbed subjects, the depth population refuses by name, the declared-input neighbour's bytes are proven unmoved, and the verifier-exemption ticket's repair is landed and reviewed alongside.
