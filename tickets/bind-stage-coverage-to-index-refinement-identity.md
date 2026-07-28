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

**The question, atomic:** what shape does a covered occurrence take once it must carry its refinement evidence?

Today `StageData::coverage` is `Vec<SemanticOccurrence>` — a bare graph-local ordinal. Binding the evidence makes each element a pair, and the exact spelling is the decision:

1. **`Vec<(SemanticOccurrence, IndexRefinementIdentity)>`** — smallest change, but the pairing is positional and nothing stops a caller transposing the two lists at a future call site.
2. **A named `CoveredOccurrence { occurrence, refinement }` record** — self-describing at every use, and the builder can refuse a half-populated one. Costs a public type in `tiler_ir::program`.
3. **A map keyed by occurrence** — enforces one evidence per occurrence structurally, but loses the declared order that the coverage encoding currently folds into identity.

**This is reserved rather than decided here because the ticket says so in terms:** "Changing stage coverage changes the public `tiler_ir::program` surface and the artifact identity that cross-references it. Tom reviews the exact record and builder shape before acceptance." It is `tiler_ir::program`'s public surface *and* an ADR 0075 always-ask, and the program and artifact layers have independent stage encoders, so the choice lands twice.

**Recommendation: option 2.** The evidence is the point of the change, and a positional pair invites exactly the transposition the binding exists to prevent; a map's structural guarantee is not worth losing the declared order that identity already folds.

**One constraint the shape must respect, whichever is chosen.** `OccurrenceEvidence::BudgetStopped` means no refinement proof exists. It must be unrepresentable as verified coverage — the candidate is absent or refused with that typed reason — rather than encodable as a placeholder that reads as proved.

**Ready to build once decided.** The compiler already derives `IndexRefinementIdentity` from a verified index region and its occurrence; it currently terminates in planning and explain output. What remains is threading it into both stage encoders and advancing the affected identity domains with their reason — the same shape as `bind-the-artifact-variant-abi-to-the-program-abi`, which landed today, and whose `formulas()` lesson applies: expect fixture churn where a caller stops supplying what is now derived.
