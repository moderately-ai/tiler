---
id: place-index-refinement-evidence-under-an-ir-owned-verifier
title: Place index-refinement evidence under an IR-owned verifier
status: in-progress
priority: p1
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-index-receipt
lease_expires_at: 1785786688
---
## User-visible outcome

An executable-program builder can accept one opaque checked receipt proving that a semantic occurrence and a verified index region were refined together, without making `tiler-ir` depend on `tiler-compiler` or letting a caller pair unrelated identities.

## Correctness derivation

**Fact:** `IndexRefinementIdentity` and `OccurrenceEvidence` are currently owned by `tiler-compiler::legality`, while `KernelProgramBuilder` and the proposed `CoveredOccurrence` live in `tiler-ir::program`. The dependency direction is `tiler-compiler -> tiler-ir`; storing the compiler type in IR would create a cycle. Private fields do not solve cross-crate minting because the compiler cannot call a crate-private constructor.

**Fact:** current `OccurrenceEvidence` has only `Refined`; budget and proof gaps fail before `ResolvedLowering`. The downstream ticket's `OccurrenceEvidence::BudgetStopped` premise is stale and cannot define a valid placeholder case.

**Inference:** the evidence identity or receipt must be owned by the lowest layer that stores and verifies it. The compiler may select and carry that receipt, but cannot remain its type authority if verified IR and artifact identity retain it.

## Implementation keys

- Move only the dependency-neutral receipt subject and verifier needed to bind one semantic occurrence to one verified index-region/refinement result into `tiler-ir`; do not move compiler search, provider selection, frontier policy, or explain attribution.
- Give the receipt opaque private storage and proof-derived construction through an IR-owned verifier that sees both governed inputs. Do not expose a constructor accepting independent occurrence and refinement identity bytes.
- Preserve provider/capability attribution only if its governing types can move without reversing dependencies; otherwise keep attribution in a compiler-owned envelope whose identity includes the IR receipt. Two same-shaped records are not interchangeable.
- Make a failed or unavailable refinement produce no receipt. The downstream program builder accepts only a receipt, so a proof gap is structurally unable to become executable coverage.
- Define exact canonical identity ownership and the correspondence between any compiler envelope, IR receipt, program coverage, and artifact stage. Unknown or mismatched receipt domains reject fail-closed.

## Required evidence

Two occurrences refined through the same content produce distinct occurrence-bound receipts; the same occurrence with changed verified index-region content moves the receipt; an unrelated occurrence/receipt pairing cannot be constructed through public APIs; a forged, wrong-domain, or stale receipt cannot enter a verified program; and a compiler proof gap yields no receipt. Every new check is perturbed once and observed failing.

## Closes when

The receipt authority sits below both compiler planning and executable-program storage without a dependency cycle; the compiler consumes rather than defines the retained receipt; the exact public verifier/type/error boundary is presented to Tom for review; targeted `tiler-ir` and `tiler-compiler` nextest and Clippy pass; and `bind-stage-coverage-to-index-refinement-identity` is unblocked against the accepted receipt.

## Graph maintenance

Make `bind-stage-coverage-to-index-refinement-identity` depend on this ticket. Update ADR 0071 and artifact identity contracts only after the exact authority move is accepted; do not describe a compiler-owned identity as directly storable in `tiler-ir`.
