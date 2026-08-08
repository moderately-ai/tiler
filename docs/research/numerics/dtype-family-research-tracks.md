---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.dtype-family-research-tracks"
kind: "research"
title: "Dtype-family research tracks"
topics: ["numerics", "dtypes", "taxonomy", "roadmap"]
catalog_group: "dtypes-quantization"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.numerical-semantics"]
ticket: "derive-dtype-family-research-tracks-from-the-mature-taxonomy"
---

# Dtype-family research tracks

- **Status:** ownership map over an already-enumerated inventory. It selects no dtype, registers nothing, and moves no ledger cell.
- **Ticket:** [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](../../../tickets/derive-dtype-family-research-tracks-from-the-mature-taxonomy.md).
- **Research date:** 2026-08-04.

## Traceability

- **Current disposition:** pending. The tracks below are a partition and an owner assignment, not adopted contract text.
- **Inventory this record consumes:** the [mature tensor dtype taxonomy](mature-dtype-taxonomy.md) owns the semantic universe. Every row of its `## Enumerated catalog at a glance` is accounted for in [Coverage](#coverage-every-taxonomy-row-reaches-a-track) below.
- **Delivered state this record consumes and never restates:** the [dtype support ledger](../../dtype-support.md) owns what is built at each layer, its cell vocabulary, its thirteen-rung addition recipe, and its four family dry runs. Where this record names an obligation, the ledger's rung is cited so the two vocabularies stay joinable rather than parallel.
- **Companion inventory on the other axis:** the [mature tensor operation and signature taxonomy](../semantic-graph/mature-operation-and-signature-taxonomy.md) owns the operation universe and its twelve `RQ-OP` questions. Five of those questions gate a dtype track here; [Where the tracks meet the operation axis](#where-the-tracks-meet-the-operation-axis) states the joins rather than duplicating the questions.
- **Normative destination, when a track eventually delivers:** [Numerical semantics](../../numerical-semantics.md).
- **Preserved primary sources:** [dtype primary-source record](sources/README.md). This record introduces no new source; every format fact it relies on is already pinned by the taxonomy.

## Purpose and boundary

The taxonomy enumerates the dtype universe and the ledger records what is delivered. Neither answers the question this record answers: **for each family, who owns the next step, and what would have to become true for that step to start.** Without that answer a family with no current workload is indistinguishable from a family nobody thought about, and the two need opposite treatment — the first is a deferral with a trigger, the second is a defect.

Three things this record deliberately does not do.

- **It does not move a ledger cell.** Every maturity claim below is quoted from the [dtype support ledger](../../dtype-support.md), which remains the sole owner of delivered state. A track's existence is not evidence of support, and the ledger's closing rule applies unchanged: an implemented generic provider, opaque byte carrier, enum variant, or target measurement never promotes an unregistered family.
- **It does not select a dtype or file an implementation ticket.** The ledger's `## Graph policy` fixes that a cell becomes an implementation ticket only when a named producer and consumer select the exact dtype, operation set, workload, target, physical layout, numerical contract, runtime predicates, cost boundary, and conformance corpus. Every track below whose trigger has not fired is filed `deferred`, so the scheduler cannot offer it as work.
- **It does not propose a key, a Rust spelling, or a public boundary.** Those are reserved to Tom under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md).

## How the partition was derived

The governing ticket names seven partitions — booleans; signed and unsigned integers; IEEE and reduced-precision floats; FP8/FP6/FP4; complex; quantized compound values with their scales and zero points; and opaque or extension carriers. A partition is only useful if its members genuinely share obligations, because the point of a track is that answering it once answers it for every member. Applying that test to the taxonomy's rows splits three of the seven and adds three the seven do not name.

**Splits.**

- **IEEE binary floats split from IEEE decimal floats.** They share an authority (IEEE 754-2019) and nothing else that a track pays for: decimal carries two distinct storage encodings of one logical format — the taxonomy records that "DPD and BID are separate storage encodings, and a bit-preserving operation or ABI must distinguish them" — while a binary float has one. A single track would owe the decimal storage question to every binary format that does not have it.
- **Reduced-precision OCP floats split from IEEE binary floats.** The taxonomy classifies `f8E8M0FNU` as "unsigned exponent-only scale value... not an ordinary signed arithmetic dtype", and records that the OCP FP4/FP6 formats have "neither NaN nor infinity". A track that assumed the IEEE special-value set would carry a false exceptional-value obligation for four of its six members.
- **Affine quantized values split from block-scaled compound values.** The ledger states the ground directly: "The only parameter-index map that exists is per-tensor, which is the wrong association for a 32-element block", and "a non-per-tensor map is a prerequisite for MX rather than a substitute for it". An MX scheme identity is a `QuantSchemeKey` under [ADR 0038](../../decisions/0038-recognize-ocp-mx-schemes.md), not a widening of the affine scheme, so the two tracks share a representation layer and no numerical obligation.

**Additions the taxonomy forces.**

- **Sub-byte and packed storage encodings are a cross-cutting track, not a member of any element track.** The taxonomy's conclusion 6 fixes that "packing belongs to storage/encoding contracts", and `bool`, `i2/u2`, `i4/u4`, and the FP4/FP6 elements each have the same packing obligations and none of the same value obligations. Folding packing into four element tracks would answer bit order, tail, alignment, and neighbour-safe writes four times.
- **Execution-only and target ABI formats are a track precisely because they are not element types.** TF32, PTX `.ue4m3`/`.ue8m0`, `x86_fp80`, and `ppc_fp128` share exactly one obligation — staying out of logical identity while remaining statable as a physical fact — and the taxonomy's conclusion 7 is what says so.
- **Nonnumeric tensor element domains are a track because they are real dtypes in real ecosystems.** ONNX has string tensors; the taxonomy lists five such domains and classifies each. Their shared obligation is that each needs offsets, buffers, lifetimes, and its own operation family before any of them is a tensor element at all.

**What is deliberately not a track.** Non-tensor graph values — tokens, resources, handles, typed PRNG keys, shape and index values, tuples, futures, and control values — and sparse or ragged representations are outside the dtype axis by the ledger's own `### Sparse, ragged, and non-tensor values` section, which places them there "instead of occupying misleading reservation cells". [Routed elsewhere](#families-routed-off-the-dtype-axis) names the owner each of them does have, so that "no family disappears" holds without inventing a dtype track for a non-dtype.

## The nine obligations, and how they join the ledger's rungs

The governing ticket requires nine facts per partition. They are not a second vocabulary competing with the ledger's thirteen rungs; they are the same obligations at the granularity a family track is scheduled at. The join is stated once here so no track restates it.

| Obligation | What the cell fixes | Ledger rung |
| --- | --- | --- |
| **Semantic identity** | The canonical key, its immutable descriptor, and the normative reference the descriptor's fields are read from. | 1 Recognition, 2 Descriptor adequacy |
| **Host/reference carrier** | The exact value set and the host oracle that decides it, together with the evidence class that oracle can honestly claim. | 4 Reference semantics |
| **Conversion behaviour** | The ordered directional pairs into and out of the family, each with its own rounding, exactness, overflow, and special-value fields. | 5 Operation signatures, restricted to the conversion families |
| **Exceptional values** | NaN, infinity, signed zero, subnormal, overflow, and — for the integer and quantized families — zero divisor and out-of-domain codes. | 3 Accuracy-metric compatibility, 6 Numerical policy |
| **Constant encoding** | What literal a program may write for the family and what bytes it becomes, given that an unrepresentable literal is a construction refusal rather than a rounded value. | 5 Operation signatures, at the constant family |
| **Artifact ABI** | Binding, transport, the storage tag, and what the family contributes to durable program identity. | 9 Physical carrier, 10 ABI and artifact identity |
| **Scalar/KIR support** | The kernel type, its spelling, and every total map the type must enter before an emission gap becomes an unstated fact. | 12 Backend lowering, at the kernel-vocabulary half |
| **Backend capability** | Per-`(target family, dtype)` dispatchability and the honourable numerical realization the target can be asked for. | 6 Numerical policy, 11 Target dispatchability, 12 execution |
| **Conformance** | The corpus that would catch a regression, and whether it can be exhaustive-finite or must state a bounded profile. | 13 Conformance |

Two properties of the join matter for reading the tables below. The obligations are **not a total order** — the ledger's own non-monotonicity holds here, and U4/F32 having a tested physical carrier with no optimizer legality is the worked case. And an obligation marked *reuses* is a claim about the shape of the answer, never about the answer: BF16 and F16 reuse the exact-rational oracle shape, and they have different value sets.

## The tracks

Fifteen tracks. Three have exact owners already and gain no ticket; twelve are newly filed, and one of those twelve is the unowned remainder of a family whose other members are owned rather than a new family.

### Semantic obligations

| Track | Semantic identity | Host/reference carrier | Conversion behaviour | Exceptional values | Constant encoding |
| --- | --- | --- | --- | --- | --- |
| **D-0** IEEE `f32` | Delivered and tested | Delivered and tested | Owed for every pair leaving F32; none exists | Delivered and tested for the governed contract | Delivered and tested |
| **D-1** Predicate `bool` | Registered, two-valued, deliberately no logical width | Owed: boolean algebra, trivially exhaustive over two values | Owed as *conversion families*, not promotion: no operation admits `bool` at any arity today | None — no numerical content; a ULP contract over it is intentionally invalid | Owed; a two-valued literal is not an `i1` literal |
| **D-2** Signed and unsigned integers | Registered for all twelve widths with signedness class and logical width | Owed: exact integer arithmetic at unbounded precision, then the family's own overflow rule | Owed in both directions; float-to-integer is four accepted families under [ADR 0041](../../decisions/0041-separate-float-to-integer-conversion-families.md) | Owed and **different in kind**: overflow is a semantic choice, not a rounding mode; zero divisor and signed `MIN rem -1` are contract text already | Owed; the code domain is derived from class and width rather than stored |
| **D-3** IEEE `f16`, `f64`, `f128` | Registered with complete structural and special-value descriptors | Owed; reuses BF16's exact-rational shape, **but F64 and F128 cannot reuse its exhaustive round trip** | Owed per ordered pair; [ADR 0091](../../decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md)'s disjoint-field derivation is specific to BF16/binary32 and does not transfer — walked and confirmed at every F16, F64, and F128 pair by [conversion family decomposition across pairs](conversion-family-decomposition-across-pairs.md), which also finds `bf16`/`f16` the one pair whose two directions owe **intersecting** field sets | Reuses the IEEE set; F16's subnormal behaviour is measured and differs from BF16's on the same Apple row | Reuses |
| **D-4** BF16 | Delivered | Delivered for three keys, exhaustive-finite over 65,536 encodings | Accepted by ADR 0091, **registered in neither direction** | Delivered for three keys | Delivered for `tiler::constant-bf16@1` |
| **D-5** OCP reduced-precision floats and E8M0 scale data | Registered: six identities with widths, bias, and a Boolean per special member | Owed; 256 or fewer encodings each, so exhaustive-finite is available to every member | Owed; the FP8 pair are commonly conversion or MMA operands without general scalar arithmetic | Owed and **member-specific**: E4M3FN has NaN with no infinity, FP4/FP6 have neither, E8M0FNU has NaN and no zero, sign, infinity, or subnormals | Owed; E8M0FNU is scale data and a scale literal is not an arithmetic literal |
| **D-6** Complex | Registered as `tiler::complex@1<ComponentTypeKey>` over exactly f16/f32/f64 | Owed over the component's own oracle, per component | Owed; a component-changing conversion is two conversions or one, and that is the family's first question | Owed; the component's set, plus the branch cuts the taxonomy's trigger already names | Owed as an ordered real-then-imaginary pair, which the descriptor already fixes |
| **D-7** IEEE decimal32/64/128 | Registered with interchange width and coefficient precision | Owed; decimal128 is not exhaustively enumerable | Owed; decimal-to-binary is not a widening or a narrowing in the binary sense | Owed; the IEEE decimal set is not the binary set | Owed, and **twice**: a decimal literal's DPD bytes and its BID bytes differ |
| **D-8** Affine quantized values, scales, and zero points | Delivered for two per-tensor strict-affine contracts; per-axis selected and unregistered | Delivered for U4/F32 and U8/F32 | `Quantize`, `Dequantize`, `Assemble` delivered; `Requantize` and integer `Rescale` are reservations | Delivered: NaN refused, infinity saturating, positive-normal scale, nearest-even | Delivered for the two contracts |
| **D-9** Block-scaled compound values | Six OCP MX scheme identities registered; **no MX value can be constructed** | Owed, and different in kind: a value is a block, so the oracle must model the shared scale | Owed; the conversion, rounding, and saturation rules stay in the pinned `ocp-mx-v1.0` record and are not restated in any descriptor | Owed, including block-wide special-value rules that have no scalar analogue | Owed; a block literal is not an element literal |
| **D-10** Sub-byte and packed storage encodings | Not an identity layer at all — `StorageEncodingKey`, per the admission policy | Not applicable; packing changes bytes, not values | Not applicable; a repack is not a numeric conversion | Not applicable | Owed indirectly: a packed constant's bytes depend on bit order and tail |
| **D-11** Execution-only and target ABI formats | **Deliberately none.** Promoting one into logical identity requires a separate semantic decision | Not applicable | Owed as a *boundary*: where a compute precision is entered and left | Owed for the format as a physical fact, never as an element contract | Not applicable |
| **D-12** Reserved numeric extension families | Owed through the extension boundary, not the built-in catalog | Owed per family; posit's exact-accumulation `quireN` has no analogue in any admitted family | Owed per family | Owed per family; UNORM/SNORM endpoint behaviour is the whole content of those two | Owed per family |
| **D-13** External and vendor-owned identities | **Owed, and it is the track's whole first question**: ownership direction is fixed, registration and collision governance are not | Owed by the provider, not by Tiler | Owed by the provider | Owed by the provider | Owed by the provider |
| **D-14** Nonnumeric tensor element domains | Owed; each domain is a separate identity question | Owed; a string oracle is not a numeric oracle | Owed, and mostly parsing rather than rounding | Not applicable in the numeric sense; validity and encoding errors replace them | Owed; a string literal needs offsets and a buffer |

### Physical and execution obligations

| Track | Artifact ABI | Scalar/KIR support | Backend capability | Conformance |
| --- | --- | --- | --- | --- |
| **D-0** IEEE `f32` | Delivered and tested | Delivered and tested | Delivered and tested on one measured macOS row | Delivered and tested |
| **D-1** Predicate `bool` | Owed, and it is the decision the family turns on: bit, byte, or another ABI width. `AbiType::Boolean` is a control predicate and does not answer it | `KernelType::Bool` exists as a control predicate and is not tensor support | Owed per family; reuses rung 11 unchanged | Cheap and exhaustive over two values, once the carrier is decided |
| **D-2** Signed and unsigned integers | Owed; byte-width integers reuse, sub-byte widths depend on D-10 | Owed; `KernelType::{Index,U8,I32}` are address, carrier, and widened-subtract machinery | **Owed and unstated**: the honourability subject vocabulary covers floating-point scalar arithmetic only, so no target can declare that it honours a wrapping or saturating add | Reuses the float shape and is cheaper; exhaustive-finite is available at the narrow widths |
| **D-3** IEEE `f16`, `f64`, `f128` | Reuses | Reuses | Owed per row; F16 has measured Apple evidence and F64 is absent on that row | **F16 reuses (65,536 encodings); F64 and F128 must state a bounded profile** |
| **D-4** BF16 | Owed; a live ticket carries it | Owed; a live ticket carries it | Dispatchability and the subnormal tables are declared on one macOS Apple9 profile row; lowering and execution are owed | Owed end to end; a live ticket carries it |
| **D-5** OCP reduced-precision floats and E8M0 scale data | Owed; FP4 and FP6 carriers depend on D-10 | Owed | Owed; a format may be an MMA operand with no general scalar arithmetic, which is a rung-12 distinction rather than a rung-11 one | Exhaustive-finite is available per element format |
| **D-6** Complex | Owed; planar versus interleaved is physical under [ADR 0037](../../decisions/0037-parameterize-complex-dtype-identity.md), so one logical value may be two buffers | Owed | Owed | Owed; component-wise exhaustiveness does not compose to the pair |
| **D-7** IEEE decimal32/64/128 | Owed, and **twice** — DPD and BID are distinguishable encodings a bit-preserving ABI must not conflate | Owed | Owed, and weak everywhere in current GPU arithmetic | Owed; decimal32 is enumerable, decimal128 is not |
| **D-8** Affine quantized values, scales, and zero points | Delivered for U4 with role-preserving structural ABI; per-axis component access is a live ticket | Delivered for the U4 dequantization path | Refused by the governed Metal profile because the strict contract's F32 subnormal preservation is unavailable; the selected profile's device claim is owed | Delivered for what is tested; the selected profile's exact-bits criterion is owed |
| **D-9** Block-scaled compound values | **Owed and structurally different: one logical value maps to two physical buffers**, so the one-buffer-per-value assumption breaks here | Owed; the block map has no per-block parameter-index representation today | Owed | Owed; the corpus is over blocks, not elements |
| **D-10** Sub-byte and packed storage encodings | **The track's core**: bit order, cross-byte layout, tail, alignment, unaligned access, neighbour-safe writes, and repacking beyond the governed whole-component U4 path | Owed for every packed carrier the vocabulary admits | Owed per target; an extraction expression that has never been dispatched is not backend capability | Owed; a packing corpus is over layouts, not values |
| **D-11** Execution-only and target ABI formats | Owed as a physical fact if a backend operation needs one | Owed as a kernel-vocabulary reservation | **This is where the whole track lives**: an execution precision is a target capability and nothing else | Owed as target evidence, not as format conformance |
| **D-12** Reserved numeric extension families | Owed by the extension provider | Owed by the extension provider | Owed by the extension provider | Owed by the extension provider, and versioned |
| **D-13** External and vendor-owned identities | Owed by the provider | Owed by the provider | Owed by the provider | **Versioned conformance is what establishes equivalence.** Similar spelling, width, or descriptor shape never does |
| **D-14** Nonnumeric tensor element domains | Owed; offsets and variable-length buffers are not the tensor ABI | Owed, and probably not in the numeric kernel vocabulary at all | Owed | Owed |

### Owners and triggers

Each entry names the exact owner, the ground in the taxonomy or the ledger, and — where the track is deferred — the trigger, stated so that a reader can check whether it has fired rather than judge whether it feels near.

#### D-0 — IEEE `f32`

**Owner: delivered; no track.** Ground: the taxonomy's `f32` row, "established portable core logical/storage/compute format". The ledger's `### IEEE f32` records a tested guarantee at every layer but runtime semantic validation and target-family dispatchability, bounded to one Apple M4 Max host under one explicitly selected flush-to-zero contract. Widening happens only from a named workload, per that section's own trigger.

#### D-1 — Predicate `bool`

**Owner: [`scope-the-predicate-tensor-vertical`](../../../tickets/scope-the-predicate-tensor-vertical.md), deferred.** Ground: the taxonomy's Predicate row and its conclusion 3, "Predicate is a logical scalar independent of physical bit/byte packing". The ledger records `tiler::bool@1` registered with deliberately no logical width, `tiler::i1@1` carrying no authority, and no registered operation admitting a `bool` operand at any arity.

**Trigger.** A named workload requires a `Select`, a comparison, a logical reduction, or a boolean mask as a tensor value. **It has not fired, and the elimination is recorded rather than assumed:** the first attention program vertical binds a host-built **additive** causal mask as an `f32` input of extent `[T, S]`, so the live workload reaches masking without a predicate tensor at all.

**Operation-axis intersection.** `RQ-OP-03` gates F-13 comparison, F-14 logical operations, F-16 classification predicates, F-17 elementwise selection, F-28's logical-reduction case, and F-36's mask case on the same missing decision, and states that this ledger trigger and that question "must close together or neither has". The operation record calls the group "the single highest-leverage unblocking decision in the inventory". Neither side may be closed alone.

#### D-2 — Signed and unsigned integers

**Owner: [`define-the-integer-numerical-contract-and-honourability-subject`](../../../tickets/define-the-integer-numerical-contract-and-honourability-subject.md), deferred, for the numerical and reference obligations. The storage obligation has a separate live owner.** Ground: the taxonomy's Signed and Unsigned integer rows and its conclusion 2. The ledger records all twelve widths registered with a signedness class and a logical width, U4's and U8's code domains tested in the reference provider, and no general integer evaluator, optimizer, or backend vertical.

**The exact gap, stated so it can be refuted.** [Numerical semantics](../../numerical-semantics.md) `### Per-dimension honourability` closes with "The subject vocabulary covers floating-point scalar arithmetic. Integer overflow families, boolean semantics, quantized compound schemes, and any future policy family have their own contracts elsewhere in this document and acquire no honourability declaration by standing beside this one." The operation contracts do exist — `## Integer and index arithmetic` names the wrapping, saturating, checked, and widening families and the six division families, under [ADR 0039](../../decisions/0039-explicit-integer-overflow-operations.md) and [ADR 0040](../../decisions/0040-specialize-integer-division-families.md). What does not exist is a *subject* an integer family could be declared honourable at, which is the same shape of seam `admit-a-bf16-scalar-arithmetic-subject` opened for floats — and the measured evidence is already on the far side of it, because [`measure-code-domain-integer-arithmetic-on-the-qualified-apple-row`](../../../tickets/measure-code-domain-integer-arithmetic-on-the-qualified-apple-row.md) measured a `u8` read, an `int` subtraction, and an `int`-to-`float` conversion on the qualified Apple row with nowhere to declare the result.

**Trigger.** A named tensor workload selects an exact width, an operation family, an overflow, division, or conversion behaviour, a storage, a target, and a corpus. **It has not fired.** The ledger's own trigger says why the nearest candidate does not count: "Quantized codes alone do not trigger an integer arithmetic vertical."

**Partial owner that is live.** [`admit-a-storage-carrier-for-integer-program-inputs`](../../../tickets/admit-a-storage-carrier-for-integer-program-inputs.md) owns the storage half for one operand shape — a `[T]` token-ID input reaching a program as an integer — and states the carrier question as a public boundary for Tom. It selects no arithmetic family, so it advances D-2's artifact-ABI obligation and none of the numerical ones.

**Operation-axis intersection.** `RQ-OP-01` owns whether a checked-overflow operation returns one result plus a validated precondition or two results with an explicit overflow predicate, and it blocks F-08. F-09 is decided by ADR 0040 and F-15's bitwise and shift families are gated on D-1's predicate decision only for their result type, not their operands.

#### D-3 — IEEE `f16`, `f64`, and `f128`

**Owner: [`state-the-non-enumerable-float-conformance-profile`](../../../tickets/state-the-non-enumerable-float-conformance-profile.md), deferred.** Ground: the taxonomy's binary-float table and the ledger's `### Other IEEE binary floats and BF16`, which records all four registered with complete descriptors, F16 and BF16 conformance evidence on the exact Apple host and family rows, and no F64 or F128 evidence at all.

**The obligation that is family-specific rather than reused.** The ledger's dry run states it in its own words: reference semantics "reuses the exact-rational shape... but F16 alone is exhaustively enumerable, so **F64 and F128 must state a bounded profile** in place of BF16's 65,536-encoding round trip", and rung 13 repeats it for conformance. Nothing in the corpus says what a bounded profile for a non-enumerable format must state, and the four evidence classes it could claim are not interchangeable — `exhaustive-finite` is unavailable, and a sampled corpus is `bounded-measurement` whose universe must be named.

**Trigger.** A named workload selects `f16`, `f64`, or `f128`. **It has not fired.** A second target measurement alone does not trigger registration, which the ledger's own trigger for this section says explicitly.

#### D-4 — BF16

**Owner: no live rung ticket; every remainder names an authority instead.** ~~Owner: the live BF16 track; no new ticket.~~ **Corrected 2026-08-08 — that read "the live BF16 track", which was true when written and is not now:** all eight rung owners below are `done`, so it routed a reader to a terminal set for work that is real. Ground: the taxonomy's `bf16` row and the ledger's `### Other IEEE binary floats and BF16`, whose dated `Fact` paragraphs record the semantic-signature, reference-evaluation, dispatchability, numerical-honourability, carrier, ABI, kernel-vocabulary, lowering, optimizer-legality, runtime-validation, execution, and conformance movements and say in each case which cell moved and which did not. (That clause counted "four" dated paragraphs until 2026-08-08; the count is dropped rather than re-pinned, because a tally maintained by hand in a second document is the coupling that produced this defect.)

**Why it is the pattern the other float tracks cite.** The ledger's thirteen-rung recipe is derived from the completed U4/F32 vertical and from the BF16 second-dtype spike, and its four dry-run columns are the reusability claim per rung. D-3 and D-5 cite that derivation rather than repeating it; D-9 and D-13 are where it breaks, and the ledger names both breakages.

**Rung owners, and all eight are delivered.** [`admit-bf16-into-the-schedule-and-kernel-vocabulary`](../../../tickets/admit-bf16-into-the-schedule-and-kernel-vocabulary.md) and [`admit-the-bf16-type-and-carrier-into-every-total-map`](../../../tickets/admit-the-bf16-type-and-carrier-into-every-total-map.md) carried rungs 9 and 12's vocabulary half; [`carry-bf16-through-the-artifact-encoding-and-identity`](../../../tickets/carry-bf16-through-the-artifact-encoding-and-identity.md) carried rung 10; [`establish-bf16-optimizer-legality`](../../../tickets/establish-bf16-optimizer-legality.md) carried rung 8; [`lower-bf16-to-metal`](../../../tickets/lower-bf16-to-metal.md) and [`validate-bf16-at-the-runtime-routing-boundary`](../../../tickets/validate-bf16-at-the-runtime-routing-boundary.md) carried rung 12's execution half; [`conform-the-bf16-vertical-end-to-end`](../../../tickets/conform-the-bf16-vertical-end-to-end.md) carried rung 13; and [`state-and-check-a-bf16-numerical-contract`](../../../tickets/state-and-check-a-bf16-numerical-contract.md) carried rung 6's public half. **Every one is `done` as of 2026-08-08**, so this paragraph records which ticket carried which rung rather than naming live work, and the tense is the load-bearing part. Re-check it rather than trusting it:

```sh
for t in admit-bf16-into-the-schedule-and-kernel-vocabulary admit-the-bf16-type-and-carrier-into-every-total-map \
         carry-bf16-through-the-artifact-encoding-and-identity establish-bf16-optimizer-legality lower-bf16-to-metal \
         validate-bf16-at-the-runtime-routing-boundary conform-the-bf16-vertical-end-to-end \
         state-and-check-a-bf16-numerical-contract; do grep -m1 '^status:' tickets/$t.md; done
```

**What remains, and the authority each remainder sits with rather than a ticket.** The conversion families [ADR 0091](../../decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md) accepted are registered in neither direction, and their two keys and spellings remain **Tom's** under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md) — no ticket may mint them, so their absence from the graph is the policy working rather than an owner missing. The accumulator is an operation's registered fact under the same ADR, and no BF16 reduction, contraction, or fold family is registered to carry one. The end-to-end vertical is hand-assembled against the authoritative profile, and what would change it is widening the ledger's own BF16 rows — a new measurement on the measured row, gated by the ledger's Trigger. The [dtype support ledger](../../dtype-support.md)'s `### Other IEEE binary floats and BF16` Trigger states all three with the same authorities and is the single statement of them; this record points at it rather than forking it.

**Partial owner that is live.** [`declare-the-bf16-ios-family-answers-on-authoritative-ios-profiles`](../../../tickets/declare-the-bf16-ios-family-answers-on-authoritative-ios-profiles.md) owns the iOS-family dispatchability answers, which the ledger's macOS-only rows deliberately do not cover, and it is the only non-terminal BF16 ticket in the graph. It is `blocked` on [`first-authoritative-ios-metal-compile-declaration`](../../../tickets/first-authoritative-ios-metal-compile-declaration.md), which is `deferred` and satisfies no dependent, so it is a named owner rather than dispatchable work. Both iOS families answer `Unknown` by absence meanwhile, which is what a profile that has not spoken should resolve rather than a gap.

#### D-5 — OCP reduced-precision floats and E8M0 scale data

**Owner: [`scope-the-ocp-reduced-precision-float-vertical`](../../../tickets/scope-the-ocp-reduced-precision-float-vertical.md), deferred.** Ground: the taxonomy's reduced-precision table and the ledger's `### OCP reduced-precision formats`, which records all six registered with widths, bias, and an explicit Boolean per special member, and nothing beyond identity.

**Why the six are one track and the FNUZ variants are not in it.** The six share an authority, an acquisition route, and a value-set shape small enough that every one of them can claim `exhaustive-finite` at rung 4. `f8E3M4`, `f8E4M3`, `f8E4M3FNUZ`, `f8E5M2FNUZ`, and `f8E4M3B11FNUZ` share none of that: the admission policy classifies them "Recognized external owner-namespaced candidates", so their track is **D-13**, not this one. That boundary is checkable from the ledger's own negative check rather than from this record — the registered catalog is 27 nominal scalars, and `bool` plus twelve integer widths plus four IEEE binary formats plus BF16 plus six OCP formats plus three decimal formats is exactly 27, leaving no room for a seventh FP8 spelling.

**Trigger.** A selected model or kernel names the exact format, its operations, its conversion and accumulation policy, its physical representation, its runtime refusal rules, its target dispatchability, and its conformance corpus. **It has not fired.**

**Source boundary this track inherits.** Both OCP specifications are `metadata-only`: acquired by hand on 2026-07-31, licence-reviewed document by document, and digested, with the bytes discarded because neither carries a self-contained redistribution grant. Re-deriving the pinned value sets requires re-acquiring through the recorded route and checking against the recorded digest. That is a real constraint on this track's rung 2 and not a blocker on its rung 1, which ADR 0036 already discharged.

#### D-6 — Complex

**Owner: [`scope-the-complex-arithmetic-vertical`](../../../tickets/scope-the-complex-arithmetic-vertical.md), deferred.** Ground: the taxonomy's complex table and conclusion 4, and the ledger's `### Decimal, complex, and other reserved numeric families`, which records the constructor registered with its ordered real-then-imaginary component contract over exactly f16/f32/f64, every other component including `complex<bf16>` and nested complex refused by typed reason, and no operation admitting it.

**Trigger.** A named operation and component type, plus the branch-cut, exceptional-value, accuracy, storage, ABI, target, and conformance choices the ledger's trigger already enumerates. **It has not fired.**

**Operation-axis intersection.** F-40 spectral transforms is "the first family that *requires* the complex identity to be more than recognized", because a real transform's result is complex. F-42's `RQ-OP-12` also carries complex operands, and its own conclusion is that the family is consumer-owned by derivation. Separately, an ordering comparison over complex is on the operation record's **intentionally invalid** list, so this track must not be read as owing one.

#### D-7 — IEEE decimal32, decimal64, and decimal128

**Owner: [`scope-the-ieee-decimal-vertical`](../../../tickets/scope-the-ieee-decimal-vertical.md), deferred.** Ground: the taxonomy's decimal table, which fixes that "IEEE permits densely packed decimal and binary-integer-decimal encodings for the same logical decimal formats, so storage encoding must remain explicit", and the ledger's decimal cells — registered identity with interchange width and coefficient precision, an architectural seam at the physical carrier, absent everywhere else.

**Trigger.** A named frontend or accelerator consumer. **It has not fired**, and the taxonomy is explicit that current GPU tensor arithmetic does not imply execution support or justify treating decimal as a core binary-float variant.

#### D-8 — Affine quantized values, scales, and zero points

**Owner: the live quantization track; no new ticket.** Ground: the taxonomy's `AffineQuantized` structure and conclusion 5, ADRs 0029 through 0033, and the ledger's three affine sections. Per-tensor strict-affine U4/F32 and U8/F32 are tested guarantees at the semantic and reference layers; per-axis is selected by a measured workload and is not yet a statable contract, because the registered scheme validator admits only the two per-tensor forms.

**Live owners, in the dependency order the ledger's graph policy requires.** [`implement-workload-selected-quantized-parameter-maps`](../../../tickets/implement-workload-selected-quantized-parameter-maps.md) and [`widen-the-physical-vocabulary-for-per-axis-quantized-component-access`](../../../tickets/widen-the-physical-vocabulary-for-per-axis-quantized-component-access.md) precede [`implement-first-quantized-backend-profile`](../../../tickets/implement-first-quantized-backend-profile.md), which additionally depends on runtime semantic enforcement, a measured `(target family, dtype)` row, and calibrated costs before any device-optimal claim. `Requantize`, integer `Rescale`, alternate widths, and alternate expressed types remain reservations under Q-SEM-006.

**Operation-axis intersection.** F-20 covers quantize, dequantize, requantize, rescale, and assemble as separate atomic families and carries no `RQ-OP` question, which is consistent with this being the one compound family with a delivered vertical.

#### D-9 — Block-scaled compound values

**Owner: [`scope-the-block-scaled-compound-value-vertical`](../../../tickets/scope-the-block-scaled-compound-value-vertical.md), deferred.** Ground: the taxonomy's `## Packed and block-scaled encoded tensors`, which fixes that "shared scales change the represented numerical values, so MX/NVFP identities combine a `NumericInterpretation` with a `StorageEncoding` rather than belonging to storage alone", that "`MXFP4` is not an alias for a tensor of independent `f4E2M1FN` values", and that NVFP4 is a distinct vendor recipe with a different group size and scale format from MXFP4. The ledger records the six scheme identities registered, every static contract offered to one refused with `microscaling.unsupported-contract`, and **no MX value constructible**.

**Trigger, in two parts, both measurable.** A selected model format names its exact constituent scheme, **and** a non-per-tensor parameter-index map exists — which is a prerequisite rather than a substitute, and the per-axis map now being implemented is not a 32-element block map. The ledger records a third route that would reopen the eliminated per-block and per-group maps specifically: a caller granting reassociation, since those maps were eliminated on legality — a scale varying along the contracted axis makes a fused contraction partition that axis into contiguous intervals merged in order — and not on accuracy, where they measured best.

**Where the recipe breaks, which is why this is a track and not a widening.** The ledger's dry run records that rung 9 "**fails: one logical value maps to two physical buffers**, so a one-buffer-per-value assumption breaks here", that rung 10's transport mapping is not one-to-one, and that rung 3's ULP metric is not applicable until a block-aware metric exists.

#### D-10 — Sub-byte and packed storage encodings

**Owner: [`generalize-the-sub-byte-storage-encoding-contract`](../../../tickets/generalize-the-sub-byte-storage-encoding-contract.md), deferred.** Ground: the taxonomy's `### Bit-packed scalar storage`, whose `BitPacked` sketch names element, bits per element, bit order, byte order, row or block alignment, and padding, and which records that "Shape, offset, and stride legality differs among these encodings". ONNX specifies LSB-first packing for its int2 and int4 tensors; DLPack describes sub-byte packing and separately flags padded storage; other runtimes use byte-padded shell types.

**Why it has no ledger row, and why that is not a defect.** The ledger's rows are dtype families and packing is a per-family *column* — `Physical carrier and encoding`. The obligation is nonetheless recorded: [`own-the-dtype-support-maturity-matrix`](../../../tickets/own-the-dtype-support-maturity-matrix.md)'s consumer-driven follow-ups name "generalized sub-byte bit order, cross-byte layout, tail, alignment, unaligned access, neighbour-safe writes, and repacking beyond the governed whole-component U4 path" as a currently unowned surface. This track is that surface's owner.

**Trigger.** A selected profile chooses a packed code width, or a predicate or sub-byte element acquires a carrier. **It has not fired**, and the reason is recorded in the selection rather than inferred: the first quantized language-model profile chose **unpacked** `StorageScalar::U8`, and the physical-vocabulary ticket it filed states that the selected profile "needs no new carrier, no new encoding, and no new kernel type". The one packed construct that exists — the U4 extraction expression in Metal emission — is checked at the string level, absent from the compiled golden fixtures, and has never been dispatched.

#### D-11 — Execution-only and target ABI formats

**Owner: [`place-execution-only-numeric-formats-in-the-physical-layers`](../../../tickets/place-execution-only-numeric-formats-in-the-physical-layers.md), deferred.** Ground: the taxonomy's `### Target ABI and execution-only floating formats` and its conclusion 7, and the ledger's `### Execution-only formats`, which keeps TF32, the PTX scale encodings, `x86_fp80`, and `ppc_fp128` out of logical built-in identity and records an architectural seam at the numerical contract with type-system reservations at the physical layers.

**Trigger.** A selected backend operation needs one of them **and** can state its conversion boundaries, delivered numerical behaviour, target detection, artifact identity, and refusal. **It has not fired.** Promoting any of them into logical identity is a separate semantic decision that this track does not pre-authorize.

#### D-12 — Reserved numeric extension families

**Owner: [`route-the-reserved-numeric-families-through-the-extension-boundary`](../../../tickets/route-the-reserved-numeric-families-through-the-extension-boundary.md), deferred.** Members: `i128`/`u128` and arbitrary-width `iN`/`uN`; binary fixed-point; decimal fixed-point; UNORM and SNORM; `positN` and `quireN`; and the logarithmic-number, unum, rational, and arbitrary-precision families. Ground: the taxonomy's wide-integer and arbitrary-width rows, its `### Fixed-point, normalized integer, and decimal fixed-point` section, and its `### Posit and other tapered formats` section, plus the admission policy's `### Initial extension-only families` list, which is what makes them one track: they share exactly one route, the extension boundary, and no built-in admission gate.

**Why they are not silently equivalent to families that resemble them.** The taxonomy states it directly: fixed-point and normalized integers "are not equivalent to affine ML quantization merely because each can be implemented with integer storage and a scale", and older `posit<n, es>` research notation "is not automatically the same contract as the ratified standard".

**Trigger, per member.** An exact producer and consumer for that member, and — because the route is the extension boundary — the registration governance D-13 owes. This track therefore depends on D-13.

#### D-13 — External and vendor-owned identities

**Owner: [`govern-external-dtype-namespace-registration-and-equivalence`](../../../tickets/govern-external-dtype-namespace-registration-and-equivalence.md), deferred.** Members: the MLIR/StableHLO IEEE-convention `f8E3M4` and `f8E4M3`; the FNUZ variants; IBM HFP8 `f8E4M3B11FNUZ`; NVFP4 and other vendor block recipes; GGML-family and other project codecs; and learned or codebook quantizers with no admitted canonical descriptor.

**What is settled and what is not, stated precisely because the ledger's dry run compresses it.** [ADR 0034](../../decisions/0034-tiler-governed-built-in-dtype-keys.md) fixes the ownership direction — built-ins use Tiler-governed keys with mandatory normative references, and an already-published external identity is supported in place and never rekeyed — and the admission policy's own closing sentence records the remainder: "Namespace registration and collision governance for external providers remain an API-design task, but the ownership direction is fixed." So the dry run's "no vendor namespace policy exists" is right about registration and wrong if read as covering ownership. ADR 0034's own realization section records the same split from the other side: no external identity, alias table, or equivalence evidence exists to exercise the policy, and no same-format owner check runs before minting a key — the correctly-external OCP spellings are preserved by a test asserting non-registration rather than by an admission check.

**Trigger.** A real consumer publishes a stable owner-namespaced identity with an immutable descriptor, a normative reference, encode and decode vectors, an operation set, storage and ABI, runtime refusal rules, target evidence, and versioned conformance. **It has not fired.** The registration and collision-governance design is an API-design task and therefore a public boundary reserved to Tom under ADR 0075, which is the second reason this track is deferred rather than dispatchable: the same-format owner check ADR 0034 requires is vacuous while no external identity exists to collide with.

#### D-14 — Nonnumeric tensor element domains

**Owner: [`scope-the-nonnumeric-tensor-element-domain-vertical`](../../../tickets/scope-the-nonnumeric-tensor-element-domain-vertical.md), deferred.** Members: string and bytes; object and variant; temporal; structured and record; and categorical or dictionary. Ground: the taxonomy's `## Nonnumeric tensor element domains` table, which classifies each and records that recognizing them "does not require admitting them to the initial tensor-kernel optimizer", and the ledger's `### Nonnumeric and non-tensor domains`.

**Trigger.** A named frontend or product workload requires the exact domain and can define its operation and lifetime contracts. **It has not fired**, and the ledger's trigger states the anti-trigger explicitly: "Numeric dtype breadth does not trigger it."

## Families routed off the dtype axis

These appear in the taxonomy and are deliberately not dtype tracks. Each has an owner on another axis, so recording them here is what keeps "no family disappears" true without inventing a dtype reservation for a non-dtype.

| Family | Why it is not a dtype track | Where it is owned |
| --- | --- | --- |
| Effect and order tokens | No runtime data payload; it is an effect vocabulary the accepted model does not have | Q-SEM-011; the operation record's F-44 and F-46 |
| Resources, device handles, pointers | Runtime identity and lifetime, not element semantics | Q-SEM-011; F-45's typed ABI, effect, alias, and placement contracts |
| Typed PRNG keys | JAX's own precedent rejects ordinary arithmetic on them; the state is a value threaded through a pure operation | The operation record's F-43 |
| Shape and index values | Distinct newtypes even where the physical representation is `i64` | [IR](../../ir.md)'s shape environment; the ledger's `KernelType::Index` note |
| Tuples, futures, control values | Graph value kinds and control constructs | Q-SEM-012; F-47 |
| Data-dependent extents | A shape and allocation mechanism, not an element type | `RQ-OP-10`; Q-SHAPE-004 and Q-SHAPE-005 |
| Sparse and ragged representations | Container, shape, layout, or value representation | The ledger's `### Sparse, ragged, and non-tensor values`, which places them outside the axis rather than in a reservation cell |

## Where the tracks meet the operation axis

The [operation taxonomy](../semantic-graph/mature-operation-and-signature-taxonomy.md) enumerates forty-seven families on the other axis and asks twelve questions. Five of them join a dtype track, and each join is a genuine shared blocker rather than a topical resemblance. The remaining seven — `RQ-OP-02`, `RQ-OP-05` through `RQ-OP-09`, and `RQ-OP-11` — turn on structure, arity, or region support and are independent of which dtype the family carries.

| Question | Dtype track it joins | The shared blocker |
| --- | --- | --- |
| `RQ-OP-03` — is a predicate tensor a first-class graph value, and what is its storage and ABI? | **D-1** | Identical: the operation record states that this question and the ledger's `bool` trigger "must close together or neither has" |
| `RQ-OP-01` — does a checked-overflow integer operation return one result plus a precondition, or two results? | **D-2** | The answer fixes F-08's arity, which fixes what an integer honourability declaration is a declaration *about* |
| `RQ-OP-04` — does ADR 0091's directional-pair decision generalize to every conversion pair? **Answered 2026-08-05: partly.** | **D-3**, and every track whose conversion obligation is owed | D-3's pairs were the nearest candidates and supplied the answer. [Conversion family decomposition across pairs](conversion-family-decomposition-across-pairs.md) confirms this record's "does not transfer" cell by walking it: the per-ordered-pair key generalizes, ADR 0091's widening/narrowing field assignment does not, and the exception is `bf16`/`f16` — a D-3 member against D-4's — whose two directions owe intersecting field sets. The join stands, because what each track still owes per ordered pair is unchanged |
| `RQ-OP-12` — are dense linear-algebra decompositions semantic operations at all? | **D-6** | Both turn on complex operands and on a realization publishing a numerical guarantee that refines an expressible contract |
| `RQ-OP-02` — is bit reinterpretation a semantic family, given its result depends on a physical representation? | **D-10** | Its test is whether two targets with different sub-byte packings can honour one registered key, which is D-10's contract restated from the operation side |

`RQ-OP-02` appears in both lists deliberately: it is independent of *which* element type is reinterpreted and entirely dependent on the packing contract, so it joins D-10 and no element track. The operation record's `## Join key for the delivery graph` also carries one row that reads as a dtype claim and is not — "Arithmetic over reduced-precision floats" maps to "the dtype axis of F-05 through F-12", which is this record's D-3, D-4, and D-5 seen from the operation side.

## Findings a reader should act on

Five things this pass established that were not previously statable. Each is refutable from a source named beside it.

1. **Every taxonomy row reaches an owner, and twelve owners did not exist before this pass.** The [coverage table](#coverage-every-taxonomy-row-reaches-a-track) is the check; a row with no track is a defect in this record, not a gap in the inventory.
2. **Twelve of fifteen tracks have an unfired trigger, and in five cases the non-firing is a recorded elimination rather than an absence of interest.** The predicate track's mask is additive by selection; the packing track's profile is unpacked by selection; the block-scaled track's maps were eliminated on legality with their reopening condition stated; the integer track's nearest candidate is excluded by name; and the decimal track's weak GPU adoption is a taxonomy finding rather than an oversight. A deferral backed by an elimination is stronger evidence than a deferral backed by silence.
3. **Five FP8-family spellings in the taxonomy are external identities, not built-ins, and the ledger row that covers them is `External or vendor formats`.** A reader who takes the OCP row to cover all of `f8E*` will conclude that `f8E4M3FNUZ` is recognized. It is not, and the arithmetic against the ledger's own asserted catalog size is the one-line check.
4. **The integer family's missing authority is a honourability *subject*, not an operation contract.** ADRs 0039 and 0040 already fix the overflow and division families. What no target can do is declare that it honours one, because the subject vocabulary is floating-point only — and the measured Apple integer evidence already exists on the far side of that seam.
5. **Both OCP specifications became `metadata-only` on 2026-07-31 and three citing records still said `pending-acquisition`.** The three in this scope are corrected in the same change as this record; [ADR 0036](../../decisions/0036-recognize-standard-binary-and-microscaling-formats.md) and [ADR 0038](../../decisions/0038-recognize-ocp-mx-schemes.md) carried the stale claim past this record's own correction, because they are outside this ticket's scopes; [`correct-the-ocp-source-status-in-adrs-0036-and-0038`](../../../tickets/correct-the-ocp-source-status-in-adrs-0036-and-0038.md) corrected both on 2026-08-06, so every citing record now states the metadata-only classification.

## Coverage: every taxonomy row reaches a track

Rows are the taxonomy's own `## Enumerated catalog at a glance` block, read in its order. A bracketed taxonomy entry stays bracketed here: it is cataloged without implying portable product support.

| Taxonomy row | Track | Ledger row |
| --- | --- | --- |
| `bool` | D-1 | Logical `bool` |
| `i2 i4 i8 i16 i32 i64`, `u2 u4 u8 u16 u32 u64` | D-2; packed widths also D-10 | Signed and unsigned exact-width integers |
| `[i128, u128, bounded iN/uN extensions]` | D-12 | Wide or bounded integer extensions |
| `f32` | D-0 | IEEE `f32` |
| `f16`, `f64`, `f128` | D-3 | IEEE `f16/f64/f128` |
| `bf16` | D-4 | BF16 |
| `f8E4M3FN`, `f8E5M2`, `f6E2M3FN`, `f6E3M2FN`, `f4E2M1FN`, `f8E8M0FNU` | D-5; FP4 and FP6 carriers also D-10 | OCP E4M3FN, E5M2, E2M3FN, E3M2FN, E2M1FN, and E8M0FNU scale data |
| `f8E3M4`, `f8E4M3`, `f8E4M3FNUZ`, `f8E5M2FNUZ`, `f8E4M3B11FNUZ` | D-13 | External or vendor formats |
| `decimal32 decimal64 decimal128` | D-7 | IEEE decimal32/64/128 |
| `complex<f16> complex<f32> complex<f64>` | D-6 | `tiler::complex@1<ComponentTypeKey>` |
| `[positN]` and other tapered formats | D-12 | Other reserved numeric families |
| `ue4m3`, `ue8m0`, `tf32`, `x86_fp80`, `ppc_fp128` | D-11 | Execution-only formats |
| Affine quantized: per-tensor, per-axis, per-block | D-8; per-block also D-9 | Strict-affine U4/F32, U8/F32, and other affine schemes |
| Binary fixed-point, decimal fixed-point, UNORM, SNORM | D-12 | Other reserved numeric families |
| Bit-packed `bool`, `i2/u2`, `i4/u4` | D-10 | Carried in the physical-carrier column of each element row |
| `MXFP8 MXFP6 MXFP4 MXINT8` | D-9 | OCP MX compound schemes |
| `NVFP4` and versioned vendor block-quantized extensions | D-9 for the block obligations, D-13 for identity | OCP MX compound schemes; External or vendor formats |
| String and bytes, object and variant, temporal, structured and record, categorical and dictionary | D-14 | Nonnumeric tensor element domains |
| Token, resource, pointer or handle, typed PRNG key, opaque extension, shape or index, tuple, future, control value | Off-axis; see [Families routed off the dtype axis](#families-routed-off-the-dtype-axis) | Non-tensor graph values |

## What would make this record wrong

Three assumptions, named so a later reader can test them rather than inherit them.

- **That a track's members share obligations at the granularity a ticket is scheduled at.** If a future workload needs `f16` and only `f16`, D-3's grouping with `f64` and `f128` costs a split rather than buying reuse — and the correct response is to split the track, not to widen the ticket.
- **That a deferral's trigger is checkable by a reader who was not here.** Every trigger above is written as a state of the corpus, not as a judgement. A trigger that requires knowing what somebody intended has already failed.
- **That the ledger stays the single owner of delivered state.** The moment a maturity claim is stated here and not there, this record becomes a second ledger that nothing keeps honest. The tables above therefore quote and never compute.
