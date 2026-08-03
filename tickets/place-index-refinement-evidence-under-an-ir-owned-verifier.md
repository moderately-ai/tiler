---
id: place-index-refinement-evidence-under-an-ir-owned-verifier
title: Place index-refinement evidence under an IR-owned verifier
status: review
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

## Draft outcome

**Fact — the retained receipt authority is now dependency-neutral.** `tiler_ir::index::IndexRefinementVerifier` checks the complete semantic occurrence against the actual `VerifiedIndexRegion`, revalidates scalar authority, binds ordered operand aliases and result roots, and returns either an opaque `IndexRefinementReceipt` or a `PendingIndexRefinementReceipt`. The pending type carries the exact region and exposes its canonical obligations, but no receipt or receipt identity. `complete` independently evaluates every residual under `MAX_REFINEMENT_PROOF_CELLS` and mints nothing on a disproof, unsupported fragment, symbolic extent, or resource stop.

**Fact — compiler attribution remains above the receipt.** `tiler_compiler::legality::IndexRefinement` retains the IR receipt and separately retains selected provider, capability revision, admission provenance, compiler proof records, reference-oracle bindings, and explain identity. No compiler search, provider, frontier, or explain type moved into `tiler-ir`, and the crate dependency remains `tiler-compiler -> tiler-ir` only. The compiler-to-IR projection is explicit rather than a same-shaped type alias; the IR verifier rechecks it before minting.

**Fact — association and failure evidence.** Tests prove two sites over reusable content have different receipt identities; the same site over two verified but structurally different equivalent regions has a different receipt; the receipt names the exact occurrence and canonical region; an occurrence paired with a region exposing the wrong interface is refused; a retained proof gap exposes no receipt; and an IR exact-proof resource stop refuses completion. A `trybuild` fixture proves neither the receipt nor its identity can be forged from public fields or bytes.

**Identity analysis.** New dependency-neutral domains are `tiler.ir.index-refinement-receipt.v1` and `tiler.ir.index-refinement-domain-proof.v1`. The compiler envelope advances `tiler.compiler.index-refinement-occurrence.v1` to `v2` because it now folds the IR receipt identity before compiler-owned content and attribution. The reproducible sweep `rg -n 'tiler\.compiler\.index-refinement-occurrence\.v1|index-refinement-occurrence' . --glob '!target/**'` found only the new `v2` encoder; no golden, artifact schema, version pin, or cached digest names this internal compiler identity. Program and artifact identities do not move in this ticket because stage coverage does not consume the draft receipt until the dependent ticket.

**Public draft requiring Tom.** The exact new surface is `IndexRefinementVerifier::{verify, complete}`, `IndexRefinementVerificationOutcome`, `PendingIndexRefinementReceipt`, `IndexRefinementReceipt`, `IndexRefinementReceiptIdentity`, `IndexRefinementDomainProof`, `IndexRefinementVerificationError`, the occurrence/value/binding records re-exported beside them, and `IndexRefinement::receipt`. This implementation and its documentation are a tested draft, not acceptance. ADR 0071 and artifact contracts remain unchanged as the graph maintenance requires; the dependent coverage ticket remains blocked until Tom accepts or reshapes this boundary.

**Unsupported draft case.** IR completion currently accepts only its own bounded exhaustive-finite derivation. The compiler's dormant sealed `Sound` proof lane is not an accepted input to the IR verifier; that lane must not be activated until its proof authority can be checked below the compiler boundary rather than trusted through independent bytes. The production compiler authority uses the same exact-finite limit as this draft, so no currently reachable successful discharge exceeds the IR verifier's support.

**Checks.** On the unmodified draft, `cargo check -p tiler-ir -p tiler-compiler`, `cargo nextest run -p tiler-ir -p tiler-compiler` (1,264 passed, 1 skipped), `cargo test -p tiler-ir -p tiler-compiler --doc`, `cargo clippy -p tiler-ir -p tiler-compiler --all-targets -- -D warnings`, `cargo test -p tiler-ir --test index_region_ui`, `cargo fmt --all -- --check`, `tkt lint`, and `git diff --check` pass. Four deliberate perturbations proved the new guards can say no: removing the occurrence or canonical-region components from receipt identity made their identity tests fail; bypassing the pending barrier made `proof_budget_gap_mints_no_receipt` fail by minting a receipt; and removing operand/result shape checks made `unrelated_occurrence_and_region_cannot_mint_an_ir_receipt` fail by accepting the mismatched pairing. Each perturbation was restored before the passing commands above.
