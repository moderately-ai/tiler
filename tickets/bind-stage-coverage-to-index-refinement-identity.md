---
id: bind-stage-coverage-to-index-refinement-identity
title: Bind kernel-program stage coverage to its refinement evidence
status: todo
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
