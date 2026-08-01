---
schema: "tiler-doc/v1"
id: "ADR-0080"
kind: "decision"
title: "Treat distributivity as a third numerical dimension"
topics: ["numerics", "reductions", "contraction", "optimizer"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics", "tiler.contract.optimizer", "tiler.contract.fusion-and-scheduling"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality", "tiler.research.numerics.operation-conformance-matrix"]
depends_on: ["ADR-0011", "ADR-0014", "ADR-0015"]
ticket: "record-distributivity-dimension-adr"
---

# 0080: Treat distributivity as a third numerical dimension

**Status:** accepted. Every clause below is derived from the numerical contract's own definitions and from ADRs 0011, 0014, and 0015; none of it is a product choice, and the one product choice in the neighbourhood — whether to admit a distributivity permission at all — is explicitly reserved by item 4 and owned elsewhere. This record supersedes nothing: neither ADR 0014 nor ADR 0015 claims exhaustiveness over the dimension set, so adding a third contradicts no accepted decision.

## Context

**Fact — the derivation is already in the contract and this record is the missing custody.** [`settle-contraction-chain-distributivity-permission`](../../tickets/settle-contraction-chain-distributivity-permission.md) resolved the question and landed the result as [Numerical semantics](../numerical-semantics.md#distributivity-is-outside-the-order-contract)'s "Distributivity is outside the order contract" section, and four further documents were updated to cite it: [the optimizer's logical-exploration rule](../compiler/optimizer.md#logical-exploration), [fusion and scheduling](../compiler/fusion-and-scheduling.md)'s contraction-planning paragraph, [the roadmap](../roadmap.md)'s Milestone 6 framing, and [Q-SEM-015](../open-questions.md). That ticket held `contracts/numerics` and `contracts/optimizer` and never `contracts/decisions`, so it structurally could not write this record and said so in its own Outcome.

**Why the gap matters rather than being a formality.** [Documentation metadata](../document-metadata.md) states that in a `mixed` contract "only accepted-ADR-derived invariants and sections explicitly labeled accepted are normative", and `docs/numerical-semantics.md` is `contract_status: mixed`. Until an accepted ADR derives it, the distributivity section is a well-argued paragraph that four documents treat as settled and the metadata contract classifies as proposed. This record is what closes that discrepancy; it adds no claim the contract does not already make.

**Fact — the two order-contract dimensions are narrow, checked by reading rather than by grep.** `docs/numerical-semantics.md` defines **reassociation** as changing grouping while preserving logical operand order and **permutation** as changing logical operand order. Two further sentences in the same section fix the scope: "Reassociation without permutation may combine only contiguous contributor intervals in order", which presupposes a fixed contributor *sequence* to take intervals of, and the exact-transformation rule that normalization must not reorder floating-point operations merely because they are associative over the reals, which names associativity specifically. [ADR 0014](0014-reassociation-vs-permutation.md) states the same transform in scalar terms — `(a + b) + c` to `a + (b + c)`, three operands, same order, different grouping. Both dimensions hold the contributor *values* fixed and vary only how those same values are grouped or ordered.

**Fact — the motivating rewrite holds no contributor sequence fixed.** For output `[i, l]`, `(AB)C` forms the rounded partials `T[i, k] = sum over j of A[i, j] * B[j, k]` and then sums the contributors `T[i, k] * C[k, l]` over `k`. `A(BC)` forms `U[j, l] = sum over k of B[j, k] * C[k, l]` and then sums the contributors `A[i, j] * U[j, l]` over `j`. The two sequences share no value, have different lengths in general, and are indexed by different axes. Not one rounded product is common to both. Neither is a grouping of the other, and no third sequence exists of which both are groupings.

**Fact — the identity relating them is distributivity, which the arithmetic does not satisfy.** The two forms agree over the reals by `(x + y) * c = x * c + y * c`. Round-to-nearest floating-point multiplication does not satisfy that identity, so the two forms are different computations with different roundings rather than two spellings of one.

## Decision

### 1. Distributivity is a numerical dimension, and it is a third one

**Proposal.** Tiler's numerical vocabulary recognizes **distributivity** as a dimension alongside reassociation and operand permutation. It authorizes exchanging a product of a sum for a sum of products in either direction. It changes which values are multiplied and therefore where roundings fall, which is precisely what puts it outside a contract that speaks about one contributor sequence.

**Why it cannot be folded into either existing dimension.** Both order-contract dimensions are defined over a contributor sequence, and the rewrite that motivates this dimension produces two sequences with no common value and no common refinement. There is nothing for reassociation or permutation to be a statement about, so the question is not which of the two covers it — neither can be evaluated.

### 2. It is additional to the other two, not a substitute for either

**Proposal.** A tensor-contraction chain regroup consumes all three permissions: distributivity, reassociation, and operand permutation. Naming only distributivity would be as incomplete as naming only reassociation.

**Why all three.** Routed through the flat form, a chain regroup also changes the nesting order over the flat reduction domain, so reassociation is consumed. Grouping the canonical lexicographic contributor order by the outer axis combines non-contiguous intervals, and the contract's own rule that "reassociation without permutation may combine only contiguous contributor intervals in order" assigns that to permutation. So reassociation is necessary, permutation is generally necessary, and a third dimension is necessary — which is the exact resolution of the predecessor question's "necessary but not sufficient".

### 3. The independence is forced by accepted decisions, not chosen here

**Proposal.** Granting reassociation does not grant distributivity, granting distributivity does not grant reassociation or permutation, and consuming distributivity requires both an operation capability declaring the algebraic property and an effective numerical permission to use it, exactly as ADR 0014 requires of the other two.

**Why it is forced.** [ADR 0011](0011-per-operation-numerical-permissions.md) requires every semantic rewrite to declare which effective permission it consumes, and `docs/numerical-semantics.md` states that one permission never implies another. [ADR 0015](0015-fma-vs-contraction.md) settles the structurally identical question for fused multiply-add: contraction permission is independent of reassociation and does not authorize regrouping an expression to create additional contraction opportunities — a permission over an existing pattern does not authorize manufacturing a new one. Reusing the reassociation permission to authorize distributivity would contradict all three at once.

### 4. No distributivity permission is admitted, and admitting one is reserved

**Proposal.** The dimension is defined; no permission grants it. The canonical policy has no such field, and `NumericalPermission` in `crates/tiler-ir/src/schedule/numerics.rs` supplies the two general resolutions `Forbidden` and `Permitted` that each *declared* dimension takes — there is no distributivity dimension for either resolution to apply to. Whether to admit one at all, and if so whether one permission covers both directions of the identity or the factoring and expanding directions are cut apart, is a product choice that does not follow from any definition above. It is reserved to the accepted decision that admits a tensor-contraction family under Q-SEM-015, and is owned by [`decide-whether-to-admit-a-distributivity-permission`](../../tickets/decide-whether-to-admit-a-distributivity-permission.md).

**Why the second half is not derivable.** ADR 0014 split reassociation from permutation on evidence that some combiners have one capability and not the other. No analogous asymmetry has been established for the two directions of the distributive identity, so cutting this dimension in two would be a preference wearing a derivation's clothes.

### 5. A rewrite consuming distributivity rejects, and the rejection names the missing dimension

**Proposal.** A tensor-contraction chain regroup is rejected under every contract Tiler can express, and the rejection names the missing distributivity dimension. Reporting a forbidden reassociation is not an acceptable substitute.

**Why the wording is normative rather than cosmetic.** A rejection naming reassociation implies that a contract permitting reassociation would admit the rewrite. That inference is false and is exactly what item 3 forbids, so the two explanations are not interchangeable phrasings of one outcome. This is the one clause of this record with an observable consequence for a caller, and it is an explainability requirement of the kind the correctness priorities already demand.

### 6. This is a settled legality position, not a pending or unexplored one

**Proposal.** The rejection is durable and does not depend on today's incidental limits. **Fact, read at `43f685f`.** `StrictF32NumericalContract::governed_profile` in `crates/tiler-compiler/src/request.rs` returns the exact set of contracts this build registers, and it holds two — `governed` and `governed_flush_to_zero` — both of which set `reassociation: NumericalPermission::Forbidden`. `normalize_serial_sum` in the same file independently rejects any program without exactly one input, so no tensor contraction reaches the compiler at all. Both limits will lift as the compiler grows, and neither is the reason. The distributivity gap is what survives them: the rewrite would still fail closed on a compiler that accepted contractions under a contract permitting both reassociation and permutation.

**Two citations of this fact were stale and are corrected alongside this record.** [Numerical semantics](../numerical-semantics.md) and [the optimizer model](../compiler/optimizer.md) both wrote that `StrictF32NumericalContract::governed` "remains the only numerical contract the compiler registers". Both were accurate when written on 2026-07-24 and stopped being so on 2026-07-25 at 08:12, when `aa7c4f0` unified admission behind `governed_profile` and registered a second contract. The conclusion each drew is unaffected, because both registered contracts forbid reassociation; only the arithmetic in the premise moved. Correcting them here rather than leaving a fourth stale copy is why this record's own statement of the fact names the profile function instead of one constant.

## Consequences

- The distributivity section of `docs/numerical-semantics.md` becomes accepted-ADR-derived and therefore normative under the `mixed`-contract rule, together with the optimizer's third logical-exploration rule, fusion-and-scheduling's contraction-planning legality boundary, and the roadmap and open-question text that cite it. Nothing in any of those documents changes; what changes is that they now rest on an accepted decision instead of on a `done` ticket's Outcome.
- Contraction-order exploration is illegal rather than unimplemented. That distinction is load-bearing for planning: it means the work is blocked on a decision Tom has not made, not on engineering nobody has done, and building the search first would produce a search with no legal input.
- The dimension is defined while its permission is withheld, which is a type-system-and-vocabulary reservation rather than implemented support. `implementation_status` is `not-started` and that is the honest value: no field, variant, or capability names distributivity anywhere in `crates/`, and the only thing the tree contains is the *absence* the rejection is required to name.
- A future contract that permits reassociation does not become able to regroup a contraction chain. This is the specific wrong inference the record exists to prevent, and item 5's rejection wording is what makes the compiler say so at the point of contact rather than leaving it to a reader.
- The conclusion is independent of a decision the roadmap reserves for Tom. Were a multi-operand contraction node defined as the flat sum over `(j, k)` of `A[i, j] * B[j, k] * C[k, l]`, its contributors would be triple products that neither binary association ever computes, and factoring `C[k, l]` out of the `j`-sum would again be distributivity. So this record does not preempt whether a semantic contraction node may consume more than two operands; it settles what an association is a choice *about*, not whether it needs a permission Tiler cannot express.
- "Contraction" is the tensor sense throughout — summation over indices shared by two or more operands — and never ADR 0015's fused-multiply-add permission. That permission governs exactly one thing about a tensor contraction: whether its own per-contributor `accumulator + a * b` step may round once. It says nothing about that reduction's order and nothing about which products are formed.

## Alternatives considered

**Read the reassociation permission as covering a chain regroup.** This is what [`qualify-contraction-association-reassociation-permission`](../../tickets/qualify-contraction-association-reassociation-permission.md) and the roadmap's Milestone 6 framing originally assumed, and it is the reading a reader arrives at from the word "association" alone. Rejected on the contract's own definitions: reassociation preserves logical operand order over a fixed contributor sequence, and the rewrite produces two sequences sharing no value. Every reading of "reassociation" available in the corpus is the narrow one, so this alternative requires widening an accepted definition rather than applying it.

**Define the dimension and admit a permission for it in the same record.** Attractive because it would make Milestone 6's first bullet reachable and would leave no half-answered question. Rejected because admitting a permission is a product choice with no derivation behind it, and because the second half of that choice — one permission or two directional ones — needs evidence of a capability asymmetry that nobody has looked for. Bundling it would put a preference inside a record whose whole authority is that it contains none.

**Leave the derivation in `docs/numerical-semantics.md` with no ADR.** The status quo, and it is not merely untidy. Under the `mixed`-contract rule the section is not normative without an accepted decision behind it, so four documents would continue to cite as settled a position the metadata contract classifies as proposed. That is the duplicated-and-unclear-authority failure the documentation contract exists to prevent.

**Write one ADR carrying both this dimension and the admission choice.** Deferred rather than rejected, and the choice is Tom's. If he prefers a single record, this one is the natural place for the admission clause to land later as an amendment, and [`record-distributivity-dimension-adr`](../../tickets/record-distributivity-dimension-adr.md) says so. Writing it that way now would mean either holding the settled half hostage to an unanswered product question, or publishing an ADR whose second half is blank.

## Traceability

The [reduction semantics and legality research](../research/numerics/reduction-semantics-and-legality.md) is the evidence behind the order contract this record adds a dimension outside of: it establishes what a reduction's contributor sequence is, what ordered-fold contributors mean, and why reassociation and permutation are separately permissioned — which is what makes "no common sequence exists" a conclusion rather than an assertion. The [initial operation conformance matrix](../research/numerics/operation-conformance-matrix.md) is the evidence ADRs 0011 and 0015 rest on, and this record's independence argument is theirs applied to a third dimension rather than a new one. Neither research record discusses distributivity by name; the derivation is from the contract's definitions and is reproduced in full above and in the numerical contract.

[Numerical semantics](../numerical-semantics.md) owns the dimension's definition and the worked counterexample. [The optimizer model](../compiler/optimizer.md) owns the logical-exploration rule that names all three permissions and the required rejection wording. [Fusion and scheduling](../compiler/fusion-and-scheduling.md) owns the statement that a schedule may not select a contraction order at all today. The work records are [`settle-contraction-chain-distributivity-permission`](../../tickets/settle-contraction-chain-distributivity-permission.md) for the derivation, [`record-distributivity-in-the-navigation-contracts`](../../tickets/record-distributivity-in-the-navigation-contracts.md) for the roadmap and open-question routing, [`decide-whether-to-admit-a-distributivity-permission`](../../tickets/decide-whether-to-admit-a-distributivity-permission.md) for the reserved product choice, and [`record-distributivity-dimension-adr`](../../tickets/record-distributivity-dimension-adr.md) for this record.

**Item 4's reservation was discharged on 2026-08-01 and this record is not superseded.** [ADR 0095](0095-decline-a-distributivity-permission.md) answers the choice item 4 reserved: Tom declined at the live review, so no distributivity permission is admitted and admitting one is no longer pending a decision — it is pending a *reopening trigger*, the first workload whose natural spelling is a directly regroupable contraction chain. Item 4's text is preserved exactly as accepted, because what it decided is that the admission does not follow from any definition here and belongs elsewhere; that is still true, and ADR 0095 is the elsewhere. A reader who arrives at item 4 expecting an open question should read ADR 0095 instead. Nothing else in this record moves: the dimension, its independence, the three-permission accounting, and the rejection-wording requirement are exactly what a decline leaves standing, and are exactly what an admission would have had to build on.
