---
id: derive-the-operation-family-and-signature-delivery-graph
title: Derive the operation-family and signature delivery graph
status: done
priority: p1
dependencies: [enumerate-the-mature-tensor-operation-and-signature-taxonomy]
related: [own-operation-family-support-matrix, admit-the-registered-unary-families-at-the-compiler-request-boundary]
scopes: [contracts/navigation, research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, roadmap, ticket-graph]
---
## User-visible outcome

The operation taxonomy becomes an executable research and delivery plan rather than
a long aspirational list.

For every taxonomy family/signature partition, map the maturity rungs separately:
semantic identity, validation and shape inference, reference semantics, logical
rewrite participation, index/access lowering, minimum physical realization, backend
realization, and bounded conformance evidence. Group signatures only where one
correctness argument and one implementation really cover them; split when numerical
contracts, compound storage, effects, or backend feasibility differ.

Create at least one design/spike/audit owner for every unsupported family. Create
implementation tickets only when their prerequisite contracts and acceptance
boundaries are resolved. Give deferred families explicit activation triggers and
file them as `deferred`, not dispatchable `todo`. Connect existing tickets instead of
duplicating them, and correct the operation-family support matrix where the taxonomy
shows that a row was too broad.

## Closes when

Every taxonomy row reaches a live owner or a justified deferred node, dependencies
run from semantics and reference correctness toward lowering/backends, exact dtype
signatures are visible, and no umbrella "support all operations" ticket hides an
unbounded implementation scope.

## Outcome (2026-08-05)

[Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md) partitions the [mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s forty-seven families into **forty delivery tracks**, maps each track across the ticket's eight maturity rungs in two tables, joins those eight to the [support matrix](../docs/roadmap.md#operation-family-support-matrix)'s seven so the vocabularies stay comparable, and closes with a coverage table mapping every family to a track. Twelve tracks reuse exact owners, one is a cross-cutting dtype-axis note, and twenty-seven are newly owned.

**Twenty-nine tickets filed; twenty-six are `deferred` because no trigger has fired**, each carrying an activation trigger written as a checkable state of the corpus and a `## Trigger check log` line ending in the command that reproduces the verdict. The three that are not deferred are [`scope-the-concatenate-fusion-role-and-lowering`](scope-the-concatenate-fusion-role-and-lowering.md) (`todo`: a family at R4 whose R5 and R6 the matrix names and no ticket owned, with two live p1 decode tickets above it), [`test-the-directional-conversion-pair-generalization`](test-the-directional-conversion-pair-generalization.md) (`todo`: `RQ-OP-04`'s closure test names no workload, no target, and no measurement, and a second dtype is already registered and reference-evaluated), and [`repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row`](repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row.md) (`todo`: an out-of-scope catalog defect the required reconciliation check found, filed rather than papered over).

**Dependencies run from semantics toward lowering, and five are hard rather than decorative.** The bit-reinterpretation track depends on the sub-byte packing track, because `RQ-OP-02`'s test needs two distinct packings and the corpus has one. The spectral track depends on the complex vertical, because a real transform's result is complex. The resampling track depends on the indirect gather, because its physical route *is* a gather. The checked-overflow arity question and the bitwise/shift track both depend on the integer honourability track, because there is no consumer to write their worked programs for until its trigger fires. Every one of the five is a trigger that is another ticket's completion, so it is in `dependencies:` and the wave query answers with a shortlist rather than a reading exercise.

**All twelve `RQ-OP` questions now have a named owner**, where nine previously had none: `RQ-OP-01` and `RQ-OP-02` are new deferrals, `RQ-OP-04` is the dispatchable test, `RQ-OP-06` through `RQ-OP-12` sit inside their families' new tracks, `RQ-OP-03` is the existing predicate track, and `RQ-OP-05` closes on Q-SHAPE-008 under the existing sub-tensor selection row.

**Six support-matrix rows narrowed, two by splitting, and no rung moved.** `Fused multiply-add` split out of the pointwise algebra row (different oracle obligation, different physical precondition, different failure class, and a different coverage class in the [physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md)); `Bit reinterpretation` split out of the cast-and-convert row at **R1**, a demotion for that member because the row's R2 rests on three ADRs that do not mention it; `Reductions beyond strict sum` narrowed because it listed physical topologies as family members and mixed two blockers; `Structural and data-movement families` narrowed because *views* are physical and no family; `Effectful and stateful operations` narrowed because F-43 counter-based generation is pure; and `Sub-tensor selection`'s title narrowed because "other non-surjective coordinate maps" reaches gather.

**One number the corpus repeats was wrong by two, in both documents carrying it.** "Twenty-three of forty-seven families have no matrix row" is the taxonomy's stated main practical output and the matrix quotes it. Twenty-three is the count of families *with* a row; the join table's own no-row cell listed twenty-four; and moving F-43 out of the effectful row makes **twenty-five**. Corrected in the taxonomy (two spans and the join table), in the matrix, and in the new record, each stating the check — count the `F-nn` tokens in the join table's no-row cell.

**One stop condition fired and was converted into structure rather than escalated.** Track O-07's lowering has two surviving alternatives — a piecewise read, or two write roots partitioning one output — which decide whether [Q-SHAPE-006](../docs/open-questions.md#q-shape-006--finite-piecewise-access-maps) fires. It is not Tom's, because the elimination has not been run and running it is research; the ticket was filed at `todo` with the fork in its body and the reachable remainder continued. The second stop condition — the taxonomy contradicting an accepted ADR — was checked family by family and did not fire; three near misses are recorded in the record so a later reader does not re-derive them.

**One gap was considered and deliberately not filed.** The roadmap's sentence that "out-of-crate registration" of `OpaqueCallDeclaration` and `OpaqueCallRegistry` is absent reads as unowned work and is [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md)'s 2026-07-31 correction: opaque declaration and registration are compiler-owned and crate-private by decision. Recorded in the record's findings so the next reader does not file it either.

**Checks.** `tkt lint` clean after every ticket edit and at the end; `git diff --check` clean; `tkt guard` reported no scope escape; every local link in the new record (75), the taxonomy (116), the roadmap (202), and the research catalog (384) resolves, and all self-anchors resolve; the reconciliation check from [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md) reports 84 research rows against 84 research records with **zero** research-side discrepancies, and its one experiment-side survivor is the pre-existing defect filed above — reproduced at this branch's base `b63dd5d0`, so it is not this change's. The check's failing perturbation was watched: removing the new catalog row makes it report `MISSING rows for ['tiler.research.semantic-graph.operation-family-delivery-graph']` over 83 rows against 84 records.
