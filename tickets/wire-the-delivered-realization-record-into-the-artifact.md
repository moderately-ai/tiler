---
id: wire-the-delivered-realization-record-into-the-artifact
title: Wire the delivered-realization record into the artifact
status: done
priority: p1
dependencies: [accept-the-delivered-realization-artifact-surface, construct-and-bind-the-first-authoritative-metal-compile-profile]
related: [record-delivered-numerical-realization, redesign-the-delivered-realization-record-from-typed-evidence]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, implementation/frontend, contracts/numerics, contracts/artifacts, contracts/decisions, implementation/runtime, research/cache, research/target-profiles, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, numerics]
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

**Stage three — the ratified record, its codec, and its refusal evidence.** The stale four-dimension draft is removed and the packet's record landed in its place as accepted public surface, with the shape item 1 names. `tiler-artifact` now names the shared `tiler_ir::numerics` vocabulary by re-export, so the artifact's four-case dimension set — the last remaining copy stage one recorded — is gone and the workspace has one dimension authority. The 38-perturbation table runs against the production types, each watched failing on its own rule identifier, counted against both `ALL_RULES` inventories and against a stated population of 38 perturbations over 25 distinct rules. The coverage check was itself watched failing: removing the `entry-rebound` perturbation makes the suite report `these rules were never watched refusing: ["entry-rebound"]`. **No identity moved and no wire format changed**: `ARTIFACT_DOMAIN` is still `tiler.artifact-program.v14`, `MANIFEST_SCHEMA` is still `(12, 0)`, and every pin in the set below is untouched.

Stage three stopped there deliberately. The wiring items are one unit whose first byte of wire change forces the manifest-schema step, and a wire change landed without that step is exactly the framing desync `docs/artifact-abi.md` refuses. So the boundary taken is the last one *before* the unit begins rather than a boundary inside it.

## The remainder, in landing order

The remaining wiring is one unit rather than four: the record only becomes *required* when the builder, the model, the codec, the fixtures, and the identity step all move, and the contract text below describes that landed state rather than the current one. **Every item is now done**; the section is kept as written because each item's derivation is what the landed shape rests on, and the stage-four record below states what each turned into.

1. **Replace `crates/tiler-artifact/src/program/realization.rs`.** Remove the stale four-dimension, dtype-free, opaque-means draft entirely — including its convention-7 file allow at `:1-4` — and land the packet's record: canonical sorted subject slice, dense dimension-indexed resolution and disposition arrays through the one exhaustive `NumericalDimension::index`, sparse obligation rows sorted by `(subject, dimension, locus)`, deduplicated referenced evidence, dispositions derived at `build()` rather than declared. Public per `accept-the-delivered-realization-artifact-surface` (`done`), so no `dead_code` allow survives.
2. **The codec.** Decode rejecting shuffled, duplicated, missing, malformed, mismatched, and dangling rows with typed causes; unknown family, subject-kind, dimension, disposition, means, provenance, locus, phase, authority, validity, and behaviour tags failing closed.
3. **Builder and model wiring.** Required on construction and on decode, cross-checked against the artifact's single `TargetProfileRef`, with `VerifiedArtifactProgram` and `DecodedArtifact` returning the record directly and no absent-record state surviving a successful decode. Fold `canonical_bytes` into `encode_identity`.

   Four facts this stage established that the wiring rests on, each read at source rather than inferred:

   - **There is no artifact-wide `TargetProfileRef` accessor and no global entry ordinal.** The single profile is an *invariant* `check_subject` enforces (`builder.rs:1253-1255`, `TargetProfileMismatch`) and is stored only per variant (`VariantData.profile`, `model.rs:807`; read through `VariantRef::target_profile`, `model.rs:1284`, and `DecodedVariant::target_profile`, `view.rs:501`). Entries are per variant throughout: `EntryRef { variant, entry }` (`model.rs:1375`) and `DecodedEntry { variant, entry }` (`view.rs:655`), neither publishing its ordinal. The flat `enumerate()` in `validate.rs:527-541` is diagnostic-only and is not an ordinal space.
   - **So the record's entry ordinal has to be *defined* by the wiring, and the precedent is `DeferredPredicateData.entry`.** It is stated in declared order, range-checked per variant (`builder.rs:935-940`), and remapped into canonical stage-key order by `canonical_entry_positions` at projection (`codec/model.rs:532`, `:960-968`). A flat packaged-entry ordinal over (routing rank, canonical entry) is definable the same way and is what the ratified `EntryPolicyBinding` needs; the record itself is deliberately agnostic and `validate_against_artifact` reads `entries` in its caller's order.
   - **`ArtifactProgramData` must gain the record**, because `assemble` (`builder.rs:803`) currently drops `PortfolioSubject`'s `numerical` and `profile` on the floor and keeps only the interface. `ArtifactEnvelope` (`codec/model.rs:475`) must gain it too, and `encode_manifest` (`codec/encode.rs:229`) is the only manifest writer — any field added there forces `MANIFEST_SCHEMA` to `(13, 0)`.
   - **A record carrying strings needs a budget row.** `check_text_budgets` (`codec/budget.rs:153-203`) bounds every `&str` the manifest writes by `MAX_TEXT_BYTES`; a provenance identity, implementation, version, build, or environment field that skips it can encode and then fail to decode, breaking the symmetry that module promises.
4. **`tiler_build::realization::translate`.** Exhaustive over every subject, dimension, disposition, structured means, and provenance variant; never reconstructing evidence from flags, target names, neighbouring dtypes, profile digests, or outer value shape. Dispositions are not translated — carrying them beside the obligations would be the same claim twice.

   **There is exactly one artifact-construction site in `tiler-build`**, and it already holds every argument the translation needs: `assemble_plan_artifact` (`plan_artifact.rs:152`) takes `plan: PlanAlternative<'_>`, derives `profile` at `:165`, and reaches the packaged program at `:179`; the builder is created at `:168` and built at `:227`. Every other caller — three in `metal_plan.rs`, four across `tests/custom_backend`, two in `examples/identity_join_producer.rs` — routes through it, so none of them changes. `PlanAlternative::delivered_realization` (`session.rs:956`) has no production caller today.
5. **Every `tiler-artifact` fixture and the `program` module doctest**, the latter through the accepted typed producer path. The population was surveyed on this tree and is larger than "every fixture": **8 helper constructors** (`program/tests.rs:1761`, `:1866`, `:1917`, `:4284`, plus the draft-shaping `:3774`; `codec/tests.rs:181`, `:213`, `:1822`) and **19 inline complete-artifact builds** (17 in `program/tests.rs`, 2 in `codec/tests.rs`, plus the inline closure at `codec/tests.rs:311`). `proof/tests.rs` builds no artifact of its own and is affected only transitively through `build_artifact`/`default_artifact`. Outside the crate: `tiler-runtime`'s `tests/adapter_route/fixture.rs:594` (which *can* name the vocabulary — `tiler-ir` is a dev-dependency there) and `prototypes/serial-sum-run/src/proof.rs:6014`.

   **Four spike harnesses also build artifacts and the gate will not catch them**, because each is a nested workspace `make full` never reaches: `spikes/cache/build-tool-exercise/envelope/src/lib.rs:118`, `spikes/cache/envelope-digest-coverage/harness/src/envelope.rs:210`, `spikes/cache/hot-path-efficiency/harness/src/envelope.rs:205`, and `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs:289`. A required record breaks all four at `build()`. They are retained evidence whose only drift detector is re-running them by hand, so the wiring stage updates them in the same commit and re-runs each from its own directory; a green workspace gate is not evidence about any of them.
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

**Stage four — the wiring and the identity step, landed as one commit.** Items 3, 4, 5, 7, and 8 are done, in one commit because the first encoded byte forces the schema and there is no boundary inside them.

- **The record is required.** `ArtifactProgramBuilder::declare_realization` is the producer seam; `build` refuses a draft without one and reports `ArtifactDiagnostic::MissingDeliveredRealization`. `ArtifactProgramData.realization` and `ArtifactEnvelope.realization` are non-`Option` fields; `VerifiedArtifactProgram::delivered_realization` and `DecodedArtifact::delivered_realization` are total readers.
- **The entry ordinal space is defined, on `DeferredPredicateData.entry`'s precedent.** A producer states a flat **declared** ordinal over (variant declaration rank, declared entry ordinal); `build` remaps it once through `packaged_entry_positions`, which is built on the codec's own `canonical_entry_positions` rather than a second definition of the stage-key order, into the flat **canonical** ordinal over (routing rank, canonical entry). The remap is `DeliveredRealizationRecord::remap_entries`, which re-sorts because a remap does not preserve the canonical `(entry, subject)` order. `ArtifactDiagnostic::DeliveredRealizationEntryOutOfRange` refuses a binding naming no packaged entry.
- **The cross-check runs on the envelope, from both sides.** `ArtifactEnvelope::check_realization` compares the record's profile against **every** variant's — which is stronger than comparing it against a portfolio-wide copy the decoder does not re-prove — and then calls the ratified `validate_against_artifact` over the flat canonical entry sequence. `build` calls it after projecting; `codec::validate` calls it last, after the entry table's own structural obligations, because running it first reported a forged *extra* entry as an unbound one.
- **`overlapping_behaviour` gained a subject.** It took `NumericalRealization`, which a decoder cannot hold: a decoded entry's contract key arrived as bytes and `NumericalFacts` owns it. `EntryRealization` names the eight behaviours both sides do hold, with `EntryRealization::of` projecting the shared-IR record by exhaustive field-named destructuring and `NumericalFacts::entry_realization` projecting the dispatch record the same way, so widening either is a build error at both.
- **The codec carries it as one framed run**, written after the variant table, decoded by the record's own codec and reported as `ArtifactCodecError::DeliveredRealization` — the record failing on its own terms, kept distinct from `ModelObligation`, which is its disagreement with the artifact around it. `check_text_budgets` gained the record's profile key and every provenance text run its evidence rows write, through one exhaustive `push_provenance_text` match over `FactEvidenceBasis`.
- **`tiler_build::realization::translate`** is the one transcription, called from the single artifact-construction site `assemble_plan_artifact`. It forwards `DimensionBehaviour`, `HonouringMeans` with its relaxation payload, `NumericalObligationKey`, and `FactSourceProvenance` by value rather than matching over them — the stronger form of exhaustive, since widening any of them is a build error at their own total encoders. It refuses a profile the compiler evidence does not name, an obligation naming a subject outside the view, a view with no policy subject, and a view with several and nothing deciding which governs an entry.

**The fixture population, and the four spikes.** Every `tiler-artifact` fixture declares a record through one `realization_record` helper that *derives* the eleven resolutions from the packaged program's own scheduled realization, so a fixture whose contract changes cannot leave a record describing the old one; `declare_realization` and `declare_realization_over` are the two call shapes. `tiler-runtime`'s `tests/adapter_route/fixture.rs` builds its record entirely through `tiler_artifact::program` re-exports — no `tiler_ir` path in it — which is what proves the record is reachable from a consumer whose closure ADR 0081 item 2 fixes at `[tiler-artifact]`. `prototypes/serial-sum-run` uses `tiler_build::realization::translate`, because it holds a `PlanAlternative` and assembles by hand. All four spike harnesses were repaired in this commit and re-run by hand from their own directories:

| Spike | Verdict |
| --- | --- |
| `spikes/cache/build-tool-exercise` | re-run `--skip-analyzer --concurrency 3`; every counted cell of all five cargo scenarios identical to the 2026-08-05 row. The three analyzer scenarios were not re-run and are unverified at this base. |
| `spikes/cache/envelope-digest-coverage` | re-run and recorded under a new label; all 36 verdict rows identical, the two whole-run substitutions still the only `only-bundle-digest` members. One quantity moved: the envelope 113,303 → 118,225 bytes, so the sweep is 236,450 decodes rather than 226,606, all refused. |
| `spikes/cache/hot-path-efficiency` | repaired, and then **aborts on its own precondition**: `SIZES` is the measured 32,136–47,803 envelope band and one envelope's fixed overhead is now 114,025 bytes. Proved to predate this ticket — the record contributes 2,453 canonical bytes carried twice, ≈4.9 KB of the 114,025. Filed as [`re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps`](re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps.md). |
| `spikes/target-profiles/scalar-cpu-vertical` | re-run green, bit-for-bit agreement on all twelve elements; fixture re-recorded at its stable path. Envelope 82,918 → 87,338 and artifact identity 40,622 → 42,832, which is the record folded into the identity the manifest also carries. The `CanonicalizeF32Nan` perturbation was re-applied and the run exited 1 naming exactly one differing element. |

**Every moving pin, recomputed on this tree.**

| Pin | Was | Now |
| --- | --- | --- |
| `ARTIFACT_DOMAIN` (`crates/tiler-artifact/src/program/model.rs`) | `tiler.artifact-program.v14` | `tiler.artifact-program.v15` |
| `MANIFEST_SCHEMA` (`crates/tiler-artifact/src/program/codec/encode.rs`) | `(12, 0)` | `(13, 0)` |
| domain literal (`crates/tiler-artifact/src/program/codec/tests.rs`) | `tiler.artifact-program.v14` | `tiler.artifact-program.v15` |
| schema assertion (`crates/tiler-artifact/src/program/codec/tests.rs`) | `(12, 0)` | `(13, 0)` |
| `IDENTITY_DOMAIN` (`crates/tiler/src/route/tests.rs`) | `tiler.artifact-program.v14` | `tiler.artifact-program.v15` |
| `ARTIFACT_IDENTITY` golden (`crates/tiler-build/src/metal_plan.rs`) | `1c84ec3a…c481d` | `d22c0d11…ce832` |
| `CACHE_SUBJECT` golden (`crates/tiler-build/src/metal_plan.rs`) | `2700a51f…e4ff1` | `6dee9552…79d68` |
| identity ledger (`docs/artifact-abi.md`) | artifact v14, manifest 12.0 | artifact v15, manifest 13.0 |
| identity ledger (`docs/status.md`) | artifact program v14, manifest schema 12.0 | artifact program v15, manifest schema 13.0 |

`crates/tiler-artifact/src/proof/codec.rs`'s `MANIFEST_SCHEMA = (1, 0)` is the proof sidecar's and did **not** move; the exact check `grep -rn "tiler.artifact-program.v1[45]\|MANIFEST_SCHEMA" --include="*.rs" crates/ prototypes/` returns it unchanged. No pin outside the table above moved.

**Four scopes were added to this ticket at stage four**, each required by work the ticket already authorizes and recorded here rather than asked about: `implementation/runtime` for `crates/tiler-runtime`'s fixture and `prototypes/serial-sum-run`, both in the surveyed fixture population; `research/cache` and `research/target-profiles` for the four spike harnesses the survey names, which map to those scopes rather than to a spikes-wide one; and `contracts/navigation` for `docs/status.md`, which the pin table above already required. `tkt list --status in-progress` showed no other live ticket holding any of the four at the time they were added.

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
