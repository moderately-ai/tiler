---
id: bind-stage-coverage-to-index-refinement-identity
title: Bind kernel-program stage coverage to its refinement evidence
status: blocked
priority: p1
dependencies: [correct-adr-0071-retained-lower-layer-identity-cardinality, place-index-refinement-evidence-under-an-ir-owned-verifier, admit-a-strict-affine-index-realization-law]
related: [bind-the-scheduled-region-to-the-verified-index-region-identity]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/artifacts, implementation/runtime, implementation/frontend, contracts/decisions, research/program-planning]
shared_scopes: [project/tickets, contracts/navigation, implementation/cargo-lock]
paths: []
tags: [implementation, ir, identity]
claimed_from: todo
assignee: agent-stage-coverage
lease_expires_at: 1785804091
---
## User-visible outcome

Every executable stage names the exact checked index-refinement receipt for each semantic occurrence it covers, so neither compiler planning nor artifact replay can silently substitute unrelated evidence.

A verified executable stage must name the exact index-refinement evidence that
proves each semantic occurrence it claims to implement.

## Implementation keys

**The question, atomic:** what shape does a covered occurrence take once it must carry its refinement evidence?

Today `StageData::coverage` is `Vec<SemanticOccurrence>` (`crates/tiler-ir/src/program/model.rs:329-330`) — a bare graph-local ordinal. Binding the evidence makes each element a pair, and the exact spelling is the decision.

| Option | Enables | Prevents |
| --- | --- | --- |
| **1. `Vec<(SemanticOccurrence, IndexRefinementIdentity)>`** | Smallest change; no new public type in `tiler_ir::program`; both stage encoders keep iterating one vector. | Nothing names which element is which. The pairing is positional at every construction site, so a future caller can transpose the two values and produce a stage that claims evidence for the wrong occurrence — the precise failure this binding exists to prevent. A half-populated pair is also not expressible as a refusable state; it is simply a wrong pair. |
| **2. A named `CoveredOccurrence { occurrence, refinement }` record** | Self-describing at every use. It is the only candidate under which a half-populated value is *constructible at all*, which is what lets the builder refuse it with a typed reason rather than silently accepting a transposition. Both encoders read named fields, so the two independent stage encoders cannot drift on field order. | Costs one public type on the `tiler_ir::program` surface, which is an ADR 0075 always-ask and lands twice (program and artifact). |
| **3. A map keyed by occurrence (`BTreeMap<SemanticOccurrence, IndexRefinementIdentity>`)** | Enforces one evidence per occurrence structurally, without a builder check. | Two grounds, both surviving a read of the current code — see below. |

**Option 3's original elimination was false and has been deleted.** It claimed a map "loses the declared order that the coverage encoding folds into identity". There is no declared order to lose: `crates/tiler-ir/src/program/builder.rs:891-892` is `let mut ordered = coverage.to_vec(); ordered.sort_unstable();`, and the *sorted* vector is what is stored — `model.rs:329-330` documents the field as "Covered occurrences in ascending order", and both identity encoders iterate it in that stored order (`model.rs:1108-1109`, `:1303-1304`). A `BTreeMap` over the same key type iterates in exactly that order. The caller's order is already discarded.

**Option 3 is still eliminated, on two grounds that do survive the read.** First, the ordered wire encoding is currently a property of the *type* — a `Vec` the builder sorted — and moving to a map makes it a property of an *iteration contract* that must be documented and held identically by two independent encoders, or the program and artifact identities diverge for the same stage. That is a weaker guarantee bought for no gain, since the sort already provides it. Second, and decisively for this ticket: a map insert makes `KernelProgramBuildError::DuplicateCoverage` (`builder.rs:903`, guard opening at `:899`) unreachable, replacing an explicit typed refusal with a silent last-writer-wins overwrite. A duplicate covered occurrence under this ticket's semantics means two different refinement identities claiming the same occurrence — exactly the ambiguity the binding is meant to make impossible — and option 3 resolves it by discarding one without saying so.

**Outcome: a named record with private fields and proof-derived construction.** The former transposition argument for a tuple was overstated because the two field types differ, and a public named record with two required fields is not half-populatable either. The real invariant is stronger: callers must not be able to pair an arbitrary occurrence with an unrelated refinement receipt. `place-index-refinement-evidence-under-an-ir-owned-verifier` first moves the retained dependency-neutral receipt authority below both compiler planning and program storage. This ticket then constructs `CoveredOccurrence` only from that IR-owned checked receipt and exposes borrowed readers, not a free public constructor.

**One constraint the shape must respect, whichever is chosen.** Current `OccurrenceEvidence` has only `Refined`; budget and proof gaps fail before `ResolvedLowering`. A failed refinement therefore produces no receipt and must remain unrepresentable as verified coverage rather than being encoded through any placeholder.

The implementation can now proceed as a concrete draft. Tom reviews the exact record, construction boundary, readers, error, and program/artifact identity changes before acceptance.

## Fact

The compiler already derives `IndexRefinementIdentity` from a verified index
region and its occurrence. That identity terminates in compiler planning and
explain output. Kernel-program stage coverage and the corresponding artifact
stage identity carry only semantic occurrences.

Coverage already has the correct one-stage-to-many-occurrences cardinality.
The program and artifact layers have independent stage encoders, so both must
carry the stronger meaning deliberately.

## Outcome

Represent each covered occurrence together with its refinement identity as one
inseparable coverage record. Do not use parallel vectors whose positions can
disagree.

A compiler proof gap produces no checked refinement receipt. It cannot be encoded as valid executable coverage or made to look proved.

## Public boundary

Changing stage coverage changes the public `tiler_ir::program` surface and the
artifact identity that cross-references it. Tom reviews the exact record and
builder shape before acceptance.

## Closes when

Every covered occurrence in a verified program and artifact is paired with its
exact refinement evidence; program and artifact identities distinguish stages
that differ only in that evidence; proof gaps cannot become verified coverage;
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

## Preserved stop — strict-affine coverage has no receipt authority

Static reconstruction exposed one real prerequisite rather than a fixture inconvenience. The governed strict-affine U4 dequantization operation has no `IndexRealizationLaw` row or compiler lowering registration, so `compile_governed` cannot mint coverage for the artifact fixture that preserves one builder/component test and three codec tests. A foreign or synthetic receipt would violate this ticket's exact-evidence outcome, and deleting the tests would erase the only artifact-level component ABI evidence. `admit-a-strict-affine-index-realization-law` now owns the missing consequential public/product boundary. This implementation draft is preserved but is not complete or mergeable until that dependency lands and these four tests use its real receipts.
