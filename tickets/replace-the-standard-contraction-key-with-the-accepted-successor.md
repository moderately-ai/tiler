---
id: replace-the-standard-contraction-key-with-the-accepted-successor
title: Replace the standard contraction key with the accepted successor
status: in-progress
priority: p1
dependencies: [accept-the-tensor-contraction-successor-public-surface, implement-the-adr-0013-plan-determinism-stability-subject]
related: [decide-the-semantic-order-contract-for-relaxed-contractions]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/artifact, contracts/numerics, contracts/decisions, contracts/navigation, contracts/optimizer, contracts/artifacts, research/numerics, research/program-planning, research/region-search, research/scheduling, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-contraction-replacement
lease_expires_at: 1787104037
---
## User-visible outcome

The standard vertical carries `tiler::tensor-contraction-f32@1` — strict cell bit-identical to the retired key's answer, reassociation-permitted cell denoting the exact ordered-tree result set — and `tiler::strict-tensor-contraction-f32@1` is completely removed from the standard semantic, reference, law, lowering, compiler-recognition, and frontend vertical.

## Why this exists

Tom accepted the complete replacement on 2026-08-18 (recorded in `decide-the-semantic-order-contract-for-relaxed-contractions`). This carrier owns the atomic migration the packet specifies: the successor registration and thirteen-field facts; the descriptor/effective-profile/witness/evaluator boundary exactly as accepted through `accept-the-tensor-contraction-successor-public-surface`; `standard-reference@7`→`@8` with the contraction capability revision 7→8; every frontend/law/lowering/fixture/test/pin moved on one tree; the successor-key negative controls and movement watches the packet enumerates; and the decision-record/documentation sweep (ADR 0087 traceability, support-matrix contraction row, every catalog naming the retired key). Blocked on the surface acceptance and on `implement-the-adr-0013-plan-determinism-stability-subject` per the accepted graph.

## Closes when

The packet's identity-cascade table is realized on one tree with every pin recomputed, the old key is absent from the standard vertical with generic historical bytes still decodable, all perturbation controls land with quoted failure text, and the documentation sweep is complete.

## Forced scopes — scheduling metadata

The accepted documentation sweep (packet downstream item 4, restated in this carrier) forces scopes beyond the dispatch set: `contracts/navigation` (the roadmap support-matrix row and the decisions README index rows for ADR 0112), `contracts/optimizer` (`docs/compiler/optimizer.md`'s recognition-routing sentence), `contracts/artifacts` (`docs/backends/metal.md`'s owning-classification fact), and the five research scopes for the dated key-rename notes in the live research records that spelled the retired key (`research/numerics`, `research/program-planning`, `research/region-search`, `research/scheduling`, `research/shapes`). The dated 2026-08-10 audit-report snapshots under `research/documentation` were deliberately not edited — they are historical records of the tree at their date — so that scope is not added.

## Implementation record — 2026-08-19, base `50327207`

Delivered on `tkt/replace-the-standard-contraction-key-with-the-accepted-successor` from exact base `5032720702ab5210389cf08c1d4d67914a029d9d`, as five staged commits: `38276968` (tiler-ir: successor key, thirteen-field facts, sole decoder, registrar guard, effective profile, ordered tree, plan witness), `22bb2237` (tiler-reference: strict cell through the sole decoder, `standard-reference@7→@8`, contraction capability revision `7→8`, topology evaluator), `a54fa32d` (tiler-compiler: recognition/policy/lowering/fusion migration, appended `StrategyDeclineCause::AlgebraicCapabilityUnsupported` at tag `0x06`), `ea217e94` (negative controls, movement watches, result-population fixtures, end-to-end witness agreement), `b75e0963` (ADR 0112 and the documentation sweep).

### Per-Fact re-audit of the packet's identity-cascade table at this base

The packet's table was verified at `075d2d44`; every row was re-derived from the current encoders at `50327207` before editing, per the packet's own instruction that implementation rederives rather than copies. Re-derived constants that MOVED since the packet's audit: `tiler.schedule.v7` (table said v6), `tiler.artifact-program.v20` (table said v18; ADR-0013 landing took v19→v20), `ArtifactSchema::GOVERNED` components `1.0/1.0/2.0/3.0` (table said `1.0/1.0/1.0/3.0`; guard-and-routing moved to 2.0 with the ADR 0013 plan-determinism cells). Constants re-verified unchanged: `tiler.semantic-graph.v3`, `tiler.semantic-definition-projection.v6`, `tiler.semantic-registry.v8`, `standard-semantics@8`, `tiler.reference-registry.v2`, `tiler.scalar-definition-projection.v2`, `tiler.scalar-registry-snapshot.v1`, `tiler.scalar-reference-registry.v1`, `standard-scalar-reference@1` (scalar capability revision 1), `tiler.ir.index-realization-law-registry.v1`, `tiler.compiler.lowering-capability-registry.v2`, `REQUEST_SCHEMA_VERSION = 2`, `tiler.compiler.request-subject.v6`, `EXPLAIN_SCHEMA_VERSION = 11`, renderer `tiler-explain-v9`, `tiler.kernel-program.v12`. None of the drifted values changes any "stays"/"moves" verdict: they are the same grammars one version later, and the replacement still moves content only. Structural verdicts re-verified at the encoders: `tiler.semantic-graph.v3` encodes each occurrence's operation key (`encode` path in `crates/tiler-ir/src/semantic/identity.rs` consumers), so a key replacement moves graph bytes; `tiler.reference-registry.v2` embeds the semantic snapshot length-framed immediately after its domain tag plus per-row operation key/signature/authority/provider/capability revision (`compute_reference_identity` in `crates/tiler-reference/src/identity.rs`); the scalar outer identity embeds the scalar snapshot the same way (`compute_scalar_reference_identity` in `crates/tiler-reference/src/oracle.rs`); `ShapeEnvIdentity` is built from bindings/provenance/constraints and cannot move under a key replacement (verified live by the pinned control below); and the law row encodes operation key + law payload + revision + provider, so the contraction row moved by exactly the seven bytes the `strict-` prefix carried while its law payload stayed byte-identical (row width 106→99).

### Realized cascade — moved pins recomputed on this tree

- Semantic registry snapshot digest (law pin test domain): `e2e2b842…` → `3b7f49b2c9dd802bfd01bcbabbebcce16a8050986708e9a6ede5a5c5f9bfd0d1`.
- Law-registry identity digest: `0b8eba7d…` → `1e771f9e787a8f4b9fccaa3f8b0085b76d17e9ceb25bcf704fc053424d2479b4`; sidecar 1,766 → 1,759 bytes; contraction row `("tiler::tensor-contraction-f32@1", 99, 3b55fa5ae89e131c545ac0b5a2261d96613bd7c5967fdc3e75a091006a63f314)` replacing `("tiler::strict-tensor-contraction-f32@1", 106, d6b5dd49…)`.
- Explain request qualifier golden: `tiler-explain-v9 request=e96618a4c50cd8a4` → `request=ba45e5043054d8d5` (schema 11 and renderer v9 unchanged).
- Program-alternative stable ids in `pipeline::tests`: `program-alternative:46e6724372a67204`/`3de1c7941b7aeced` → `375b3a2bd8575034`/`cc9b86b61e7ddf9d`.
- New outer fixed-byte pins (none existed at this base, as the packet recorded): `CanonicalReferenceRegistryIdentity` digest `77c9352c0eba4bd9d8eaec7baee8b7c716a33a82be622d5e7dabd08286afac5a`; `CanonicalScalarReferenceRegistryIdentity` digest `ebd9a727624707dc730f97455e0d81f84c3dd40fc3ab96f63416a2f68478683a` (`outer_reference_identities_pin_bytes_and_watch_their_nested_snapshots`, `crates/tiler-reference/src/tests.rs`), each beside a position-anchored embedded-snapshot watch and a live semantic-only movement perturbation.
- New graph pins: successor-key occurrence graph `1d3bd76985de28d7ca0c86eb1c6b763af2b17bcfc57c12fc615440e40f87b0bf`, retired-key twin `cdcdd0da639451f7772686e3a01bcc00336d951f229aa8f4285e845f1755cb75` (`replacing_only_the_occurrence_key_moves_the_semantic_graph_identity`), with `ShapeEnvIdentity` asserted byte-identical.
- Scalar subjects: byte-identical, held by the scalar outer pin above.
- `tiler-build`/`tiler-artifact`/`tiler-runtime` carried no fixed-byte pins on the moved subjects; the full workspace suite (3,794 tests) passes with no further pin edits.

### Strict-key consumer census and dispositions

`grep -rn "strict-tensor-contraction|strict_tensor_contraction" crates/ prototypes/ docs/ spikes/` at the base named 26 crate files and 30 documents; zero hits in `prototypes/` and `spikes/`. Dispositions:

- **Migrated (crates):** `tiler-ir` semantic (`contraction.rs`, `contraction/tests.rs`, `semantic.rs`, `registry.rs`, `standard_operations.rs`, `catalog.rs`), `index/law.rs` (constructor renamed `tensor_contraction_f32()`), `tests/index_region.rs` doc; `tiler-reference` (`contraction.rs`, `contraction/tests.rs`, `standard.rs`, `error.rs`, six integration tests); `tiler-compiler` (`policy.rs`, `request.rs`, `governed.rs`, `governed/contraction_conformance.rs`, `capability.rs`, `fusion_legality.rs`, `tests/contraction_direct_path.rs`).
- **Stays — names the strict realization, not the retired key:** `ScalarProgram::StrictTensorContraction` (schedule/kernel vocabulary; the strict left fold remains the successor's sole registered realization and its strict-cell answer) and its kernel-test label; `IndexRealizationLaw::StrictTensorContractionF32` variant (law payload unchanged; constructor and registration key migrated).
- **Stays — deliberate retirement references:** the two dated notes in `crates/tiler-ir/src/semantic/contraction.rs` naming the retired key as retired.
- **Docs — migrated live contract sentences:** `docs/numerical-semantics.md`, `docs/backends/metal.md`, `docs/compiler/optimizer.md`, `docs/roadmap.md` (including the support-matrix contraction row advance), plus ADR 0087's traceability correction and dated key-rename notes in ADR 0091, ADR 0095, and nine live research records.
- **Docs — dated records stay:** the twelve `docs/research/documentation/ticket-audit-2026-08-10/reports/**` snapshots keep their historical spellings.

### Perturbation controls, with quoted failure text

Live subject perturbations, each run against the shipped registration and restored byte-identically (`git status` clean after each):

1. Field-15 row 4 flipped to `unsupported` in `register_standard_contraction`: the standard registry itself refuses to build — `the standard registry builds: InvalidGovernedContractionDescriptor { source: ContradictoryFields { first: Reduction(AttributeFieldId(3)), second: Reduction(AttributeFieldId(4)) } }`.
2. Stability row 1 flipped to `timing-deterministic`: `the standard registry builds: InvalidGovernedContractionDescriptor { source: UnsupportedValue { field: Stability(AttributeFieldId(1)) } }`.
3. `.with_algebraic_capabilities(…with_ordered_associativity())` added to the successor registration: `the_contraction_declares_no_algebraic_capability` fails — `the successor's operand-level record must stay none(): the fold's algebraic authority is the reduction descriptor's order-freedom maxima, and an operand-chain regrouping claim would consume distributivity, which ADR 0095 declines` (message text as re-rationalized for the successor; the perturbation run showed the prior message, same assertion site).
4. Pin-probe movements (subject = the shipped encoders, perturbed by this very migration): the law pin test failed at the old snapshot digest with `left: "3b7f49b2…" right: "e2e2b842…"`, the explain golden with `request=ba45e5043054d8d5` vs `request=e96618a4c50cd8a4`, and the reference identity pins with the probed values above — each check reached its subject and said no before its pin was recomputed.

Landed refusal controls (asserted typed values; each is a subject the accepted contract requires refused): the field-8/field-9 successors — descriptor supports reassociation × strict ceiling → `StrictLeftFold`, permissive ceiling → `OrderedFullBinaryTrees`, permutation forbidden under both ceilings with the raw ceiling retained unchanged, malformed descriptor never resolves (decode fails first) — in `the_effective_profile_joins_descriptor_maximum_and_ceiling` and `the_descriptor_decoder_refuses_every_deviation_by_name` (every decoder variant reachable and named, including `UnexpectedField` on a revived retired field 9); the strict cell refused at the topology route as `ResultClass { expected: OrderedFullBinaryTrees, actual: StrictLeftFold }`; all four caller budgets refused one step short with exact `{limit, actual}`; witness graph binding refused on both construction (`SemanticGraphMismatch`) and evaluation (`SemanticSubjectMismatch`); and the F32 result-population fixture with the packet's exact bits — grouping `0x00000000` vs `0x3f800000` on `[2^24, 1, -2^24]`, lane-strided `0x40000000` outside the tree-spellable set (`NonAdjacentChildren { node: 2 }`), permutation observably different (`0x3f800000` vs `0x00000000`) and structurally unspellable.

**Artifact join, old/successor cross — delivered as a pinned composition.** `ArtifactProgramBuilder::push_variant`'s first check compares exactly `semantic_graph_identity()`; `rejects_a_variant_realizing_another_semantic_graph` (tiler-artifact) proves any graph-identity difference refuses as `SemanticSubjectMismatch`; and the new pinned control proves a key-only old/successor replacement is such a difference. A directly-executed cross with two verified kernel programs is unconstructible at this base without rebuilding the retired vertical by hand: the compiler cannot compile an old-key program (recognition refuses it — which is itself the no-fallback half of the acceptance), and semantic programs of extension keys cannot be built outside `tiler-ir`'s facades. The composition is stated in both tests' documentation.

### Commands and results

At the final commit: `cargo nextest run --workspace` — 3,794 tests run, 3,794 passed, 8 skipped; `cargo test --workspace --doc` — pass; `cargo clippy --all-targets` for the three touched crates — zero warnings; `cargo fmt --check` — clean; `make citations` — 1,167+ pinned citations and every local link resolve; `tkt lint`, `git diff --check`, `tkt guard tkt/replace-the-standard-contraction-key-with-the-accepted-successor --format json`, and `make full` — recorded in the closing commit.

### Unsupported and unchanged

Everything the packet excludes stays typed unsupported: permutation, commutativity authority, FMA, distributivity, signed-zero elimination, exceptional-value absence assumptions at the topology route (`ExceptionalAssumptionUnsupported`), live-`K` witnesses (`LiveContributorCount`), padding, coordinate-dependent trees, runtime-selected widths, seeded/empty folds, non-F32 arithmetic, historical-key fallback. The executable realization remains the strict direct fold; no split is admitted (that is `revise-contraction-split-admission-to-contiguous-only-delivery`'s delivery); no kernel-performance claim is made. `MissingRealization`, `PaddedCoverageUnsupported`, `PermutationUnsupported`, `ArrivalNotFixed`, and `PerOutputTopologyUnsupported` are reserved refusal vocabulary documented as unreachable at this base — no verified program encoding can state their subjects yet — and `ContractionF32PlanWitnessError::Witness`-side revalidation in the evaluator fires only for a witness/occurrence K disagreement that graph binding already excludes.
