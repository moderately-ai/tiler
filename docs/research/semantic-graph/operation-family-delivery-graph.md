---
schema: "tiler-doc/v1"
id: "tiler.research.semantic-graph.operation-family-delivery-graph"
kind: "research"
title: "Operation-family delivery graph"
topics: ["semantics", "operations", "delivery", "roadmap", "ticket-graph"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.ir", "tiler.contract.operation-extensions"]
ticket: "derive-the-operation-family-and-signature-delivery-graph"
---

# Operation-family delivery graph

- **Status:** ownership map over an already-enumerated inventory. It registers nothing, selects no operation, and moves no support-matrix rung.
- **Ticket:** [`derive-the-operation-family-and-signature-delivery-graph`](../../../tickets/derive-the-operation-family-and-signature-delivery-graph.md).
- **Research date:** 2026-08-05, against the tree at `b63dd5d0`.

## Traceability

- **Current disposition:** pending. The tracks below are a partition and an owner assignment, not adopted contract text.
- **Inventory this record consumes:** the [mature operation and signature taxonomy](mature-operation-and-signature-taxonomy.md) owns the operation universe, its eight signature dimensions, and its twelve `RQ-OP` questions. Every one of its forty-seven families is accounted for in [Coverage](#coverage-every-taxonomy-family-reaches-a-track) below, and no classification is re-derived here.
- **Delivered state this record consumes and never restates:** the [operation-family support matrix](../../roadmap.md#operation-family-support-matrix) owns what is built at each layer and its seven rungs. Where a cell below says *delivered*, the matrix is the citation, and the matrix stays the sole maturity ledger.
- **The minimum-route classification this record consumes rather than repeats:** the [minimum correct physical realization profile](../program-planning/minimum-correct-physical-realization-profile.md) already sorts all forty-seven families into four coverage classes for the baseline physical route, from the taxonomy's own D7 cells. Rung **M6** below is that classification read per track, not a second one.
- **Companion ownership map on the other axis:** [Dtype-family research tracks](../numerics/dtype-family-research-tracks.md) does for the element-type universe exactly what this record does for the operation universe, and already states five joins between the two. [Where the tracks meet the dtype axis](#where-the-tracks-meet-the-dtype-axis) reads those joins from this side and adds nothing to them.
- **Normative destinations, when a track eventually delivers:** [IR stack and invariants](../../ir.md) owns the operation/value model, [Operation extensions](../../operation-extensions.md) owns the definition and capability contract, and [Numerical semantics](../../numerical-semantics.md) owns per-operation numerical meaning.
- **Primary sources:** none new. Every operation fact this record relies on is already pinned by the taxonomy, whose own preservation boundary — including the two re-checks that cost it two claims — is stated there and is not restated here.

## Purpose and boundary

The taxonomy enumerates what a mature Tiler must be able to express and the matrix records what is built. Neither answers the question this record answers: **for each family and each signature partition, which of the eight delivery rungs are owed, who owns the next one, and what would have to become true for that owner to start.** Without that answer a family with no current workload is indistinguishable from a family nobody has thought about, and the two need opposite treatment — the first is a deferral with a trigger, the second is a defect.

Four things this record deliberately does not do.

- **It does not move a matrix rung.** Every *delivered* claim below is quoted from the matrix. A track's existence is not evidence of support, and the matrix's own rule holds unchanged: listing a family authorizes nothing.
- **It does not register anything or propose a public boundary.** No `OpKey`, no attribute schema, no Rust spelling. Those are reserved to Tom under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md), and this record's tracks each say so in their own non-goals.
- **It does not file work whose prerequisites are unresolved.** Twenty-six of the twenty-nine tickets this pass filed are `deferred`, so the scheduler cannot offer them. The three that are not are the concatenate lowering fork, whose demand is live and whose blocking question is stated; `RQ-OP-04`'s generalization test, whose closure test names no workload, no target, and no measurement; and one catalog defect the required reconciliation check found, which is not an operation-family track at all and is filed rather than absorbed.
- **It does not create an umbrella.** There is no "support all operations" node. Every track is bounded by its families, its rungs, and its trigger, and a track that could not state all three would be a defect in this record.

## The eight delivery rungs, and how they join the matrix's seven

The governing ticket names eight rungs. They are not a second maturity ladder competing with the matrix's `R1`–`R7`; they are the obligations at the granularity a *ticket* is scheduled at, which is finer in two places and coarser in none. The join is stated once here so no track restates it.

| Rung | What the cell fixes | Matrix rung |
| --- | --- | --- |
| **M1 Semantic identity** | A governed `OpKey` with an attribute schema and a normative reference the definition's fields are read from. | R3, first half |
| **M2 Validation and shape inference** | A deterministic inference routine deriving result shapes from operands rather than accepting declared ones, and every malformed case refused at construction under its own named diagnostic. | R3, second half |
| **M3 Reference semantics** | A host oracle for the exact signature, together with the evidence class that oracle can honestly claim. | R4 |
| **M4 Logical rewrite participation** | A registered fusion role and derived legality, so a region containing the family resolves to something other than `Unknown`. | R5 |
| **M5 Index/access lowering** | An index-access capability that emits a region for the occurrence, in the access classes the [index and access model](../indexing/index-access-model.md) admits. | *(between R5 and R6; the matrix has no rung for it)* |
| **M6 Minimum physical realization** | A deliberately simple valid route that exists for any program the semantic layer admits — or an explained refusal where the family's D7 names none. | *(no rung; the [physical profile](../program-planning/minimum-correct-physical-realization-profile.md) owns it)* |
| **M7 Backend realization** | A structured-kernel construct, a backend emission, and a target whose declared numerical realization does not reject it. | R6 |
| **M8 Bounded conformance evidence** | The corpus that would catch a regression, bounded by exactly what it exercises. | R7 |

**Fact — M5 and M6 are the two places the matrix's ladder does not resolve, and that is not a cosmetic gap.** The derivation this paragraph was written on, on 2026-08-05, was three families at R5 with three different actual blockers: `tiler::silu-f32@1` had a registered lowering capability and no `ScalarProgram` spelling, so no region could realize it; `tiler::rms-norm-f32@1` and `tiler::softmax-f32@1` realized as *two* and *three* regions respectively while `GovernedIndexAccess` emitted exactly one region per occurrence; and `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` had lowering capabilities that were never resolved, because the request boundary admitted no program shape containing one. **All three blockers were discharged by 2026-08-06 — [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](../../../tickets/admit-the-registered-unary-families-at-the-compiler-request-boundary.md), [`lower-a-two-region-occurrence-through-one-index-access-capability`](../../../tickets/lower-a-two-region-occurrence-through-one-index-access-capability.md), and [`reach-a-verified-kernel-through-the-structural-families`](../../../tickets/reach-a-verified-kernel-through-the-structural-families.md) are all `done` — and the derivation is what survived them.** The activation and the two structural families are at R6 and compile through the ordinary path. The one-region-per-occurrence limit is gone: `IndexAccessLoweringProvider::lower_sequence` emits an ordered chain, and the normalization carries a registered region-sequence law, so its blocker moved twice on 2026-08-06 — the missing scheduled-region spelling (an M6 fact) landed under [`admit-a-scheduled-region-for-a-staged-elementary-family`](../../../tickets/admit-a-scheduled-region-for-a-staged-elementary-family.md), and the live one is now program assembly's missing staged-realization declaration, owned by [`account-for-a-staged-realization-stage-in-the-kernel-program`](../../../tickets/account-for-a-staged-realization-stage-in-the-kernel-program.md) — while the softmax's is a missing `IndexRealizationLaw` — an M5 fact, owned by [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](../../../tickets/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md) and [`admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence`](../../../tickets/admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence.md). Both families still report R5, and the matrix reports one rung for both. **Inference.** A reader scheduling from the rung alone would treat them as one wall. The M5/M6 split is what makes them two, and it is why this record's rung vocabulary is eight rather than seven.

**A second property of the join matters for reading the tables.** The rungs are **not a total order** for a family, only for a layer. The quantization track's non-monotonicity is the worked case the matrix already records: separately tested non-monotone physical evidence exists for U4 dequantization without the family reaching M4 at all. A track whose M7 cell says something while its M4 cell says *owed* is reporting that, not contradicting itself.

## How the partition was derived

The governing ticket fixes the rule: group signatures only where one correctness argument and one implementation genuinely cover them, and split when numerical contracts, compound storage, effects, or backend feasibility differ. Applied to the taxonomy's forty-seven families it yields **forty tracks**, which is neither one per family nor one per taxonomy group.

**Where the rule grouped.** F-22 and F-23 are one track because a bijection and a many-to-one read map share a bit-preserving oracle, a coordinate-relation fusion role, and a lowering that moves no data; F-13, F-14, F-16, and F-17 are one track because none of them is expressible until a predicate tensor is a graph value, which is the same missing decision seen four times; F-18 and F-19 are one track because both are keyed by an ordered `(source, destination)` pair and both owe their contract fields by the same derivation, which is what `RQ-OP-04` asked about and [answered](../numerics/conversion-family-decomposition-across-pairs.md) — **corrected 2026-08-05:** this sentence previously grouped them on the ground that "a float-to-integer conversion is a directional pair under [ADR 0041](../../decisions/0041-separate-float-to-integer-conversion-families.md) exactly as a float-to-float conversion is under [ADR 0091](../../decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md)", and ADR 0041's four families are all float-to-integer, so they decompose one ordered pair by *mode* rather than decomposing a pair by direction; the grouping is unaffected and only its ground moves; F-37 and F-38 are one track because F-38's own D7 makes its physical fallback "a full sort followed by a slice", so the sort's implementation contains the selection's; F-03 and F-27 are one track because neither has any numerical content and both realize as the read map or copy the two admitted structural families already emit.

**Where the rule split, and these are the interesting ones.**

- **F-06 splits across two tracks, and the split is a signature partition rather than a compromise.** The taxonomy classifies binary float arithmetic as "atomic per operation", so `add` and `multiply` — delivered, executed, and bit-compared on one measured host — and `subtract` and `divide` — no key at all — are different signatures of one family at opposite ends of the ladder. A track holding all four would have to report one rung for both ends.
- **F-07 splits out of pointwise float algebra.** The matrix currently holds `Subtract`, `Divide`, negation, and required `Fma` in one row at R2. A fused multiply-add owes an oracle the others do not (exact-rational is *required*, because a host `f32` route double-rounds), has a physical precondition the others do not (a target offering single rounding, which [ADR 0015](../../decisions/0015-fma-vs-contraction.md) makes non-negotiable), and fails in a different class (hard infeasibility, never approximate feasibility). The [physical profile](../program-planning/minimum-correct-physical-realization-profile.md) reaches the same split independently, placing F-05 and F-06 in *covered — direct scalar or map route* and F-07 in *covered only under a stated precondition*.
- **F-28 splits three ways.** The delivered strict serial sum, the undelivered numeric reducers (product, standalone extrema, non-identity seed, and the variadic question `RQ-OP-06` owns), and the logical `any`/`all` case, which `RQ-OP-03` blocks along with four whole families and which therefore belongs to the predicate track rather than to a reduction track. One row cannot carry two blockers.
- **F-25 splits from every other structural family.** F-22, F-23, F-24, F-26, and F-27 have no numerical content; padding introduces a value into the result that downstream arithmetic reads, and [Numerical semantics](../../numerical-semantics.md) already carries the counterexample that makes it non-neutral. That is a numerical contract, which is the rule's first split criterion.
- **F-39 splits from F-37 and F-38.** Sorting *produces* an order and owes a total order and a tie-break; an ordered search *consumes* an order it cannot see and owes a validated precondition under [ADR 0021](../../decisions/0021-validated-value-assumptions.md). The implementations differ too — a sort against a binary search parallel over the needles.
- **F-44, F-46, and F-47 stay three tracks.** The taxonomy's conclusion 7 forbids the merge in its own words: "Effect, region, and non-tensor-value support are three separate reservations, not one 'advanced features' bucket."

**What is deliberately not a track.** The taxonomy's join table carries one row — *Arithmetic over reduced-precision floats* — that maps to "the dtype axis of F-05 through F-12" rather than to any family. That is a signature axis crossing many tracks, so it is recorded as **O-13**, a cross-cutting note whose owners are the live BF16 chain, and it contributes no family to the coverage table.

## The tracks

Forty tracks. Twelve have exact owners already and gain no ticket, one is a cross-cutting note, and twenty-seven are newly owned — twenty-six by a new track ticket and one, the concatenate lowering, by a new ticket beside an already-delivered semantic half.

### Semantic rungs

*owed* = no work exists. *live* = a ticket is open on it. *delivered* = the matrix records it, and the matrix is the citation.

| Track | Families | M1 identity | M2 validation | M3 reference | M4 rewrite |
| --- | --- | --- | --- | --- | --- |
| **O-00** Pointwise F32 constants and separate-rounding arithmetic | F-01, F-06{`add`, `multiply`} | delivered | delivered | delivered | delivered (`ValueSource`, `ElementwiseArithmetic`) |
| **O-01** Strict serial F32 sum reduction | F-28{`sum`} | delivered | delivered | delivered | delivered (`OrderedReduction`) |
| **O-02** Named activations and normalizations | F-12 | delivered, three keys | delivered | delivered, exact-rational enclosures | delivered, three distinct roles |
| **O-03** Tensor contraction | F-32 | delivered, one keyed family with an index structure | delivered, five structural rules refusing at construction | delivered | **owed** — no `FusionOperationRole`; live ticket |
| **O-04** Strict-affine quantization transitions | F-20 | delivered for two per-tensor contracts | delivered | delivered | owed; live tickets |
| **O-05** Axis-structural bijections and broadcast | F-22, F-23 | delivered | delivered, six mapping forms and three relations | delivered, bit-preserving | delivered (`CoordinateRelation`) |
| **O-06** Sub-tensor selection | F-24 | delivered for the literal-offset form | delivered, nine named refusals | delivered | owed |
| **O-07** Sequence extension | F-26 | delivered | delivered | delivered | delivered (`CoordinateRelation`), 2026-08-06 |
| **O-08** Indirect gather | F-34 | **delivered** for the F32 source / `tiler::u32@1` index form, 2026-08-07 | **delivered**, nine named refusals | **delivered** at the reference layer, with the bounds rule enforced there | **not owed, and deliberately absent** |
| **O-09** Predicate-producing and predicate-consuming families | F-13, F-14, F-16, F-17, F-28{`any`, `all`} | owed, gated on `RQ-OP-03` | owed | owed | owed |
| **O-10** Integer data arithmetic and division | F-08, F-09 | owed; ADRs 0039 and 0040 accepted, no key | owed; `RQ-OP-01` fixes the checked form's arity | owed | owed |
| **O-11** Collective and cross-device | F-46 | owed; needs an effect vocabulary and tokens | owed | owed | owed |
| **O-12** Opaque, custom, and composite calls | F-45 | declared per instance; crate-private by [ADR 0078](../../decisions/0078-name-the-intended-public-extension-seams.md) | declared per instance | none generic, by design | owed |
| **O-13** *(cross-cutting)* Reduced-precision float arithmetic | the dtype axis of F-05–F-12 | delivered for three BF16 keys | delivered | delivered, exhaustive-finite over 65,536 encodings | owed; live ticket |
| **O-14** Structural index generation | F-02 | owed | owed | owed | owed |
| **O-15** Remaining bit-preserving structural families | F-03, F-27 | owed | owed | owed | owed |
| **O-16** Bit reinterpretation | F-04 | owed, and `RQ-OP-02` asks whether it is a semantic family at all | owed, including the rank change a width mismatch forces | owed | owed |
| **O-17** Remaining elementwise float algebra | F-05, F-06{`subtract`, `divide`, negation} | owed | owed | owed | owed |
| **O-18** Fused multiply-add | F-07 | owed; ADR 0015 accepted, no key | owed | owed, and **must** be exact-rational | owed |
| **O-19** Extrema and clamp | F-10 | owed for all five identities; ADR 0023 accepted | owed, including the unordered-bounds refusal | owed | owed |
| **O-20** General elementary functions | F-11 | owed; the contract carrier is delivered and no key is | owed | owed | owed |
| **O-21** Bitwise and shift | F-15 | owed | owed, including the shift-amount range | owed | owed |
| **O-22** Numeric and float-to-integer conversion | F-18, F-19 | owed; ADRs 0010, 0041, 0091 accepted, no key in either direction | owed | owed | owed |
| **O-23** In-type precision reduction | F-21 | owed | owed | owed | owed |
| **O-24** Padding and cropping | F-25 | owed | owed | owed | owed |
| **O-25** Index-producing reduction | F-29 | owed; `RQ-OP-07` fixes the result count | owed | owed, a `(value, index)` carrying fold | owed; cannot reuse F-28's permissions |
| **O-26** Statistical and normed composites | F-30 | owed; `RQ-OP-08` fixes atomicity | owed | owed, the stable formulation | owed |
| **O-27** Scans and cumulative reductions | F-31 | owed | owed | owed, a serial prefix fold | owed, and constrained: the parallel form consumes reassociation |
| **O-28** Windowed reduction, pooling, convolution | F-33 | owed; `RQ-OP-09` fixes the mechanism | owed | owed | owed |
| **O-29** Scatter and indexed update | F-35 | owed; pure by the corpus's position | owed, including declared uniqueness | owed | owed |
| **O-30** Data-dependent-extent selection | F-36 | not expressible; `RQ-OP-10` owns the representation | blocked on the same | owed | owed |
| **O-31** Ordering and rank selection | F-37, F-38 | owed; `RQ-OP-11` fixes fixed-order versus comparator region | owed | owed, under a stated total order | owed |
| **O-32** Ordered search | F-39 | owed | owed, plus a sortedness value assumption | owed | owed |
| **O-33** Spectral transforms | F-40 | owed; gated on complex arithmetic that does not exist | owed | owed | owed |
| **O-34** Geometric resampling | F-41 | owed, four attributes or the family is underspecified | owed | owed | owed |
| **O-35** Dense linear-algebra decompositions | F-42 | owed; `RQ-OP-12` asks whether it is a semantic family | owed | **no exact reference exists** in the F-05 sense | owed |
| **O-36** Counter-based random generation | F-43 | owed; pure, with the algorithm in identity | owed | owed | owed |
| **O-37** Implicitly stateful and environment-observing | F-44 | **unrepresentable** — `OperationEffect` has one variant and refuses a second at three encoders | not applicable | not applicable | not applicable |
| **O-38** Non-tensor values and control constructs | F-47 | not a tensor-element question; value kinds and regions | not applicable | not applicable | not applicable |
| **O-39** Monoid reducers beyond the strict sum | F-28{product, standalone extrema, non-identity seed, variadic} | owed; the schema is fixed and uninstantiated | owed | owed | owed |

### Physical and evidence rungs

| Track | M5 index/access lowering | M6 minimum physical route | M7 backend realization | M8 bounded conformance |
| --- | --- | --- | --- | --- |
| **O-00** | delivered | covered, direct scalar route | delivered | delivered, bounded to one host and one contract |
| **O-01** | delivered | covered, direct fold | delivered | delivered, thirty bit-compared cases on one host |
| **O-02** | one capability delivered; the two- and three-region occurrences are owed | covered under the pinned formula | owed at the request boundary | measured corpora retained per family, with two divergences recorded |
| **O-03** | delivered, the eighth governed capability | covered, direct nested fold | delivered for the `direct` realization | bit-identical against retained device measurements; **no execution row** |
| **O-04** | owed | covered under a packing and parameter-broadcast route | tested non-monotone U4 evidence only | owed for the selected profile |
| **O-05** | delivered and never resolved, because no admitted program shape contains one | covered, an access-map change with no data movement | owed | delivered at the reference layer, ranks one through four |
| **O-06** | **delivered for the literal-offset form** — [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](../../../tickets/lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md) supplies the exact provider, and [`admit-an-index-realization-law-for-the-literal-offset-slice`](../../../tickets/admit-an-index-realization-law-for-the-literal-offset-slice.md) independently reconstructs `WholeAxis -> d` and `Window -> d + offset`; exact refinement compares the two, while strided and source-bearing offsets remain excluded | covered under a view where the ABI expresses the offset, a copy otherwise | owed | delivered at the reference layer |
| **O-07** | **owed, and the alternative is a fork** — a piecewise read or two write roots | covered, a windowed write with unique ownership | owed | delivered at the reference layer |
| **O-08** | **still owed, and now bounded rather than open** — [ADR 0107](../../decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md) admits the family above the index language and leaves the indirection contract to a decision constrained by ADR 0046's non-weakening condition | **not covered** — the refusal is the guarantee, and it is now a *typed* refusal at a named enforcement boundary rather than an absence | owed | **delivered at the reference layer** |
| **O-09** | owed | **not covered** — until `RQ-OP-03` closes there is nothing for a route to write into | owed | cheap and exhaustive once the carrier is decided |
| **O-10** | owed | covered, direct scalar route | owed; no target can be asked, the honourability subject is float-only | owed |
| **O-11** | owed | **not covered** — D7 is empty | owed | owed |
| **O-12** | owed | **not covered** — a call declaring no numerical contract is inadmissible | owed; lowering an opaque call is unimplemented | owed |
| **O-13** | owed; live ticket | reuses the F32 route | owed; live tickets for the vocabulary, lowering, and routing | owed end to end; a live ticket |
| **O-14** | trivially available — the coordinate is already an affine index expression | covered, an index expression | owed | owed |
| **O-15** | reuses the two admitted structural capabilities | covered, a read map or a copy | owed | owed |
| **O-16** | owed | covered **only** under a declared storage encoding | owed | owed |
| **O-17** | owed | covered, direct scalar route | owed | owed |
| **O-18** | owed | covered **only** under a target offering single rounding | owed, and a target that cannot honour it makes the operation infeasible | owed, and needs an input distinguishing one rounding from two |
| **O-19** | owed | covered, direct scalar route | the exact fixup is **delivered** for the embedded fold and reusable | owed for every standalone identity |
| **O-20** | owed | covered **only** under a vendor bound refining the declared contract | owed | owed |
| **O-21** | owed | covered, direct scalar route | owed | owed |
| **O-22** | owed | covered, a scalar convert | owed; the only realized construct is an `f32`-to-`f32` NaN canonicalization | owed |
| **O-23** | owed | covered, a scalar sequence | owed | owed |
| **O-24** | owed | covered, a guarded read; elision owes a neutrality proof | owed | owed |
| **O-25** | owed | covered, direct fold | owed; a parallel topology needs an identity-less combine | owed |
| **O-26** | owed | covered **only** under the stable formulation | owed; may need two passes | owed |
| **O-27** | owed | covered, the **serial** form only under a non-reassociating contract | owed | owed |
| **O-28** | owed | covered, a direct nested loop | owed; im2col and Winograd are different contracts, not realizations | owed |
| **O-29** | owed | covered **only** under proved unique ownership or the atomic-combine contract | owed | owed |
| **O-30** | owed | **not covered** — an allocation size is unknown before execution | owed | owed |
| **O-31** | owed | covered **only** under a stated total order and tie-break | owed | owed |
| **O-32** | owed | covered **only** under a validated sortedness assumption | owed | owed |
| **O-33** | owed | covered **only** under complex arithmetic | owed; an FFT is numerically different from the direct transform | owed, as a bound rather than an equality |
| **O-34** | owed; its route *is* a gather, so it inherits O-08's | covered **only** under the gather its coordinates are read through | owed | owed |
| **O-35** | owed | **not covered** — a vendor decomposition publishes no accumulation order, so its evidence is `Unknown` | inadmissible rather than expensive | owed |
| **O-36** | owed | covered, the named algorithm per output element | owed | owed |
| **O-37** | not applicable | **not covered** — D7 is empty | not applicable | not applicable |
| **O-38** | not applicable | **not covered** — D7 is empty | not applicable | not applicable |
| **O-39** | owed | covered, a serial pass | owed | owed |

## Owners and triggers

Each track names its owner and, where it is deferred, its trigger — written as a state of the corpus a reader can check rather than a judgement they must share. Every one of the twenty-six deferred tickets carries its trigger in its own `## Trigger check log`, ending in the command that reproduces the verdict; this section names the owner and the ground, and does not duplicate the log.

### Tracks with exact owners already

- **O-00, O-01** — delivered. The matrix records both at R6 with R7 bounded to checked target-neutral layers and one prototype execution row on one Apple M4 Max host under the flush-to-zero contract. Widening happens per target and numerical realization, never by inheritance.
- **O-02** — the three activation and normalization keys, live at R5. [`lower-a-two-region-occurrence-through-one-index-access-capability`](../../../tickets/lower-a-two-region-occurrence-through-one-index-access-capability.md) owns M5 for the multi-region occurrences, [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](../../../tickets/admit-the-registered-unary-families-at-the-compiler-request-boundary.md) owns the activation's request-boundary spelling, and [`carry-the-elementary-numerical-dimensions-in-the-region-realization`](../../../tickets/carry-the-elementary-numerical-dimensions-in-the-region-realization.md) owns the profile-level accuracy assessment M8 needs.
- **O-03** — live, and the deepest chain on the board. [`admit-a-fusion-role-for-the-tensor-contraction`](../../../tickets/admit-a-fusion-role-for-the-tensor-contraction.md) is M4, [`integrate-the-contraction-vertical-into-the-runtime`](../../../tickets/integrate-the-contraction-vertical-into-the-runtime.md) is M8's remaining residual, and the `tiled` realization sits behind two deferred staging-relation tickets. The multi-operand question is [`decide-whether-a-contraction-may-consume-more-than-two-operands`](../../../tickets/decide-whether-a-contraction-may-consume-more-than-two-operands.md) and is Tom's.
- **O-04** — live. The dtype axis's D-8 names the dependency order, which this record does not repeat.
- **O-05** — live. [`reach-a-verified-kernel-through-the-structural-families`](../../../tickets/reach-a-verified-kernel-through-the-structural-families.md) and [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](../../../tickets/admit-the-structural-families-into-the-scheduled-region-vocabulary.md) own M5's resolution and M7.
- **O-06** — fully delivered at M5 for the literal-offset grammar. [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](../../../tickets/lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md) supplies the unary-F32 capability and exact provider; [`admit-an-index-realization-law-for-the-literal-offset-slice`](../../../tickets/admit-an-index-realization-law-for-the-literal-offset-slice.md) adds the independent one-region law, revision-one standard row, and exact refinement evidence. The law sidecar grows from fifteen to sixteen rows under the unchanged `tiler.ir.index-realization-law-registry.v1` domain: append-only tag `13` preserves every old law row and the semantic snapshot while moving the complete law-registry identity and its consumers. `IndexRealizationLaw::Slice` and `slice_f32()` remain a labelled public draft awaiting Tom at [`accept-the-literal-offset-slice-realization-law`](../../../tickets/accept-the-literal-offset-slice-realization-law.md). This M5 claim includes only `WholeAxis -> d` and literal `Window -> d + offset`; the strided and symbolic relations stay at R1 behind [Q-SHAPE-008](../../open-questions.md) and the separate source-bearing selection decision respectively, and both are refused by name rather than approximated.
- **O-08** — **semantic and reference halves delivered 2026-08-07; the next step moved rather than closed.** [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](../../../tickets/admit-an-indirect-gather-family-for-tied-embedding-lookup.md) registered `tiler::gather-f32@1` under [ADR 0107](../../decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md) (`proposed`), discharging Q-SHAPE-007's bounds, determinism, and validation rules for reads and stating the duplicate-write rule it does not implement. **The *Fusion role* cell reads "not owed" rather than "owed", and the distinction is this row's most useful content.** Every other coordinate-relation family here takes `CoordinateRelation`, whose contract is that it discharges no obligation because the aliasing it introduces is the index verifier's. That is false of a gather — the verifier cannot bound a coordinate it cannot see — so the family is deliberately unclassified, `classify` returns `None`, and no region containing one derives legality at all. The next step is [`admit-the-indirect-access-class-into-the-index-layer`](../../../tickets/admit-the-indirect-access-class-into-the-index-layer.md), which is a *decision* about whether the index layer admits a data-dependent access class rather than an implementation ticket, and [`emit-the-indirect-gather-on-metal`](../../../tickets/emit-the-indirect-gather-on-metal.md) depends on it.
- **O-09** — [`scope-the-predicate-tensor-vertical`](../../../tickets/scope-the-predicate-tensor-vertical.md), deferred. `RQ-OP-03` and the [dtype ledger](../../dtype-support.md)'s `Logical bool` trigger "must close together or neither has", and the taxonomy calls this "the single highest-leverage unblocking decision in the inventory". This record adds one member the dtype-side framing did not name: F-28's logical `any` and `all` reductions belong here rather than to any reduction track.
- **O-10** — [`define-the-integer-numerical-contract-and-honourability-subject`](../../../tickets/define-the-integer-numerical-contract-and-honourability-subject.md), deferred, for the numerical and honourability obligations; [`admit-a-storage-carrier-for-integer-program-inputs`](../../../tickets/admit-a-storage-carrier-for-integer-program-inputs.md), live, for one operand shape's storage half. **The arity question that ticket excludes by name is newly owned** by [`decide-the-checked-overflow-operation-result-arity`](../../../tickets/decide-the-checked-overflow-operation-result-arity.md), which depends on it because the closure test is a worked consumer program and there is no consumer until the integer trigger fires.
- **O-11** — [`multi-device-and-sharding-scope-gate`](../../../tickets/multi-device-and-sharding-scope-gate.md), deferred, whose activation gate covers semantic collectives directly. [Vision](../../vision.md) lists the whole class as a first-implementation non-goal, so the row records a non-goal rather than an oversight.
- **O-12** — owned and deliberately closed at the boundary this record was tempted to file against. ADR 0078 classifies opaque declaration and registration as compiler-owned and crate-private, [`register-opaque-calls-on-the-compile-path`](../../../tickets/register-opaque-calls-on-the-compile-path.md) closed the in-crate reachability gap without adding a public item, and the out-of-crate seam is a decided classification rather than a missing owner. **No ticket was filed for it**, and that is the finding: the roadmap sentence naming the gap reads as unowned work and is a recorded decision.
- **O-13** — the live BF16 chain, whose eight remaining tickets the matrix's reduced-precision row enumerates by rung. This record adds nothing to it and defers to the dtype axis's D-4.

### Newly owned tracks

Each of the twenty-seven entries below is a new ticket. The ground and the trigger live in the ticket; what this list adds is the one sentence saying why the track exists as a track. Two further tickets this pass filed are not tracks and are absent from the table: [`decide-the-checked-overflow-operation-result-arity`](../../../tickets/decide-the-checked-overflow-operation-result-arity.md), which is `RQ-OP-01` inside track O-10, and [`repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row`](../../../tickets/repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row.md), a catalog defect found by this ticket's own required check. Twenty-nine tickets in total, twenty-six of them `deferred`.

| Track | Owner | Why it is a track rather than a note |
| --- | --- | --- |
| **O-07** | [`scope-the-concatenate-fusion-role-and-lowering`](../../../tickets/scope-the-concatenate-fusion-role-and-lowering.md), `todo` | A family at R4 whose R5 and R6 the matrix names and no ticket owns, with two live p1 decode tickets above it and a lowering fork that decides whether [Q-SHAPE-006](../../open-questions.md#q-shape-006--finite-piecewise-access-maps) fires |
| **O-14** | [`scope-the-structural-index-generation-family`](../../../tickets/scope-the-structural-index-generation-family.md) | Its physical route already exists as an affine index expression, which makes it the family most likely to be admitted casually and least likely to have a producer |
| **O-15** | [`scope-the-remaining-bit-preserving-structural-families`](../../../tickets/scope-the-remaining-bit-preserving-structural-families.md) | The matrix row that names bit-preserving copies also names *views*, which are physical; and repetition, a peer of broadcast, appears in no row at all |
| **O-16** | [`scope-the-bit-reinterpretation-family-against-its-storage-carrier`](../../../tickets/scope-the-bit-reinterpretation-family-against-its-storage-carrier.md) | The only family whose *semantic* result depends on a physical fact, and the only one in genuine tension with an accepted ADR |
| **O-17** | [`scope-the-remaining-elementwise-float-algebra-families`](../../../tickets/scope-the-remaining-elementwise-float-algebra-families.md) | Two signatures of F-06 are delivered and two have no key, and `divide` owes a reciprocal permission on which two delivered families already sit on opposite sides |
| **O-18** | [`scope-the-fused-multiply-add-semantic-family`](../../../tickets/scope-the-fused-multiply-add-semantic-family.md) | A different oracle obligation, a different physical precondition, and a different failure class from the algebraic families it currently shares a row with |
| **O-19** | [`scope-the-standalone-extrema-and-clamp-families`](../../../tickets/scope-the-standalone-extrema-and-clamp-families.md) | The expensive half — the exact Metal fixup that implements the total order without `fmax` — is delivered and reusable, and none of the five identities exists |
| **O-20** | [`select-the-first-general-elementary-function-keys`](../../../tickets/select-the-first-general-elementary-function-keys.md) | Q-SEM-004's machinery is delivered and its *selection* has no ticket; three landings each minted no general key on purpose |
| **O-21** | [`scope-the-bitwise-and-shift-families`](../../../tickets/scope-the-bitwise-and-shift-families.md) | The shift-amount type and range are signature fields the ecosystem routinely omits, and the float-operand form is intentionally invalid and must never become work |
| **O-22** | [`test-the-directional-conversion-pair-generalization`](../../../tickets/test-the-directional-conversion-pair-generalization.md), **answered 2026-08-05**; the rule it derived was carried to an ADR by [`land-the-conversion-pair-decomposition-adr`](../../../tickets/land-the-conversion-pair-decomposition-adr.md) and accepted as [ADR 0102](../../decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md) on 2026-08-06 | `RQ-OP-04`'s closure test named no workload, no target, and no measurement, and a second dtype was already admitted at the semantic and reference layers. The answer is in [conversion family decomposition across pairs](../numerics/conversion-family-decomposition-across-pairs.md): the per-ordered-pair key generalizes, ADR 0091's widening/narrowing field assignment does not, and no rung of this track moved — the acceptance moved none either, because the record fixes the family's shape and registers nothing |
| **O-23** | [`scope-the-in-type-precision-reduction-family`](../../../tickets/scope-the-in-type-precision-reduction-family.md) | It changes neither the type nor the numeric interpretation, so collapsing it into either conversion row would hide a rounding the row's signature does not mention |
| **O-24** | [`scope-the-padding-and-cropping-family`](../../../tickets/scope-the-padding-and-cropping-family.md) | The one structural family with a numerical participant, and the corpus already carries the counterexample proving the pad value is not neutral |
| **O-25** | [`scope-the-index-producing-reduction-family`](../../../tickets/scope-the-index-producing-reduction-family.md) | An identity-less fold with heterogeneously typed results cannot reuse the monoid reduction's topology permissions |
| **O-26** | [`scope-the-statistical-and-normed-composite-families`](../../../tickets/scope-the-statistical-and-normed-composite-families.md) | The group where "it decomposes" and "it is correct" most often disagree, with a closure test that is one worked input per member |
| **O-27** | [`scope-the-scan-and-cumulative-reduction-family`](../../../tickets/scope-the-scan-and-cumulative-reduction-family.md) | The only family whose interesting realization *consumes* a numerical permission by construction, so a strict contract leaves it with the serial form alone |
| **O-28** | [`scope-the-windowed-reduction-and-convolution-family`](../../../tickets/scope-the-windowed-reduction-and-convolution-family.md) | Its question decides whether ADR 0087's index structure is the corpus's general mechanism or the contraction's own |
| **O-29** | [`scope-the-scatter-and-indexed-update-family`](../../../tickets/scope-the-scatter-and-indexed-update-family.md) | Q-SHAPE-007's own text records that no ticket proposes one, so duplicate-write and write-determinism rules stay reserved |
| **O-30** | [`scope-the-data-dependent-extent-representation`](../../../tickets/scope-the-data-dependent-extent-representation.md) | A shape-and-allocation decision whose cheaper candidate makes *every* downstream consumer mask-aware, which is why it precedes admitting any member family |
| **O-31** | [`scope-the-ordering-and-rank-selection-families`](../../../tickets/scope-the-ordering-and-rank-selection-families.md) | Neither primary source supplies a float order to adopt, and the alternative to fixing one is the first nested region in the public graph |
| **O-32** | [`scope-the-ordered-search-family`](../../../tickets/scope-the-ordered-search-family.md) | It returns an in-range index of the right type for every input, sorted or not, which is exactly the silent-wrongness shape a declared precondition exists to prevent |
| **O-33** | [`scope-the-spectral-transform-family`](../../../tickets/scope-the-spectral-transform-family.md) | The first family that requires the complex identity to be more than recognized, and one whose accuracy contract must bound two algorithms against each other |
| **O-34** | [`scope-the-geometric-resampling-family`](../../../tickets/scope-the-geometric-resampling-family.md) | Four commonly conflated attributes decide the result, and its physical route is a gather that does not yet exist |
| **O-35** | [`scope-the-dense-linear-algebra-decomposition-family`](../../../tickets/scope-the-dense-linear-algebra-decomposition-family.md) | The sharpest case where a fallback is most tempting and the optimizer's own admissibility rule already forbids it |
| **O-36** | [`scope-the-counter-based-random-generation-family`](../../../tickets/scope-the-counter-based-random-generation-family.md) | It is **pure**, and the matrix row that would otherwise absorb it is named for effects it does not need |
| **O-37** | [`scope-the-effect-signature-opening`](../../../tickets/scope-the-effect-signature-opening.md) | Q-SEM-011 had no ticket, and the widening is a coordinated identity-domain step across three encoders rather than an enum edit |
| **O-38** | [`scope-the-non-tensor-value-kinds-and-control-constructs`](../../../tickets/scope-the-non-tensor-value-kinds-and-control-constructs.md) | Q-SEM-012 had no ticket, and its two halves — value kinds and regions — are separable and have different identified consumers |
| **O-39** | [`scope-the-monoid-reducers-beyond-the-strict-sum`](../../../tickets/scope-the-monoid-reducers-beyond-the-strict-sum.md) | The matrix row it narrows lists four semantic families and two physical topologies together, and two of its five members belong to other owners |

## Support-matrix rows this pass narrowed

The governing ticket requires correcting rows the taxonomy shows were too broad. Six rows were narrowed, and each correction is a claim about *what the row asserts*, never about a rung: **no rung moved in this pass, and none should have.** The corrections land in [the matrix](../../roadmap.md#operation-family-support-matrix) itself; this section records the derivation so a reader can refute the narrowing rather than only see it.

1. **`Remaining pointwise float algebra` held `Fma` at the same rung as `Subtract`, `Divide`, and negation.** They differ in oracle obligation, physical precondition, and failure class, and the physical profile already sorts them into different coverage classes. Split into two rows.
2. **`Cast and convert` held bit reinterpretation among the numeric conversions.** The row's R2 rests on ADRs 0010, 0041, and 0091, none of which mentions bit reinterpretation; the family has no ADR, an open `RQ-OP-02`, a rank change no conversion has, and a genuine tension with [ADR 0018](../../decisions/0018-portable-bitwise-nans.md). Split out at **R1**, which is a demotion for that member and the point of the correction. The same row's name reaches no in-type precision reduction, which has no row at all and is now owner-linked.
3. **`Reductions beyond strict sum` listed tree and multi-pass topologies as members.** Topology is physical and never semantic under [ADR 0012](../../decisions/0012-physical-reduction-topology.md) and the taxonomy's cross-family invariant 3; and logical `any`/`all` is gated on `RQ-OP-03` while product and extrema are not. One row, two axes and two blockers. Narrowed in place with each member routed to its owner.
4. **`Structural and data-movement families` listed *views*.** A view is a physical realization of a selection or a copy, not a semantic family; the taxonomy has no view row and F-03's semantic content is the bit-preserving identity. The same row reaches no repetition, which is a peer of broadcast with no row.
5. **`Effectful and stateful operations` absorbs counter-based random generation through the taxonomy's own join table.** F-43 is pure, atomic, and classified "with confidence"; reading it into this row makes a family that needs no effect vocabulary look like it waits on Q-SEM-011.
6. **`Sub-tensor selection: Slice and other non-surjective coordinate maps` names a class wider than the family.** A data-dependent gather is also a non-surjective coordinate map and is a different family with a live owner; the row's own body says *injective* non-surjective, and the title did not.

**One count moved, and it moved because it was wrong rather than because this pass changed anything.** Both the matrix and the taxonomy's join table said twenty-three of forty-seven families have no row. Twenty-three is the count of families *with* a row; the join table's own no-row cell listed twenty-four, which is one line to check — count the `F-nn` tokens in the cell. Correction 5 then moves F-43 out of the effectful row, making **twenty-five**. Neither split changed the count, because splitting a row moves a family between rows without giving one to a family that had none.

**What was deliberately not corrected.** No row was added for any of the twenty-five. Doing so would convert a tracked gap into twenty-five ledger entries claiming R1, which is a maturity claim none of them has earned — an owner is not a rung, and this record's whole job is to supply the first without implying the second.

## Product questions this pass did not answer

**One stop condition fired, and it fired as designed.** Track **O-07**'s lowering has two surviving alternatives — a piecewise read, or two write roots partitioning one output — which encode different priorities rather than one dominating the other: the first widens the access language and fires Q-SHAPE-006 for every later family; the second stays inside the admitted language and needs a multiple-writer coverage proof the model states but has never exercised. **It is not escalated to Tom**, because the elimination has not been run yet and running it is research rather than a decision — the ticket's deliverable is the elimination, and only a genuine two-survivor result after it would be Tom's. The alternative was recorded, the ticket was filed at `todo` with the fork in its body, and the reachable remainder of this pass continued.

**No taxonomy row contradicts an accepted ADR.** The second stop condition was checked family by family and did not fire. Three near misses are worth recording because each *looks* like a contradiction and is not. F-04's bit preservation against ADR 0018's NaN canonicalization is a tension the taxonomy states and resolves by inference — canonicalization is a property of arithmetic result materialization and F-04 performs none — and the record is explicit that this is "an inference from two accepted positions, not an accepted rule". F-28's contributor-order requirement against StableHLO's implementation-defined ordering is a disagreement with a *source*, not with an ADR, and the corpus's opposite choice is what ADRs 0012, 0013, and 0014 accept. F-32's contraction-order exploration reads as blocked by an unmade decision and is not: [ADR 0095](../../decisions/0095-decline-a-distributivity-permission.md) **declined** the distributivity permission on 2026-08-01, so the regroup is withheld by decision rather than pending one.

**Two questions remain Tom's and are unchanged by this pass.** Whether a semantic contraction may consume more than two operands ([Q-SEM-015](../../open-questions.md), owned by [`decide-whether-a-contraction-may-consume-more-than-two-operands`](../../../tickets/decide-whether-a-contraction-may-consume-more-than-two-operands.md)), and the acceptance of every public boundary any track above would eventually reach. Neither was reopened and no track pre-authorizes either.

## Where the tracks meet the dtype axis

[Dtype-family research tracks](../numerics/dtype-family-research-tracks.md) already states five joins between the two axes, from the dtype side. Read from this side they are the same five, and this record adds no sixth: `RQ-OP-03` joins the predicate track **D-1** (the two must close together or neither has), `RQ-OP-01` joins the integer track **D-2**, `RQ-OP-04` joins **D-3** and every track whose conversion obligation is owed, `RQ-OP-12` joins the complex track **D-6**, and `RQ-OP-02` joins the packing track **D-10** and no element track at all.

**One asymmetry between the two records is worth naming.** The dtype record's coverage table maps every catalog row to a track and every track to at most one ticket. This record's O-13 maps to *no* families and O-09 and O-10 map to tickets the dtype axis filed. That is not duplication: a predicate tensor is one decision with an operation half and a dtype half, and filing an operation-side ticket for it would have created the second half of a decision the corpus has already said must close as one.

## Findings a reader should act on

Seven things this pass established that were not previously statable. Each is refutable from a source named beside it.

1. **The support matrix's seven rungs do not resolve the two rungs that are currently binding.** When this pass ran, three families sat at R5 with three different blockers — a missing `ScalarProgram` spelling, a one-region-per-occurrence lowering limit, and a request boundary admitting no program shape containing the occurrence — each owned by a different ticket. **All three were discharged by 2026-08-06 and the finding held through them.** The activation and the two structural families reached R6, and the two families still at R5 are separated by exactly this split: the normalization is blocked at M6 — since later on 2026-08-06 by the missing program-assembly declaration rather than the scheduled-region spelling, which landed — and the softmax at M5 by a missing `IndexRealizationLaw`. The M5/M6 split is what separates them, and a reader scheduling from the rung alone would treat them as one wall.
2. **Twenty-six of the twenty-nine tickets filed are deferred, and in eight cases the non-firing is a recorded elimination rather than an absence of interest.** The workload samples its logits on the consumer side (O-25, O-31); its mask is an additive `f32` input (O-09); its KV growth is a concatenation over consumer-owned tensors (O-29); its rotary tables and positions are bound inputs (O-14); three delivered composite families each minted no general elementary key on purpose (O-20); the one measured vendor-library candidate published nothing admissible (O-35); the first quantized profile selected unpacked storage (O-16, transitively); and the sole in-graph multiply-add adjacency declares ADR 0015's permission forbidden (O-18). A deferral backed by an elimination is stronger evidence than one backed by silence.
3. **Two open questions in `docs/open-questions.md` had no ticket at all, and both are reservations the corpus depends on.** Q-SEM-011's effect signature and Q-SEM-012's non-tensor value kinds and control constructs are now O-37 and O-38. Until this pass, the only thing standing between the corpus and a widened `OperationEffect` was a compile error at three encoders — which is a good mechanism and is not an owner.
4. **`RQ-OP-04` is the one taxonomy question whose closure test is already satisfiable, and it is nearer to load-bearing than its priority suggests.** Its test names no workload, no target, and no measurement; a second dtype is registered and reference-evaluated at three keys; and the matrix's own cast-and-convert row states that admitting a second dtype into a profile forces the conversion row. Whichever shape the first registered conversion takes becomes the precedent every later pair is read against. **Resolved 2026-08-05, and the prediction held while the test did not.** [Conversion family decomposition across pairs](../numerics/conversion-family-decomposition-across-pairs.md) answered it without a workload, a target, or a measurement, exactly as this conclusion said it could. What that record also found is that `RQ-OP-04`'s stated falsification test could not decide the question it was written for: the test fires on `bf16`/`f16`, whose two directions' field sets intersect at `rounding`, and the candidate it hands victory to is one ADRs 0010, 0041, and 0091 had each already rejected. A closure test is a claim like any other, and this is the first one in this corpus observed to be unsound.
5. **One roadmap sentence that reads as an unowned gap is a decided classification.** "What remains absent is out-of-crate registration — `OpaqueCallDeclaration` and `OpaqueCallRegistry` are crate-private" describes ADR 0078's correction, not a missing owner. This pass considered filing against it and did not, which is recorded here so the next reader does not file it either.
6. **Six matrix rows asserted a shared rung across members that share no correctness argument, and none of the six was a rung error.** Every correction narrows what a row *says*; not one moves what it *claims*. That is the shape a ledger correction should have, and it is worth stating because the opposite — quietly moving a rung while narrowing a row — would be indistinguishable in a diff.
7. **The corpus's most-cited number about this axis was wrong by two, in both documents that carried it.** "Twenty-three of forty-seven families have no matrix row" is the taxonomy's stated main practical output and the matrix repeats it; twenty-three is the count of families *with* a row, the join table's own no-row cell listed twenty-four, and reclassifying F-43 makes twenty-five. The check is one line — count the `F-nn` tokens in the join table's no-row cell — and it was available from the day the number was written. A number that names its own population is checkable; this one named a population it did not count.

## Coverage: every taxonomy family reaches a track

Families are the taxonomy's own `## Enumerated inventory at a glance` block, read in its order. Every family appears exactly once except F-06 and F-28, whose signature partitions are split across tracks for the reasons [How the partition was derived](#how-the-partition-was-derived) states; a family with no track would be a defect in this record rather than a gap in the inventory.

| Family | Track | Support-matrix row |
| --- | --- | --- |
| F-01 dense constant | O-00 | Pointwise `f32` constants and separate-rounding arithmetic |
| F-02 structural index generation | O-14 | *(none)* |
| F-03 identity and bit-preserving copy | O-15 | Structural and data-movement families |
| F-04 bit reinterpretation | O-16 | Bit reinterpretation *(split out of Cast and convert by this pass)* |
| F-05 unary float arithmetic | O-17 | Remaining pointwise float algebra |
| F-06 binary float arithmetic | O-00 for `add` and `multiply`; O-17 for `subtract`, `divide`, negation | Pointwise `f32` …; Remaining pointwise float algebra |
| F-07 fused multiply-add | O-18 | Fused multiply-add *(split out by this pass)* |
| F-08 integer arithmetic | O-10 | Integer data arithmetic |
| F-09 integer division and remainder | O-10 | Integer division and remainder |
| F-10 extrema and clamp | O-19 | `Minimum`/`Maximum`, `MinimumNumber`/`MaximumNumber` |
| F-11 elementary transcendentals | O-20 | Pointwise transcendentals as general keys |
| F-12 named activations and normalizations | O-02 | Elementwise activation; Normalization; Attention normalization |
| F-13 comparison | O-09 | *(none)* |
| F-14 logical operations | O-09 | *(none)* |
| F-15 bitwise and shift | O-21 | *(none)* |
| F-16 classification predicates | O-09 | *(none)* |
| F-17 elementwise selection | O-09 | `Select` and bit-selecting operations |
| F-18 numeric conversion | O-22 | Cast and convert |
| F-19 float-to-integer conversion | O-22 | Cast and convert |
| F-20 quantize/dequantize/requantize/rescale/assemble | O-04 | `QuantizeStrictAffine` and siblings |
| F-21 in-type precision reduction | O-23 | *(none)* |
| F-22 axis-structural bijections | O-05 | Structural and data-movement families |
| F-23 broadcast | O-05 | Structural and data-movement families |
| F-24 sub-tensor selection | O-06 | Sub-tensor selection |
| F-25 padding and cropping | O-24 | *(none)* |
| F-26 concatenation and stacking | O-07 | Sequence extension |
| F-27 repetition and tiling | O-15 | *(none)* |
| F-28 monoid axis reduction | O-01 for `sum`; O-39 for the remaining numeric reducers; O-09 for `any` and `all` | Strict serial `f32` `Sum`; Reductions beyond strict sum |
| F-29 index-producing reduction | O-25 | *(none)* |
| F-30 statistical and normed composites | O-26 | *(none)* |
| F-31 scans and cumulative reductions | O-27 | *(none)* |
| F-32 tensor contraction | O-03 | Tensor contraction |
| F-33 windowed reduction, pooling, convolution | O-28 | *(none)* |
| F-34 gather and indexed read | O-08 | Indirect gather |
| F-35 scatter and indexed update | O-29 | *(none)* |
| F-36 data-dependent-extent selection | O-30 | *(none)* |
| F-37 sort and argsort | O-31 | *(none)* |
| F-38 top-k and rank selection | O-31 | *(none)* |
| F-39 ordered search | O-32 | *(none)* |
| F-40 spectral transforms | O-33 | *(none)* |
| F-41 geometric resampling | O-34 | *(none)* |
| F-42 dense linear-algebra decompositions | O-35 | *(none)* |
| F-43 counter-based random generation | O-36 | *(none; **not** the effectful row)* |
| F-44 implicitly stateful and environment-observing | O-37 | Effectful and stateful operations |
| F-45 opaque, custom, and composite calls | O-12 | *(none)* |
| F-46 collective and cross-device | O-11 | *(none)* |
| F-47 non-tensor values and control constructs | O-38 | *(none)* |

Twenty-five families have no matrix row — the count that reads *(none)* in the column above, corrected from twenty-three for the two reasons [Support-matrix rows this pass narrowed](#support-matrix-rows-this-pass-narrowed) states. None gained a row in this pass, and each gained an owner.

## What would make this record wrong

Four assumptions, named so a later reader can test them rather than inherit them.

- **That a track's families share obligations at the granularity a ticket is scheduled at.** If a workload needs `subtract` and only `subtract`, O-17's grouping with `divide` costs a split rather than buying reuse — and the correct response is to split the track, not to widen the ticket. The same test applies to O-15 and O-31, the two other groupings this record made on an implementation-sharing argument rather than a numerical one.
- **That every trigger is checkable by a reader who was not here.** Each is written as a state of the corpus and each track ticket's log ends in the command that reproduces its verdict. A trigger requiring knowledge of what somebody intended has already failed, and the check most at risk is O-28's second route — "a proposal is made to widen the contraction's index structure" — which is a state of the board rather than of the tree.
- **That the matrix stays the single owner of delivered state.** The moment a maturity claim is stated here and not there, this record becomes a second ledger nothing keeps honest. The tables above therefore quote and never compute, and the six corrections narrow assertions without moving a rung precisely so that the ledger's own claims stay the only ones.
- **That the eight rungs are the right eight.** They come from the governing ticket and they resolved two walls the matrix's seven did not. If a future family's blocker falls between M6 and M7 — a schedule vocabulary gap that is neither a baseline route nor a backend emission — the vocabulary is short by one and the honest repair is a ninth rung, not a cell that quietly means two things.
