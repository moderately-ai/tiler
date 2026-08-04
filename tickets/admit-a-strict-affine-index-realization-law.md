---
id: admit-a-strict-affine-index-realization-law
title: Admit a strict-affine index-realization law
status: in-progress
priority: p1
dependencies: [place-index-refinement-evidence-under-an-ir-owned-verifier]
related: [bind-stage-coverage-to-index-refinement-identity, prototype-quantized-value-vertical]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, correctness]
claimed_from: todo
assignee: agent-strict-affine-law
lease_expires_at: 1785851573
---

## User-visible outcome

The already-governed strict-affine U4 dequantization operation can obtain an
IR-owned checked index-refinement receipt. Until that law exists, the compiler
and artifact layers must continue to refuse strict-affine executable coverage
rather than attach unrelated evidence or preserve a manually asserted proof gap.

## Fact

`tiler_ir::index::IndexRealizationLaw` is a closed, non-exhaustive public enum
whose current variants cover F32 constants, binary pointwise F32, precise SiLU,
strict serial sum, reindex, broadcast, and strict F32 contraction. Reading that
enum and running
`rg -n 'dequantize_strict_affine_op|strict.affine' crates/tiler-compiler/src crates/tiler-ir/src/index`
finds no strict-affine realization law or compiler registration. The compiler's
physical frontier separately recognizes `ScalarProgram::StrictAffineU4Dequantize`
only to refuse it as a frontier expression; a physical spelling is not logical
index-refinement evidence.

`bind-stage-coverage-to-index-refinement-identity` makes the absence observable:
`CoveredOccurrence` can be derived only from a completed
`IndexRefinementReceipt`. Its artifact tests retain four strict-affine claims
that cannot honestly be reconstructed after that boundary: one builder/component
projection test and three codec tests covering round trip, typed corruption
refusals, and component-order identity. The exact population is
`rg -n 'strict_affine_u4_dequantize_artifact|strict_affine_components' crates/tiler-artifact/src/program`;
the helper has one builder consumer and three codec consumers. Deleting or
diluting them would erase established component-ABI evidence, while a synthetic
or foreign receipt would test the invalid path this prerequisite exists to
remove.

## Inference

The missing row is not a one-line registration. Strict-affine dequantization
reads the ordered codes, scale, and zero-point components of one encoded logical
operand and produces a dense F32 result. The candidate-blind law must therefore
state that compound logical access and scalar meaning in the dependency-neutral
index vocabulary before the compiler may advertise a capability. Reusing a
plain F32 pointwise law, treating the encoded operand as dense, or validating
only equal shapes would silently prove a different operation.

## Proposed work

1. Derive the exact logical index/scalar semantics of governed
   `dequantize_strict_affine_op()` from its semantic definition and existing
   component contract. Decide whether the current index tensor/scalar vocabulary
   can express it without collapsing logical components into physical storage;
   if not, present the smallest typed vocabulary extension and its unsupported
   cases.
2. Add a candidate-blind `IndexRealizationLaw` variant and canonical-region
   interpreter that binds the one encoded operand, all ordered component roles,
   and the dense result. Register it through the owning semantic provider and
   admit a compiler lowering capability only after the IR verifier can reproduce
   the exact canonical region.
3. Prove the check can say no: perturb the operation, encoded value contract,
   component role/order, shape, scalar operation, canonical region, and numerical
   contract independently. Each mismatch must produce a typed refusal and mint
   no receipt; retained domain obligations must remain pending rather than being
   converted into coverage.
4. Restore the four preserved artifact/component tests using real governed
   receipts, and show the ordinary compiler path retains the receipt rather than
   reconstructing its identity.
5. Enumerate identity consequences before editing a separator. Adding a new
   closed-law encoding tag may be append-only only if per-tag injectivity and
   registry framing make old and new subjects incomparable; otherwise advance
   the owning index-law/receipt domain completely. No kernel-program or artifact
   identity step belongs here merely because this ticket makes a previously
   unsupported subject reachable: those grammars are owned by the dependent
   coverage-binding ticket.

## Public boundary and stop condition

This work adds a consequential case to the public non-exhaustive
`IndexRealizationLaw` vocabulary and promotes strict-affine lowering from a
physical/type-system reservation to checked executable support. Eliminate the
representation alternatives against correctness, performance, and long-term
maintainability, preserve a tested draft, and present the exact enum/data shape,
construction authority, canonical-region semantics, compiler registration, and
unsupported cases to Tom before acceptance. Do not treat an implementation or
green gate as acceptance of that boundary.

## Closes when

A real strict-affine receipt is minted only for the exact governed semantic
operation and exact canonical logical region; all named perturbations refuse;
the compiler retains that receipt; the four dependent artifact/component tests
run without a forge or semantic substitution; identity analysis is recorded and
any required step is complete; the public boundary is accepted; and `make full`
passes.
