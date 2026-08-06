---
schema: "tiler-doc/v1"
id: "tiler.research.program-planning.general-compilation-boundary"
kind: "research"
title: "General compilation boundary with bounded capability support"
topics: ["program-planning", "compiler-api", "capabilities", "extensions"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "adopted"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.architecture", "tiler.contract.optimizer"]
adopted_by: ["ADR-0069"]
ticket: "prototype-target-neutral-baseline-slice"
---

# General compilation boundary with bounded capability support

**Status:** research complete; accepted by ADR 0069

**Evidence boundary:** the precedents and dependency argument below are
primary-source synthesis. A bounded compiler slice exercises part of the
accepted boundary, but no retained experiment supports this report as an
`executable-model` of the general mature contract.

**Correction — 2026-07-31.** The evidence boundary above said *private*, and the
slice is no longer private: `prototype-public-compiler-api` landed the reviewed
`tiler_compiler::session` boundary, so an out-of-crate caller composes a request,
installs its own lowering registry, states an ordered numerical-contract
preference and its own target profiles, and reads the typed outcome. That is the
general entry point this report recommended, reached by the route it recommended.
What is unchanged is the evidence class: reachability is not coverage, the
admitted program subject is still the bounded one — a single input and a single
output over an `f32` pointwise or scale-bias-then-strict-serial-sum shape — and
every other valid program is still rejected without approximation, exactly as the
accepted disposition below requires.

**Correction — 2026-08-04, by the general-pipeline audit.** The admitted subject named in the paragraph above is no longer the one the compiler admits, and the sentence would now understate the boundary in one direction and overstate it in another. Recognition is a *general occurrence walk*, not a shape: `select_supported_strategy` in `crates/tiler-compiler/src/request.rs` checks three program-wide properties — at least one declared input, exactly one output, `f32` throughout — and then classifies the occurrence producing the output, walking outward through the occurrences feeding it. Any declared input arity is admitted, the elementwise dimension is the general `PointwiseF32Expression` vocabulary at any depth with shared subexpressions, and a two-operand `tiler.strict-tensor-contraction-f32` root is admitted through its own normalization. `normalize_serial_sum`, the function the 2026-07-31 correction and its consumers named, does not exist at this revision; `grep -rn 'normalize_serial_sum' crates/` returns one historical mention in a test's doc comment. What did *not* move is the report's actual claim: reachability is still not coverage, and every unadmitted program is still refused under a named rule without approximation.

**The evidence class also did not move, and the reason is now downstream of this boundary rather than at it.** The recognizer's generality is not yet reachable through the physical layer. Region formation and the general DAG partition search enumerate covers of an arbitrary verified DAG, and since 2026-08-04 and 2026-08-05 the physical provider answers for every region a cover places and program assembly derives its whole program from the cover; [the optimizer contract](../../compiler/optimizer.md#what-each-stage-is-general-over-today) states that per stage. What still bounds the reachable set is the *region vocabulary*: a schedule whose semantic members are not one of the partitions this recognizer pre-computed fails the request-subject binding, so no such region is ever proposed and no cover containing one is ever retained. A program the recognizer admits therefore still compiles only when the strategy it was classified into supplies a partition the schedule vocabulary can spell.

## The critical path to a naive but general compiled MIMO program

**Inference, 2026-08-04 — derived from the stage generality above, ordered by what blocks what.** "Naive but general" means: an arbitrary acyclic program over the supported operation set, several ordered named outputs, no fusion or parallel strategy required, one dispatch per region, conservative materialization at every cover edge. Four things stand between the current compiler and that program, and only the first is architectural.

1. ~~**A physical provider that proposes for an arbitrary region rather than for a pre-computed member set.**~~ **Landed 2026-08-04 by [`derive-physical-proposals-from-the-cover-region-subject`](../../../tickets/derive-physical-proposals-from-the-cover-region-subject.md).** This was the load-bearing item: until it existed, every other item below admitted programs nothing could implement. It was not a new seam — the provider trait, the neutral schedule vocabulary, the feasibility authority, and the typed decline channel were all landed and exercised — and what it needed was a proposal derived from the region subject the provider is handed, together with the region builder generalization the [minimum correct physical realization profile](minimum-correct-physical-realization-profile.md) identified: the builders took only the request and read the recognizer's own subject, so generalizing the provider alone would only have moved the match. Every region a cover places now receives a verified proposal or a typed decline naming the region-vocabulary wall it hit, and the written tensor role comes from the cover rather than from which whole-program recognizer matched. **This did not widen what compiles.** The three walls of item 4 are still walls; what changed is that each is now a refusal a reader can act on instead of an absence, and item 2 became reachable.
2. ~~**Program assembly generalized past three plan shapes.**~~ **Landed 2026-08-05 by [`assemble-a-kernel-program-from-an-arbitrary-cover`](../../../tickets/assemble-a-kernel-program-from-an-arbitrary-cover.md).** Assembly is derived from the cover's own regions, materialization edges, and named outputs for a cover of any region count, through one description (`CoverAssembly`) that the build path and the artifact receipt path both read, and `tiler-ir` needed no widening as predicted. The refusal is reclassified with it: a cover the assembler cannot express is a missing compilation capability naming the region, not invalid compiler output.

   **What it did not do is make a four-region program compile, and the ordering premise this item was filed under is refuted by that.** The item's dependency on 1 was justified by "a plan reaches assembly only once every region of its cover has an admitted implementation", with the implication that item 1 landing would make the new paths reachable. It did not: item 1 converted silence into typed refusals without widening the covered set, so the reachable covers are still the one- and two-region ones and the longest assembled program is still the three-stage split. The generality is a seam with its obligations proven — not a tested guarantee over four-region programs, which item 4 is what unlocks.
3. **Both `output_count() != 1` guards relaxed together** — the request boundary's and the artifact refinement's — with ordered output identity carried through the artifact encoding first. [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](../../../tickets/admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) owns it and states why relaxing either alone is worse than refusing.
4. **The named region-vocabulary wall**, already owned: no `ScalarProgram` spelling for the registered unary families. It bounds *which* programs the general path would then compile; it does not block the path itself. *Corrected 2026-08-06:* this item read "the three named walls" and closed with "and no reduction directly over a declared input". [`admit-a-reduction-over-a-declared-input-tensor`](../../../tickets/admit-a-reduction-over-a-declared-input-tensor.md) widened `tiler-ir`'s serial `StrictSerialSum` arm to the fold's declared contributor domain, so that wall is gone and `sum(x)` compiles to one region binding the input directly. *Corrected again 2026-08-06, second entry:* the remaining item read "no elementwise region over a materialized intermediate", which was one name covering two halves, and only one half survives. [`admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`](../../../tickets/admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary.md) separated a pointwise read's access position from the declared input its `TensorRole` names, so an elementwise region *may* now read one materialized intermediate and the epilogue's consumer half is expressible. What is still unspellable is the producer half of the reduction shape — a `StrictSerialSum` whose owning write targets an intermediate — which [`admit-a-strict-serial-fold-that-writes-a-materialized-intermediate`](../../../tickets/admit-a-strict-serial-fold-that-writes-a-materialized-intermediate.md) owns. A contraction already writes one, so `contract(a, b) * 2.0` is the first epilogue shape whose regions the vocabulary can spell; building them from a recognized program is the compiler-side dependent's work rather than a further vocabulary wall. *Corrected again 2026-08-06, third entry:* the producer half is gone too, so this item now names **one** wall — the unary families — and the materialized-intermediate entry is discharged in both halves. [`admit-a-strict-serial-fold-that-writes-a-materialized-intermediate`](../../../tickets/admit-a-strict-serial-fold-that-writes-a-materialized-intermediate.md) replaced the `write.tensor == TensorRole::Output` literal at every *committing* fold pass — the four serial arms, the split's final pass, and the cooperative tile — with `CommittedTensor::CoverAssigned`, on the derivation that where a fold's result goes is a property of the surrounding cover and not of the fold. The split's *partial* pass keeps `CommittedTensor::Exactly(Intermediate)`, because a partial is an unfolded fragment and is no cover's declared output. Both halves of `sum(x * x) * scale` are therefore spellable regions, and what remains for [`admit-elementwise-epilogues-over-a-materialized-intermediate`](../../../tickets/admit-elementwise-epilogues-over-a-materialized-intermediate.md) is entirely compiler-side: recognizing the shape, threading `RegionWrite` into the reduction spellings that still hard-code `TensorRole::Output`, and assembling the chain.

Items 1 and 2 had no owner when this list was written; both had one as of 2026-08-04, item 1 landed the same day, and item 2 landed 2026-08-05. Items 3 and 4 have live owners and dependency-correct edges, and **item 4 is now the one that gates the path**: with both compiler halves general, the first four-region program to compile is the first one whose regions the schedule vocabulary can spell. Nothing on this path requires a memo, a calibrated cost model, a boundary-property enforcer, or shared-work duplication — each of those improves a baseline that does not yet exist.

*Corrected 2026-08-06 — the gate moved off item 4 for the reduction-epilogue shape, and naming which item gates is now shape-dependent.* The sentence above was written when every four-region program was blocked on a region the vocabulary could not spell. That is no longer true of the epilogue chain: with the materialized-intermediate read and the committing-fold write both admitted, `prologue -> fold -> epilogue` is four spellable regions and nothing in `tiler-ir` refuses it. What gates *that* shape is now [`admit-elementwise-epilogues-over-a-materialized-intermediate`](../../../tickets/admit-elementwise-epilogues-over-a-materialized-intermediate.md) — a recognizer that classifies an operand produced by a fold or contraction, the `RegionWrite` threading its reduction spellings still lack, and the `regions` budget literal that ticket's own body records as spelled `3` rather than derived. Item 4 still gates the shapes that need a unary family. The distinction matters because "the vocabulary cannot spell it" and "the recognizer will not build it" are different refusals with different owners, and this list previously had only one name for both.

## Question

Should Tiler expose its first compiler slice through a graph-specific entry
point such as `profiles::serial_sum_baseline`, or should the compiler accept a
general semantic program and reject unsupported capability combinations
explicitly?

The serial `Sum` materialized baseline is deliberately narrow. The question is
whether that coverage boundary should become public compiler vocabulary.

## Existing Tiler constraints

**Fact:** the architecture contract already defines a consumer-independent
`CompilationRequest` over an immutable `SemanticProgram`, numerical contract,
shape environment, target profiles, frozen operation capabilities, budgets,
and options.

**Fact:** ADR 0026 separates representability from operation and product
support. ADR 0044 makes semantic and optional compilation capabilities
explicit in a frozen registry. A valid operation may therefore lack a selected
access, scheduling, target, or kernel-lowering provider.

**Fact, as of this report's writing and since superseded:** the executable model
then recognized one exact graph and used fixed two-stage and three-buffer Rust
arrays. Those cardinalities were evidence about the first strategy, not
invariants of `CompilationRequest` or a mature compiler product.

**Correction — 2026-07-31.** Both halves have moved, in the direction the
accepted disposition below asked for. Recognition is no longer one exact graph:
`ResolveLoweringCapabilities` resolves one index/access capability per recognized
occurrence against the registry the request carries, so which programs compile is
a property of the installed authority rather than of a hard-coded shape.
Cardinalities are no longer fixed arrays: `tiler_ir::program` carries `Vec`
stages and buffers, and the governed deterministic budgets that bound them are
request fields — widened from two regions and three buffers to three and four by
[`enumerate-the-split-reduction-on-the-planning-frontier`](../../../tickets/enumerate-the-split-reduction-on-the-planning-frontier.md)
so a split reduction's three stages over four values fit. That widening is
itself the evidence for the original sentence's point: the numbers changed, and
nothing downstream treated them as invariants.

**Inference:** publishing the fixed normalized graph or its cardinalities would
make current coverage look like the compiler's abstraction. Renaming the same
types behind a general `compile` function would only hide that coupling.

## Primary precedents

### Apache DataFusion

**Fact:** DataFusion's `PhysicalPlanner::create_physical_plan` accepts a
general `LogicalPlan`. `ExtensionPlanner::plan_extension` returns `None` when
one provider does not know a node, allowing another provider to try; the
default planner reports an error when no installed planner can produce an
execution plan.

Source inspected at DataFusion commit
`c3a288b97a1127c11b8c967f64c530d1cb8671b5`:
`datafusion/core/src/physical_planner.rs`.

**Inference:** general planner input does not imply universal physical support.
Capability resolution and explicit failure preserve extensibility without
creating an entry point for every supported logical pattern.

### Apache TVM

**Fact:** TVM exposes `tvm.compile(mod, target, ...)` as a unified entry point
for a `PrimFunc` or `IRModule`. Pipelines and targets determine which contents
can be lowered; the public entry point is not named after the currently
selected operation pattern.

Source: [TVM driver API](https://tvm.apache.org/docs/reference/api/python/driver.html).

### MLIR dialect conversion

**Fact:** MLIR conversion applies a target legality contract to general input
IR. Full conversion succeeds only if every required operation is legalized;
partial and analysis modes expose different incomplete-lowering contracts.
Legality may be dynamic for a particular operation instance.

Source: [MLIR dialect conversion](https://mlir.llvm.org/docs/DialectConversion/).

**Inference:** support is best represented as an instance-sensitive compiler
outcome, not as a claim that every representable operation has a realization.

## Options assessed

### Graph-specific public entry points

This accurately advertises the first executable coverage and can make a small
demo difficult to misuse. It also makes an exact graph pattern part of public
module and type identity, fragments extension dispatch, and creates migrations
whenever coverage grows from one pattern to arbitrary graphs.

### General entry point with an implicit support envelope

This keeps the public boundary aligned with semantic IR. The frozen registry,
compiler/provider revisions, target, numerical contract, and selected options
already determine realizability and output identity. Typed outcomes must
distinguish invalid input, missing capability, target infeasibility, and
internal verifier failure so that “general” does not imply silent fallback.

### General entry point plus a named support policy

A versioned support policy could preserve an intentionally maintained
acceptance envelope across compiler upgrades. Without two real maintained
envelopes, it duplicates the frozen provider set and compiler version and
creates a compatibility promise whose only member is today's test graph.
Search policy or product certification may justify this later; the serial-Sum
pattern does not justify it now.

## Accepted disposition

Use one general consumer-independent compilation boundary. Keep serial `Sum`
as a private strategy, conformance fixture, and explain-rule identity. Do not
publish an `experimental` namespace, a serial-Sum compiler profile, or the
current fixed-cardinality normalized/product types.

Before public exposure, generalize the request and result seams, use
variable-length verified collections for program cardinalities, and classify
unsupported capability and no-feasible-plan outcomes explicitly. The compiler
may initially realize only the accepted serial-Sum slice and must reject every
other valid program without approximation.

Defer a selectable support-policy type until at least two deliberately
maintained policies, a product-certification requirement, or a compatibility
need independent of the pinned compiler/provider identity exists.
