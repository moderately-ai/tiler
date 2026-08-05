---
id: wire-the-delivered-realization-record-into-the-artifact
title: Wire the delivered-realization record into the artifact
status: in-progress
priority: p1
dependencies: [accept-the-delivered-realization-artifact-surface, construct-and-bind-the-first-authoritative-metal-compile-profile]
related: [record-delivered-numerical-realization, redesign-the-delivered-realization-record-from-typed-evidence]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, implementation/frontend, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, numerics]
claimed_from: todo
assignee: agent-wire-realization
lease_expires_at: 1785964303
---
`record-delivered-numerical-realization` built the first delivered-realization record and staged it crate-private in `crates/tiler-artifact/src/program/realization.rs`. That four-dimension, dtype-free draft was later disproved and is historical evidence rather than the shape to wire. `redesign-the-delivered-realization-record-from-typed-evidence` owns its replacement. This ticket makes a produced artifact carry the replacement boundary Tom accepts, which is what ADR 0076 item 4 asks for and what a staged draft alone does not supply.

Both prerequisites are now satisfied; the sentence below said otherwise and is corrected here (2026-08-05). `accept-the-delivered-realization-artifact-surface` ratifies every public item below under ADR 0075 and reached `done`, so the public boundary this ticket lands is accepted rather than proposed. `construct-and-bind-the-first-authoritative-metal-compile-profile` supplies real Metal evidence whose quantitative, dispatchability, numerical, compiler, and environment claims are authoritative enough to package, and is `done`; do not re-derive it. Whether the evidence it produced is authoritative enough for *this* ticket's packaging step was not re-checked here and remains this ticket's to confirm. The record redesign may use checked synthetic evidence; production wiring must not turn the low-level caller-vouched F32 projection into an authenticated target claim.

## The work

- **Implement the accepted shared and producer boundaries.** Land the ratified shared scalar-arithmetic dimension/subject vocabulary, compiler evidence view, artifact record, and exhaustive `tiler-build` translation together. Do not leave sibling crates translating through copied tags or strings.
- **Make the record required and versioned.** Every executable artifact rests on
  declared honouring means. The builder and decoder must both reject an
  artifact that does not carry a validated record; decoded bytes are not a
  special optional case.
- **Verify the record's profile against the artifact's.** `check_subject` already pins one `TargetProfileRef` across every variant. The record names the profile that declared its means, and the two must be the same profile; a mismatch is a typed rejection, not a tolerated duplication. This is the check that turns the record's copy of the profile from a second statement into evidence.
- **Cross-check every existing realization statement.** Every packaged entry/variant references an existing policy subject, and each record resolution among the eight dimensions already carried by widened `NumericalFacts` equals every overlapping entry statement. Artifact construction and decode reject a missing subject, profile disagreement, dangling obligation/evidence reference, or overlapping behaviour mismatch. The compiler proves operation/policy-locus meaning; `tiler-build` proves its translation; the neutral artifact validates the encoded associations without pretending to re-run compiler consumption analysis.
- **Fold `canonical_bytes` into `encode_identity`.** Two artifacts delivering one contract by different means are not the same artifact. This changes every artifact identity in the workspace, so the pinned and golden values that view it must be recomputed on the merged tree rather than taken from either side.
- **Carry it across the codec.** Choose the canonical versioned encoding that
  fits the envelope's existing compatibility rules, with explicit budgets and
  validation. This ticket requires the behavior, not a particular section
  layout or feature-flag mechanism.
- **Reject unknown numerical families.** Unknown family, schema, subject, dimension, disposition, locus, means, or provenance tags fail closed. Compatibility never skips an unknown numerical contract and still treats the artifact as executable.
- **Update the durable authorities.** Refine ADR 0076's “each dimension's means” language to the accepted complete assessment-disposition contract, update the numerical and artifact contracts, and advance the exact artifact identity domain and manifest schema from their merged-tree authorities.
- **Expose total readers.** Both `VerifiedArtifactProgram` and
  `DecodedArtifact` return the record directly. Untrusted bytes are rejected by
  decoding; a successfully decoded artifact must not preserve an
  `UnrecordedRealization` state for every caller to rediscover.
- **Update the fixtures and the module doctest.** Every fixture in `crates/tiler-artifact` gains a record. The `program` module doctest must construct it through the accepted typed producer path; it must not teach a caller to invent opaque means bytes or provenance.

## Staged landing, and what each stage established

The ticket is being landed in gated stages because a half-executed identity step is worse than none. `project/tickets` was added to `shared_scopes` when the first stage recorded its outcome here: every claimed ticket's own file is edited by the worker holding it, and the guard does not treat that file as implicitly shared.

**Stage one — the one shared vocabulary (merged, `2ff6bd97` and `895f88b6`).** `tiler_ir::numerics` is the single authority for the eleven governed dimensions, their behaviour spaces, the scalar-arithmetic policy subject and its serialized identity, the relaxation requirement, the honouring means, the policy locus and obligation key, and the structured fact provenance. `tiler-compiler` names them by re-export; the artifact's four-case dimension set is the only remaining copy. `HonouringMeans::key` became `label` and `encode` now carries the relaxation payload. No identity moved.

**Stage two — the compiler evidence view (`dc3d81ab`).** `PlanAlternative::delivered_realization` returns one borrowed, `Copy`, constructor-free `DeliveredRealizationView` exposing the profile key and descriptor, every selected scalar-arithmetic subject with its dense eleven-dimension contract, and the locus-keyed obligations with their exact checked facts. This was genuine new plumbing: `SelectedPlan::honoured` had no production caller.

Two facts the stage established that the remainder depends on:

- **The evidence cannot be materialized on `SelectedPlan`.** An obligation is keyed by a `SemanticOccurrence`, which `crate::lowering::OccurrenceLowering::covered_occurrence` mints from a completed `IndexRefinementReceipt`. A `SelectedPlan` is assembled in `selection.rs` from cover regions, before any receipt exists. The materialization sits on `ProgramAlternative`, built in `pipeline/planning.rs::build_alternative_for_origin`, which is the first point holding both the retained plan's honoured facts and the packaged program's proof-derived coverage.
- **Today's obligations are whole-program, spelled per occurrence.** `policy::dimension_requirements` takes a contract and no occurrence. The view therefore states one obligation per honoured dimension at `PolicyLocus::Computation` of *every* occurrence the packaged program covers — over-stating which occurrences consume a dimension and never under-stating it, because a missing obligation would let the artifact builder derive that dimension's disposition as `NotRequired`. `derive-per-locus-numerical-obligations` owns narrowing it.

## The remainder, in landing order

Stage three is the rest, and it is one unit rather than four: the record only becomes *required* when the builder, the model, the codec, the fixtures, and the identity step all move, and the contract text below describes that landed state rather than the current one.

1. **Replace `crates/tiler-artifact/src/program/realization.rs`.** Remove the stale four-dimension, dtype-free, opaque-means draft entirely — including its convention-7 file allow at `:1-4` — and land the packet's record: canonical sorted subject slice, dense dimension-indexed resolution and disposition arrays through the one exhaustive `NumericalDimension::index`, sparse obligation rows sorted by `(subject, dimension, locus)`, deduplicated referenced evidence, dispositions derived at `build()` rather than declared. Public per `accept-the-delivered-realization-artifact-surface` (`done`), so no `dead_code` allow survives.
2. **The codec.** Decode rejecting shuffled, duplicated, missing, malformed, mismatched, and dangling rows with typed causes; unknown family, subject-kind, dimension, disposition, means, provenance, locus, phase, authority, validity, and behaviour tags failing closed.
3. **Builder and model wiring.** Required on construction and on decode, cross-checked against the artifact's single `TargetProfileRef`, with `VerifiedArtifactProgram` and `DecodedArtifact` returning the record directly and no `UnrecordedRealization` surviving a successful decode. Fold `canonical_bytes` into `encode_identity`.
4. **`tiler_build::realization::translate`.** Exhaustive over every subject, dimension, disposition, structured means, and provenance variant; never reconstructing evidence from flags, target names, neighbouring dtypes, profile digests, or outer value shape. Dispositions are not translated — carrying them beside the obligations would be the same claim twice.
5. **Every `tiler-artifact` fixture and the `program` module doctest**, the latter through the accepted typed producer path.
6. **The 38-perturbation table against production types**, each watched failing on its own rule identifier, counted against both `ALL_RULES` inventories. Six shapes are wire-only and cannot be reached through the typed producer path.
7. **ADR 0076 and the two contract corrections**, applied byte-faithfully from the packet's drafted text where it drafted exact text, dated-correction idiom where it amends.
8. **The identity step, executed completely in one commit.** The pin set below was re-derived on this tree rather than taken from the dispatch brief, and the brief was wrong in one place: it named `route/tests.rs:44` without a crate, and that file is `crates/tiler/src/route/tests.rs`, which maps to `implementation/frontend` — a scope this ticket does not declare. **Stage three must add `implementation/frontend` to `scopes` before touching it**, and the reason is that the artifact identity domain is pinned as a literal in the frontend crate's route tests. The exact check is `grep -rn "tiler.artifact-program.v14\|MANIFEST_SCHEMA" --include="*.rs" crates/ prototypes/`, which returns nine source sites in five files, of which these move:

   | Pin | Site |
   | --- | --- |
   | `ARTIFACT_DOMAIN` = `tiler.artifact-program.v14` | `crates/tiler-artifact/src/program/model.rs:222` |
   | `MANIFEST_SCHEMA` = `(12, 0)` | `crates/tiler-artifact/src/program/codec/encode.rs:85` |
   | domain literal | `crates/tiler-artifact/src/program/codec/tests.rs:263` |
   | schema assertion | `crates/tiler-artifact/src/program/codec/tests.rs:297` |
   | `IDENTITY_DOMAIN` literal | `crates/tiler/src/route/tests.rs:44` |
   | `ARTIFACT_IDENTITY` golden | `crates/tiler-build/src/metal_plan.rs:1153` |
   | `CACHE_SUBJECT` golden | `crates/tiler-build/src/metal_plan.rs:1155` |
   | identity ledger | `docs/artifact-abi.md:217` |
   | identity ledger | `docs/status.md:20` |

   `implementation/frontend` was added to `scopes` for that one pin. It is required by work this ticket already authorizes — closes-when 3 folds `canonical_bytes` into `encode_identity`, which moves the artifact identity domain, which this literal restates — so the declaration is scheduling metadata rather than a product-scope expansion, and it is declared now so stage three does not discover a scope escape at its guard.

   `crates/tiler-artifact/src/proof/codec.rs:65` also spells `MANIFEST_SCHEMA`, at `(1, 0)`. It is the **proof sidecar's** manifest, a different subject, and it does **not** move; it is named here so a reader sweeping for the constant does not move it by pattern. Every moving pin is recomputed on the tree the step lands into and enumerated in the report. A pin outside this set moving is a stop.

## Closes when

1. Every executable artifact carries a validated, versioned record: the builder refuses to produce one without it and the decoder refuses to accept one without it, with decoded bytes given no optional path of their own.
2. The record's profile is checked against the artifact's single `TargetProfileRef` and a mismatch is a typed rejection, so the record's copy of the profile is evidence rather than a second statement.
3. `canonical_bytes` is folded into `encode_identity`, two artifacts delivering one contract by different means have different identities, and every pinned or golden identity is **recomputed on the merged tree** rather than taken from either branch.
4. The record crosses the codec under the envelope's existing compatibility rules, with explicit budgets and validation on decode.
5. `VerifiedArtifactProgram` and `DecodedArtifact` both return the record directly — total readers, with no `UnrecordedRealization` state surviving a successful decode for callers to rediscover.
6. Every `tiler-artifact` fixture carries a record, and the `program` module doctest uses the accepted typed producer path without inventing means or provenance.
7. The convention-7 file allow at `crates/tiler-artifact/src/program/realization.rs:1-4` is removed or narrowed to whatever remains genuinely unreached, and `make full` passes.

## What this does not close

This ticket does not redesign the compiler evidence, provenance vocabulary, or artifact record; its dependencies deliver and ratify those. It wires the accepted shape, advances the exact merged-tree domains and schema, and rebaselines the resulting identities.

## User-visible outcome

An artifact consumer can determine which declared numerical realization the
artifact delivers, and two otherwise identical artifacts with different
honouring means have different canonical identities. Missing, malformed, or
profile-mismatched records fail during construction or decode.
