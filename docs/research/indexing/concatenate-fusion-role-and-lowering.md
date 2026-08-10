---
schema: "tiler-doc/v1"
id: "tiler.research.indexing.concatenate-fusion-role-and-lowering"
kind: "research"
title: "Concatenate fusion role and lowering"
topics: ["indexing", "access", "fusion", "lowering", "operation-families", "concatenate", "write-ownership"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "partially-adopted"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.ir", "tiler.contract.fusion-and-scheduling"]
depends_on: ["tiler.research.indexing.index-access-model", "tiler.research.shapes.sequence-extending-tensor-family", "tiler.research.semantic-graph.operation-family-delivery-graph"]
ticket: "scope-the-concatenate-fusion-role-and-lowering"
---

# Concatenate fusion role and lowering

- **Status:** completed scoping record for track **O-07**'s M4 and M5 questions. Its M4 `CoordinateRelation` proposal was adopted and delivered R5; its M5 lowering conclusion remains separately incomplete. This record itself registered nothing, moved no support-matrix rung, and accepted no public boundary. Its outcome is two eliminations, one restated open-question trigger, and four filed tickets.
- **Ticket:** [`scope-the-concatenate-fusion-role-and-lowering`](../../../tickets/scope-the-concatenate-fusion-role-and-lowering.md).
- **Research date:** 2026-08-05, against the tree at `d5960e81`.

## Traceability

- **The partition this record sits in:** [Operation-family delivery graph](../semantic-graph/operation-family-delivery-graph.md) track **O-07**, whose M4 cell read *owed, and newly owned* and whose M5 cell read *owed, and the alternative is a fork* at this record's base. That record owns the partition and the rung vocabulary; this one answers only O-07's two cells. **Restated 2026-08-10:** M4 landed under [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](../../../tickets/admit-a-fusion-role-for-the-sequence-extension-concatenate.md) and the support matrix records R5; the lowering work remains separate.
- **The delivered state this record consumes and never restates:** the [operation-family support matrix](../../roadmap.md#operation-family-support-matrix)'s `Sequence extension` row. It is the sole maturity ledger and **no rung moves here.**
- **The access language this record is bounded by:** [Symbolic index and access model](index-access-model.md) and [ADR 0046](../../decisions/0046-separate-logical-access-from-storage-addressing.md). The piecewise reservation this record tests against is that record's own — "Finite piecewise/guarded maps may be added as an explicit ordered-or-disjoint case set, but only with a verifier proving cases cover the domain and overlap consistently."
- **The semantic elimination this record inherits rather than reopens:** [Sequence-extending tensor family](../shapes/sequence-extending-tensor-family.md) eliminated the windowed-mutation and out-of-program mechanisms and selected a pure semantic join. That elimination stands; this record starts from the family as registered and asks only what fusion role it takes and how it lowers.
- **The question whose firing condition this record restates:** [Q-SHAPE-006](../../open-questions.md#q-shape-006--finite-piecewise-access-maps).
- **Inspected source, in full, at `d5960e81`:** `crates/tiler-ir/src/semantic/concatenate.rs`; `crates/tiler-ir/src/index/{model.rs,builder.rs}`; `crates/tiler-ir/src/index/builder/proof.rs`; `crates/tiler-ir/src/index/mod.rs`; `crates/tiler-ir/src/program/verify.rs`; `crates/tiler-compiler/src/{fusion_legality.rs,policy.rs,capability.rs,governed.rs,explain.rs,request.rs}`.

Claims are labelled **Fact** when traced to inspected source at that commit, **Inference** when derived from stated facts, and **Proposal** when not yet accepted or tested. This record takes no measurements and states none.

## What the family is, before either question

**Fact.** `tiler::concatenate-f32@1` is registered at `crates/tiler-ir/src/semantic/concatenate.rs:383-409` with `OperationEffect::Pure`, an inclusive operand arity of two through eight (`:67`, `:79`), exactly one result, one required `u32` axis attribute (`:82`), and six unconditional definition facts (`:90-100`). Its inferencer refuses any operand whose resolved type is not `tiler::f32@1` (`:507-517`) and derives the result shape from the operands rather than accepting a declared one (`:322-380`). Its normative definition (`:417-444`) states that every result element is an operand element unchanged, that an exceptional payload arrives bit for bit, that operand order is semantic, and — in its own last sentence — that whether the join has a contiguous byte window "is an applicability predicate over a physical candidate rather than part of this identity".

**Fact.** The registration declares no algebraic capability, and says why at `:404-408`: a concatenation performs no arithmetic, so it has no associativity or commutativity *of rounding* to declare, and a missing declaration reads as unknown rather than as the inverse law.

**Fact.** It has no `OperationNumericalCapability` row and is listed in `UNPLANNED_OPERATIONS` (`crates/tiler-compiler/src/policy.rs:788-817`), whose own doc states that it is unplanned "because nothing *physical* realizes it at all". That list is `#[cfg(test)]` and has exactly three consumers, all tests in the same module (`policy.rs:843`, `:863`, `:866`); it is a guard against table drift, not a runtime gate.

## Question 1 — the fusion role

### What a role has to answer for

**Fact — at this record's `d5960e81` research base.** The fusion authority is `crates/tiler-compiler/src/fusion_legality.rs`. `FusionOperationRole` (`:132-223`) has six variants; `FusionNumericalCapabilities::governed()` (`:268-335`) maps nine operation keys onto them; `classify` (`:349-351`) is a checked lookup and `derive_member` returns `Ok(None)` for an unregistered family (`:1037-1039`), which `derive_fusion_legality` converts into `FusionLegality::Unknown` with obligation `OperationCapabilitiesResolved` and reason `"unsupported-operation-capability"` (`:940-953`). That is exactly the state the ticket's user-visible outcome asked to leave.

**Restated 2026-08-10 — live census.** `rg -c -F 'roles.insert(' crates/tiler-compiler/src/fusion_legality.rs` returns **15**, and `is_exact_governed_same_family_pointwise`'s `CoordinateRelation` arm is closed over `reindex_f32_op()`, `broadcast_f32_op()`, `concatenate_f32_op()`, and `slice_f32_op()`. The nine-key premise is the record's historical research state, not its live inventory.

**Fact.** Nine obligations exist (`:372-391`) and `derive_obligations` (`:1063-1163`) discharges each one from the members' roles, their reached definitions, their derived purity, their dtype homogeneity, and the numerical contract. Nothing in that function, or in `derive_fusion_legality` (`:922-967`), resolves an index-access lowering capability, consults a realization law, or reaches the request boundary.

**Inference — M4 is independent of M5, and this is the most schedulable finding in the record.** The fusion role can be registered and its legality derived with no lowering in existence. The two rungs are separately dispatchable and the fusion role does not wait on the lowering fork, on Q-SHAPE-006, or on anything in `crates/tiler-ir/src/index/`. The corpus already runs in this configuration in both directions: the contraction has a lowering and no role, and the two structural families have both and are still unreachable at the request boundary.

### The elimination

Four candidates. Each is tested against what `derive_obligations` actually does, not against what a role's name suggests.

| Candidate | Survives? | Ground |
| --- | --- | --- |
| **No role — leave the family `Unknown`** | **No.** | It is the present state and it is what the ticket exists to end. `Unknown` is a fail-closed refusal, so it is not *wrong*; it is an absence of derivation. The family is pure, bit-preserving, dtype-homogeneous by construction, and reduction-free, so every obligation the authority can ask has an answer. Declining to state it is conservatism with no premise. |
| **`ValueSource`** | **No.** | Its contract (`fusion_legality.rs:139-141`) is a member that "contributes a value and no reordering, conversion, or reduction obligation of its own", and the `CoordinateRelation` doc (`:205-212`) draws the distinction explicitly: "a value source contributes a value the region did not otherwise have, while a coordinate relation contributes an access map over a value the region already has." Every element of a concatenation's result is an element of an operand already at the region's boundary. Classifying it as a value source would make `region_structure` (`:1303-1325`) report a region as holding one more independent value than it does — the failure the doc names. |
| **A new seventh role** | **No.** | A new role must earn itself by either deriving an obligation differently or falling outside the four structural buckets. It does neither. Every one of the nine obligations derives for a concatenate exactly as it does for a reindex (walked below), so a new variant would carry no distinct derivation. And `FusionRegionStructure`'s four role counts must sum to `members` (`:511-538`, and the reason is stated twice at `:184-188` and `:522-529`): a fifth count field moves the content identity of **every** region this vocabulary can already encode. That is an identity-domain step paid for zero derivational difference. |
| **`CoordinateRelation`** | **Yes.** | It is the role whose stated contract the family satisfies term for term, and the one whose derivation is already correct for it. |

**Fact — the obligation walk, `CoordinateRelation` against `derive_obligations`.** Nine obligations, each traced to the code that decides it.

1. `OperationCapabilitiesResolved` (`:1070-1073`) — discharged as `SoundProof` once any role resolves.
2. `ReferentialTransparency` (`:1075-1082`) — requires every member pure. `concatenate.rs:401` declares `OperationEffect::Pure`, and `derive_member` (`:1032-1036`) hard-errors on a disagreement between the graph's derived purity and the reached definition's effect, so this cannot pass on a stale fact. Discharged.
3. `ConversionBoundaryPreservation` (`:1084-1094`) — requires `member_is_homogeneous` (`:1054-1060`), every operand and result type encoding equal to the governed dtype. The inferencer refuses a non-`f32` operand at construction (`concatenate.rs:507-517`) and the result is `f32`, so **every admissible occurrence is homogeneous by construction**. Discharged, and on a stronger premise than the arithmetic families have.
4. `ArithmeticContraction` (`:1102-1117`) — the only obligation needing a decision, treated below.
5. `ExceptionalValues` (`:1146-1159`) — compares the contract's canonical arithmetic NaN pattern against the governed one. A concatenation performs no arithmetic and therefore reaches no arithmetic result boundary at which a canonicalization could be added or removed; its normative definition guarantees an exceptional payload survives bit for bit. Discharged as `NormativeGuarantee` under the governed contract, on the same premise as the two existing coordinate relations.
6-9. The four reduction obligations (`push_reduction_obligations`, `:1217-1266`) — `is_reduction()` is false for `CoordinateRelation` (`:230-237`), so identity/empty-domain and contributor order discharge as `SoundProof` vacuously, reassociation discharges because `!has_reduction`, and operand permutation discharges unconditionally.

**Fact — the one decision the role forces, and where.** `is_exact_governed_same_family_pointwise` (`:1165-1214`) is deliberately closed over exact keys, and its `CoordinateRelation` arm (`:1187-1189`) matches only `reindex_f32_op()` and `broadcast_f32_op()`. Its comment states both the soundness argument — "inserting a pure data movement between two adds cannot introduce a product to fuse" — and the reason for the closure — "a future capability could classify another contraction-capable family as a coordinate relation". A member that falls through reaches the exhaustive arm at `:1204-1210` and returns `false`.

**Inference — the arm should be extended to the concatenate key, and this must be decided rather than inherited.** The soundness argument transfers verbatim: a concatenation introduces no multiply, no add, and no adjacency between them, so inserting one between two adds cannot create a product to fuse. The closure exists so that this transfer is stated per key rather than assumed by role, which is precisely what extending it does.

**Fact — what not extending it would cost, and it is not zero.** `strict_contract` (`policy.rs:714-734`) sets `contraction: NumericalPermission::Forbidden`, so under the governed contract the `else if` branch at `:1107-1111` discharges `ArithmeticContraction` as a `NormativeGuarantee` anyway. Under a contract permitting contraction, a member falling through returns `Unknown` with reason `"unrealized-contraction"` (`:1113-1116`), and `first_unknown` (`:1286-1300`) makes the whole candidate `Unknown`. **Inference:** leaving the arm unextended would silently defer every fused candidate containing a concatenate under any contraction-permitting contract, for a reason the family's own semantics refute.

**Fact — arity is not an obstacle, checked rather than assumed.** Every family holding a role today is fixed-arity (zero, one, or two operands) and the concatenate is two through eight. Nothing in the derivation is arity-sensitive: `region_structure` (`:1303-1325`) counts members by role predicate and reads `boundary_inputs` from the candidate rather than from any operation's arity, `derive_obligations` iterates members, and `member_is_homogeneous` iterates whatever operand encodings the member has. A variadic member changes the candidate's boundary-input count, which is already a per-candidate quantity.

**Fact — registering the role does not move the pinned explain digest.** `ExplainWriter::new` (`explain.rs:1219-1235`) folds `FusionNumericalCapabilities::governed().provider()` — the `ProviderIdentity`, namespace, name, and revision — into the allowed-provider set. It does not fold the role table. `GOVERNED_PROVIDER_REVISION` has been `1` since the module was introduced (`git log -S "GOVERNED_PROVIDER_REVISION: u32 = " -- crates/tiler-compiler/src/fusion_legality.rs` returns exactly one commit, `1f541d60`), including across the landing that added the reindex and broadcast roles. **Inference:** on that precedent, adding a role is not an output-affecting revision bump, and the pinned `"tiler-explain-v7 request=45467875b9574962"` at `explain.rs:4050` is unaffected. The implementation ticket must still confirm this on its own merged tree rather than inherit it, because the ledger comments at `explain.rs:4008-4021` record two occasions on which a concatenate-related change moved that digest for a different reason. *(Dated 2026-08-06: the implementation landed, confirmed the non-movement on its tree, and the quoted pin value has since moved several times for unrelated changes — read the current value from `explain.rs`'s own ledger, never from this record.)*

**Proposal — the disposition.** *The sequence-extension concatenate takes the existing `CoordinateRelation` fusion role, with `is_exact_governed_same_family_pointwise`'s coordinate-relation arm extended to its key.* No new role, no new obligation, no new structural count, no public boundary. `FusionOperationRole` is a private enum and `FusionNumericalCapabilities` is `pub(crate)`, so this proposal reaches no public item; the matrix rung it would satisfy remains the matrix's to move.

**Fact — partly adopted 2026-08-06.** [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](../../../tickets/admit-a-fusion-role-for-the-sequence-extension-concatenate.md) registered the existing role and the per-key arm decision, delivering the support matrix's R5 criterion. The lowering conclusion remains separate and is why this record is only partially adopted.

## Question 2 — the lowering fork

### What one lowering capability can emit

**Fact — restated 2026-08-06, and the restatement withdraws a premise rather than a conclusion.** At this record's base `IndexAccessLoweringProvider` had one emission method, `lower` (`capability.rs:313-321`), which emits through one `IndexAccessLoweringContext` wrapping one canonical region builder; this record read that as one capability producing **one index region per occurrence**, the same limit [`lower-a-two-region-occurrence-through-one-index-access-capability`](../../../tickets/lower-a-two-region-occurrence-through-one-index-access-capability.md) owned for the normalizations. That ticket landed and the limit is gone. The trait now carries a defaulted `lower_sequence` beside `lower`, each stage built and verified by its own canonical builder, and `GovernedRootMeanSquareScaleF32` (`crates/tiler-compiler/src/governed.rs`) is a shipped provider that overrides it to emit an ordered two-region chain for one occurrence while implementing `lower` as an explicit refusal. **One capability may therefore emit a region *sequence*, and how many regions it emits is not a fact about the trait at all.** Nothing in this record rested on the withdrawn clause: the registration cost below derives from the resolver's key, which the next paragraph states on its own.

**Fact.** `resolve_index_access` (`capability.rs:1115-1144`) matches on the exact triple `(family, operation, signature)`, and `LoweringSignature` carries the exact operand and result type lists (`governed.rs:230-334`). **Inference — the concatenate needs seven registered index-access capabilities, one per admitted arity two through eight**, exactly as `MAX_CONCATENATE_OPERANDS`'s own doc comment (`concatenate.rs:69-79`) explains for the reference provider. No existing capability is variadic and nothing in the resolver is; this is a concrete registration cost nothing in the corpus had recorded.

### The four sites that refused a partitioned write at this record's base

The delivery graph said the partitioned alternative "stays inside the admitted language". Q-SHAPE-006 said it "is available". Those were different claims and, at this record's base, only the first was true. **Fact — the partitioned write was refused at four named sites, each read in full at that base.** The list is retained as the evidence the elimination below ran against; the dated correction after it states which refusals have since been discharged.

1. `crates/tiler-ir/src/index/builder.rs:1899-1901` — `output()` inserts into `output_tensors` and returns `IndexBuildError::DuplicateOutputTensor` on a repeat. One output tensor admits at most one output root. `MAX_OUTPUT_ROOTS` is `4_096` (`index/mod.rs:110`), so multiple *roots* are permitted; multiple roots over *one tensor* are not.
2. `crates/tiler-ir/src/index/builder.rs:1308-1310` — `prepare_access` returns `IndexBuildError::InvalidWriteDomain` unless a write's domain set equals the region's complete parallel dimension set. A write cannot iterate a sub-range, and two writes in one region share one iteration domain, so an operand-sized write and a result-sized write cannot coexist there.
3. `crates/tiler-ir/src/index/builder/proof.rs:702-712` — the exhaustive ownership walk allocates its `seen` bitset over every element of the whole output tensor (`:640-641`) and reports `WriteOwnershipNotProven` for any element left uncovered. A write that is not a coordinate permutation reaches this path via the `unresolved_ownership` gate at `:199-200`, and `write_is_permutation` (`:806-823`) requires every coordinate to be a bare `IndexNode::Dimension` with a proved-equal extent — which an offset write never is.
4. `crates/tiler-ir/src/program/verify.rs:197-199` — `KernelProgramDiagnostic::MultipleWriters`. If the partitions are separate regions rather than separate accesses, one value with two writing stages is refused at the program layer instead.

**Correction — 2026-08-06, under [`correct-the-write-domain-rule-in-the-indexing-corpus`](../../../tickets/correct-the-write-domain-rule-in-the-indexing-corpus.md): sites 1 and 2 are discharged, site 3 is split, and site 4 stands, so the refusal list above is history and the contract L-B is priced against below now exists.** [`admit-sub-range-write-domains-for-unequal-partitions`](../../../tickets/admit-sub-range-write-domains-for-unequal-partitions.md) relaxed site 2's equality rule to a subset rule: a write's domain may be any subset of the region's parallel dimensions (`crates/tiler-ir/src/index/builder.rs:1337-1343`), each root carries its own iteration space, and `InvalidWriteDomain` survives meaning only that a domain names a non-parallel dimension — so an operand-sized write and a result-sized write coexist in one region, which is the exact clause item 2's conclusion rested on. Site 1's construction-time refusal is gone with it: several roots may name one output tensor (`builder.rs:1909-1927`), a repeat is not a construction error, and the partition obligations are discharged at verification instead, under `IndexRegionDiagnostic::OutputPartitionUncovered`, `OutputPartitionRangesOverlap`, and `OutputPartitionDoubleWritten`, decided by interval reasoning (`builder/proof.rs:329`). Site 3's exhaustive walk now governs only a root that owns its output alone (`proof.rs:271`); a root whose output several roots partition is decided by that partition path rather than required to be a coordinate permutation. Site 4 is unchanged in meaning (`program/verify.rs:203`): one value written by two stages still refuses at the program layer, which partitioned accesses within one region do not trigger. The verdict below is unaffected — the elimination selected the partitioned write, and these landings are that selection executed rather than a new premise.

**Fact — the coordinate arithmetic the partitioned write needs already exists.** Operand *k*'s write coordinate on the concatenated axis is `t + offset_k`, where `offset_k` is the sum of the preceding operands' extents on that axis. Every extent a semantic occurrence can carry is a static `Extent` — `concatenate_result_shape` (`concatenate.rs:322-380`) computes over `Shape` and refuses an unrepresentable sum — so `offset_k` is a literal, and `IndexNode::LinearCombination { constant: IndexInteger, terms }` (`index/model.rs:97-100`) carries a literal exact-integer constant. The expression stays `Affine`. **Inference:** the partitioned write asks the coordinate-expression language for nothing it does not already have. The carrier gap Q-SHAPE-006 names for the slice family's symbolic-offset half — `SourcedExtent` appearing only in the `FloorDiv` and `Modulo` divisor positions (`index/model.rs:101-108`) — bites only if a concatenate occurrence ever carries a symbolic extent, which it cannot today.

**Corrected 2026-08-08 by [`correct-the-symbolic-coefficient-era-index-vocabulary-claims`](../../../tickets/correct-the-symbolic-coefficient-era-index-vocabulary-claims.md): the final carrier-gap sentence above is stale. A symbolic slice offset no longer awaits a coordinate-expression widening: `SourcedIndexInteger` reaches `LinearTermData::coefficient`, and the sourced builder represents a symbolic addend as `symbol * 1`; `SourcedExtent`'s divisor-only placement remains true but no longer establishes the conclusion. The concatenate inference rule still accepts only literal operand shapes because it calls `OperationInferenceRequest::static_operand_shape`, which is not a claim that every semantic occurrence is static. The symbolic slice instead fails earlier: its literal-only selection grammar has no source-bearing offset and `decode_axis` refuses `symbolic-window` by name.

### Why the piecewise read is not one widening but two

**Fact.** `AccessData` (`index/model.rs:126-132`) binds exactly one `tensor: u32` per access. A piecewise read for a concatenation must select a different **operand tensor** per output coordinate, not merely a different coordinate into one tensor. ADR 0046's piecewise reservation, as the [index and access model](index-access-model.md) states it, is over the *map* — `TensorAccess { tensor_value, mode, map }` places the tensor outside the map — so per-case tensor selection is a widening that reservation does not reserve.

**Fact — the alternative spelling is closed too.** Reading every operand unconditionally over the result-sized domain and selecting between the loaded scalars fails twice. The out-of-range reads are refused: the exhaustive verifier reports `CoordinateOutOfBounds` for a coordinate at or above the axis extent (`proof.rs:662-665`, `:675-683`), and operand zero's extent on the concatenated axis is strictly smaller than the result's whenever any other operand is non-empty. Clamping the coordinate back into range would pass the bounds proof and read the wrong element. And the select itself is unrepresentable: `ScalarValueDefinition` has exactly two variants, `AccessRead` and `OperationResult` (`index/model.rs:155-165`), with no conditional, and a predicated select needs a predicate tensor the registry cannot type — the decision the delivery graph's O-09 calls "the single highest-leverage unblocking decision in the inventory", blocking four whole families behind `RQ-OP-03`.

**Inference.** The piecewise read costs Q-SHAPE-006 **plus** one of {per-case tensor selection, a predicate dtype}. It is not the cheaper option; it is the one whose second cost is hidden behind the first.

### The elimination

| Alternative | Survives? | Ground |
| --- | --- | --- |
| **L-A — one total write with a piecewise read** | **No.** | It is insufficient on its own: the case selects a tensor, which neither `AccessData` nor ADR 0046's reservation expresses, and the read-both-and-select spelling is refused by the bounds proof and needs a predicate dtype four other families are already queued behind. What it widens is the access language itself — the expression vocabulary, its canonical identity, its classification lattice, every consumer that matches on `IndexNode`, and the realization-law templates — which is the broadest seam in the index layer. And it does not compose with the realization the workload wants: a single total write into a fresh output is a full materializing copy by construction. |
| **L-B — a partitioned write, one write root per operand over one output** | **Yes.** | It asks the coordinate language for nothing (offsets are literal, expressions stay `Affine`) and asks the *write-ownership contract* for one new thing: totality relative to a declared partition, plus joint coverage and disjointness across the partition set. For a concatenate the partitions are contiguous ranges fixed by static extents, decidable by interval reasoning without enumeration. It also composes with the copy-free physical candidate, below. |

**Inference — three arguments, and the third is the decisive one.**

- **Containment.** L-B's widening is confined to write ownership and multi-writer verification: `WriteOwnershipProof` and its public view (`index/model.rs:140-144`, `:970-979`), the four refusal sites above, and their proof code. L-A's widening reaches the expression language and everything downstream of it. Two owners against one.
- **Sufficiency.** L-B needs exactly one new mechanism. L-A needs two, and the second is blocked behind a decision the corpus has already identified as its highest-leverage one.
- **Composition with the physical candidate, which is why this is not a close call.** [Sequence-extending tensor family](../shapes/sequence-extending-tensor-family.md) established that the ABI already addresses a byte window rather than a whole value end to end — `accessible_offset` beside `accessible_bytes`, folded into artifact identity, re-proven at decode, applied at the binding call — and that "a kernel writing `[8, T, 128]` into a window of an `[8, S, 128]` value is already representable", with `MultipleWriters`, `ExternalValueWritten`, and the absence of a proof over untouched bytes being what refuses it. That copy-free realization **is** a partitioned write. Choosing L-A pays for a language widening and still owes L-B's contract before the copy can be elided; choosing L-B pays once. The record's own residency arithmetic — a naive copy at the B1-d row is 1.60× the model's entire F32 weight traffic per decode token — is what makes the elision matter rather than being a refinement.

**Inference — the two are not competing at the same rung, which the fork's framing obscured.** The windowed, copy-free binding is an M6/M7 physical candidate; the partitioned write is the M5 index-access lowering that makes it expressible. They compose. L-A composes with neither.

**Proposal — the disposition.** *The sequence-extension concatenate lowers as a partitioned write: one write root per operand over one output value, each total over its own contiguous partition of the concatenated axis, with joint coverage and disjointness proved across the roots. The piecewise read is eliminated.* This is a research disposition. The public boundary it would eventually reach — a variant on the `#[non_exhaustive]` `WriteOwnershipProofView` — is Tom's, and is named in the filed ticket rather than pre-authorized here.

## Question 3 — the inner-axis realization, checked rather than inherited

The ticket requires checking, not inheriting, the matrix row's assertion that an inner-axis concatenate's loss of the contiguous-window realization is an applicability predicate on a physical candidate rather than a second semantic identity.

**Fact.** The index region L-B produces is axis-uniform. For a concatenation on axis *a*, operand *k*'s write coordinate is `t_a + offset_k` on axis *a* and the bare dimension on every other axis; the read is the identity over the operand. Nothing in that construction distinguishes the slowest-varying axis from an inner one: the coordinate is a `LinearCombination` with a literal constant either way, the bounds obligation is the same interval question, and the ownership partition is a contiguous coordinate range on axis *a* either way.

**Fact.** What differs is on the other side of the two-map boundary. Whether the partition's coordinates map to a contiguous element-offset range is decided by the element strides in the storage half — the `ByteWindow` and `StorageEncoding` owners in `crates/tiler-ir/src/program/`, which the [index and access model](index-access-model.md)'s implementation-status section names as the storage owner. Under a row-major layout a contiguous window exists only for the slowest-varying axis.

**Inference — the assertion holds, and the check that establishes it is one sentence.** The axis changes no index-region fact and no semantic fact; it changes only which physical candidates are applicable. So it is an applicability predicate over a candidate, exactly as `concatenate.rs:441-443` and the matrix row state, and admitting a second key for the inner-axis case would give one meaning two identities for a reason that lives entirely in the storage half. Nothing here needs a second family.

## Q-SHAPE-006's firing condition, restated

**Fact — the sentence being corrected.** [Q-SHAPE-006](../../open-questions.md#q-shape-006--finite-piecewise-access-maps) read, at this record's base: "The second alternative is available, so the trigger has not fired; it fires if that alternative is eliminated." Read as "still on the table as a design option" it was defensible; read as "expressible today" it was refuted by the four sites inventoried above. The accurate claim was the delivery graph's — the second alternative *stays inside the admitted access language*, which is a statement about what it does not widen, not about what already works.

**Proposal — the verbatim-landable replacement for that bullet.** The paths below are written relative to `docs/open-questions.md`, which is where this text lands; they do not resolve from this record. Carried by [`carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus`](../../../tickets/carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus.md); the transfer executed and was byte-identical at landing. The landed bullet has since been corrected in place — the write-ownership contract it said the surviving alternative "owes" landed, and the four-site refusal inventory it repeated is discharged as the dated correction above states — so the quotation below is the transfer record, not the bullet's current text.

**The span is fenced rather than blockquoted, which is what keeps its destination-relative link from being a promise this page cannot keep.** A fenced block is this corpus's spelling for content whose links belong to another file — `check-citations.sh "fenced block is content proposed for somewhere else"` states the rule, and [`scope-transformer-nonlinear-normalization-and-reductions`](../../../tickets/scope-transformer-nonlinear-normalization-and-reductions.md) fences a catalog row for exactly that reason. Repointing the link at this record instead would send a reader to the page they are already on — neither what landed nor useful — and would spend the byte-identity the transfer claim above rests on. The fence's one cost is counted rather than hidden: the four retired line pins inside the span leave the citation matcher's reach with it. Three of them (`:1899`, `:1308`, `:702`) were being checked; the fourth, `:197`, already was not, because two tracked files end with `program/verify.rs` and an ambiguous partial path is skipped. All four are line numbers the dated correction above discharges, and the delta was measured rather than estimated: `./check-citations.sh` moved its `docs` population from 702 checked citations to 700, three retired pins out against the one anchor above in.

```text
- The one live piecewise *pressure* is resolved and does not fire this trigger. [Concatenate fusion role and lowering](research/indexing/concatenate-fusion-role-and-lowering.md) ran the elimination on 2026-08-05 at `d5960e81` and selected the partitioned write over the piecewise read, so the concatenate lowering asks nothing of the coordinate-expression language: an operand's write coordinate on the concatenated axis is `t + offset` for a literal offset, and `IndexNode::LinearCombination`'s exact-integer constant already carries it. The piecewise read was eliminated as **insufficient rather than merely expensive** — the case selects a different operand *tensor* per coordinate, which `AccessData`'s single `tensor` field does not express and which ADR 0046's piecewise reservation, being over the map rather than over the tensor, does not reserve; the alternative spelling that reads every operand and selects is refused by the bounds proof and additionally needs a predicate dtype `RQ-OP-03` owns. What the surviving alternative owes is a write-ownership contract rather than an access class: `WriteOwnershipProof` proves one access total over a whole output, and a partition needs partition-relative totality plus joint coverage and disjointness across roots. Two corrections to the previous wording of this bullet: the partitioned write is not *available* but *inside the admitted language*, being refused at four named sites (`index/builder.rs:1899` `DuplicateOutputTensor`, `index/builder.rs:1308` `InvalidWriteDomain`, `index/builder/proof.rs:702` total-coverage ownership, `program/verify.rs:197` `MultipleWriters`); and the family's own rungs are now owned rather than unassigned. **Restated trigger:** this question fires on the first family whose *read* map is genuinely case-split over one tensor — padding and cropping, track O-24, is the named candidate and its physical route is already recorded as "a guarded read" — and no longer on the concatenate.
```

## Reproducible checks

Each is one command from the repository root, with the positive control that proves it can return something.

```sh
# 1. Historical census: this record's base held nine keys without concatenate;
#    the 2026-08-06 restatement recorded eleven keys with it. Live, the table
#    holds fifteen keys and includes concatenate as a CoordinateRelation.
grep -n 'roles.insert(' -A 1 crates/tiler-compiler/src/fusion_legality.rs
#    Positive control: the same read finds reindex_f32_op and broadcast_f32_op,
#    so a key is read out of a list with members rather than out of an empty
#    result.
#    Restated 2026-08-06: this read "nine keys and no concatenate", which was
#    the state the elimination below ran against. The disposition it supports
#    landed under `admit-a-fusion-role-for-the-sequence-extension-concatenate`,
#    so the eleven-key intermediate state observed the registration. Restated
#    2026-08-10: the source-safe live census is fifteen registrations.
#    `-A 1` is needed because most entries span two lines and the bare match
#    shows no key.

# 2. Historical census: the arm had two keys at this record's base and three
#    after concatenate's admission. Live, the CoordinateRelation arm is closed
#    over four exact keys, including concatenate and slice.
grep -n 'fn is_exact_governed_same_family_pointwise' -A 50 crates/tiler-compiler/src/fusion_legality.rs
#    Positive control: the same read finds the ValueSource arm's constant guard,
#    so the match is being read rather than a comment.
#    Restated 2026-08-06: this read "closed over two keys" — reindex and
#    broadcast — which is what the extension proposed below started from. The
#    extension landed with the per-key transfer this record argued for, making
#    the arm three-key at that point. Restated 2026-08-10: slice is the fourth
#    exact key; the arm remains closed.

# 3. The refusal list's current state: the subset write-domain rule, the
#    partition path that replaced the construction-time refusals, the
#    sole-owner exhaustive walk, and the program-layer multiple-writers rule.
grep -n 'InvalidWriteDomain' crates/tiler-ir/src/index/builder.rs
grep -n 'OutputPartition\|decide_partition_by_interval' crates/tiler-ir/src/index/builder/proof.rs
grep -n 'fn verify_access_exhaustively' -A 105 crates/tiler-ir/src/index/builder/proof.rs
grep -n 'MultipleWriters' crates/tiler-ir/src/program/verify.rs
#    Positive controls: the first returns the subset rule's doc and its one
#    refusal site; the second returns the partition diagnostics and their
#    interval decision; the third's tail shows the uncovered-element scan. And
#    `grep -rn DuplicateOutputTensor crates/` returns nothing at all, which is
#    the discharged construction-time refusal observed rather than assumed.

# 4. No conditional exists in the scalar value vocabulary.
grep -n 'enum ScalarValueDefinition' -A 10 crates/tiler-ir/src/index/model.rs
#    This shows exactly two variants. Positive control: the same read finds
#    AccessRead, so an absent Select is an absence from an enumerated list.

# 5. Resolution is by exact signature, so a variadic family needs one
#    capability per admitted arity. A `LoweringSignature` is the ordered
#    operand and result type lists, compared for equality; `resolve` filters
#    the registry on the exact (family, operation, signature) triple. Two
#    arities of one operation are therefore two signatures and two
#    capabilities, and the conclusion rests on this alone.
grep -n 'pub struct LoweringSignature' -B 2 -A 5 crates/tiler-compiler/src/capability.rs
grep -n 'fn resolve_index_access' -A 25 crates/tiler-compiler/src/capability.rs
#    Positive control: the second read reaches the MissingCapability
#    construction, so the failure path is present rather than inferred.
#    Restated 2026-08-06: this check opened "one index-access capability emits
#    one region, and resolution is by exact signature". The first clause is
#    refuted — the trait carries `lower` beside a defaulted `lower_sequence`,
#    so one capability may emit an ordered chain of regions for one occurrence
#    — and it was never load-bearing here. The third read is what shows that,
#    and is a control on the premise rather than on the conclusion.
grep -n 'fn lower_sequence' -B 18 -A 6 crates/tiler-compiler/src/capability.rs

# 6. The explain writer folds the fusion provider identity, not the role table.
grep -n 'FusionNumericalCapabilities::governed().provider()' -B 8 crates/tiler-compiler/src/explain.rs
#    Positive control: the surrounding comment names both governed providers, so
#    the read reaches the allowed-provider construction rather than a use site.
```

## What this record does not decide

- **Any matrix rung.** O-07 stays where the matrix has it. A record delivers nothing; the tickets below do.
- **The public boundary of a partitioned write-ownership proof.** `WriteOwnershipProofView` is `pub` and `#[non_exhaustive]`; a variant on it is a public boundary and is Tom's under ADR 0075. The filed ticket names it as a stop rather than pre-authorizing it.
- **The in-place append into a caller-retained allocation.** Still [`scope-an-in-place-append-into-a-caller-retained-allocation`](../../../tickets/scope-an-in-place-append-into-a-caller-retained-allocation.md)'s under Q-PLAN-015. This record establishes that the partitioned-write contract is a shared prerequisite of both, not that either is authorized.
- **Whether a partitioned write is proved per-partition or jointly, and by which proof form.** The obligation is stated — partition-relative totality, joint coverage, disjointness — and its decidability for contiguous static ranges is argued; the exact proof kind, its canonical encoding, and its budget are implementation work.
- **Any cost claim.** No schedule for a concatenation has been measured at any shape. The residency arithmetic cited above is the sequence-extending record's, restated with its label intact.
- **Whether the second dtype ever needs a concatenate.** A BF16 concatenation would be a second registered family under ADR 0091's directional-pair reading and is out of this record's subject entirely.

## Work this record filed

| Ticket | Rung | Why it is separate |
| --- | --- | --- |
| [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](../../../tickets/admit-a-fusion-role-for-the-sequence-extension-concatenate.md) | M4 / R5 | The derivation is independent of every index-layer question, so holding it behind the lowering would park a landable rung behind a public-boundary decision it does not need. |
| [`admit-a-partitioned-write-ownership-contract`](../../../tickets/admit-a-partitioned-write-ownership-contract.md) | M5, `tiler-ir` half | The four refusal sites and the third proof kind are one owner's work, they are what the in-place append also needs, and the public boundary they reach is Tom's. |
| [`lower-the-concatenate-occurrence-through-partitioned-writes`](../../../tickets/lower-the-concatenate-occurrence-through-partitioned-writes.md) | M5, `tiler-compiler` half | Seven per-arity capabilities and a realization law against a contract that must exist first. |
| [`carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus`](../../../tickets/carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus.md) | none | This record's scopes do not reach `contracts/navigation`, and the three navigation edits it forces — this record's catalog row, Q-SHAPE-006's bullet, and the matrix row's owner links — are one step that must land together. |

## What would make this record wrong

- **That `CoordinateRelation`'s contract is about computing no value rather than about reading one tensor.** Every sentence of the role's doc comment is about arithmetic, rounding, and value contribution, and none is about arity or source count. If a later obligation is added that reasons over an access's source tensor, the role would owe a re-derivation — and the check is to re-walk `derive_obligations` against a concatenate member, which is the walk this record spells out so it can be re-run rather than re-derived.
- **That the partitioned write's joint-coverage obligation stays decidable.** It is argued for contiguous ranges over static extents. A concatenate occurrence carrying a symbolic extent would move the offsets out of `LinearCombination`'s literal constant and into the carrier gap Q-SHAPE-006 already names for the slice family, and the argument would have to be re-made rather than extended.

**Corrected 2026-08-08 by [`correct-the-symbolic-coefficient-era-index-vocabulary-claims`](../../../tickets/correct-the-symbolic-coefficient-era-index-vocabulary-claims.md): the preceding symbolic-concatenate contingency is false in its stated ground.** `LinearCombination` still stores an exact literal constant, but `SourcedIndexInteger` now admits a coordinate addend as a `symbol * 1` term, so a future source-bearing concatenate would not reopen Q-SHAPE-006's carrier gap. `Concatenate` currently calls `OperationInferenceRequest::static_operand_shape`, so no such occurrence is constructible today. If that rule changes, the joint-coverage argument must be re-derived for its source-bearing ranges; that is a proof and semantic-shape question, not a missing coordinate-expression vocabulary.
- **That one capability per arity is the right shape rather than a variadic lowering signature.** Seven registrations is what the resolver's exact-signature key forces today. If a variadic `LoweringSignature` is ever admitted, the seven collapse to one and this record's registration-cost finding becomes an artifact of a limit that moved.
- **That the copy-free windowed realization remains the reason to prefer L-B.** The composition argument is the decisive one, and it rests on the ABI facts the sequence-extending record established rather than on a measurement taken here. If the windowed binding turns out to be blocked by something that inventory missed, L-B keeps the containment and sufficiency arguments and loses the strongest one.
