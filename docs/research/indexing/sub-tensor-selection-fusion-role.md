---
schema: "tiler-doc/v1"
id: "tiler.research.indexing.sub-tensor-selection-fusion-role"
kind: "research"
title: "Sub-tensor selection fusion role"
topics: ["indexing", "access", "fusion", "operation-families", "slice", "sub-tensor-selection", "coordinate-relation"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.ir", "tiler.contract.fusion-and-scheduling"]
depends_on: ["tiler.research.indexing.concatenate-fusion-role-and-lowering", "tiler.research.indexing.index-access-model", "tiler.research.semantic-graph.operation-family-delivery-graph"]
ticket: "scope-the-sub-tensor-selection-fusion-role"
---

# Sub-tensor selection fusion role

- **Status:** scoping record for track **O-06**'s M4 cell, and for that cell alone. It registers nothing, moves no support-matrix rung, and accepts no public boundary. Its outcome is one elimination, one arm decision, one correction to the precedent record's key count, and two filed tickets.
- **Ticket:** [`scope-the-sub-tensor-selection-fusion-role`](../../../tickets/scope-the-sub-tensor-selection-fusion-role.md).
- **Research date:** 2026-08-05, against the tree at `3cca2a3f`.

## Traceability

- **The partition this record sits in:** [Operation-family delivery graph](../semantic-graph/operation-family-delivery-graph.md) track **O-06**, whose M4 cell reads *owed* and whose M5 cell reads *owed*. That record owns the partition and the rung vocabulary; this one answers only O-06's M4 cell, and files M5's owner rather than deciding it.
- **The delivered state this record consumes and never restates:** the [operation-family support matrix](../../roadmap.md#operation-family-support-matrix)'s `Sub-tensor selection` row. It is the sole maturity ledger and **no rung moves here.**
- **The method this record reuses and the answer it does not inherit:** [Concatenate fusion role and lowering](concatenate-fusion-role-and-lowering.md) ran a four-candidate elimination against `derive_obligations` for track O-07 and selected `CoordinateRelation`. The *method* transfers; the *answer* is re-derived below against the authority at this record's own base, because a role is a per-key claim and the two families differ in arity, in mapping class, and in how much of the classification their registered definition facts carry.
- **The semantic elimination this record inherits rather than reopens:** [`admit-the-sub-tensor-selection-family`](../../../tickets/admit-the-sub-tensor-selection-family.md) settled the family's form — one keyed family carrying a canonical total per-axis selection, on ADR 0087's rule — and refused the strided and symbolic relations by name. That elimination stands; this record starts from the family as registered and asks only what fusion role it takes.
- **The access language this record is bounded by, and does not reach:** [Symbolic index and access model](index-access-model.md) and [ADR 0046](../../decisions/0046-separate-logical-access-from-storage-addressing.md). Nothing below reaches the index layer, and the walk at *M4 does not wait on M5* is the reason.
- **Inspected source, in full, at `3cca2a3f`:** `crates/tiler-ir/src/semantic/slice.rs`; `crates/tiler-ir/src/semantic/concatenate.rs`; `crates/tiler-compiler/src/fusion_legality.rs`. Read in part, at the sites cited: `crates/tiler-compiler/src/{policy.rs,region.rs,explain.rs,request.rs}`; `crates/tiler-reference/src/standard.rs`; `crates/tiler-ir/src/index/builder/proof.rs`.

Claims are labelled **Fact** when traced to inspected source at that commit, **Inference** when derived from stated facts, and **Proposal** when not yet accepted or tested. This record takes no measurements and states none. No source it needed was unfetchable, so it raises no acquisition request.

## What the family is, before the question

**Fact.** `tiler::slice-f32@1` is registered at `crates/tiler-ir/src/semantic/slice.rs:814-839` with `OperationEffect::Pure` (`:831`), an exact operand arity of one and exactly one result (`:820-821`), one required `Record` selection attribute (`:822-825`), and five unconditional definition facts (`:882-905`). Its inferencer refuses any operand whose resolved type is not `tiler::f32@1` (`:948-953`) and derives the result shape from the operand and the selection rather than accepting a declared one (`:954-957`). Its normative definition (`:847-880`) states that every result element is an operand element unchanged, that no value is computed, converted, rounded, or canonicalized, that an exceptional payload arrives exactly as it left the operand, and — in its own last sentence — that the family "makes no claim that storage was copied, viewed, or left alone".

**Fact.** The registration declares no algebraic capability, and says why at `:834-838`: a selection performs no arithmetic, so it has no associativity or commutativity *of rounding* to declare, and a missing declaration reads as unknown rather than as the inverse law. That is the concatenate's reasoning at `concatenate.rs:404-408` in the same words, applied to a different structural identity.

**Fact — and this is the one registration fact that has no concatenate counterpart.** One of the five definition facts is `SLICE_FACT_MAPPING_CLASS` (`slice.rs:185`), whose canonical value is `total-over-the-result-domain-and-injective-not-surjective-into-the-operand-domain` (`:888-893`). The concatenate's six facts (`concatenate.rs:446-473`) name value behaviour, operand order, result extent, the empty operand, dtype promotion, and the storage claim; **none of them names a mapping class.** So the slice family declares in canonical, unconditional attribute bytes that it is an output-to-input coordinate relation, where the concatenate's equivalent claim lives only in normative prose.

**Fact.** It has no `OperationNumericalCapability` row and is listed in `UNPLANNED_OPERATIONS` (`crates/tiler-compiler/src/policy.rs:811-817`) beside `tiler::concatenate-f32@1`. That list is `#[cfg(test)]` and guards table drift; it is not a runtime gate. Its doc comment (`:789-810`) explains the BF16 entries and the concatenate entry and **says nothing about the slice entry**, which is a documentation gap this record files rather than repairs.

## Question — the fusion role

### What a role has to answer for

**Fact.** The fusion authority is `crates/tiler-compiler/src/fusion_legality.rs`. `FusionOperationRole` (`:132-223`) has six variants; `FusionNumericalCapabilities::governed()` (`:268-335`) maps **nine** operation keys onto them — constant, multiply, add, silu, strict-serial-sum, rms-norm, softmax, reindex, and broadcast — and the slice is not among them. `classify` (`:349-351`) is a checked lookup, `derive_member` returns `Ok(None)` for an unregistered family (`:1037-1039`), and `derive_fusion_legality` converts that into `FusionLegality::Unknown` with obligation `OperationCapabilitiesResolved` and reason `"unsupported-operation-capability"` (`:944-953`). That is exactly the state the ticket's user-visible outcome asks to leave.

**Fact — the state above is reachable, which is not the same as saying the role's absence is inert.** Region formation is key-agnostic: `RegionGraph`'s construction (`crates/tiler-compiler/src/region.rs:652-692`) admits every occurrence whose key the registry defines, reading only the definition's effect, and holds no operation allowlist. So a program containing a slice does form candidates containing it, and every such candidate is `Unknown` today for the reason above. Registering the role changes a real outcome rather than a hypothetical one.

**Fact — but what it does *not* change is worth stating in the same breath.** The compiler request boundary refuses a program stating a coordinate-mapping family under `operation-set`, because the region vocabulary's `LogicalAccess` cannot spell the family's access relation — `crates/tiler-compiler/src/request.rs:4898-4922` records the reindex case and its test asserts the refusal. A slice program refuses there for the same reason. **Inference:** registering the role delivers R5's criterion — a derivable legality instead of a fail-closed `Unknown` — and delivers nothing at the request boundary, which is the state the two existing coordinate relations are already in.

**Fact.** Nine obligations exist (`:372-391`) and `derive_obligations` (`:1063-1163`) discharges each one from the members' roles, their reached definitions, their derived purity, their dtype homogeneity, and the numerical contract. Nothing in that function, or in `derive_fusion_legality` (`:922-967`), resolves an index-access lowering capability, consults a realization law, or reaches the request boundary.

**Inference — M4 is independent of M5, on the same reading the concatenate record made of the same two functions.** The fusion role can be registered and its legality derived with no slice lowering in existence, so O-06's two owed cells are separately dispatchable and neither waits on the other.

### The elimination

Four candidates. Each is tested against what `derive_obligations` does at `3cca2a3f`, not against what a role's name suggests and not against the concatenate's answer.

| Candidate | Survives? | Ground |
| --- | --- | --- |
| **No role — leave the family `Unknown`** | **No.** | It is the present state and it is what the ticket exists to end. `Unknown` is a fail-closed refusal, so it is not *wrong*; it is an absence of derivation. The family is pure by registration, bit-preserving by normative definition, dtype-homogeneous by construction, and reduction-free, so every obligation the authority can ask has an answer — the walk below supplies all nine. Declining to state it is conservatism with no premise. |
| **`ValueSource`** | **No.** | Its contract (`fusion_legality.rs:139-141`) is a member that "contributes a value and no reordering, conversion, or reduction obligation of its own", and the `CoordinateRelation` doc (`:206-211`) draws the distinction explicitly: "a value source contributes a value the region did not otherwise have, while a coordinate relation contributes an access map over a value the region already has." Every element of a selection's result is an element of its single operand, already at the region's boundary — which the family's own `SLICE_FACT_VALUE_BEHAVIOUR` states as `none-every-result-element-is-an-operand-element-unchanged` (`slice.rs:884-887`). Classifying it as a value source would make `region_structure` (`:1303-1325`) report a region as holding one more independent value than it does, which is the failure the doc names. |
| **A new seventh role** | **No.** | A new role must earn itself by deriving an obligation differently or by falling outside the four structural buckets, and the one property that could plausibly make this family different from the two roles already holding `CoordinateRelation` — that its map is *non-surjective*, so operand elements go unread — derives nothing differently. Walked below: all nine obligations derive for a slice exactly as they do for a reindex, and none of them reads a mapping class, an operand count, or a read's coverage of its source. And `FusionRegionStructure`'s four role counts must sum to `members` (`:511-538`, with the reason stated at `:172-174`, `:199-201`, and `:522-529`, and asserted by the existing test at `:1592-1600`): a fifth count field moves the content identity of **every** region this vocabulary can already encode. That is an identity-domain step paid for zero derivational difference. |
| **`CoordinateRelation`** | **Yes.** | It is the role whose stated contract the family satisfies term for term, whose derivation is already correct for it, and whose class the family's own registered mapping-class fact names. |

**Fact — the obligation walk, `CoordinateRelation` against `derive_obligations` at `3cca2a3f`.** Nine obligations, each traced to the code that decides it and to the slice fact that answers it.

1. `OperationCapabilitiesResolved` (`:1070-1073`) — discharged as `SoundProof` once any role resolves.
2. `ReferentialTransparency` (`:1075-1082`) — requires every member pure. `slice.rs:831` declares `OperationEffect::Pure`, and `derive_member` (`:1032-1036`) hard-errors on a disagreement between the graph's derived purity and the reached definition's effect, so this cannot pass on a stale fact. Discharged.
3. `ConversionBoundaryPreservation` (`:1084-1094`) — requires `member_is_homogeneous` (`:1054-1060`), every operand and result type encoding equal to the governed dtype. The inferencer refuses a non-`f32` operand at construction (`slice.rs:948-953`) and the result is built as `ValueFact::new(F32::resolved_type(), shape)` (`:957`), so **every admissible occurrence is homogeneous by construction** — over exactly one operand rather than a two-through-eight range, so the predicate is decided by a single comparison. Discharged.
4. `ArithmeticContraction` (`:1102-1117`) — the only obligation needing a decision, treated below.
5. `ExceptionalValues` (`:1146-1159`) — compares the contract's canonical arithmetic NaN pattern against the governed one. A selection performs no arithmetic and therefore reaches no arithmetic result boundary at which a canonicalization could be added or removed; its normative definition (`slice.rs:850-853`) guarantees that a non-canonical NaN, a signalling NaN, a signed zero, and a subnormal all arrive exactly as they left the operand. Discharged as `NormativeGuarantee` under the governed contract, on the same premise as the two existing coordinate relations.
6-9. The four reduction obligations (`push_reduction_obligations`, `:1217-1266`) — `is_reduction()` is false for `CoordinateRelation` (`:230-237`), so identity/empty-domain and contributor order discharge as `SoundProof` vacuously, reassociation discharges because `!has_reduction`, and operand permutation discharges unconditionally.

**Fact — non-surjectivity is checked against the one place it could have mattered, rather than waved past.** The exhaustive index verifier allocates its coverage bitset only for a write: `crates/tiler-ir/src/index/builder/proof.rs:640-641` reads `(access.mode == AccessMode::Write).then(...)`, and the uncovered-element scan that reports `WriteOwnershipNotProven` is guarded by that same `Option`. A *read* is therefore never required to be total over its source. **Inference:** the property that distinguishes this family from a reindex — that at least one operand element is not read at all — is not a fact any obligation in this authority or any coverage rule in the index verifier consults. It is a semantic admission rule, which is where the family already keeps it.

**Fact — the reference layer reached the same classification independently, and said so.** `crates/tiler-reference/src/standard.rs:114-125` registers the slice under the *same* unary signature as the reindex and the broadcast, with a comment stating that what separates the three "is the class of map the attribute may state — bijective, many-to-one, and injective-not-surjective — which is a semantic admission rather than a signature, so nothing here needs to tell them apart." That is a different layer, a different authority, and the same conclusion about the same three families.

### The one decision the role forces, and where

**Fact.** `is_exact_governed_same_family_pointwise` (`:1165-1214`) is deliberately closed over exact keys, and its `CoordinateRelation` arm (`:1187-1189`) matches only `reindex_f32_op()` and `broadcast_f32_op()`. Its comment states both the soundness argument — "inserting a pure data movement between two adds cannot introduce a product to fuse" — and the reason for the closure: "a future capability could classify another contraction-capable family as a coordinate relation." A member that falls through reaches the exhaustive arm at `:1204-1210` and returns `false`. The existing test at `:1666-1682` proves the closure bites, by showing a foreign coordinate relation disqualifying the proof.

**Inference — the arm should be extended to the slice key, and this must be decided rather than inherited.** The soundness argument transfers: a selection introduces no multiply, no add, and no adjacency between them, so inserting one between two adds cannot create a product to fuse. The closure exists so that this transfer is stated per key rather than assumed by role, which is precisely what extending it does.

**Fact — what not extending it would cost, and it is not zero.** `strict_contract` (`policy.rs:714-734`) sets `contraction: NumericalPermission::Forbidden`, so under the governed contract the `else if` branch at `:1107-1111` discharges `ArithmeticContraction` as a `NormativeGuarantee` anyway. Under a contract permitting contraction, a member falling through returns `Unknown` with reason `"unrealized-contraction"` (`:1113-1116`), and `first_unknown` (`:1286-1300`) makes the whole candidate `Unknown`. **Inference:** leaving the arm unextended would silently defer every fused candidate containing a slice under any contraction-permitting contract, for a reason the family's own semantics refute.

**Fact — registering the role does not move the pinned explain digest, on the same reading and one further check.** `ExplainWriter::new` (`explain.rs:1219-1235`) folds `FusionNumericalCapabilities::governed().provider()` — the `ProviderIdentity`, namespace, name, and revision — into the allowed-provider set. It does not fold the role table. `GOVERNED_PROVIDER_REVISION` has been `1` since the module was introduced (`git log --oneline -S "GOVERNED_PROVIDER_REVISION: u32 = " -- crates/tiler-compiler/src/fusion_legality.rs` returns exactly one commit, `1f541d60`). The pinned `"tiler-explain-v7 request=45467875b9574962"` at `explain.rs:4054` is a *request-subject* digest, and the ledger comments above it (`:4008-4020`) record that it moved for the concatenate because the request subject folds the **semantic registry snapshot** — which a fusion-role addition does not touch, the role table living in `tiler-compiler` and the snapshot in `tiler-ir`. **Inference:** the digest is unaffected. The implementation ticket must still confirm this on its own merged tree rather than inherit it, because the same ledger records two occasions on which a structural-family change moved that value for a reason its author had not predicted. *(Dated 2026-08-06: the quoted pin value and its line number are the state this record was written against; the qualifier has since moved several times for unrelated registry and request-subject changes — read the current value from `explain.rs`'s own ledger, never from this record.)*

### Where the premises differ from the concatenate's, stated honestly

The ticket's hypothesis was that the obligations would discharge "on the same or stronger premises". They do, but only two of the differences are genuine strengthenings and one is merely a simplification; conflating the three would overstate the result.

| Premise | Slice against concatenate | Class |
| --- | --- | --- |
| The family is a coordinate relation rather than a value source | The slice states it as a **registered definition fact** in canonical attribute bytes (`SLICE_FACT_MAPPING_CLASS`); the concatenate states it only in normative prose | **Stronger** |
| Dtype homogeneity (obligation 3) | Both are homogeneous by construction. The slice decides it over one operand; the concatenate over a two-through-eight range, which is why that record had to check that nothing in the derivation is arity-sensitive | **Simpler, not stronger** — the concatenate's arity check concluded the same thing, and a check that concludes correctly is not a weaker premise |
| Bit preservation (obligation 5) | Identical wording in both normative definitions; both discharge as `NormativeGuarantee` | **Same** |
| The contraction-arm extension (obligation 4) | Identical: neither family performs arithmetic, so neither can create a multiply-plus-add adjacency | **Same** |
| Corroboration outside this authority | The reference layer's own registration comment already classes the slice with the reindex and broadcast by map class; the concatenate has no such neighbour, being registered per-arity for a different reason | **Stronger** |

**Proposal — the disposition.** *The sub-tensor selection slice takes the existing `CoordinateRelation` fusion role, with `is_exact_governed_same_family_pointwise`'s coordinate-relation arm extended to its key.* No new role, no new obligation, no new structural count, no public boundary. `FusionOperationRole` is a private enum and `FusionNumericalCapabilities` is `pub(crate)`, so this proposal reaches no public item; the matrix rung it would satisfy remains the matrix's to move.

## A correction to the precedent record

**Fact.** [Concatenate fusion role and lowering](concatenate-fusion-role-and-lowering.md) states that `FusionNumericalCapabilities::governed()` "maps eight operation keys onto them", and [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](../../../tickets/admit-a-fusion-role-for-the-sequence-extension-concatenate.md) repeats the number. The table holds **nine** keys, both at this record's base and at `d5960e81`, the commit the concatenate record names as its own tree: `git show d5960e81:crates/tiler-compiler/src/fusion_legality.rs | grep -c 'roles.insert('` returns `9`, and `grep -c 'roles.insert(' crates/tiler-compiler/src/fusion_legality.rs` returns `9`. The missing key in both statements is `softmax_f32_op()`, whose `ExtremumShiftedOrderedReduction` role is registered at `fusion_legality.rs:324-327` and asserted by name in `softmax_role_tests` (`:2272-2293`).

**Inference — the error is arithmetic and changes no conclusion of either document.** Both texts use the count only to say that the concatenate is absent from the table, which is true at nine keys as it was at eight. The number is corrected in both places in the same change as this record, rather than left to be copied a third time. *Restated 2026-08-06: the live `grep -c` above returns `11` on the current tree — the concatenate's own role and the softmax's landed after this record — while the `d5960e81` count and the arithmetic correction it grounds are unchanged; the reproducible checks below carry the current counts.*

## What this record does not decide

- **Any matrix rung.** O-06 stays where the matrix has it. A record delivers nothing; the tickets below do.
- **The M5 index-access lowering, beyond one bounded observation made in passing.** O-06's M5 cell is owed and this record does not derive it. What it establishes is that the two cells do not block each other. What it *observed*, without reading the lowering implementations in full, is that `governed_index_access_capabilities` (`crates/tiler-compiler/src/governed.rs:222-337`) registers the reindex and broadcast with the unary `f32` signature a slice also has and with a deliberately empty emitted set, so the shape looks like one capability rather than the concatenate's seven-per-arity fork. That observation is carried into [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](../../../tickets/lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md) with the stop conditions that would refute it, not asserted as a conclusion here. The related observation that a selection's read coordinates are refused out of bounds at construction (`slice.rs:652-659`) is likewise an *M5* fact recorded there rather than an M4 conclusion drawn here.
- **The request boundary.** A slice program is refused under `operation-set` before any of this is reached, and lifting that is the region vocabulary's work, tracked for the two existing coordinate relations by [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](../../../tickets/admit-the-structural-families-into-the-scheduled-region-vocabulary.md). Registering the role neither lifts it nor depends on it.
- **The strided and symbolic relations.** Both stay reserved and refused by name under their own triggers — `RQ-OP-05` on Q-SHAPE-008 for the stride, an `IndexNode` variant carrying a `SourcedExtent` in a coordinate position for the offset. A fusion role is stated over a key, and the key admits neither relation, so nothing here touches either trigger.
- **[Q-SHAPE-006](../../open-questions.md#q-shape-006--finite-piecewise-access-maps).** A slice's read map is not case-split: one operand, one access, one affine coordinate per axis. This record reaches no piecewise pressure in either direction and leaves that question's restated trigger exactly as the concatenate's carrier left it.
- **Any cost claim.** No schedule containing a selection has been measured at any shape.
- **Whether a second dtype ever needs a selection.** A BF16 slice would be a second registered family under ADR 0091's directional-pair reading and is out of this record's subject entirely.

## Reproducible checks

Each is one command from the repository root, with the positive control that proves it can return something.

```sh
# 1. The fusion-role table holds eleven keys and no slice.
grep -n 'roles.insert(' -A 1 crates/tiler-compiler/src/fusion_legality.rs
#    Positive control: the same read finds reindex_f32_op and broadcast_f32_op,
#    so a missing key is an absence from a list with members rather than an
#    empty result.
#    Restated 2026-08-06: this read "nine keys", the count at this record's
#    base. The concatenate's role and the softmax's landed since, so the list
#    is eleven; the slice's absence — the fact this check observes — holds
#    unchanged.

# 2. The coordinate-relation arm of the contraction proof is closed over three
#    exact keys, and the slice is not among them.
grep -n 'fn is_exact_governed_same_family_pointwise' -A 50 crates/tiler-compiler/src/fusion_legality.rs
#    Positive control: the same read finds the ValueSource arm's constant guard,
#    so the match is being read rather than a comment.
#    Restated 2026-08-06: this read "closed over two keys" — reindex and
#    broadcast, the membership at this record's base. The concatenate's
#    admission added a third; the arm is still closed and the slice is still
#    outside it, which is what this check observes.

# 3. The family declares its mapping class as a registered definition fact, and
#    the concatenate declares no such fact.
grep -n 'fn slice_facts' -A 24 crates/tiler-ir/src/semantic/slice.rs
grep -n 'fn concatenate_facts' -A 28 crates/tiler-ir/src/semantic/concatenate.rs
#    Positive control: both reads return a nonempty record of named fields, so
#    the absent mapping-class field is an absence from an enumerated list.

# 4. The exhaustive index verifier requires total coverage of writes only.
grep -n 'let mut seen' -B 4 -A 2 crates/tiler-ir/src/index/builder/proof.rs
#    Positive control: the same read shows the AccessMode::Write test that
#    guards the allocation, so the read is reaching the guard and not a
#    declaration.

# 5. Region formation holds no operation allowlist.
grep -n 'fn operation_definition' -A 4 crates/tiler-compiler/src/region.rs
grep -n 'pure: matches!(definition.effect()' -B 6 crates/tiler-compiler/src/region.rs
#    Positive control: the second read shows the only property of a definition
#    the graph reads, so "no allowlist" is a statement about what the
#    construction does rather than about what a search failed to find.

# 6. The reference layer classes the three coordinate-mapping families together.
grep -n 'The third coordinate-mapping family' -A 10 crates/tiler-reference/src/standard.rs
#    Positive control: the read lands inside the registrar call it comments, so
#    the claim is about a registration and not a stray note.

# 7. The explain writer folds the fusion provider identity, not the role table.
grep -n 'FusionNumericalCapabilities::governed().provider()' -B 8 crates/tiler-compiler/src/explain.rs
#    Positive control: the surrounding comment names both governed providers, so
#    the read reaches the allowed-provider construction rather than a use site.
```

## Work this record filed

| Ticket | Rung | Why it is separate |
| --- | --- | --- |
| [`admit-a-fusion-role-for-the-sub-tensor-selection-slice`](../../../tickets/admit-a-fusion-role-for-the-sub-tensor-selection-slice.md) | M4 / R5 | The derivation is independent of every index-layer question, so holding it behind the lowering would park a landable rung behind work it does not need. |
| [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](../../../tickets/lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md) | M5 | O-06's M5 cell is owed and, unlike O-07's, the delivery graph names no owner for it. It is filed as implementation rather than scoping because the capability table's existing coordinate-relation entries already have the shape a selection needs, and it carries the two stop conditions that would convert it back into a fork. |

The `UNPLANNED_OPERATIONS` doc comment's silence about the slice entry (`policy.rs:789-810`) is folded into the first ticket rather than filed separately: it is one comment in the file that ticket already edits, and a ticket smaller than its own brief costs more than the change.

## What would make this record wrong

- **That no obligation ever reads a member's mapping class or its source coverage.** Every one of the nine is about rounding, order, purity, dtype homogeneity, or an exceptional value produced by arithmetic, and the check that establishes it is the walk above re-run against a slice member. If an obligation is added that reasons over how much of a source an access reads — a materialization-elision obligation is the plausible shape — this family would owe a re-derivation and the reindex would not.
- **That `FusionRegionStructure`'s four counts stay the identity constraint they are.** The seventh-role elimination rests on a fifth count moving every encodable region's content identity. If the structure identity is ever versioned for another reason, that argument loses its force and a seventh role would have to be re-eliminated on derivational grounds alone — which it still is, but on one argument rather than two.
- **That the contraction arm's soundness argument is about arithmetic rather than about data movement's shape.** It is stated as "inserting a pure data movement between two adds cannot introduce a product to fuse". A selection moves strictly less data than a reindex and introduces no operation; if the argument turns out to depend on the movement being *total*, the arm extension would need re-deriving, and the check is to re-read the arm's own comment against a non-surjective member.
- **That the request-boundary refusal is genuinely orthogonal.** The claim that registering the role delivers R5 and nothing else rests on `operation-set` refusing for a missing access-relation vocabulary rather than for a missing capability. If the region vocabulary's admission ever consults the fusion table, the two would stop being independent and the ticket ordering here would be wrong.
