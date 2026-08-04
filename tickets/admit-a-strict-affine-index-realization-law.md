---
id: admit-a-strict-affine-index-realization-law
title: Admit a strict-affine index-realization law
status: done
priority: p1
dependencies: [place-index-refinement-evidence-under-an-ir-owned-verifier]
related: [bind-stage-coverage-to-index-refinement-identity, prototype-quantized-value-vertical]
scopes: [implementation/ir, implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, correctness]
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
4. Show the ordinary compiler path retains the real governed receipt rather than
   reconstructing its identity. The dependent stage-coverage ticket owns rebasing
   the four preserved artifact/component tests onto that receipt.
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

## Scope declarations during implementation

`implementation/reference` declares the independent scalar oracle required by
the new strict-affine scalar meaning: adding a governed scalar definition
without its executable reference capability would make the standard refinement
profile structurally claim arithmetic that its correctness oracle cannot run.
The dependent stage-coverage ticket, not this prerequisite, declares
`implementation/artifact` and owns rebasing the four preserved fixture consumers
onto this receipt. The reference scope addition describes work already required
by this ticket's accepted outcome; it does not accept the consequential public
boundary.

## Public-boundary acceptance — 2026-08-04

Tom accepted the exact consequential public boundary at reviewed commit
`5626957d05593c8ab1e6d4a1f993ba1e120a9a8c` in the live T3 Code session on
2026-08-04, after receiving the complete inventory below, the two green
fixed-point reviews, the unsupported-case boundary, and the identity blast
radius. That exact commit includes the final `OperandBindingsTooLarge`
fail-closed boundedness correction and its two green fixed-point reviews.

Acceptance authorizes integration of this tested boundary. It does not accept
U8 or per-axis parameter maps, physical packing, Metal executability, nested
encoded components, or the downstream stage-coverage/artifact identity step.

## Accepted public boundary

**Fact:** the existing region boundary could bind exactly one dense tensor to
each distinct semantic input. A strict-affine operand is one encoded logical
value whose contract already declares three ordered typed components, so the
old boundary could express neither the component reads nor an honest receipt.

**Elimination:** treating the encoded value as a dense scalar proves a different
operation and is rejected by correctness. Adding a second caller-authored role
list to the index region duplicates the semantic contract, permits the two lists
to drift, grows every ordinary tensor boundary, and would force an index-region
identity step for no new information. Decomposing the scalar meaning into a new
general U4/I32 conversion vocabulary would expose several public operations and
intermediate types before another consumer exists; it is less maintainable than
one exact atomic scalar definition and gives physical planning no benefit,
because the schedule/KIR layer already owns the decomposed execution spelling.

**Proposal:** an encoded semantic input expands, only during receipt binding,
into the component tensors already declared by its contract, in exact contract
order. `OperandBinding` retains `component_role: Option<EncodedComponentRole>`;
ordinary values remain one `None` binding, while strict-affine U4 yields ordered
codes/scale/zero-point bindings. `IndexRealizationLaw` adds
`StrictAffineU4Dequantize { codes_role, scale_role, zero_point_role, scalar }`
and the standard constructor fixes those fields to the governed role constants
and `tiler.scalar::strict-affine-u4-dequantize@1`. The law accepts only the exact
U4 contract, empty attributes, one operand, same-shaped dense F32 result, the
exact ordered component declarations, and the strict F32 numerical contract.
The compiler registers one provider for that signature and emits the identical
candidate-blind canonical region. No physical packing enters this logical law.

**Identity analysis:** law encoding tag 8 is append-injective: tags 1 through 7
and their payloads remain byte-identical. Rows are self-delimiting through the
actual canonical operation and provider encodings, the fixed-width revision,
and the tagged law payload; there is no outer row-length frame.

The existing scalar operation keys and definition payloads, semantic registry
snapshot, existing per-operation law rows, canonical regions, old-operation
refinement subjects, and old-operation resolution identities remain stable. The
complete `CanonicalScalarRegistrySnapshotIdentity`,
`CanonicalScalarReferenceRegistryIdentity`,
`IndexRealizationLawRegistryIdentity`, and `CanonicalLoweringRegistryIdentity`
move because each gains a row or binds a complete snapshot that did. Every
`IndexRealizationAuthority` moves because it binds the complete scalar snapshot,
and old-operation `ScalarAuthorityEvidence` and receipt identities move for the
same reason even though their reached-definition projections stay stable. The
strict-affine subject moves from an absent law row to the new row and gains its
first resolvable resolution and receipt. Canonical request subjects move because
they bind the complete realization/lowering authorities; every derived artifact
identity or cache entry that binds one of those moved requests, authorities, or
receipts must miss rather than replay. The deterministic request qualifier moves
to `fb0b64dd69649785`. No separator or domain version step is required because
the newly tagged payload cannot collide with an old row; all deterministic pins
in this diff were recomputed from the complete tree.
The residual ceiling remains 6,144: strict-affine has two rank-wide accesses
(codes read and result write), while scale and zero-point reads are rank zero,
so it does not exceed the existing three-rank-wide-access closed-law maximum.

**Unsupported cases:** strict-affine U8, other schemes or component maps, nested
components, non-strict F32 contracts, alternate component order or roles, and
semantically equivalent noncanonical index regions all refuse. Quantization and
encoded-value production remain outside this lowering. Metal executability is
unchanged; this receipt proves logical realization only. The registry currently
admits one realization-law row per semantic operation, so the selected future
per-axis U8 profile must replace this exact U4 row with a generalized law rather
than append a competing row. `implement-workload-selected-quantized-parameter-maps`
owns that triggered broadening and depends on this initial exact authority.

**Exact consequential public inventory:** `IndexRealizationLaw` gains the
`StrictAffineU4Dequantize` variant and its standard constructor;
`strict_affine_u4_dequantize_scalar_op()` and its standard scalar-definition row
name the atomic scalar meaning; `OperandBinding::component_role()` exposes the
generic encoded-component expansion performed during receipt binding; and the
standard semantic-law, scalar-reference, and compiler-lowering populations each
gain the corresponding exact row. U8, non-per-tensor parameter maps, other
schemes, and nested encoded components remain unsupported. The public
`OperandArity` errors in both IR verification and compiler refinement now report
`expanded_inputs` (ordinary semantic inputs plus ordered encoded components),
and `OperandInterface::position` names that same expanded boundary order rather
than a distinct semantic-operand ordinal. Both layers also gain
`EmptyEncodedOperandComponents { input }`, because a zero-component encoded
contract has no expanded position that `OperandInterface` could truthfully
name. Expansion first checked-counts component declarations, saturating only on
host arithmetic overflow, and refuses any population above the existing public
`MAX_BOUNDARY_TENSORS` region limit or unequal to the actual verified input
population before reserving component vectors or deriving component shapes.
Alias expansion is then checked independently before the retained receipt
bindings are allocated: public `MAX_INDEX_REFINEMENT_OPERAND_BINDINGS` currently
equals `MAX_BOUNDARY_TENSORS`, but has its own name because repeated uses of one
encoded input can exceed the distinct region input population. Both IR
verification and compiler refinement expose the typed public
`OperandBindingsTooLarge { actual, limit }` refusal. Its count saturates on host
arithmetic overflow and the final binding vector reserves exactly the accepted
population. The new constant, error variants, and accounting order add no
identity bytes and move no identity domain or deterministic pin.

## Draft verification

The affected four-package nextest population passed 1,786 of 1,786 tests with 5
configured skips. Package doctests passed 20 with 1 ignored; Clippy passed for
all targets with warnings denied; formatting, `git diff --check`, `tkt lint`,
and true-base guard passed. The four preserved artifact/component tests remain
the dependent stage-coverage ticket's closure evidence: after its rebase they
must run against this real receipt before that ticket closes.

The negative population independently refused a U8 encoded contract, a changed
component role, a changed scalar operation, a non-strict numerical contract, a
capability applied to the wrong semantic operation, a reversed but semantically
equivalent noncanonical traversal, swapped component boundaries, and a scalar
codes boundary with the wrong shape. The scalar oracle additionally covered the
exact centered-code boundaries `-15` and `+15` and refused positive and negative
zero, a negative normal, both signed subnormal classes, NaN, and both infinities
as typed invalid applications. Removing the scale-domain check made that focused
test fail (exit 100); restoring it made the same one-test population pass.
Sixteen 1,024-component encoded inputs exactly reached the 16,384-boundary
region maximum in the count-only pass; a seventeenth refused with typed arity
before any component shape was materialized. A safe zero-component encoded
contract refused with the new input-indexed error and the compiler mapping
preserved it exactly. Removing the boundary-ceiling check made the over-limit
test return `Ok(17408)` and fail (exit 100); restoring it made the same focused
population pass. Separately, one 1,024-component encoded input aliased sixteen
times exactly reached the 16,384 receipt-binding limit; a seventeenth use
refused at 17,408 before final collection, while a synthetic host arithmetic
overflow saturated to `usize::MAX` and refused. Removing that independent
binding-ceiling check made the 17-use case return `Ok(17408)` and fail (exit
100); restoring it made the same focused population pass.
Before the frozen explain qualifier was
updated, its existing exact assertion failed and reported the recomputed
`fb0b64dd69649785` value; that deliberate observation proves the identity check
detects the new authority rows.

## Closes when

A real strict-affine receipt is minted only for the exact governed semantic
operation and exact canonical logical region; all named perturbations refuse;
the compiler retains that receipt; identity analysis is recorded and any
required step is complete; the public boundary is accepted; and `make full`
passes. Rebasing stage coverage and its artifact fixtures is downstream work and
is not a prerequisite for this law authority to close.
