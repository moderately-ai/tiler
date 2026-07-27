---
schema: "tiler-doc/v1"
id: "tiler.contract.optimizer"
kind: "contract"
title: "Optimizer model"
topics: ["optimizer", "search", "planning"]
contract_status: "accepted"
implementation_status: "partial"
evidence: ["tiler.research.region-search.exhaustive-region-oracle", "tiler.research.reference.normative-reference-slice", "tiler.research.cost-model.bootstrap-cost-model", "tiler.research.program-planning.general-compilation-boundary"]
---

# Optimizer model

**Status:** accepted research contract; bounded prototype implementation

The first private compiler slice now retains complete materialized and fused program alternatives, carries exact structural metrics, and selects the fused program only when it strictly Pareto-dominates the baseline. Its stable policy key makes no latency claim. A missing per-operation *fusion numerical* capability, candidate-budget exhaustion, or fused target infeasibility rejects only the fused alternative; failure of a compiler-produced verifier remains a hard compiler error. General memo search, partitioning, and calibrated cost estimation remain unimplemented.

**Fact — no installed-provider constant gates the fused alternative.** `crates/tiler-compiler/src/request.rs` used to carry two named lowering-provider constants, one materialized and one optionally installed fused serial-sum provider, and an absent fused constant suppressed the whole-program candidate before its numerical equivalence was ever proved. Both constants are gone. Whether a whole-program candidate is retained is now decided by [fusion legality](fusion-and-scheduling.md#legality) and typed target feasibility alone, and each retained alternative's lowering authority is whatever the request's installed registry resolved for its member occurrences. A missing *lowering* capability is not in this class at all and never rejects a single alternative; [capability resolution](#lowering-capability-resolution-and-index-region-refinement) states what it does instead.

The bounded slice rederives alternative identity, structural cost, KIR,
program, artifact receipt, and selection from the verified semantic/request
subject before returning a portfolio. Selection authority is the verified
alternative identity under a named cost model, not a caller-editable vector
index or stored cost. Explain records retain typed subjects, evidence class,
budget actual/limit pairs, feasibility facts, and provenanced cost values.

## Ownership boundary

This document owns planning phases, rule contracts, alternative retention,
search bounds, costing inputs, and explainability. It consumes verified IR
schemas from the IR contract and does not redefine their fields or backend
resource limits.

Tiler borrows selected techniques from property-aware database optimizers while
using a tensor operation/value DAG, access-aware fusion regions, and explicit
GPU schedules. DataFusion is useful vocabulary for semantic/executable
separation and boundary enforcement, but it is not the structural template for
Tiler's graph or search algorithm.

The contract synthesizes the [region oracle](../research/region-search/exhaustive-region-oracle.md),
[index/access model](../research/indexing/index-access-model.md),
[scheduled-region model](../research/scheduling/scheduled-region-model.md),
[whole-program plan](../research/program-planning/kernel-program-buffer-plan.md),
and [structured-kernel verifier](../research/kernel-ir/structured-kernel-ir-verifier.md).

## Compilation boundary and failure classes

Everything below is reached through one general, consumer-independent
compilation boundary over a verified `SemanticProgram` and explicit request
inputs. Under ADR 0069 there is no graph-specific entry point, `experimental`
namespace, or serial-Sum support profile. A bounded vertical slice remains a
private strategy, conformance fixture, and explain identity; its fixed region,
stage, entry, and buffer cardinalities are not request or result invariants.

The boundary returns either general target-neutral program products or a typed
outcome drawn from five distinct failure classes:

- **invalid request:** the semantic program, resolved numerical contracts,
  shapes, frozen registry, or request inputs are malformed;
- **missing compilation capability:** the program is valid, but no installed
  access, scheduling, lowering, or provider capability covers it;
- **infeasible plan:** every candidate is intrinsically invalid or rejected by
  typed target feasibility;
- **exhausted bounded search:** a declared candidate or expansion budget
  stopped exploration before a complete plan was selected; and
- **compiler IR verification failure:** a compiler-produced index, schedule,
  kernel, or program value failed its authoritative verifier.

These classes are not interchangeable. A valid program that lacks coverage is
never reported as malformed, and an unsupported case fails closed with an
explainable reason rather than being approximated to retain a fast path. A
budget that stops one growth path while complete coverage survives is an
explain reason on the selected plan, not this failure class. A verifier failure
is invalid compiler output and remains a hard error rather than a costed
rejection.

An exhausted *proof* budget is a sixth thing, and it is deliberately not a failure class. When an analysis cannot afford to decide a predicate, nothing about its subject has been disproved: the analysis stopped. Compilation therefore continues, and the trace carries a typed budget stop naming the exhausted resource beside an explicit `Unknown` assessment of the predicate that stayed unproven. Reporting that as an infeasible plan would confuse hard feasibility with an exhausted analysis, which is the confusion the separation of feasibility from cost exists to prevent; admitting it silently would report an absent proof as a proof. [Index-region refinement](#refinement-is-exhaustive-finite-evidence-with-an-explicit-gap) is the implemented instance.

## Planning model

```text
SemanticTensorGraph
  -> deterministic normalization
  -> bounded logical exploration
  -> overlapping RegionCandidates
     |-> independent complete-cover enumeration ---------|
     `-> checked schedules + ImplementationFrontier -----|
  -> compatible complete physical-plan selection
  -> structured KIR refinement
  -> KernelProgram or guarded ProgramPortfolio
```

The optimizer must distinguish:

- **logical equivalence:** expressions compute the same tensor under a stated
  numerical policy;
- **fusion legality:** a region can be implemented correctly as one kernel;
- **physical feasibility:** a schedule fits target capabilities and resources;
- **profitability:** the complete plan is preferable to legal alternatives.

## Named stages and verifier boundaries

The initial optimizer pipeline has explicit stage names and cannot skip their
verification boundaries:

1. `VerifySemanticRequest` checks the graph, resolved numerical contracts,
   shapes, and frozen operation registry.
2. `NormalizeSemantics` produces one deterministic canonical graph.
3. `ExploreLogicalAlternatives` adds only proved contract-preserving forms.
4. `EnumerateRegionCandidates` forms connected convex semantic regions and
   retains complete singleton coverage.
5. `ResolveLoweringCapabilities` resolves exactly one index/access lowering
   capability per recognized occurrence against the frozen lowering registry the
   request carries. It selects an authority and drives none, so it proves
   nothing about emitted work.
6. `LowerIndexRegions` drives each resolved provider through the canonical index
   builder, derives width-independent domains/access maps, and proves read
   bounds plus exact unique ordinary writes against the occurrence its
   capability was resolved for.
7. `EnumerateCompleteCovers` independently enumerates legal whole-graph covers;
   it does not select schedules or implementations.
8. `ExploreScheduledRegions` intrinsically verifies normalized schedules for
   individual legal regions. Typed target-feasibility assessment then admits
   bounded per-region physical frontiers. This authority does not require a
   previously selected global cover.
9. `SelectCompletePhysicalPlans` joins complete covers with compatible local
   implementations, boundary contracts, proposed materializations,
   dependencies, and guards. It emits a checked selected-plan or portfolio
   receipt for cover/implementation compatibility, not final executable-program
   authority. Buffer requirements remain provisional at this stage.
10. `RefineStructuredKernels` lowers each selected scheduled kernel and proves typed,
    effect-safe refinement of exactly that schedule before backend emission.
11. `AssembleKernelPrograms` constructs verified executable programs from the
    checked physical-plan receipt and verified KIR. Only this post-KIR verifier
    authoritatively checks executable stage coverage, buffers, initialization,
    lifetimes, aliasing, storage handoffs, ABI/launch references, and routing.

Stages 5 and 6 run together, per recognized occurrence, before the first cover is enumerated: grouping occurrences the installed authority cannot lower would enumerate plans nothing could realize. They are two authorities in one pass, not one stage — resolution answers *which* authority lowers an occurrence and refinement answers whether the work that authority emitted realizes it.

The explain vocabulary spells these two stages `CapabilityResolution` and `KernelRefinement`. `KernelRefinement` carries both stage 6 and stage 10, which are different obligations over different subjects: the rule key `kernel.index-region-refinement.v1` names an index region refining a semantic occurrence, and `kernel.plan-refinement` names a structured kernel refining a selected schedule. A trace reader must not read one as evidence for the other.

Semantic, index, schedule, program/buffer, and structured-kernel verifiers have
separate authority. Target feasibility cannot repair intrinsic invalidity;
costing observes only candidates that have passed the applicable gates.
`Intrinsic` and structured-kernel refinement failures therefore remain invalid
compiler output; only a checked target/resource rejection can contribute to a
valid empty physical frontier.

An index-region refinement refusal at stage 6 is the one failure both words could name and neither class covers. It is not invalid compiler output, because the compiler produced nothing wrong: a provider it resolved emitted a region that does not realize the occurrence. It is a missing compilation capability — the installed authority could not lower a program it was handed — and it is reported as one, at the refinement stage and against the exact occurrence. It is never a target rejection.

Search implementations may interleave cover and local-frontier exploration,
feed pruning information in either direction, and lazily schedule only regions
retained by viable covers. Such feedback is implementation freedom: it cannot
make a cover receipt prove schedule feasibility, or a local frontier prove
whole-program coverage.

## Lowering capability resolution and index-region refinement

Stages 5 and 6 answer different questions about one recognized occurrence, and the contract keeps them apart. Resolution selects *which* installed authority lowers the occurrence. Refinement proves that the work that authority emitted *realizes* it. Neither registration nor a successful builder construction is refinement evidence.

### Resolution is unconditional and fails closed

**Fact.** `crates/tiler-compiler/src/lowering.rs` resolves exactly one `LoweringFamily::IndexAccess` capability for every recognized occurrence, against the frozen lowering-capability registry the compilation request carries, and does so for every occurrence before the first cover is enumerated. There is no shape recognizer behind it, no default provider, no approximate provider, and no priority order between candidates.

Two dispositions are distinct, and neither is a preference:

- **absent** — no installed capability matches the occurrence's operation and signature. This is a *deferred* capability: the installed authority was never extended to this occurrence.
- **contended** — more than one matches. This is a *disproved* checked predicate: the authority was extended, and its extensions contradict each other. Reporting it as deferred would suggest a missing registration when the defect is a contradiction between two present ones.

Both stop the compilation with a typed missing-compilation-capability failure attributed to the exact occurrence. Neither narrows the portfolio, because an occurrence nobody can lower has no valid plan at all — retaining a smaller portfolio would return plans for a program the installed authority cannot compile. This is why a missing *lowering* capability behaves unlike a missing per-operation *fusion numerical* capability: the latter leaves every occurrence lowerable and only makes one fused grouping unprovable, so it rejects one alternative.

**Fact — lowering provenance is a registry resolution, not a compile-time table.** An artifact construction plan's `lowering_providers` is the set of `{provider identity, capability revision}` pairs resolution returned, deduplicated in canonical ascending order. `crates/tiler-compiler/src/program.rs` re-derives that set from the request's own installed registry when the plan is built and again when the portfolio is re-verified, and refuses a plan whose recorded provenance differs, so a receipt cannot name a provider the registry never resolved. Several occurrences of one family contribute one entry; one provider owning two capabilities at different revisions contributes two. ADR 0072 is why both halves are retained: a provider revision is the admitting authority's own output-affecting revision, and a capability revision covers the exact lowering that provider registered for one family and signature.

### Refinement is exhaustive finite evidence with an explicit gap

**Fact.** `legality::refine_index_region` drives the resolved provider through the canonical `tiler-ir` index builder and then proves, independently of the provider, that the emitted region realizes the occurrence: the ordered operand and result interface agrees in type, shape, arity, and aliasing; the reached scalar authority stays inside what the capability declared it may emit; the capability's and the region's semantic type authorities agree; and every ordinary write carries complete unique-ownership evidence. A refined occurrence's explain record carries exhaustive finite evidence, which is the strongest class this stage can produce and is weaker than a sound proof.

A malformed region, or a well-formed region that does not realize its occurrence, is a genuine rejection and fails closed. The artifact plan names the resolved provider as that occurrence's lowering authority, and that claim has to be true.

**An exhausted proof budget is neither an admission nor a rejection.** `tiler_ir::index` charges an exhaustive access proof against `MAX_EXHAUSTIVE_PROOF_CELLS`, and a region whose proof exceeds that budget has not been disproved — the enumeration stopped before it could decide. The stage records two things and leaves the plan standing:

- a typed budget stop at the refinement stage naming the exhausted proof resource, its governed limit, and the amount the proof would have required; and
- an explicit `Unknown` assessment of the refinement predicate, naming the predicate that stayed unproven and the reason it did.

Read that pair as exactly what it says and nothing more. The occurrence carries no refinement evidence, so no later stage may treat it as refined or cite it as one. Nothing about the emitted region was disproved, so the plan that contains it stays valid and is still costed, selected, verified, and returned on the same terms as any other. Rejecting here would report an exhausted analysis budget as hard infeasibility; admitting here would report an absent proof as a proof; and either reading loses the one fact the records exist to carry, which is that the question is open.

A budget stop found beside any other verification diagnostic is not a budget stop. The region was independently refused, and reporting the pair as an open question would hide a real refusal behind an exhausted analysis, so the refusal is what fails closed.

**Measurement.** Cells are charged only where the cheaper interval proof fails or a write is not a proved coordinate permutation. Every governed lowering's writes are coordinate permutations and its reads are bounded by its own dimensions, so the governed profile charges nothing at any recognized size — measured at `[70_000, 2]` in `pipeline::conformance::governed_lowerings_never_charge_the_exhaustive_proof_budget`. That is why refinement is attempted for every occurrence rather than gated on a size threshold, and it bounds the claim to the governed lowerings: a registered provider whose emitted access is neither interval-provable nor a proved permutation can and does trip the budget.

### Scalar-authority conformance is containment, not equality

**Fact.** A capability declares the scalar operations it may emit, and refinement requires the region's reached scalar authority to be *contained in* that declaration — the region must reach nothing beyond what was declared. The rule formerly required the two sets to be equal.

Equality is unsatisfiable for a shape-general provider. One capability lowers every occurrence of its operation family and signature, while which of the declared scalar operations a given occurrence actually needs depends on that occurrence's shapes and attributes. A `tiler.strict-serial-sum-f32` occurrence with a single contributor reaches no scalar operation at all, one over an empty reduced domain reaches only the identity constant, and one over many contributors reaches the add: three reached sets, one capability, one declaration that must cover all three. Requiring equality would have forced a provider registration per program shape, which is the opposite of what a declared emitted set is for.

The safety property is unchanged, because equality never carried it. Containment still refuses every region that reached an authority the capability was not admitted to emit, which is what makes the declaration a bound on what a lowering can compute. What equality added was a *completeness* requirement — that every declared operation actually be exercised — and that was the defect: it is a claim about one occurrence rather than about the capability, and no correctness argument rested on it.

### Maturity boundary

Resolution and refinement are implemented and unconditional on the ordinary compile path, and an index/access lowering provider written only against the public `capability` surface has driven a recognized occurrence end to end through the compiler's entry point, with the artifact plan recording that provider as the lowering authority. *Installing* such a registry from outside the crate is not reachable: the compilation request, its capability field, and the capability snapshot are all crate-private, and `tiler-compiler` exports no compile entry point. Implemented support for the emit-and-refine half and an unreachable installation path are different maturity claims and are recorded as different ones. The reviewed public compiler facade owned by [`prototype-public-compiler-api`](../../tickets/prototype-public-compiler-api.md) is what would close the second.

## Bounded hierarchical search

A Cascades-style memo is one possible implementation technique, not a committed
architecture. The durable concepts are contract-conforming semantic
alternatives, explicit region candidates, bounded implementation frontiers,
and deterministic complete-program selection. The term `memo` is reserved for
an implementation that actually groups equivalence classes and performs
goal-directed property search.

Examples of equivalent expressions include:

- consecutive reindexes versus one composed access map;
- a pointwise operation before or after a reindex when domains permit;
- alternative associations of a future multi-input einsum contraction, under a numerical policy that permits the distributivity the regrouping consumes.

Logical equivalence is policy-relative, so the third example is a group only where that policy holds; the first two hold unconditionally. No expressible policy holds for the third today, so it names a reserved equivalence group rather than an available one. See [logical exploration](#logical-exploration) for the permission each rewrite consumes.

Recomputation, materialization, fusion, and register residency are physical
implementations of one logical DAG. They do not create new logical equivalence
groups.

The first implementation should use bounded exploration: canonical operation
and value keys, deterministic rule order, small alternative sets, dominance
pruning, and explicit search budgets. Tiny graphs should have an exhaustive
oracle in tests so heuristic completeness and plan quality can be measured
before a memo architecture is chosen.

Five of the first deterministic safety budgets bound region formation, as the
`region_*` fields of `DeterministicBudgets`: 32 semantic occurrences per region
(`region_members`), 8 boundary outputs (`region_boundary_outputs`), 64 live
boundary/internal values (`region_live_values`), 32 candidates per seed
(`region_candidates_per_seed`), and 10,000 candidate expansions per compilation
request (`region_expansions`). Three more bound the stages downstream of it:
1,024 retained complete covers (`region_covers`), 100,000 partition-search
expansions (`region_cover_expansions`), and 4,096 complete-plan combinations per
cover source (`physical_plan_combinations`). A further budget, 8 nondominated
implementations per region, is forward-looking: it bounds the per-region
physical-implementation frontier's retention, which is not yet implemented, so it
becomes a real budget only when that stage lands. Producer duplication is
disabled outside oracle tests in the initial implementation. Hitting any of these
stops only that growth path, emits an explain reason, and never removes
singleton/unfused coverage. These defaults are calibration inputs, not
correctness constants.

Every budget above bounds a *search*, so exhausting one costs an alternative while complete coverage survives. `tiler_ir::index::MAX_EXHAUSTIVE_PROOF_CELLS` is not one of them and is not a request field: it bounds a *proof*, and exhausting it costs neither an alternative nor a plan but leaves one predicate about one occurrence open. [Refinement](#refinement-is-exhaustive-finite-evidence-with-an-explicit-gap) states what the compiler records in that case. Both reach the trace as typed budget stops, and a reader must not treat a lost proof as a lost alternative or the reverse.

## Rule classes

### Semantic normalization

Normalization chooses a canonical form and must terminate deterministically:

- resolve axis names and ellipses;
- canonicalize reductions and output-axis policy;
- compose permutations and legal split/merge chains;
- canonicalize explicit broadcast/repeat axis mappings;
- eliminate identity reindexes and no-op casts;
- normalize constants and dtypes;
- remove dead values.

Normalization must not silently change floating-point evaluation order.

### Logical exploration

These rules add alternatives:

- push a view through a pointwise expression;
- add contract-conforming alternatives over named pointwise operations;
- choose alternative associations of a tensor contraction only when the effective distributivity, reassociation, and operand-permutation permissions all authorize the regrouping;
- reassociate arithmetic or reductions only when numerical policy permits.

Each rule above names the effective numerical permission it consumes, as ADR 0011 requires of every semantic rewrite, and a rule that names none consumes none. Pushing a view through a pointwise expression relocates reads without changing which scalar operations compute a value, and initial floating-point operations are value-only under ADR 0020, so adding or removing an evaluation of one is not observable. This stage's guarantee that it adds only proved contract-preserving forms checks each rule's stated precondition; it does not supply a missing one.

"Contraction" in the third rule is the tensor sense — summation over indices shared by two or more operands — and its association is a numerical question before it is a search question. A reassociation permission is necessary and is never sufficient. Rewriting `(AB)C` to `A(BC)` forms entirely different rounded products rather than regrouping one reduction's contributors: the two programs' contributor sequences share no value and are indexed by different axes, so neither is a grouping of the other. [Numerical semantics](../numerical-semantics.md#distributivity-is-outside-the-order-contract) therefore classifies the rewrite as consuming distributivity — a third dimension, independent of reassociation and operand permutation, that no permission in that contract grants. [ADR 0080](../decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) is the accepted decision behind that classification and behind the rejection wording below. The rule fails closed under every contract Tiler can express, and does so as a settled position rather than pending one.

That rejection must name the missing distributivity dimension. Reporting a forbidden reassociation would be inaccurate and would imply that a contract permitting reassociation would admit the rewrite, which is exactly the inference the numerical contract forbids.

`StrictF32NumericalContract::governed_profile` in `crates/tiler-compiler/src/request.rs` returns the exact set of numerical contracts this build registers, and both members of it — `governed` and `governed_flush_to_zero` — set `reassociation` to `NumericalPermission::Forbidden`, so no registrable contract permits reassociation either. That is a property of the registered set rather than of the vocabulary: ADR 0076 item 1 widened `NumericalPermission` to `Forbidden` and `Permitted`, which changed which contracts are expressible and not which the compiler registers. `normalize_serial_sum` in the same file independently rejects any program without exactly one input, so no tensor contraction reaches the compiler at all. Both of those are incidental limits that will lift as the compiler grows. The distributivity gap is the durable reason: the rule would still fail closed on a compiler that accepted contractions under a contract that permitted both reassociation and permutation. The same contract's separate `contraction` field is ADR 0015's fused-multiply-add permission, which governs whether a tensor contraction's own `accumulator + a * b` step may round once; no one of these three permissions implies another.

### Region-candidate formation

Region rules propose, but do not automatically select, candidates with explicit
member operations, boundary values, retained results, materialized edges, and
duplication policy:

- pointwise plus pointwise;
- reindex plus pointwise;
- pointwise prologue into a reduction;
- pointwise epilogue after a reduction;
- compatible sibling consumers as a future multi-output kernel;
- supported prologue/epilogue around a semantic operation with an opaque
  library implementation;
- an explicit split/materialize alternative at eligible edges.

Each initial candidate is nonempty, connected, and convex in the operation DAG:
a path between included operations may not leave and re-enter the region.
Explicit duplication creates separately accounted occurrences; it never
silently waives convexity. Values consumed outside the region and graph results
are retained boundary outputs, so one fused region may correctly produce
several ordered values.

Producer duplication, region boundaries, and materialization belong to this
physical exploration phase rather than logical rewrite identity. A hypergraph
may index overlapping candidates internally, but membership alone is not a
complete region identity.

### Physical implementation

Implementation rules produce schedules such as:

- scalar or vectorized flat loops;
- rank-aware strided loops;
- direct or tiled rearrangement;
- serial, subgroup, threadgroup, or multi-pass reduction;
- direct or GEMM-backed contraction.

The bounded P0 frontier admits only checked `ScheduledKernel` proposals and
rejects opaque-call proposals explicitly. Its provider/body representation
must retain an additive sum-type seam so the later reviewed
[`implement-opaque-physical-call-providers`](../../tickets/implement-opaque-physical-call-providers.md)
ticket can add opaque implementations without weakening scheduled-kernel
verification.

Each implementation candidate advertises a machine-checkable numerical
guarantee, realization/provider identity, and scoped evidence. It is admitted
only when that guarantee refines every effective operation contract. A stronger
implementation may satisfy a weaker requested result set, but it does not
rewrite semantic identity.

### Enforcers

An enforcer supplies a missing required property at a cost:

- contiguous materialization;
- layout conversion;
- encoding repacking.

An enforcer may change only how a boundary value is stored, addressed, placed, or delivered, never which values that boundary carries. ADR 0001's separation of semantic planning from physical scheduling holds only because several physical schedules implement one semantic group identically, so a schedule-level step that altered a value would make one semantic program mean different things under different plans. Every entry above is value-preserving in that sense, and so is every property the [boundary-property list](#boundary-requirements-and-guarantees) admits.

A dtype cast is therefore not an enforcer, and resolved value dtype is absent from that list by construction rather than by omission. [Numerical semantics](../numerical-semantics.md#casts) makes casts semantic operations carrying resolved typed conversion contracts, and ADR 0010 forbids a later phase substituting a different conversion or letting fusion erase one that an unfused program happened to realize through a store and reload. A conversion the graph already contains is realized by ordinary lowering of that operation and supplies no missing property; a conversion the graph does not contain may not be introduced by a schedule at all. Admitting dtype to the property list would also break that list's ordering relation, because satisfaction there is subsumption and the dtype analogue of "16-byte alignment satisfies a 4-byte requirement" is a producer keeping `f32` where the boundary calls for `f16` — precisely the erased narrowing ADR 0009 and ADR 0010 forbid.

Choosing wider computation or accumulator precision inside a region is a different mechanism under a different gate: the implementation rules above already require each candidate's machine-checkable numerical guarantee to refine every effective operation contract. That is numerical conformance checked on an implementation, not a missing property supplied at a boundary.

Scalar alignment-safe execution and bounds masking are schedule alternatives or
proof obligations, not enforcers. A partial buffer plus second pass is a
multi-kernel reduction implementation.

### Cleanup

After program selection, local passes perform index-expression CSE,
loop-invariant motion, strength reduction, constant folding, bounds-check
elimination, and dead-code elimination. Schedule-affecting normalization
finishes before `ScheduledRegion` identity is formed. Later structured-kernel
cleanup is independently canonicalized and committed through codegen/artifact
identity; it must not silently mutate the already-hashed schedule.

## Boundary requirements and guarantees

A downstream region implementation requests boundary properties and each
producer implementation advertises what it guarantees. Initial boundary
contracts include:

- storage layout class and contiguous axes;
- storage encoding;
- alignment and vectorizable width;
- materialized buffer, alias/view, or opaque runtime value;
- device and address space;
- **availability** — the dependency after which a produced value may be consumed;
- **visibility** — whether a consumer's reads see the produced value without a further coherence action.

The last two complete the list rather than extending it. `AGENTS.md` names ordering and synchronization as explicit physical contracts rather than implicit node annotations, and a boundary contract that could not express them would leave a plan's ordering and coherence obligations unstated at exactly the boundary they are owed across.

**What each dimension currently establishes, distinguished by maturity.** A dimension appearing here says the optimizer models that property, not that every value in its vocabulary is served. Reading the two new dimensions honestly:

| value | maturity |
| --- | --- |
| availability *after producing dispatch* | **implemented and satisfiable** — a producer guaranteeing availability after its own dispatch discharges it |
| availability *after observed host completion* | **type-system reservation** — ADR 0033 makes host observation a separate boundary (terminal completion, a post-completion status check, error-record visibility, then interpretation) and no guarantee in this vocabulary discharges it, so a requirement naming it rejects explicitly |
| visibility *readable on the requiring affinity* | **implemented and satisfiable** — discharged by a producer coherent on its own affinity |
| visibility *requires an explicit coherence action* | **reserved, and deliberately not satisfiable** — ADR 0047 makes an affinity-to-domain edge declare its own coherence requirements, so a domain owing a flush or invalidate is guaranteed by its producer and unreadable by a consumer until an enforcer supplies the action; treating it as satisfied at a higher cost is the substitution ADR 0043 forbids |

A reserved value is not a weaker form of support. It rejects, and the rejection is the guarantee: the alternative is a plan that silently reads a value no one made visible.

**Admission rule for a new dimension.** The list is extensible, and a dimension is admitted only when all of these are stated: its requirement space, its guarantee space, the satisfaction or subsumption rule between them, how a child boundary derives it, its dominance behaviour, its identity encoding, its maturity by the classes above, and the boundary at which a value-preserving enforcer may discharge it rather than the plan being refused. A dimension without a satisfaction rule is a label, and one without an identity encoding is invisible to every consumer that compares two plans.

**Storage encoding is a distinct property, not part of layout class.** Layout
class answers which logical coordinate maps to which position; encoding answers
how one element is represented at that position. They vary independently: a
blocked layout of bit-packed `u4` and a blocked layout of unpacked `u4` share a
layout class and differ in encoding, so no layout class can express the
difference. Encoding meets the admission test that keeps dtype off this list —
a producer can realize one semantic value either way and the choice is
unobservable in the value — where a narrowing dtype change is observable in it.

Its enforcer is repacking. ADR 0047 already names "materialization/repacking" as
an enforcer family, and the
[transfer taxonomy](../research/transfers/transfer-synchronization-and-resource-lifetime.md)
already separates `MaterializeLayout` ("same logical value and dtype;
addressing/layout may change") from `RepackEncoding` ("explicitly changes
storage encoding"), keeping both distinct from `ConvertDtype`. Its
`TransferStage` also carries an explicit `PreserveStorageEncoding` semantics
field, which a transfer would have no reason to declare unless encoding were a
dimension it could otherwise change. So the enforcer was accepted before the
property it supplies was named here, and this entry closes that gap rather than
adding a new mechanism.

Encoding owes the same treatment every other property owes: canonical keys,
satisfaction and subsumption, child-requirement derivation, and dominance.
Subsumption is not automatic in either direction — an unpacked producer does not
satisfy a packed requirement merely by being cheaper to read, and a packed one
does not satisfy an unpacked requirement merely by being denser — so an encoding
relation is stated per encoding family rather than assumed to be an ordering.

**A quantized value's companion parameters are not a separate property.** Its
component roles are semantic: the [IR contract](../ir.md) makes a quantized
tensor "one first-class semantic tensor value even when its runtime
representation has several components", with the versioned scheme, component
roles, and coordinate maps named in its static type contract, and with scale and
zero-point tensors entering as ordered operands to an explicit assembly or
conversion operation. A schedule may not add, drop, or re-role a component,
because that would change which values the boundary carries. What remains
physical is that "physical packing and addressing remain storage decisions" and
that "artifact lowering may expand one logical quantized argument or result into
several verified physical bindings". Those are encoding and layout applied to
each component, so what this list owes a multi-component value is that its
properties are stated per component — not a further property naming the
companions themselves.

Logical shape, resolved value dtype, accumulation semantics, and numerical
policy are semantic traits or optimization-context constraints, not properties
supplied by a schedule.
Target capabilities, runtime guards, resource use, schedule invariants, and
cost estimates are also distinct concepts rather than entries in one universal
property bag. Iteration order and register residency are region-internal unless
they affect a boundary value.

For example, a vectorized reduction may require a unit-stride reduction axis,
16-byte alignment, and an extent divisible by four. The optimizer compares a
contiguous-materialization enforcer followed by that reduction against a
generic strided reduction.

The boundary-contract system defines canonical keys, satisfaction and
subsumption (for
example, 16-byte alignment satisfies a 4-byte requirement), child requirement
derivation, and dominance. Enforcer insertion is cycle-checked. Interesting
boundary properties such as useful unit-stride axes are retained on a bounded
Pareto frontier even when they are not locally cheapest.

One implementation dominates another only within the same semantic and
constraint region when its applicability covers the other's, its target and
boundary requirements are no stronger, its guarantees are at least as strong,
its hard resources are no worse where relevant, and its symbolic cost is no
worse throughout the compared constraint cell and strictly better somewhere.
Otherwise both remain or the constraint space is partitioned. Cost alone may
not prune the only implementation valid for a runtime region.

Target-requirement implication and evaluation phase participate in dominance.
A candidate needing a stronger or later runtime predicate does not dominate a
generic candidate merely because its estimated cost is lower. Scalar/generic
coverage is retained whenever specialized feasibility is deferred or narrower.

Numerical conformance is checked before this dominance relation. Accuracy is a
hard semantic dimension, not a Pareto cost; incomparable or unknown evidence
cannot be made legal by a lower estimated runtime.

## Possible memo contract

If a bounded memo is adopted, its conceptual key is:

```text
semantic group key = canonical semantic expression
optimization key = (group, boundary requirements, target profile,
                    numerical policy, constraint region)
candidate = region implementation + child boundary requirements
            + boundary guarantees
```

It would store a bounded Pareto set, track shared DAG cost without charging a
materialized producer once per parent, detect cycles, and retain structured
rule/candidate provenance. Search-budget exhaustion returns the best complete
plan found under deterministic fallback heuristics.

Region enumeration is already general rather than a trivial builder for a narrow
semantic graph: `EnumerateRegionCandidates` proposes every connected convex
region of an arbitrary verified DAG up to the declared budgets, with separate
content and occurrence identities and typed budget-stops, and is checked against
an exhaustive subset oracle. Goal-directed property search over those candidates
is the staged future work:
[cover enumeration](../../tickets/prototype-region-cover-enumeration.md),
[physical-implementation frontiers](../../tickets/prototype-physical-implementation-frontier.md),
and [complete physical-plan selection](../../tickets/prototype-complete-physical-plan-selection.md)
are separate later stages, not a second optimizer architecture.

## Symbolic parameters and routing

The optimizer consumes a constraint environment describing exact/ranged
extents, divisibility, equalities, and optionally common profiled values. Costs
may be symbolic or piecewise over this environment. The selected result can be
a portfolio of AOT variants plus a deterministic routing decision tree or
crossover formula. Guards establish validity; routing chooses profitability
when several variants are valid. Routing policy participates in `EXPLAIN` and
artifact identity.

## Rule interface

Semantic rules conceptually provide:

```text
match(expression) -> bindings
check(bindings, semantic_context) -> proof or rejection
apply(bindings) -> equivalent expression(s)
```

Implementation rules conceptually provide:

```text
implement(group, boundary_requirements) -> candidate {
    implementation,
    child_requirements,
    boundary_guarantees,
    legality_constraints,
    estimated_resources
}
```

Every rule needs a stable name, declared numerical preconditions, positive and
negative tests, deterministic search behavior, and explain-trace output.

## Explainability

An `EXPLAIN` report should show:

```text
logical input
normalization rules fired
equivalent alternatives retained
resolved lowering capability per occurrence
index-region refinement evidence or its recorded gap
fusion regions considered
boundary requirements/guarantees
enforcers inserted
schedules considered and rejected
per-operation reference and effective accuracy envelope
candidate numerical guarantee, realization, and evidence class
selected cost and assumptions
runtime guards and fallback
```

Structured rejection reasons are important: “threadgroup reduction rejected:
shared memory exceeds target limit” is actionable; a later MSL compiler error
is not. Numerical reasons are equally concrete, such as “claimed 3 ULP exceeds
required 1 ULP,” “domain uncovered,” or “toolchain evidence unknown,” and are
reported separately from cost rejection.

Every rejection records its stage, stable reason code, rule/provider identity,
affected operation/value or candidate, failed predicate/evidence, and whether
the result is a hard rejection, safe deferral, budget stop, dominance pruning,
or cost disadvantage. Explain output never collapses these into “not fused.”

A budget stop is the one disposition that says nothing about its subject, so it never stands alone. Whatever predicate the stopped analysis was deciding is recorded beside it with an `Unknown` evidence class and the reason its proof stopped. Emitting the stop without that assessment would leave a reader to infer either a pass or a rejection from a record that supports neither, and inferring a pass is the more dangerous of the two.

### Explain authority

Under ADR 0073 the typed explain vocabulary — records, subjects, stages,
dispositions, reason and rule keys, evidence classes, and retention bounds — is a
module of `tiler-compiler`, not a separate `tiler-explain` crate. The compiler
owns record construction, canonical identity, causal integrity, and the versioned
renderer. Emission is compiler-owned: sibling compiler modules obtain record
handles from a writer, and no provider-facing emission trait is published. Module
visibility is a public-facade question rather than a packaging one; the module is
private while the compiler boundary is private.

If a second crate must ever read canonical traces, the record, subject, and
disposition vocabulary moves into `tiler-ir` following the `AbiExpr` co-location
precedent of ADRs 0068 and 0070, with emission staying compiler-owned. A new
crate is not the expansion path. Until that trigger fires, a component that
cannot depend on `tiler-compiler` has no explain contract; it is an explicit
unsupported case rather than a licence to copy the vocabulary.

Canonical trace content is data and the renderer is presentation. Nothing in this
contract requires an explain trace to be serialized into an artifact envelope,
and the artifact contract does not carry one.

### What the public compiler boundary exposes of a trace

*Added 2026-07-25 by `prototype-public-compiler-api`, which settled the seven public-surface questions the typed-explain work deferred. Each statement below is derived from a contract or a measured property rather than chosen, and each names what would reopen it.*

**A trace is complete or absent, never partial, and a failed compilation returns the one it has.** A detail record that would exceed the retained-trace ceiling fails the compilation closed with a typed capacity error rather than being dropped, so a sealed trace is complete by construction and no truncated form exists to describe. A refusal that happens *before* a verified per-target request exists — request verification, semantic output typing, numerical-contract resolution, normalization, target selection — has no trace to seal, and reports that absence as a distinct state. Discarding a sealed trace on the failure path is not an option this contract leaves open: a rejection reported with no stage, reason code, rule, or predicate is the collapse the paragraph above forbids.

**Rendering is deterministic and total; its spelling is not a contract.** One trace renders to one string, and every retained record appears — the renderer has no filter and no bound. The rendered text is a diagnostic for a human reader and is not a parse target, and its leading `tiler-explain-v<N>` names the renderer version so a change to the rendering is visible. Committing to the text would create a second description of a trace that has to be kept in agreement with its canonical bytes, which is the duplicate-derivation hazard the data/presentation split exists to prevent.

**The renderer header's request qualifier is a correlation label, not an identity.** It is a short non-cryptographic fold of the canonical request subject, so two distinct requests may share one. ADR 0074 convention 2 governs it as a presentation label: it is never an equality, dedup, or cache-key input. Redacting it protects nothing — it is derived from the caller's own request — and removing it would leave two rendered traces in one log indistinguishable.

**Nothing in a trace is redacted.** Every provider key and revision a trace attributes is either minted by Tiler or installed by the caller's own request, because the writer refuses a rule attributed to any other provider. There is no third party's detail present to withhold, and withholding one would make a rejection unexplainable, which this contract forbids. Reconsider when a registry the caller does not control can install rules.

**There is no retention control to expose.** The configurable detail budget is gone; exceeding the ceiling is a typed compile failure. Re-introducing a control would re-introduce a trace that is silently incomplete.

**Only the compiler mints an evidence receipt, and only from a proof it derived.** A receipt carries the `SoundProof` evidence class, and this repository keeps `SoundProof`, exhaustive finite evidence, empirical evidence, normative guarantees, and `Unknown` as distinct classes. A receipt supplied by an external provider is a *claim*; recording it as `SoundProof` would convert an assertion into a proof at the boundary, and a fusion legality proof is what admits a rewrite. A provider's contribution is its identity and revision, which the compiler attributes and bounds against the request's installed registry — that is provenance, not evidence. This does not change if a provider can one day ship a machine-checkable proof: the compiler would still mint the receipt, from its own re-check.

**Every identity the boundary emits is canonical bytes, never a digest, and never both.** ADR 0074 convention 2 states the rule; a digest here would be a second identity over the same subject, requiring a stated hash and a collision argument, and the production digest implementation is not yet chosen. Two published values a consumer can disagree about is strictly worse than one.

**Public enums follow ADR 0074 convention 5's clause test, and never a parallel versioned schema view.** Such a view is a second, hand-maintained description of an enum that nothing keeps in agreement, which is convention 3's argument against encoding a projection instead of its source; and it buys compatibility, which ADR 0075 records as a rejected premise while no crate is publishable.
