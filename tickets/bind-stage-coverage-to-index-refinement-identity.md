---
id: bind-stage-coverage-to-index-refinement-identity
title: Bind kernel-program stage coverage to its refinement evidence
status: done
priority: p1
dependencies: [correct-adr-0071-retained-lower-layer-identity-cardinality, place-index-refinement-evidence-under-an-ir-owned-verifier, admit-a-strict-affine-index-realization-law, derive-a-reached-only-executable-coverage-identity]
related: [bind-the-scheduled-region-to-the-verified-index-region-identity, restore-an-executable-artifact-assembly-example, accept-the-proof-bound-stage-coverage-public-boundary]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/artifacts, implementation/runtime, implementation/frontend, contracts/decisions, research/program-planning, research/documentation, contracts/foundation, implementation/build]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [implementation, ir, identity]
---
## User-visible outcome

Every executable stage names proof-derived reached-only index-refinement evidence for each semantic occurrence it covers, so neither compiler planning nor artifact replay can silently substitute unrelated evidence or make unused authority invalidate an otherwise identical executable artifact.

A verified executable stage must name the exact index-refinement evidence that
proves each semantic occurrence it claims to implement.

## Implementation keys

**The question, atomic:** what shape does a covered occurrence take once it must carry its refinement evidence?

At the ticket base, `StageData::coverage` is `Vec<SemanticOccurrence>` (`crates/tiler-ir/src/program/model.rs:329-330`) — a bare graph-local ordinal. Binding the evidence makes each element a record, while `derive-a-reached-only-executable-coverage-identity` now owns the exact evidence identity stored in that record.

| Option | Enables | Prevents |
| --- | --- | --- |
| **1. `Vec<(SemanticOccurrence, ExecutableCoverageIdentity)>`** | Smallest change; no new public type in `tiler_ir::program`; both stage encoders keep iterating one vector. | Nothing names which element is which. The pairing is positional at every construction site, so a future caller can transpose the two values and produce a stage that claims evidence for the wrong occurrence — the precise failure this binding exists to prevent. A half-populated pair is also not expressible as a refusable state; it is simply a wrong pair. |
| **2. A named `CoveredOccurrence { occurrence, refinement }` record** | Self-describing at every use. It is the only candidate under which a half-populated value is *constructible at all*, which is what lets the builder refuse it with a typed reason rather than silently accepting a transposition. Both encoders read named fields, so the two independent stage encoders cannot drift on field order. | Costs one public type on the `tiler_ir::program` surface, which is an ADR 0075 always-ask and lands twice (program and artifact). |
| **3. A map keyed by occurrence (`BTreeMap<SemanticOccurrence, ExecutableCoverageIdentity>`)** | Enforces one evidence per occurrence structurally, without a builder check. | Two grounds, both surviving a read of the current code — see below. |

**Option 3's original elimination was false and has been deleted.** It claimed a map "loses the declared order that the coverage encoding folds into identity". There is no declared order to lose: `crates/tiler-ir/src/program/builder.rs:891-892` is `let mut ordered = coverage.to_vec(); ordered.sort_unstable();`, and the *sorted* vector is what is stored — `model.rs:329-330` documents the field as "Covered occurrences in ascending order", and both identity encoders iterate it in that stored order (`model.rs:1108-1109`, `:1303-1304`). A `BTreeMap` over the same key type iterates in exactly that order. The caller's order is already discarded.

> **Citation drift corrected 2026-08-04 by the stale-claim sweep. The argument above is unchanged and was re-read at base `c4b4bdb9`; only its line numbers had moved.** Current sites: the sort is `crates/tiler-ir/src/program/builder.rs:1078-1079`; the field and its "Covered occurrences in ascending order" doc are `crates/tiler-ir/src/program/model.rs:549-550`; the two independent stage identity encoders iterate it at `model.rs:1535-1536` and `:1734-1735`; `KernelProgramBuildError::DuplicateCoverage` is raised at `builder.rs:1090` with its guard opening at `:1087`. The `StageData::coverage` citation in the paragraph above the option table (`model.rs:329-330`) moves to `:549-550` for the same reason. Reproduce with `grep -n 'ordered = coverage\|sort_unstable\|DuplicateCoverage' crates/tiler-ir/src/program/builder.rs` and `grep -n 'Covered occurrences in ascending order\|stage.coverage' crates/tiler-ir/src/program/model.rs`. The old numbers are preserved above rather than overwritten, so a reader can see that line drift and not a changed claim is what was repaired.

**Option 3 is still eliminated, on two grounds that do survive the read.** First, the ordered wire encoding is currently a property of the *type* — a `Vec` the builder sorted — and moving to a map makes it a property of an *iteration contract* that must be documented and held identically by two independent encoders, or the program and artifact identities diverge for the same stage. That is a weaker guarantee bought for no gain, since the sort already provides it. Second, and decisively for this ticket: a map insert makes `KernelProgramBuildError::DuplicateCoverage` (`builder.rs:903`, guard opening at `:899`) unreachable, replacing an explicit typed refusal with a silent last-writer-wins overwrite. A duplicate covered occurrence under this ticket's semantics means two different refinement identities claiming the same occurrence — exactly the ambiguity the binding is meant to make impossible — and option 3 resolves it by discarding one without saying so.

**Outcome: a named record with private fields and proof-derived construction.** The former transposition argument for a tuple was overstated because the two field types differ, and a public named record with two required fields is not half-populatable either. The real invariant is stronger: callers must not be able to pair an arbitrary occurrence with unrelated refinement evidence. `place-index-refinement-evidence-under-an-ir-owned-verifier` first moved the retained dependency-neutral receipt authority below both compiler planning and program storage. The active prerequisite now derives a reached-only executable-coverage identity from that completed receipt, excluding unused registry/provider rows while retaining every selected subject needed to prevent replay. This ticket constructs `CoveredOccurrence` only through that IR-owned proof path and exposes borrowed readers, not a free public constructor.

**One constraint the shape must respect, whichever is chosen.** Current `OccurrenceEvidence` has only `Refined`; budget and proof gaps fail before `ResolvedLowering`. A failed refinement therefore produces no receipt and must remain unrepresentable as verified coverage rather than being encoded through any placeholder.

The exact-receipt implementation is preserved only as a non-mergeable draft. Further implementation is blocked until `derive-a-reached-only-executable-coverage-identity` lands its reached-only projection; Tom then reviews the exact record, construction boundary, readers, error, and program/artifact identity changes before acceptance.

## Fact

The compiler already derives `IndexRefinementIdentity` from a verified index
region and its occurrence. That identity terminates in compiler planning and
explain output. Kernel-program stage coverage and the corresponding artifact
stage identity carry only semantic occurrences.

Coverage already has the correct one-stage-to-many-occurrences cardinality.
The program and artifact layers have independent stage encoders, so both must
carry the stronger meaning deliberately.

## Outcome

Represent each covered occurrence together with its proof-derived reached-only executable-coverage identity as one
inseparable coverage record. Do not use parallel vectors whose positions can
disagree.

A compiler proof gap produces no checked refinement receipt. It cannot be encoded as valid executable coverage or made to look proved.

## Public boundary

Changing stage coverage changes the public `tiler_ir::program` surface and the
artifact identity that cross-references it. Tom reviews the exact record and
builder shape before acceptance.

## Closes when

Every covered occurrence in a verified program and artifact is paired with its
reached-only proof-derived refinement evidence; program and artifact identities distinguish stages
that differ only in that evidence; proof gaps cannot become verified coverage;
unused semantic/scalar authority does not change those identities; the preserved
strict-affine builder/component test and all three codec tests use the governed
strict-affine receipt without a forged or substituted receipt; the ordinary
compiler-to-program path retains the corresponding reached-only projection;
all affected identity domains are advanced with their reason; and `make full`
passes.

## Graph maintenance

- Advance every affected program and artifact identity domain once on the merged tree and recompute pins there.
- Preserve exhaustive independent encoders so a new coverage field is a compile error in both. **Restated 2026-08-05, because "in both" is not achievable and the first implementation quietly met neither half.** All three encoder sites originally read `CoveredOccurrence` through accessors, so a fourth field compiled silently everywhere. The IR half is now delivered: `stage_key` and `encode_identity` in `crates/tiler-ir/src/program/model.rs` destructure the record, so a new field is `error[E0027]: pattern does not mention field` at both sites, alongside `E0063` at the constructor. The artifact half **cannot** be compile-forced, and the reason is this record's own design rather than an omission: the fields are private and `tiler-artifact` is another crate, so the privacy that stops a caller assembling a record also stops a sibling crate destructuring one. Weakening it to enable the check would trade the property the ticket exists to establish for a check on that property. It is held instead by `the_artifact_stage_key_encodes_the_same_coverage_record_as_the_kernel_program` in `crates/tiler-artifact/src/program/tests.rs`, which asserts the per-record run the artifact stage key writes occurs exactly once inside the kernel-program identity — so when the IR folds a field the artifact does not, the run stops matching and the test fails. Both directions were perturbed and watched fail; the failure prints `left: 0, right: 1`. The residual gap is honest and worth stating: a field neither encoder folds is caught by the IR compile error, and a field *only the artifact* folds is caught by the same containment check, but nothing catches a widening both encoders miss because the record itself refused to change — which is the case the `E0063` constructor error covers.
- Present the exact public draft to Tom before acceptance; no additional shape choice is pending before implementation.
- Follow `place-index-refinement-evidence-under-an-ir-owned-verifier`; directly storing compiler-owned `IndexRefinementIdentity` in `tiler-ir` is a forbidden dependency inversion.

## Scope additions during implementation

The identity grammar and its source-derived ledger require `implementation/runtime` for the runtime proof adapter, `implementation/frontend` for the recorded-artifact domain pin, `contracts/decisions` for ADR 0071/0072 implementation status, `research/program-planning` for its current-domain statement, and shared `contracts/navigation` for `docs/status.md`. These declare files already required by the accepted outcome; they do not expand it. Before editing `docs/status.md`, the coordinator compared the KV worker's exact six-file branch population against base `32232577`; `docs/status.md` was absent, so the concurrent edits are file-disjoint.
The sealed receipt authority intentionally leaves no artifact-only fixture constructor: a coverage record can be derived only from an IR-owned verified receipt, and `tiler-artifact` cannot honestly mint one from bytes or an occurrence. Its tests therefore use a `tiler-compiler` **dev-dependency** to obtain real governed receipts. This adds the shared `implementation/cargo-lock` scope because Cargo records that package-local test edge. The edge is absent from `[dependencies]`, so production `tiler-artifact` remains compiler-independent; the compiler already depends on artifact in production, while Cargo permits the reverse edge only for artifact test targets rather than constructing a production dependency cycle.

## Active stop — executable identity must exclude unused authority

The preserved draft folds the exact opaque `IndexRefinementReceiptIdentity` into both stage encoders. That receipt identity contains complete semantic and scalar registry snapshots, so an unused provider revision would change kernel-program and artifact identity. Accepted ADR 0072 and the existing `an_unused_semantic_provider_revision_does_not_change_identity` test require the opposite. `derive-a-reached-only-executable-coverage-identity` is therefore the sole active prerequisite: it must mint an opaque reached-only executable identity from a completed receipt without weakening the receipt verifier or exposing caller-assembled identity fields. The exact-receipt draft must not merge, advance final pins, or be presented for acceptance before that prerequisite lands.

**Resolved 2026-08-04 — the stop above is discharged and the status moved from `blocked` to `todo`.** This section named `derive-a-reached-only-executable-coverage-identity` as "the sole active prerequisite"; that ticket is `done`, as are the other three edges — `correct-adr-0071-retained-lower-layer-identity-cardinality`, `place-index-refinement-evidence-under-an-ir-owned-verifier`, and `admit-a-strict-affine-index-realization-law`. The stale-claim sweep found this ticket `blocked` with **zero** unmet dependency edges, which is the structural shape the sweep exists to catch: `blocked` is a claim about the graph, and the graph no longer supports it. Reproduce with `for d in correct-adr-0071-retained-lower-layer-identity-cardinality place-index-refinement-evidence-under-an-ir-owned-verifier admit-a-strict-affine-index-realization-law derive-a-reached-only-executable-coverage-identity; do grep -m1 '^status:' tickets/$d.md; done`, which prints `status: done` four times. **This changes only the status, not the work.** Tom's review of the exact record, construction boundary, readers, error, and program/artifact identity changes is still owed before acceptance — that is a public boundary inside the ticket, not a graph edge, and the sections above are unamended. The stop was not verified beyond its own stated condition, so a worker claiming this ticket re-reads the ADR 0072 identity argument against the landed reached-only projection before building on it.

## Built 2026-08-05 — the exact public draft, awaiting Tom

The record is `CoveredOccurrence { graph, occurrence, refinement }` in `crates/tiler-ir/src/program/model.rs`: all three fields private, one constructor `from_receipt(&IndexRefinementReceipt)`, borrowed readers `occurrence()` and `refinement() -> &IndexRefinementExecutableCoverageIdentity`, and a crate-internal `graph()` the program builder uses and no consumer sees. The retained identity is the receipt's **reached-only** executable-coverage projection, never `IndexRefinementReceiptIdentity` — the ADR 0072 elimination the active stop recorded, held now by a check rather than by an argument.

`KernelProgramBuilder::push_stage` takes `&[CoveredOccurrence]`. `check_coverage` sorts by occurrence and refuses, in that order, a record whose retained graph is not the builder's bound subject (`KernelProgramBuildError::ForeignCoverageGraph`, new), an out-of-graph occurrence (`CoverageOutOfRange`), and a repeated occurrence (`DuplicateCoverage`). The graph check leads because it is the one that says whose proof this is: a foreign receipt can carry an in-range unclaimed occurrence and would otherwise be accepted as evidence for this program's operation of the same ordinal.

Both stage encoders write the occurrence and then the length-framed evidence: `stage_key` and `encode_identity` in `tiler-ir`, and `stage_key` in `tiler-artifact`. `OccurrenceLowering::covered_occurrence` replaced `canonical_occurrence` in the compiler, which is now the only production minting site; the superseded method is removed rather than kept beside it.

**Identity-domain step, executed completely.** `PROGRAM_DOMAIN` moves `tiler.kernel-program.v8` → `v9` and the IR's private `STAGE_KEY_DOMAIN` `v1` → `v2`; `tiler-artifact`'s `STAGE_KEY_DOMAIN` moves `v2` → `v3`. `ARTIFACT_DOMAIN` holds at `v14` and the manifest schema at 12.0, by the per-tag injectivity the canonical-coverage step recorded: `push_variant` writes each entry's stage subject with `push_slice`, so the complete stepped key arrives length-framed. Ledger documents moved in the same commit — `docs/artifact-abi.md` (current-ledger sentence, the `(v8 now)` parenthetical, the ABI-uses sentence, and a new step paragraph), `docs/status.md` (whose scheduled-region and kernel-program rows were also stale against source and are corrected), `docs/ir.md`, `docs/research/program-planning/abi-expression-ownership.md`, `docs/research/documentation/production-crate-codebase-audit.md`, ADR 0071's implementation boundary, ADR 0072's implementation-status ledger, `crates/tiler-ir/src/program/mod.rs`, `crates/tiler-artifact/src/program/mod.rs`, `crates/tiler-artifact/src/program/builder.rs`, and `crates/tiler-compiler/tests/multi_output_boundary.rs`. Two pinned identities were recomputed on this tree: the standard Metal artifact identity `886ed671…` → `1c84ec3a…` and its expansion-cache subject `f23ac9dd…` → `2700a51f…`, both in `crates/tiler-build/src/metal_plan.rs`, whose step ledger carries the reason.

**Scope corrections against the plan in "Scope additions during implementation".** `contracts/foundation` and `implementation/build` are added, both required by the already-authorized work and neither expanding it: `docs/ir.md` carried the accepted executable-coverage record's own sentence "Program and artifact stages do not consume this draft yet", which this ticket makes false, and `crates/tiler-build/src/metal_plan.rs` holds the two pinned identities the domain step forces. File-level disjointness was verified rather than assumed: every claim in `tkt claims` is expired, `tkt/raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells` (the only `in-progress` holder of `implementation/build`) has an empty `git diff --name-only main...` population, and no other branch declaring either scope touches `metal_plan.rs` or `docs/ir.md`. `research/documentation` is added for the audit finding this discharges. `implementation/cargo-lock` is dropped and no `tiler-compiler` dev-dependency is taken: `tiler-runtime`'s `the_consumer_links_no_compiler_emitter_or_build_provider` walks `Cargo.lock`, which merges normal and development edges per package, so that edge puts the compiler in the consumer's closure and fails the test. The fixtures reach the refinement verifier through `tiler_ir::index` instead, which they needed regardless: `compile_governed` refuses the provider-provenance graphs on `semantic-authority-pairing` and the dual-output graph on `output-arity`, so it could never have served three of the four fixtures. `Cargo.lock` is unmodified. The cost is **three** module walk-throughs marked `ignore` because their hidden preamble now needs receipts no documentation example can produce — `crates/tiler-ir/src/program/mod.rs:164`, `crates/tiler-artifact/src/program/mod.rs:83`, and `crates/tiler-artifact/src/proof/mod.rs:104`. An earlier draft of this note said two, counting only the artifact pair; the `tiler-ir` example is the same casualty and was never rescuable by a compiler dev-dependency in any case, since that edge is a cycle from `tiler-ir`. All three now call a helper the workspace does not have (`refined_coverage`, `proof_derived_coverage`), so they are pseudo-code rather than merely uncompiled. `restore-an-executable-artifact-assembly-example` owns all three across `implementation/ir` and `implementation/artifact`, and records the candidate resolutions — one of which is narrowing the closure walk to the root's own dev edges, a change to an accepted guard that this ticket deliberately did not make to fit its own convenience.

**Cost, so a reader is not surprised by it.** A five-occurrence stage key is 21,366 bytes, roughly 4 KB of evidence per occurrence, because the reached-only projection embeds the semantic graph identity and the verified region. Program identity is therefore linear in occurrences times graph-identity size where it was linear in occurrences. Nothing measured hits `MAX_PROGRAM_IDENTITY_BYTES` or `MAX_ARTIFACT_IDENTITY_BYTES`, and no benchmark was run; a larger graph is the case to measure before this is called cheap.

**What Tom is being asked to accept:** the `CoveredOccurrence` type and its three methods on the public `tiler_ir::program` surface, the `push_stage` signature change, the `ForeignCoverageGraph` build-error variant, and the artifact identity that cross-references the stepped stage key.

## Resolved history — strict-affine receipt authority

Static reconstruction previously found that governed strict-affine U4 dequantization lacked an `IndexRealizationLaw` row and compiler lowering registration. `admit-a-strict-affine-index-realization-law` has since landed, and the preserved artifact builder/component test plus all three codec tests now use a real verifier-minted strict-affine receipt. That historical stop is resolved; it does not unblock the active ADR 0072 identity conflict above.

## Acceptance correction — 2026-08-09

The “awaiting Tom” language in the built-draft section is historical.
[`accept-the-proof-bound-stage-coverage-public-boundary`](accept-the-proof-bound-stage-coverage-public-boundary.md)
records Tom's 2026-08-05 acceptance of `CoveredOccurrence`, its receipt-only
construction and readers, the `push_stage`/`StageRef::coverage` surface,
`ForeignCoverageGraph`, and the cross-referenced identity step. The acceptance
carried no exclusion. Later program and artifact domain steps do not reopen
that item-level boundary.
