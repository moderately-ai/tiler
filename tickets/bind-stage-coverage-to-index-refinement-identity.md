---
id: bind-stage-coverage-to-index-refinement-identity
title: Bind kernel-program stage coverage to its refinement evidence
status: awaiting-decision
priority: p1
dependencies: [correct-adr-0071-retained-lower-layer-identity-cardinality]
related: [bind-the-scheduled-region-to-the-verified-index-region-identity]
scopes: [implementation/ir, implementation/compiler, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, identity]
---
A verified executable stage must name the exact index-refinement evidence that
proves each semantic occurrence it claims to implement.

## Decision needed (2026-07-28)

**The question, atomic:** what shape does a covered occurrence take once it must carry its refinement evidence?

Today `StageData::coverage` is `Vec<SemanticOccurrence>` (`crates/tiler-ir/src/program/model.rs:329-330`) — a bare graph-local ordinal. Binding the evidence makes each element a pair, and the exact spelling is the decision.

| Option | Enables | Prevents |
| --- | --- | --- |
| **1. `Vec<(SemanticOccurrence, IndexRefinementIdentity)>`** | Smallest change; no new public type in `tiler_ir::program`; both stage encoders keep iterating one vector. | Nothing names which element is which. The pairing is positional at every construction site, so a future caller can transpose the two values and produce a stage that claims evidence for the wrong occurrence — the precise failure this binding exists to prevent. A half-populated pair is also not expressible as a refusable state; it is simply a wrong pair. |
| **2. A named `CoveredOccurrence { occurrence, refinement }` record** | Self-describing at every use. It is the only candidate under which a half-populated value is *constructible at all*, which is what lets the builder refuse it with a typed reason rather than silently accepting a transposition. Both encoders read named fields, so the two independent stage encoders cannot drift on field order. | Costs one public type on the `tiler_ir::program` surface, which is an ADR 0075 always-ask and lands twice (program and artifact). |
| **3. A map keyed by occurrence (`BTreeMap<SemanticOccurrence, IndexRefinementIdentity>`)** | Enforces one evidence per occurrence structurally, without a builder check. | Two grounds, both surviving a read of the current code — see below. |

**Option 3's original elimination was false and has been deleted.** It claimed a map "loses the declared order that the coverage encoding folds into identity". There is no declared order to lose: `crates/tiler-ir/src/program/builder.rs:891-892` is `let mut ordered = coverage.to_vec(); ordered.sort_unstable();`, and the *sorted* vector is what is stored — `model.rs:329-330` documents the field as "Covered occurrences in ascending order", and both identity encoders iterate it in that stored order (`model.rs:1108-1109`, `:1303-1304`). A `BTreeMap` over the same key type iterates in exactly that order. The caller's order is already discarded.

**Option 3 is still eliminated, on two grounds that do survive the read.** First, the ordered wire encoding is currently a property of the *type* — a `Vec` the builder sorted — and moving to a map makes it a property of an *iteration contract* that must be documented and held identically by two independent encoders, or the program and artifact identities diverge for the same stage. That is a weaker guarantee bought for no gain, since the sort already provides it. Second, and decisively for this ticket: a map insert makes `KernelProgramBuildError::DuplicateCoverage` (`builder.rs:903`, guard opening at `:899`) unreachable, replacing an explicit typed refusal with a silent last-writer-wins overwrite. A duplicate covered occurrence under this ticket's semantics means two different refinement identities claiming the same occurrence — exactly the ambiguity the binding is meant to make impossible — and option 3 resolves it by discarding one without saying so.

**Recommendation: option 2.** The evidence is the point of the change, and a positional pair invites the transposition the binding exists to prevent. **The counterpoint, stated because option 2's cost is real:** it puts a new type on a public surface Tom must review, and it is the only option that admits a half-populated value at all — options 1 and 3 make a pair structurally total. That is not an argument against it; it is the reason to choose it. A value that can be half-populated is a value the builder can *refuse with a typed reason*, and refusing is what this ticket requires. The alternatives do not make the error impossible, they make it unnameable.

**One constraint the shape must respect, whichever is chosen.** `OccurrenceEvidence::BudgetStopped` means no refinement proof exists. It must be unrepresentable as verified coverage — the candidate is absent or refused with that typed reason — rather than encodable as a placeholder that reads as proved. This binds all three options equally and is not a discriminator between them.

**This is reserved rather than decided here because the ticket says so in terms:** "Changing stage coverage changes the public `tiler_ir::program` surface and the artifact identity that cross-references it. Tom reviews the exact record and builder shape before acceptance." It is `tiler_ir::program`'s public surface *and* an ADR 0075 always-ask, and the program and artifact layers have independent stage encoders, so the choice lands twice.

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

`OccurrenceEvidence::BudgetStopped` means no refinement proof exists. It cannot
be encoded as valid executable coverage: the candidate must be absent or
refused with that typed reason rather than made to look proved.

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

## Parked 2026-07-27 — awaiting Tom

The question parked here was hoisted to `## Decision needed (2026-07-28)` at the top of this ticket. It is the same question and the same recommendation; what changed is that option 3's stated elimination was checked against the source, found false, and replaced with two grounds that hold. The recommendation for option 2 also gained the counterpoint it previously lacked.

**Ready to build once decided.** The compiler already derives `IndexRefinementIdentity` from a verified index region and its occurrence; it currently terminates in planning and explain output. What remains is threading it into both stage encoders and advancing the affected identity domains with their reason — the same shape as `bind-the-artifact-variant-abi-to-the-program-abi`, which landed today, and whose `formulas()` lesson applies: expect fixture churn where a caller stops supplying what is now derived.
