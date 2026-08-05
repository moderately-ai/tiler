---
id: bind-stage-coverage-to-index-refinement-identity
title: Bind kernel-program stage coverage to its refinement evidence
status: in-progress
priority: p1
dependencies: [correct-adr-0071-retained-lower-layer-identity-cardinality, place-index-refinement-evidence-under-an-ir-owned-verifier, admit-a-strict-affine-index-realization-law, derive-a-reached-only-executable-coverage-identity]
related: [bind-the-scheduled-region-to-the-verified-index-region-identity]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/artifacts, implementation/runtime, implementation/frontend, contracts/decisions, research/program-planning]
shared_scopes: [project/tickets, contracts/navigation, implementation/cargo-lock]
paths: []
tags: [implementation, ir, identity]
claimed_from: todo
assignee: agent-bind-stage
lease_expires_at: 1785934519
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
- Preserve exhaustive independent encoders so a new coverage field is a compile error in both.
- Present the exact public draft to Tom before acceptance; no additional shape choice is pending before implementation.
- Follow `place-index-refinement-evidence-under-an-ir-owned-verifier`; directly storing compiler-owned `IndexRefinementIdentity` in `tiler-ir` is a forbidden dependency inversion.

## Scope additions during implementation

The identity grammar and its source-derived ledger require `implementation/runtime` for the runtime proof adapter, `implementation/frontend` for the recorded-artifact domain pin, `contracts/decisions` for ADR 0071/0072 implementation status, `research/program-planning` for its current-domain statement, and shared `contracts/navigation` for `docs/status.md`. These declare files already required by the accepted outcome; they do not expand it. Before editing `docs/status.md`, the coordinator compared the KV worker's exact six-file branch population against base `32232577`; `docs/status.md` was absent, so the concurrent edits are file-disjoint.
The sealed receipt authority intentionally leaves no artifact-only fixture constructor: a coverage record can be derived only from an IR-owned verified receipt, and `tiler-artifact` cannot honestly mint one from bytes or an occurrence. Its tests therefore use a `tiler-compiler` **dev-dependency** to obtain real governed receipts. This adds the shared `implementation/cargo-lock` scope because Cargo records that package-local test edge. The edge is absent from `[dependencies]`, so production `tiler-artifact` remains compiler-independent; the compiler already depends on artifact in production, while Cargo permits the reverse edge only for artifact test targets rather than constructing a production dependency cycle.

## Active stop — executable identity must exclude unused authority

The preserved draft folds the exact opaque `IndexRefinementReceiptIdentity` into both stage encoders. That receipt identity contains complete semantic and scalar registry snapshots, so an unused provider revision would change kernel-program and artifact identity. Accepted ADR 0072 and the existing `an_unused_semantic_provider_revision_does_not_change_identity` test require the opposite. `derive-a-reached-only-executable-coverage-identity` is therefore the sole active prerequisite: it must mint an opaque reached-only executable identity from a completed receipt without weakening the receipt verifier or exposing caller-assembled identity fields. The exact-receipt draft must not merge, advance final pins, or be presented for acceptance before that prerequisite lands.

**Resolved 2026-08-04 — the stop above is discharged and the status moved from `blocked` to `todo`.** This section named `derive-a-reached-only-executable-coverage-identity` as "the sole active prerequisite"; that ticket is `done`, as are the other three edges — `correct-adr-0071-retained-lower-layer-identity-cardinality`, `place-index-refinement-evidence-under-an-ir-owned-verifier`, and `admit-a-strict-affine-index-realization-law`. The stale-claim sweep found this ticket `blocked` with **zero** unmet dependency edges, which is the structural shape the sweep exists to catch: `blocked` is a claim about the graph, and the graph no longer supports it. Reproduce with `for d in correct-adr-0071-retained-lower-layer-identity-cardinality place-index-refinement-evidence-under-an-ir-owned-verifier admit-a-strict-affine-index-realization-law derive-a-reached-only-executable-coverage-identity; do grep -m1 '^status:' tickets/$d.md; done`, which prints `status: done` four times. **This changes only the status, not the work.** Tom's review of the exact record, construction boundary, readers, error, and program/artifact identity changes is still owed before acceptance — that is a public boundary inside the ticket, not a graph edge, and the sections above are unamended. The stop was not verified beyond its own stated condition, so a worker claiming this ticket re-reads the ADR 0072 identity argument against the landed reached-only projection before building on it.

## Resolved history — strict-affine receipt authority

Static reconstruction previously found that governed strict-affine U4 dequantization lacked an `IndexRealizationLaw` row and compiler lowering registration. `admit-a-strict-affine-index-realization-law` has since landed, and the preserved artifact builder/component test plus all three codec tests now use a real verifier-minted strict-affine receipt. That historical stop is resolved; it does not unblock the active ADR 0072 identity conflict above.
