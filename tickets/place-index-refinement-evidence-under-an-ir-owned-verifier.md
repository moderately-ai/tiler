---
id: place-index-refinement-evidence-under-an-ir-owned-verifier
title: Place index-refinement evidence under an IR-owned verifier
status: todo
priority: p1
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity]
scopes:
  - implementation/ir
  - implementation/compiler
  - contracts/foundation
  - contracts/artifacts
shared_scopes:
  - project/tickets
---

## User-visible outcome

A dependency-neutral authority can eventually mint one opaque checked receipt that binds an exact verified semantic graph, its typed graph-local occurrence, the governed numerical contract, and the verified index region that realizes it. This ticket establishes that authority; the dependent `bind-stage-coverage-to-index-refinement-identity` owns program-builder consumption, stale/wrong-domain receipt refusal, and program/artifact identity changes.

## Correctness derivation

**Fact:** `IndexRefinementIdentity` and the lowering-capability authority are currently owned by `tiler-compiler`, while `KernelProgramBuilder`, `program::SemanticOccurrence`, and `SemanticGraphIdentity` live in `tiler-ir`. The dependency direction is `tiler-compiler -> tiler-ir`; storing the compiler type in IR would create a cycle.

**Fact:** a `VerifiedIndexRegion` contains index/access structure, scalar operations, boundary types/shapes, and scalar-registry authority. It contains no governed association to a tensor-semantic operation, semantic graph, graph-local occurrence ordinal, host attributes, or resolved numerical contract.

**Fact:** the compiler lowering registry resolves an operation/signature to a provider-attributed `LoweringCapabilityAuthority`. That capability and the compiler's exact occurrence are the only current authorities relating a provider-emitted region to an operation. Interface, scalar-containment, bounds, and write-ownership checks can reject malformed emissions, but cannot distinguish two same-interface operations or two occurrences differing only in attributes or numerical contract.

**Measurement:** commit `cbbe0432170259db2ddb388082ad0b5e32b111fd` implemented the narrower structural verifier as a concrete draft. Review constructed the same-interface counterexample: changing operation, attributes, or numerical contract left every verifier predicate true, so a receipt still minted. The draft also carried only caller-supplied opaque occurrence bytes; it could not prove their correspondence to `program::SemanticOccurrence(u32)` in one `SemanticGraphIdentity`. The implementation and its identity-domain step are withdrawn by the correction commit following this record.

**Inference:** folding operation, attributes, contract, graph bytes, or an ordinal into identity does not prove their association. A public caller could supply a self-consistent lie. An opaque receipt must be minted from an authority that already governs both the checked semantic subject and the lowering realization, not from independently supplied identity components.

## Eliminated candidates

1. **IR structural verification plus copied semantic bytes — eliminated by correctness.** It accepts a same-interface region for the wrong operation, attributes, contract, graph, or occurrence ordinal.
2. **An unchecked or publicly constructible compiler-token wrapper in IR — eliminated by correctness.** Private fields cannot be minted across crates; a public byte constructor recreates the forbidden pairing.
3. **Keep the receipt compiler-owned and store it directly in `tiler-ir::program` — eliminated by dependency direction.** It introduces an `tiler-ir -> tiler-compiler` cycle.
4. **Treat provider emission or registration alone as proof — eliminated by the accepted refinement contract.** Resolution says which authority was selected; successful construction alone proves neither semantic association nor realization.

## Decision accepted by Tom — 2026-08-03

Tom approved the recommended authority move in the T3 Code orchestration
conversation: the dependency-neutral portion of lowering/refinement authority
may move into `tiler-ir`, so an IR-owned verifier can bind a checked
semantic-program occurrence and the resolved lowering authority to the emitted
region.

The approval covers the ownership split below, not the eventual concrete public
API. The exact verifier, receipt, subject, identity, and error boundary remains
a tested draft that returns to Tom under ADR 0075 before acceptance.

- **Accepted — move the minimal authority.** `tiler-ir` owns the sealed subject and receipt: exact `SemanticGraphIdentity`, typed `program::SemanticOccurrence`, operation/attributes, a dependency-neutral numerical-contract identity, resolved signature, admitted scalar/type authority, and the checked region. The compiler continues to own registry search, provider selection, frontier policy, and explain attribution, but its registered capability carries an IR-owned admitted realization authority that the verifier can consume. This preserves dependency direction and makes downstream receipt consumption non-forgeable, at the cost of a larger public IR boundary than the withdrawn draft.
- **Rejected — keep all lowering authority compiler-owned.** Executable-program coverage could not retain a proof-derived per-occurrence receipt in the current crate graph. A separate lower proof crate or a relocation of executable-program assembly would become a new architecture task and public boundary before this path could proceed.

A separate shared proof crate is not recommended: it adds a crate and another dependency seam while the governed subjects—semantic graph identity, program occurrence, scalar/index region, and program builder—already belong to `tiler-ir`.

## Required evidence after the decision

- The receipt subject is derived from a verified semantic program and typed graph-local ordinal; no independent ordinal/graph byte pairing exists.
- Same-interface wrong-operation, wrong-attribute, and wrong-contract cases refuse before receipt minting, with each check deliberately perturbed once.
- Two occurrences over reusable region content remain occurrence-distinct, and changed verified-region content moves identity.
- A proof gap mints no receipt.
- The exact public verifier/type/error boundary returns to Tom as a tested draft; this ticket does not self-accept it.
- Targeted `tiler-ir` and `tiler-compiler` nextest, doctests, Clippy, formatting, ticket lint, diff checks, and scope guard pass.

## Graph maintenance

`bind-stage-coverage-to-index-refinement-identity` already depends on this ticket and remains blocked. That dependent owns `CoveredOccurrence`, program-builder receipt-domain/staleness validation, and program/artifact identity domains. Do not claim that evidence here or change those identities before this authority is accepted. Update ADR 0071 and artifact contracts only after the exact authority move is accepted.
