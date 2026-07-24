---
id: scope-einsum-contraction-support
title: Scope einsum and tensor-contraction support (Milestone 6)
status: done
priority: p1
dependencies: []
related: [own-operation-family-support-matrix, qualify-contraction-association-reassociation-permission, disambiguate-contraction-in-the-glossary]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, breadth, einsum]
---
Milestone 6 (einsum contractions) has zero tickets and zero open questions. Every
"contraction" currently in the corpus is the FMA numerical-permission sense
(whether multiply-add may fuse into one rounding), never tensor contraction. For a
"DataFusion for tensor compute", general tensor contraction — matmul, batched
matmul, and einsum — is the single most conspicuous missing operation family, and
it is invisible in both the work graph and the durable question index.

Add an owning `docs/open-questions.md` entry (and, if warranted, a `docs/roadmap.md`
Milestone 6 note) that frames the tensor-contraction / einsum question with an
explicit reconsideration trigger: what identity, validation, access-relation, and
lowering consequences a contraction operation family imposes, and what must be true
(a generic compile path, a working backend, the optimizer conformance gate) before
it can be scheduled. This is a deferred question with a trigger, not an
implementation and not a commitment to a specific einsum surface.

Coordinates with `own-operation-family-support-matrix`, which references this as
the contraction line of the broader operation-family matrix.

## Outcome

[Q-SEM-015](../docs/open-questions.md) is the durable entry, filed under "Deferred until an explicit trigger", and the substantive framing it indexes is the [Milestone 6 framing section](../docs/roadmap.md#framing-what-a-tensor-contraction-family-would-impose). The split follows the precedent `own-operation-family-support-matrix` set for Q-SEM-014: `docs/open-questions.md` states that each entry is one question with one owner and one closure or trigger, so a five-part consequence analysis belongs beside the milestone it constrains, with the question index pointing at it rather than holding a second copy. The matrix's contraction row keeps its rung and evidence, and its trigger cell now names the framing and Q-SEM-015 instead of forward-referencing this ticket; Q-SEM-014's own cross-reference now names Q-SEM-015 rather than a ticket id.

The trigger is deliberately two-part, because collapsing it into one condition would be wrong in both directions. The semantic half — identity, construction-time validation, access relation — depends on a named workload or frontend lowering and on nothing else; no backend is required to fix what a contraction *means*. The planning half is gated on `prototype-optimizer-conformance-gate` closing and on a backend having executed a compiled program. That gate is stated from exact evidence rather than as a readiness judgement: `normalize_serial_sum` in `crates/tiler-compiler/src/request.rs` rejects any program whose `input_count()` is not exactly one, so a binary contraction cannot reach the compiler at all, and `capability.rs` and `legality.rs` are `pub mod` draft authorities that `pipeline::compile` never calls, so no occurrence can resolve a lowering provider. [Fusion and scheduling](../docs/compiler/fusion-and-scheduling.md) independently requires contraction planning to follow the boundary-contract and cost infrastructure.

The five consequence areas each rest on inspected source or an accepted decision. **Identity:** an authored subscript string is not canonical under ADR 0074, because `CanonicalValue::Utf8String` holds exact bytes with no normalization, so `ij,jk->ik` and `ab,bc->ac` would be two identities for one computation; the canonical attribute must encode index *structure*, and `StrictSerialSumF32::infer` is the precedent for validating it inside the operation definition. **Validation:** a contraction is the first family whose operands are required to disagree — `BinaryF32::infer` accepts only equal shapes or the rank-zero admission — and its extent agreement is already answered by two accepted decisions in the shape-environment contract that use `MatMul` as their worked example; what remains is the structural well-formedness that precedes any extent comparison, enumerated as five construction-time rejections. **Access relation:** the family is architecturally distinct because two or more operand access maps share one reduction domain while each drops a different subset of the free coordinates, and those maps are pure projections that the admitted index vocabulary already expresses, so the gap is the absent lowering capability rather than the index language. **Lowering:** the direct/tiled and library-GEMM alternatives have different prerequisites — the first is a `ScheduledKernel`, the second an `OpaqueCall` deferred behind `implement-opaque-physical-call-providers` — and a vendor GEMM that does not publish its accumulation order is `Unknown`, which is inadmissible rather than expensive. **Numerical contract:** a contraction inherits every reduction obligation, K-padding a tiled GEMM owes a neutrality proof under the exact signed-zero counterexample the contract already gives, its signature shape is already settled by the accepted mixed-precision decision, and it would resolve to `FusionLegality::Unknown` because `OrderedReduction` is the only registered reduction role.

The naming collision is stated wherever the word is used. ADR 0015's contraction is the fused-multiply-add permission; tensor contraction is the operation family. They meet at exactly one point — a contraction's per-contributor `accumulator + a * b` — and that permission is `Forbidden` in the only numerical contract the compiler registers, so a device or library GEMM built on fused multiply-add accumulate does not implement the declared semantics. This ticket's stated premise that every corpus use of "contraction" was the numerical sense is false, and a third contract beyond the two named at dispatch also uses the tensor sense: `docs/ir.md` requires reductions and contractions to declare computation precision, accumulator and result types, and an order or algorithm contract. The matrix row's evidence cell now records that third site.

Two questions are named as Tom's rather than decided: whether a contraction is one keyed family carrying an index-structure attribute or a set of fixed-arity keys per shape class, and whether a semantic contraction node may consume more than two operands. The second determines whether Milestone 6's contraction-order bullet has a subject at all.

Two follow-ups were filed for contracts this ticket does not hold. [`qualify-contraction-association-reassociation-permission`](qualify-contraction-association-reassociation-permission.md) records that the optimizer contract lists "choose alternative contraction associations" under a stage that "adds only proved contract-preserving forms" without the reassociation qualifier its neighbouring bullet carries: regrouping a contraction chain changes which partial sums are rounded, so under the registered strict `f32` contract, where reassociation is `Forbidden`, contraction-order exploration is illegal rather than merely unexplored. [`disambiguate-contraction-in-the-glossary`](disambiguate-contraction-in-the-glossary.md) records that `docs/glossary.md` covers only the numerical sense, and only obliquely inside its "Numerical policy" row.

No `contracts/optimizer`, `contracts/foundation`, or `contracts/numerics` file was edited; all were read as evidence and cross-referenced only.
