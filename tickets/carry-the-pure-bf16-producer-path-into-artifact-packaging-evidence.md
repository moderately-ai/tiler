---
id: carry-the-pure-bf16-producer-path-into-artifact-packaging-evidence
title: Carry the pure-BF16 producer path into artifact packaging evidence
status: review
priority: p1
dependencies: [admit-a-bf16-index-realization-law-and-refinement-contract]
related: [carry-bf16-through-the-artifact-encoding-and-identity, conform-the-bf16-vertical-end-to-end]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, bf16, identity]
claimed_from: todo
assignee: agent-bf16-packaging
lease_expires_at: 1785988695
---
## User-visible outcome

A pure-BF16 semantic program reaches a `VerifiedArtifactProgram` through the ordinary producer path — built, encoded, decoded, and its identity re-derived — rather than only a hand-assembled envelope at the artifact layer.

## Why this is not the producing ticket's own evidence

**Fact.** `admit-a-bf16-index-realization-law-and-refinement-contract` made the composition reachable and proved it as far as its crate can reach: a pure-BF16 constant/multiply/add program obtains verified coverage for all four occurrences and builds a `VerifiedKernelProgram` over a `PointwiseBf16` scheduled region (`crates/tiler-ir/src/program/tests.rs`, `a_pure_bf16_program_covers_every_occurrence_and_builds_a_verified_kernel_program`).

**Fact.** `VerifiedArtifactProgram` lives in `crates/tiler-artifact/src/program/model.rs`, and `crates/tiler-ir/Cargo.toml` declares no workspace crate dependencies — the direction is `tiler-artifact → tiler-ir`. No `tiler-ir` test can reach the artifact layer, so the packaging half of the evidence is `implementation/artifact` work and was recorded as a boundary rather than absorbed.

## What to do

Add the BF16 analogue of the existing `f32` artifact fixture (`crates/tiler-artifact/src/program/tests.rs`, `build_artifact`/`default_artifact`, and the strict-affine variant at `strict_affine_u4_dequantize_artifact`): a pure-BF16 program packaged into a `VerifiedArtifactProgram` that encodes, decodes, and re-derives its identity.

Note that the candidate index region must be hand-built with `IndexRegionBuilder`, as the other artifact fixtures do: `IndexRealizationLaw::realize` and `FrozenSemanticRegistry::index_realization_law` are `pub(crate)` to `tiler-ir`.

## Stale prose to correct in the same change

**Fact.** The doc comment on `bf16_input_envelope` (`crates/tiler-artifact/src/program/codec/tests.rs:2465-2477`) states that "`NumericalContractIdentity` wraps `F32NumericalContractKey` alone, and the standard semantic provider registers index-realization laws for nine `f32` and quantization operations and none for the registered `bf16` family". Both clauses are now false: the identity admits a `bf16` key and twelve laws are registered. The test it justifies (`a_bf16_artifact_round_trips_and_its_carrier_enters_identity`) still passes; only its stated reason for hand-assembling the envelope is stale, and a hand-assembled fixture whose justification has expired is exactly the comment that misleads the next reader.

## Closes when

A pure-BF16 program reaches a `VerifiedArtifactProgram` through the builder, its round trip and identity re-derivation are asserted, and the stale justification is corrected to describe what the code now does.

## Outcome

**The producer wall is gone, and the four-byte identity difference the encoding rung pinned does not survive the crossing — which is the finding, not a discrepancy.** That number described one artifact with two tag bytes rewritten. A producer-path pair is two artifacts *derived* from two verified semantic graphs, so they differ in operation keys, refinement evidence, scheduled expression, canonical NaN payload, and buffer sizes; they are not even the same length, and no positional byte difference between them is defined. The property the number stood for — a BF16 program and its F32 twin are two artifacts — holds and is now asserted through the builder.

**Fact — the composition, end to end and with no forgery.** `PointwiseWidth` (`crates/tiler-artifact/src/program/tests.rs`) builds the same four-operation `result = input * 2.0 + 1.0` graph at either width from one parameterized construction, so the width is the only difference between the two artifacts rather than a property two hand-written twins would drift on. At `Bf16` it carries: a pure-BF16 `SemanticProgram` over the complete registered BF16 vocabulary (constant, multiply, add); four `CoveredOccurrence` records at canonical ordinals `[0, 1, 2, 3]`, each minted by the refinement verifier from a candidate region this crate built with `IndexRegionBuilder` — `IndexRealizationLaw::realize` is `pub(crate)` to `tiler-ir`, which is the stronger arrangement anyway since a caller that could obtain the law's own answer and hand it back would make the verifier a rubber stamp; a one-stage `VerifiedKernelProgram` over a `PointwiseBf16` region; and a one-variant `VerifiedArtifactProgram`. `checked_occurrence` gained the three BF16 families beside the four F32 ones, and `constant_region` took its bits attribute and scalar operation as parameters because the two constant families carry different attribute identities under one shared law template.

**Fact — the three new tests and what each holds.** `a_pure_bf16_program_reaches_a_verified_artifact_through_the_builder` asserts the coverage run, the single stage, the declared interface component at `(StorageScalar::Bf16, KernelType::Bf16)`, both entry bindings at `(Bf16, Bf16, 12)` — six BF16 elements are twelve bytes on each side — and that the delivered-realization record carries the BF16 scalar-arithmetic subject and *not* the F32 one. `the_bf16_artifact_and_its_f32_twin_are_two_artifacts` asserts the identities differ, that two builds at one width agree so nothing else in the fixture varies, and that the twin's read binding addresses the same six elements at 24 bytes. `a_producer_built_bf16_artifact_round_trips_and_re_derives_its_identity` (`codec/tests.rs`) asserts the decoded envelope equals the model, its identity re-derives from decoded content and equals the builder's stamped identity, re-encoding the decoded envelope reproduces the bytes exactly, the carrier is read back off the decoded interface, and the two widths' encodings are not the same length.

**Fact — the subject is a parameter because nothing else can catch it.** `validate_against_artifact` compares the record's behaviours to each bound entry's realization and never reads the subject's arithmetic type, so a BF16 artifact carrying `ScalarArithmeticSubject::f32()` builds, encodes, decodes, and states something false about which arithmetic its delivered numerics govern. `realization_record` therefore takes the subject; `declare_realization`/`declare_realization_over` pass the F32 one and are unchanged for every existing fixture.

**Measurement, at `21ed6264` + this branch, `cargo nextest run -p tiler-artifact`.** The BF16 producer-path artifact encodes to **90,806 bytes** with a **45,457-byte** canonical identity; its F32 twin encodes to **73,556 bytes** with a **36,832-byte** identity. Neither number is pinned in a test: an identity step would move both, and they carry no information the length inequality and the identity inequality do not. The pre-existing fused-serial-sum fixture still encodes to **97,060 bytes**, byte-for-byte the value `carry-bf16-through-the-artifact-encoding-and-identity` recorded — the sharpest available evidence that no encoding moved.

**Fact — no pin, golden, or version moved, and no encoder was touched.** The only changed source files are `crates/tiler-artifact/src/program/tests.rs` and `crates/tiler-artifact/src/program/codec/tests.rs`, both `implementation/artifact`, plus ticket files under the shared `project/tickets`. `ARTIFACT_DOMAIN` holds at `tiler.artifact-program.v15` and `MANIFEST_SCHEMA` at `(13, 0)`; the exact check is `grep -rn "tiler.artifact-program.v1[45]\|MANIFEST_SCHEMA" --include="*.rs" crates/ prototypes/`, whose results are unchanged from the base commit, and `crates/tiler-artifact/src/proof/codec.rs`'s `MANIFEST_SCHEMA = (1, 0)` is the proof sidecar's and did not move. No goldens in `crates/tiler-build/src/metal_plan.rs` or `crates/tiler/src/route/tests.rs` were touched, and the full workspace run is green. **There was no identity-domain step**, so that stop condition did not fire; nor did the public-boundary or outside-scope ones — no `crates/tiler-ir` edit was required and none was made.

**Fact — the stale justification is corrected rather than deleted.** `bf16_input_envelope`'s doc comment no longer claims a producer wall. It now states that the wall is gone, links the producer-built fixture and its round-trip test, and gives the reason the direct-assembly fixture is still the right tool where it is used: the unknown-tag and access-type-mismatch cases perturb one field at a time on an otherwise well-formed artifact, each a state the builder refuses to construct, and forging from the F32 fixture holds every other byte fixed so a refusal is attributable to the perturbed field rather than to the many things that legitimately differ between two separately derived programs.

### Verification

**Measurement — seven perturbations, each run against a restored tree and each observed failing.** Filter `cargo nextest run -p tiler-artifact -E 'test(pure_bf16_program_reaches) or test(f32_twin_are_two) or test(producer_built_bf16)' --no-fail-fast`, 3 tests in scope.

- (A) `PointwiseWidth::Bf16.storage_scalar()` returns `F32`: all 3 fail, refused by the shared IR at `push_value` as `StorageAccessType { expected: F32, actual: Bf16 }` — the carrier cannot be silently wrong, it is checked where the value is constructed.
- (B) `element_bytes()` at `Bf16` returns 4: all 3 fail as `AccessibleBytesDisagreement { expected: 12, actual: 24 }` — the two-versus-four-byte misread stopped at the program verifier.
- (C) `contract()` at `Bf16` returns the strict F32 contract: all 3 fail as `NumericalContractNotGoverned`, the refinement law refusing a contract stated for another width.
- (D) `numerical()` at `Bf16` declares the F32 canonical NaN: all 3 fail at `verify_pointwise_bf16` as `NumericalOrAccessRefinement`.
- (E) `subject()` at `Bf16` returns the F32 subject: **only** the record assertion fails, on its "must not carry the f32 one" arm — the exact isolation that proves the claim in its comment, that no other check catches a wrong subject.
- (F) the decoder drops the interface component's carrier: only the two BF16 round-trip tests fail, as `BindingComponentMismatch` — a decoder losing the carrier is refused by name before any equality assertion is reached, so the round trip is defended in depth. With that backstop and the identity and canonical-re-encode comparisons also disabled and the binding alignment dropped, `decoded == envelope` is the failing assertion, which is how that one was watched failing.
- (G) `f32_pointwise_artifact` returns the BF16 artifact: the length inequality fails with `left: 90806, right: 90806`, so the twin really being the other width is what that assertion depends on.

The tree was restored from backups after each and `git status` confirmed before committing.

### Graph maintenance

- **Filed:** [`correct-the-dtype-ledger-bf16-abi-cell-for-the-landed-producer-path`](correct-the-dtype-ledger-bf16-abi-cell-for-the-landed-producer-path.md) (`todo`, p2, `contracts/navigation`). `docs/dtype-support.md:136` still states "no producer can build a BF16 artifact — no BF16 index-realization law or refinement contract exists", every clause of which this ticket and its dependency falsify. Out of scope here; the two tickets that previously moved that cell are both `done`, so nothing live owned the correction.
- [`correct-the-artifact-abi-contracts-bf16-producer-wall-paragraph`](correct-the-artifact-abi-contracts-bf16-producer-wall-paragraph.md) (`todo`, `contracts/artifacts`) is now fully dischargeable: its own body noted the heading's claim was "narrowly still true only because the artifact-layer packaging evidence is separate work", and that work is this ticket. A dated fact recording the discharge was appended to it.
- No dependent was released by this landing beyond those two doc corrections. `validate-bf16-at-the-runtime-routing-boundary` needs a producer-built artifact and now has the fixture pattern for one, but its own dependency edges are unchanged and it stays where the graph puts it.
