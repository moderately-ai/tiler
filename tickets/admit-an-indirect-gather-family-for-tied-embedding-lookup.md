---
id: admit-an-indirect-gather-family-for-tied-embedding-lookup
title: Admit an indirect gather access family
status: done
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface, reclassify-language-model-work-as-a-conformance-track]
related: [own-operation-family-support-matrix, design-model-ingestion-and-complete-execution, implement-index-domain-predicates]
scopes: [contracts/foundation, implementation/ir, implementation/reference, implementation/compiler, implementation/metal, contracts/navigation, contracts/decisions, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, indexing, gather, language-model, breadth, class-generic-capability]
---
## User-visible outcome

A program can use one tensor's values as coordinates into another — an indirect, tensor-data-derived access class that the admitted index vocabulary rejects by construction and that no composition of admitted families can express.

**Retitled 2026-08-04 under [`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md).** The outcome above read "A language-model program can read its own input: token IDs select rows of the embedding matrix". The access class is generic; a tied embedding lookup is the occurrence that found it and is the workload evidence below, never the thing that names or owns it. **The ticket id is deliberately unchanged**: five records outside this ticket's editable scopes link to it by filename — `docs/research/shapes/transformer-operation-and-shape-surface.md:166`, `docs/research/numerics/first-quantized-lm-profile.md:182`, `docs/research/program-planning/first-attention-program-vertical.md:164`, `docs/research/program-planning/model-level-qualification.md:356`, and `docs/research/program-planning/complete-model-ingestion-and-execution.md:105`/`:305` — so renaming the file would trade a workload-flavoured identifier for broken links a reader hits and no gate reports.

## Evidence prerequisite

**Fact — the admitted access language rejects it by construction.** [`docs/ir.md`](../docs/ir.md) Layer 2 bounds the initial index vocabulary to addition and negation, multiplication by a parameter-only expression, and Euclidean floor division or modulo by a proven-positive parameter-only expression, and states that "iteration-by-iteration multiplication and tensor-data-derived indices are rejected". A token ID read from an input tensor and used as a row coordinate is exactly a tensor-data-derived index. This is not a missing key over an existing access class; it is a missing access class.

**Fact — the corpus already tracks it as absent, with no owner.** [Q-SHAPE-007](../docs/open-questions.md#q-shape-007--indirect-gatherscatter-relations) states the trigger as "gather/scatter enters an active product profile" and records that closure "needs bounds, duplicate-write, determinism, and validation rules". The [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) mentions gather only inside the structural row's trigger — "Gather and scatter stay out until Q-SHAPE-007 triggers" — and gives it no row of its own, so the family has no recorded rung at all.

**Fact — the workload evidence, from the L2 derivation.** [The transformer operation and shape surface derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md) records one gather occurrence per forward pass of the pinned `Qwen/Qwen3-0.6B-Base` profile: `[T]` token IDs index a `[151936, 1024]` F32 matrix to produce `[T, 1024]`. The bounded rows fix `T` at 10 then 1 for the conformance row and up to 8192 for the benchmark matrix, and the pinned `vocab_size` is 151,936. **Fact — the same matrix is also a contraction operand.** The checkpoint carries `tie_word_embeddings: true` and no `lm_head.weight`, so one tensor serves the gather on the input side and the vocabulary projection on the output side; the semantic graph admits one value with two consumers, and a plan that allocates two copies doubles the model's largest single allocation.

**Inference — the ratio is why this is a boundary question and not a cost question.** One gather sits against 253 contractions in the same forward pass, so its execution cost is negligible. What it decides is whether the program has a boundary at all: with no admitted access class the model's first operation is not expressible, and the alternative is to move the lookup outside Tiler and hand the compiler a materialized `[T, 1024]` input, which is a different product boundary rather than a different implementation.

## Required delivery

One vertical carrying:

- **The access class.** An indirect relation in the index layer, with the bounds, duplicate-write, determinism, and validation rules Q-SHAPE-007 names as its closure condition. A read-only gather does not need the duplicate-write rule to be *implemented*, but it does need it *stated*, so that admitting scatter later is additive rather than a reinterpretation.
- **Semantic identity and validation.** A governed `OpKey` with an index-tensor operand of an admitted integer value type, a gathered-axis attribute, and validation that the result shape composes the index shape with the source's surviving axes.
- **The bounds obligation.** An index value outside `0..151936` must refuse or be validated at a named enforcement boundary. It may not clamp, wrap, or read out of bounds. `docs/ir.md` already fixes the shape of this: a semantic precondition is proved statically or the physical plan names a supported enforcement and publication boundary, and a semantic validation failure is never a plan miss.
- **Normative reference, lowering capability, target realization, and runtime binding**, on the same terms every other family owes them.
- **Bounded conformance evidence.** Token IDs at 0, at 151935, repeated, and out of range; an empty `T`; and the tied case where the same value feeds both a gather and a contraction, verified not to duplicate the allocation.
- **A matrix row** for the family, with its rung and its trigger, since it currently has none.

## Non-goals

Scatter, and any data-dependent output shape. The workload needs neither, and `docs/ir.md` separately holds data-dependent output shapes and device-produced launch dimensions as unsupported.

## Reconsideration trigger

Active now: the selected workload's first operation requires it. If the product boundary moves so that embedding lookup happens on the consumer side and Tiler receives materialized activations, this family stops being required by this workload — and that is a decision for [`design-model-ingestion-and-complete-execution`](design-model-ingestion-and-complete-execution.md) to make explicitly rather than a gap to leave open.

## Repaired before dispatch, 2026-08-07 — the "unowned and untracked" framing was false in four ways

Verified by the coordinator against the corpus. The workload evidence, the index vocabulary quote, the bounds-obligation shape, and the non-goals all **verify unchanged**; what follows corrects the tracking claims built on top of them.

### Struck: "the corpus already tracks it as absent, with no owner"

**Four records name this ticket as the owner.** `docs/open-questions.md:280` — "The gather half's trigger **has fired**, and it is re-owned rather than left standing as a deferral (2026-08-04)"; `docs/roadmap.md:417`, `:479`, `:481`; and `docs/research/semantic-graph/operation-family-delivery-graph.md:187`. The correction is therefore not "record an absence" but "advance a tracked, owned row" — a different and smaller job.

### Struck: the Q-SHAPE-007 trigger quote

The ticket quotes the trigger as "gather/scatter enters an active product profile". **That string exists nowhere in the tree except inside this ticket.** The current Q-SHAPE-007 (`docs/open-questions.md:281`) reads "Trigger, for the half that has **not** fired: scatter." The gather trigger is spent, not pending.

### Struck: the roadmap quote and the "no recorded rung" claim

"Gather and scatter stay out until Q-SHAPE-007 triggers" **no longer exists** in `docs/roadmap.md` (`grep -c` returns 0), and the structural row now says the opposite at `:479`: "Scatter likewise stays out under Q-SHAPE-007, but **gather no longer does** … it is a family this row does not carry rather than a class held behind an unfired trigger." Gather appears at `:353`, `:408`, `:417`, `:479` and `:481` — five sites, not one.

Two rungs are recorded: `docs/research/shapes/transformer-operation-and-shape-surface.md:67` and `:87` (R1), and `operation-family-delivery-graph.md:97` carries an explicit row — `| **O-08** Indirect gather | F-34 | owed; live ticket | owed | owed | owed |`. **The surviving true clause is narrower:** no row in the *roadmap matrix* specifically.

### Link line numbers refreshed, and a sixth record added

`transformer-operation-and-shape-surface.md` is `:188` (not `:166`); `model-level-qualification.md` is `:358` (not `:356`); `complete-model-ingestion-and-execution.md` is `:105` ✓ and `:322` (not `:305`). `first-quantized-lm-profile.md:182` and `first-attention-program-vertical.md:164` are correct. The unlisted sixth is **`docs/research/semantic-graph/operation-family-delivery-graph.md:187` and `:308`** — so six records, not five.

### Two scopes added, one of them an ADR obligation this ticket did not name

- **`contracts/decisions`.** ADR 0046 is `accepted`, and its `:48-49` carries the normative rejection this work must lift — "It rejects iteration-by-iteration multiplication and **tensor-data-derived indices**" — with `:73-74` adding that "Data-dependent gather, scatter, sparse iteration, and data-dependent cardinality **require later explicit IR contracts**." Admitting this access class supersedes or extends an accepted ADR, and the ticket named no ADR obligation at all. Per AGENTS.md, if the branch cannot land in `docs/decisions/`, preserve a **verbatim-landable ADR body** and file a carrier ticket — editing during transfer creates a fork.
- **`research/semantic-graph`**, for the O-08 row at `:97` and the F-34 join row at `:308`. `docs/roadmap.md:437` makes that file the authority for which ticket holds each family's next step, so it moves when the family lands.

### "Runtime binding" is not this ticket's to deliver

Required Delivery demands "runtime binding, on the same terms every other family owes them". `crates/tiler-runtime/**` is `implementation/runtime`, **not declared here and deliberately not added** — that work is owned by `admit-a-storage-carrier-for-integer-program-inputs`, whose own `dependencies` list contains *this* ticket. So the item is both out of scope and assigned downstream. **Drop it from Required Delivery** and note the discharge. The semantic side needs no work: `tiler::u32@1`, `i32` and `i64` are already registered in `crates/tiler-ir/src/semantic/catalog.rs`.

`implementation/conformance` is **not** added: the four landed peer family tickets all kept their evidence inside `implementation/ir` + `implementation/reference` and declared no conformance scope. Follow the peers.

### Struck: the unverifiable conformance item

"the tied case where the same value feeds both a gather and a contraction, **verified not to duplicate the allocation**" is degenerate in both readings. In the workload's real shape it can never say *yes* — `complete-model-ingestion-and-execution.md:68` records that under W-C those are two programs, so the matrix is bound twice, "binding two copies costs **622,329,856 bytes** and **nothing below the consumer can tell that the two bindings name one tensor**". Non-duplication is a consumer obligation with no enforcer here. In a synthetic single-program shape it can never say *no*, because inputs are bound rather than allocated, so one value with two consumers cannot duplicate. **Replace it with the two-program binding fact**, and hand the tied case to `assemble-the-embedding-and-vocabulary-projection-programs`, which already records that cost and is the real owner.

### Struck: the Reconsideration trigger

It cites `design-model-ingestion-and-complete-execution` as still owing the product-boundary decision. That ticket is `done` and **made the decision**, under the heading "The input boundary: the gather stays inside" — `complete-model-ingestion-and-execution.md:103`, `:105` ("It is made here."), with IN-A surviving at `:109` and IN-B eliminated at `:110`, confirmed by `docs/roadmap.md:408`. As written the trigger can fire only by superseding an accepted decision. Restate it as settled.

## Worker Fact audit, 2026-08-07, at base `411e09bf`

**Citation convention for this block.** Every `path:line` below is pinned to base `411e09bf` and is *evidence about that tree*, not a durable pointer — several of these entries exist precisely because an earlier line citation drifted. Where the claim is about content rather than about a location, the quoted text is the anchor and the line number is a convenience; re-pin against the merged tree rather than trusting the ordinal. Code citations name a symbol (`AccessData`, `bind_marker`, `input_resolved`) so they survive movement.

Every source this ticket names was re-read in full at the base commit before any edit. The audit covers the original body **and** the 2026-08-07 repair block above, because a repair block is a claim like any other. Verdicts below are per-Fact; where a claim is false or imprecise the correction is stated here rather than by rewriting the claim out of existence.

### The original body's Facts

| Claim | Verdict | Evidence |
| --- | --- | --- |
| The admitted access language rejects a tensor-data-derived index by construction (`:24`) | **Verified, with one omission** | `docs/ir.md:1036` reads verbatim "Iteration-by-iteration multiplication and tensor-data-derived indices are rejected." See the qualification below. |
| The corpus tracks it as absent, with no owner (`:26`) | **False** | Already struck above; the strike is itself re-verified below. |
| The workload evidence — one gather per forward pass, `[T]` into `[151936, 1024]` (`:28`) | **Verified** | `transformer-operation-and-shape-surface.md:52` (`\| Indirect gather \| 1 \| the tied embedding lookup \|`), `:97` and `:111` (`[151936, 1024]`; vocabulary 151,936 static), `:116` (`1 ≤ T ≤ 8192` for B1; `T = 10` then `T = 1` for C1). |
| The same matrix is also a contraction operand (`:28`) | **Verified** | `first-metal-lm-workload.md:111` (`\| vocab_size \| 151936 \| tie_word_embeddings \| true \|`) and `:143` ("the inventory contains no `lm_head.weight`"). |
| One gather against 253 contractions (`:30`, Inference) | **Verified** | `transformer-operation-and-shape-surface.md:44` and `:55`. |
| Non-goals: `docs/ir.md` holds data-dependent output shapes and device-produced launch dimensions unsupported (`:45`) | **Verified** | `docs/ir.md:1085-1086`: "Data-dependent output shapes and device-produced/indirect launch dimensions are initially unsupported." |
| The Reconsideration trigger (`:49`) | **False** | Already struck above; re-verified below. |

**The omission in the surviving index-vocabulary Fact, which does not overturn it.** `docs/ir.md:1036` now opens the sentence the ticket quotes with a caveat the quotation drops: "**This paragraph states an admitted vocabulary, and the implemented one is narrower; the paragraph after it says by how much.**" The Fact's conclusion is unaffected — a narrower implementation rejects at least as much — but a reader who follows the citation finds a paragraph making a weaker claim than the ticket's framing implies. The *implemented* rejection is stronger and is the load-bearing one; it is stated as its own Fact below.

**Fact — the implemented index layer cannot express the access, and this is a structural fact rather than a vocabulary choice.** Read in full at this base: `crates/tiler-ir/src/index/model.rs:105` declares `IndexNode` with exactly five variants — `Constant`, `Dimension`, `LinearCombination`, `FloorDiv`, `Modulo` — and every operand of every one is a literal, a domain-dimension ordinal, or a `SourcedExtent`/`SourcedIndexInteger`, each of which is `Static`/`Literal` or one declared `ShapeSymbol`. No variant reads tensor data. `IndexExprClass` at `model.rs:58` has three variants — `Affine`, `QuasiAffine`, `SemiAffine` — and **no `DataDependent`**; its `join` at `:83` is an exhaustive match written so that adding a class is a build error. Decisively, `AccessData` at `model.rs:138` carries `tensor: u32` — *one* tensor ordinal — so an access cannot name a second tensor as a coordinate source at all. `LogicalAccess` at `crates/tiler-ir/src/schedule/model.rs:244` carries seven variants and none is data-dependent. The gap is therefore not a missing `IndexNode` variant; it is that the access record has no place to put a second tensor.

### The repair block's own claims

| Claim | Verdict | Evidence |
| --- | --- | --- |
| The Q-SHAPE-007 trigger quote exists nowhere but this ticket | **Verified** | `grep -rn "enters an active product profile" docs tickets crates` returns only this ticket's own two lines. `docs/open-questions.md:281` reads "Trigger, for the half that has **not** fired: scatter." |
| "Gather and scatter stay out until Q-SHAPE-007 triggers" no longer exists in `docs/roadmap.md` | **Verified** | `grep -rn "Gather and scatter stay out" docs` returns no `docs/roadmap.md` hit; the only hits are two tickets recording the correction. `docs/roadmap.md:479` carries the quoted opposite verbatim. |
| Gather appears at roadmap `:353`, `:408`, `:417`, `:479`, `:481` — five sites | **Verified** | `grep -n -i gather docs/roadmap.md` returns exactly those five. |
| Two rungs are recorded | **Verified** | `transformer-operation-and-shape-surface.md:67` ("R1; not on the matrix as its own row") and `:87` ("The indirect gather is still R1 and still has no row of its own"); `operation-family-delivery-graph.md:97` carries `\| **O-08** Indirect gather \| F-34 \| owed; live ticket \| owed \| owed \| owed \|`. |
| `docs/roadmap.md:437` makes the delivery graph the authority for each family's next step | **Verified** | Verbatim at `:437`. |
| ADR 0046 `:48-49` and `:73-74` | **Verified, and materially incomplete** | See *The ADR obligation is smaller than the repair block states* below. |
| Runtime binding is not this ticket's | **Verified** | `admit-a-storage-carrier-for-integer-program-inputs` is `todo`, declares `implementation/ir, implementation/artifact, implementation/frontend, contracts/artifacts`, and lists this ticket among its `dependencies`. |
| `implementation/conformance` deliberately not added; follow the peers | **Verified** | The slice landing `00d36766` put its evidence in `crates/tiler-reference/tests/slice_conformance.rs` and touched no conformance crate. |
| The struck conformance item, and the 622,329,856-byte figure | **Verified** | `complete-model-ingestion-and-execution.md:68` verbatim, including "(0.5796 GiB)". |
| The struck Reconsideration trigger | **Verified** | Heading at `:103`, "It is made here." at `:105`, IN-A `Yes` at `:109`, IN-B `No` at `:110`; `design-model-ingestion-and-complete-execution` is `status: done`. |

#### Three claims in the repair block are themselves wrong, and are corrected here

**1. `complete-model-ingestion-and-execution.md:322` is `:324`.** The repair block corrected the original body's `:305` to `:322`; the second link to this ticket in that file is at **`:324`**, inside the delivery-ticket table. `grep -rn "admit-an-indirect-gather-family-for-tied-embedding-lookup" docs` returns `:105` and `:324` for that file and nothing at `:322`. The repair block replaced a two-line drift with a two-line drift in the other direction.

**2. "Six records, not five" double-counts, and contradicts a scope the same repair block added.** `operation-family-delivery-graph.md:308` is the F-34 join row `| F-34 gather and indexed read | O-08 | *(none)* |` — it contains neither this ticket's filename nor a link, so it is not a link site. More importantly, the original body's claim at `:20` is about records *outside this ticket's editable scopes*, and the same repair block added **`research/semantic-graph`** as a scope, which makes `operation-family-delivery-graph.md` editable here. The count of records outside editable scopes is therefore **five, unchanged** — the delivery graph moved *into* scope rather than onto the list. The five are `transformer-operation-and-shape-surface.md:188`, `first-quantized-lm-profile.md:182`, `first-attention-program-vertical.md:164`, `model-level-qualification.md:358`, and `complete-model-ingestion-and-execution.md:105`/`:324`. (`docs/roadmap.md:417`/`:479`/`:481` and `docs/open-questions.md:280` also link by filename and are correctly excluded, being `contracts/navigation`, a declared scope.)

**3. "Four records name this ticket as the owner" counts nothing consistently.** The sites are `open-questions.md:280`, `roadmap.md:417`/`:479`/`:481`, and `operation-family-delivery-graph.md:187` — **three records at five sites**. Neither reading is four. The substance — that the family is owned and tracked rather than unowned — is verified and unaffected.

#### The ADR obligation is smaller than the repair block states, and getting this right changes what must be written

The repair block says "Admitting this access class supersedes or extends an accepted ADR". It cites ADR 0046 `:48-49` (the rejection) and `:73-74` ("Data-dependent gather, scatter, sparse iteration, and data-dependent cardinality **require later explicit IR contracts**"), both verified verbatim. What it does not cite is **ADR 0046's own Consequences section, `:86-87`**: "Indirect operations remain addable without weakening the verifier for the initial direct-access language."

**Inference, and it is the audit's most consequential finding.** ADR 0046 already contemplates this addition and states the condition under which it is not a supersession — that the verifier for the initial direct-access language is not weakened. Read together with `:73-74`, admitting a *semantic* gather family is the "later explicit IR contract" ADR 0046 defers to, **not an amendment to its status**. The obligation is therefore to write a new ADR that supplies that contract and leaves ADR 0046 `accepted` and its index-expression rejection intact — which is a materially different and smaller document from one that reopens an accepted decision. Concretely: the new record must not admit a tensor-read form into `IndexNode`, because doing so *would* weaken the verifier for the direct-access language and would put the two records in conflict.

#### One further imprecision, in the repair block's discharge of the runtime item

"The semantic side needs no work: `tiler::u32@1`, `i32` and `i64` are already registered in `crates/tiler-ir/src/semantic/catalog.rs`" is **verified as to registration** (`catalog.rs:336-339`, `:300-303`, `:306-309`) and **imprecise as to consequence**. Two things do not exist:

- **No Rust `ValueTypeMarker` is bound to any integer identity.** `StandardSemantics::register` binds exactly one marker — `registrar.bind_marker::<F32>(F32::resolved_type())` at `crates/tiler-ir/src/semantic/registry.rs:2267`. So `SemanticProgramBuilder::input::<U32>` does not exist; a `[T]` index operand must be declared through `input_resolved` (`crates/tiler-ir/src/semantic/program.rs:516`), which validates the type against the frozen registry and is sufficient.
- **No *reference* value type is registered for any integer identity.** `StandardReferenceProvider::register` registers `F32::resolved_type()` alone, so `tiler-reference` cannot hold or validate a `tiler::u32@1` tensor today. That is real work, it sits in `implementation/reference`, and it is this ticket's. The reference tensor representation itself imposes no obstacle: `crates/tiler-reference/src/tensor.rs:36` makes `ReferenceElement` an opaque `Vec<u8>` carried beside a `ResolvedValueType`, so an integer element needs a validator and a decoder rather than a new payload kind.

#### A stale claim in a record this ticket cannot edit

`model-level-qualification.md:358` states that `grep -rhoE '"tiler::[a-z0-9-]+@[0-9]+"' crates/ | sort -u` "lists 24 registered keys and none is a gather". Run at this base the command returns **26**, and the list mixes operation keys with value-type keys (`tiler::f32@1`, `tiler::bf16@1`, `tiler::complex@1`, `tiler::strict-affine@1`), so the count was never a count of operations. **The load-bearing half is verified and unchanged: none of the 26 is a gather.** The file is `research/program-planning` and outside this ticket's scopes, so the count is reported rather than repaired.

### What the audit changes about the work

Nothing in the workload evidence, the bounds obligation, or the non-goals moved. Two things about the *shape* of the delivery did: the ADR is a new subordinate contract rather than an amendment to ADR 0046, and the reference crate owes an integer value-type registration that the repair block's discharge of the runtime item reads past.

## Outcome

**Delivered at `411e09bf` + this branch.** `tiler::gather-f32@1` is a **registered semantic family with a reference evaluator and no lowering** — the coherent boundary, stated as such rather than as a way-point. Maturity and evidence tiers are given per claim below because they are five different claims and only three are made.

### What is claimed, by tier

| Claim | Maturity | Evidence tier |
| --- | --- | --- |
| A registered family | **implemented support** | exhaustive finite over the registry: `the_family_is_registered_and_carries_no_realization_law` reads the frozen registry rather than a list |
| A reference evaluator | **tested guarantee**, bounded to the covered shapes | empirical, bounded — 10 conformance cases + 19 semantic cases, each with a retained perturbation |
| A derived fusion legality | **not claimed, and deliberately absent** | — the family takes no `FusionOperationRole`, so `classify` returns `None` |
| A region reaching a `VerifiedKernel` | **not claimed** | — no index region can express the access |
| A device-verified result | **not claimed** | — nothing dispatched |

The bounds rule is an **empirical** guarantee at the reference boundary and `Unknown` everywhere else: there is no physical enforcement boundary, which is one of the two reasons no occurrence reaches a plan. The `[151936, 1024]` extents are exercised as a **shape** only; 155,582,464 elements is outside the reference evaluator's governed tensor bound, so the last-coordinate case uses its structural analogue and the conformance row says so.

### The ADR obligation, and why it was smaller than the ticket assumed

The pre-dispatch repair block framed this as "supersedes or extends an accepted ADR". The audit above found ADR 0046's *Consequences* already state that "indirect operations remain addable without weakening the verifier for the initial direct-access language", and its Decision defers "data-dependent gather … to later explicit IR contracts". So [ADR 0107](../docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md) **supplies that deferred contract** and leaves ADR 0046 `accepted` with its index-expression rejection intact. That also fixed what the record may not do: it admits **no** tensor-read form into `IndexNode`, because doing so is exactly the weakening ADR 0046 conditions on.

ADR 0107 is `proposed`. `accept-adr-0107-indirect-gather-semantic-family` is `awaiting-decision` and is Tom's.

### Public boundary — a labelled draft under ADR 0075

**Included:** `gather_f32_op`, `GATHER_AXIS_ATTRIBUTE`, `GatherAxis`, `GatherError` and its diagnostic codes, `gather_axis`, `gather_result_shape`, `decide_gather_index`, `gather_index_resolved_type`, `F32Gather`, the six `GATHER_FACT_*` field IDs, and `ReferenceOperationError::GatherIndexOutOfBounds`. **Excluded and refused by name:** scatter, signed index identities, every unsigned identity but `tiler::u32@1`, and any data-dependent output shape. Not accepted.

### Required Delivery, item by item

- **The access class** — delivered as a *semantic* class. Bounds, duplicate-write, determinism, and validation rules are all stated in the normative definition; three are implemented and duplicate-write is stated-not-implemented because a read performs no write.
- **Semantic identity and validation** — delivered. Governed `OpKey`, `tiler::u32@1` index operand, gathered-axis attribute, result shape composed from the index shape and the source's surviving axes.
- **The bounds obligation** — delivered at a named enforcement boundary (the reference evaluator). It refuses naming position, value, and extent; it never clamps and never wraps. **No physical enforcement boundary exists**, which is stated rather than implied.
- **Normative reference** — delivered. **Lowering capability, target realization** — *not* delivered, and split out rather than left implied: `admit-the-indirect-access-class-into-the-index-layer` (a decision, not an implementation) and `emit-the-indirect-gather-on-metal` (`blocked` on it). **Runtime binding** was struck pre-dispatch and stays struck.
- **Bounded conformance evidence** — delivered in `crates/tiler-reference/tests/gather_conformance.rs`, following the four landed peer families into `implementation/reference` rather than the conformance crate.
- **A matrix row** — delivered in `docs/roadmap.md`, under *Indirect gather*.

### One stale claim found in a record outside these scopes

`docs/research/program-planning/model-level-qualification.md`'s `A-token-out` row states the command `grep -rhoE '"tiler::[a-z0-9-]+@[0-9]+"' crates/ | sort -u` "lists 24 registered keys". It returned **26** before this branch and **27** after. The count also mixes operation keys with value-type keys, so it was never a count of operations. Its load-bearing half — "none is a gather" — was true and is now false by this landing, so the row needs re-reading by whoever owns `research/program-planning`.

## Outcome — done, 2026-08-07

Landed at merge **`56046f77`** (worker commits `fab1f6db` audit, `cf9578ee` family). 22 files. `make full` exit 0 on the merged tree — **1,090 release tests**, 3,127 workspace tests, 29 of them new.

### ADR 0046 was never superseded, and the coordinator's brief was wrong to say so

This is the finding that changed the shape of the work. The brief framed admitting a tensor-data-derived index class as superseding or extending an accepted ADR. **It does neither.** ADR 0046's own Consequences already state that "**Indirect operations remain addable without weakening the verifier**" for the initial direct-access language, and its Decision defers data-dependent gather to "**later explicit IR contracts**". So ADR 0107 *supplies the contract 0046 deferred*: 0046 stays `accepted`, its rejection intact and untouched by this branch — coordinator-verified at lines 74 and 86, and by confirming the file is absent from the diff.

That reframing also fixed what the record may **not** do: admit no tensor-read form into `IndexNode`.

### Three errors in this ticket's own 2026-08-07 repair block, all the coordinator's

1. The repair "corrected" a citation to `:322`. **It is `:324`** — a two-line drift replaced by a two-line drift the other way.
2. "**Six records, not five**" double-counts: `operation-family-delivery-graph.md:308` contains no link, and the same repair block added `research/semantic-graph` as a scope, moving that record *into* scope. Outside editable scopes it is **five, unchanged**.
3. "**Four records name this ticket as owner**" is **three records at five sites**.

Also corrected: "the semantic side needs no work" read past the reference crate — no integer identity had a registered reference value type, which was real work.

### The boundary is the decision, not a deferral inside it

`AccessData` carries **one tensor ordinal** and `IndexNode` has no variant reading tensor data, so the gap is the access record's *shape*, not a missing expression form. The family is admitted at the semantic layer and **deliberately nowhere below**.

**No fusion role, on purpose.** Slice and concatenate took `CoordinateRelation`, whose contract says its aliasing is the index verifier's concern — false of a gather, since the verifier cannot bound a coordinate it cannot see. `classify` returns `None`, so no region derives legality. Fail-closed.

| Claim | Maturity | Evidence |
| --- | --- | --- |
| Registered family | implemented support | exhaustive-finite over the frozen registry |
| Reference evaluator | tested guarantee, bounded | empirical, 29 cases with retained perturbations |
| Derived fusion legality | **not claimed** | deliberately absent |
| Region → `VerifiedKernel` | **not claimed** | no index region expresses it |
| Device-verified | **not claimed** | nothing dispatched |

`[151936, 1024]` is exercised as a **shape only** — materializing it exceeds the reference tensor bound.

Six perturbations, each on the subject: clamping instead of refusing returns row 4 exactly as predicted; ignoring the index operand fails 7 of 10; appending instead of composing the shape transposes the result; dropping the signed refusal changes the diagnostic code; skipping registration and removing the `UNPLANNED_OPERATIONS` entry each fire their own gate. The pinned explain digest was recomputed **on this tree** per its ledger instruction, `de9ad4cc` → `f99d1e5e`.

### Owed onward

**ADR 0107 is `proposed`** and `accept-adr-0107-indirect-gather-semantic-family` is `awaiting-decision` — Tom's, under ADR 0075, with the included and excluded surface enumerated. The remainder is mapped, not landed: `admit-the-indirect-access-class-into-the-index-layer` (itself a decision bounded by ADR 0046's condition) and `emit-the-indirect-gather-on-metal`, blocked on it. `model-level-qualification.md`'s stale key count is filed separately.

The audit's `path:line` entries are evidence about base `411e09bf` and predate `make citations`; they should be re-pinned against the merged tree if reused.
