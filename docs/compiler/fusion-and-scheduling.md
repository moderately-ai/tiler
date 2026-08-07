---
schema: "tiler-doc/v1"
id: "tiler.contract.fusion-and-scheduling"
kind: "contract"
title: "Fusion and scheduling"
topics: ["fusion", "scheduling", "optimizer"]
contract_status: "accepted"
implementation_status: "partial"
evidence: ["tiler.research.region-search.exhaustive-region-oracle", "tiler.research.scheduling.scheduled-region-model", "tiler.research.target-profiles.physical-feasibility-model", "tiler.research.program-planning.kernel-program-buffer-plan"]
---

# Fusion and scheduling

**Status:** accepted research contract; bounded prototype implementation

Region formation is now the deterministic `EnumerateRegionCandidates` stage
over an arbitrary verified semantic DAG, not a graph-specific serial-`Sum`
recognizer. It proposes every connected convex region up to the declared
budgets: both extremes of the partition lattice are emitted unconditionally
before growth starts — every operation's singleton and the whole-program
region — the regions between them are seeded from their minimum member so each
connected set is generated exactly once, and convexity is decided when a set is
emitted by re-reaching the region through its forward closure. Each candidate
carries member operations, boundary
inputs, retained outputs, an allowed-duplication policy, and separate
region-content and region-occurrence identities: content canonicalizes the
region's internal computation with members renumbered to region-local positions,
while occurrence additionally pins the exact graph site in canonical
coordinates. Five deterministic budgets bound this stage and they are **two
kinds, not one**. `region_candidates_per_seed` and `region_expansions` bound the
*search* between the two extremes, so exhausting either costs an alternative and
never a plan. `region_members`, `region_boundary_outputs`, and
`region_live_values` bound one region's admissible *shape*: they say how large a
region may be, so a program whose only implementable cover needs a bigger one is
refused by them, and that refusal is `BudgetExhausted` naming the bound rather
than a target verdict. Every budget that fires is retained as a typed explain
budget-stop, so a legal alternative lost to a bound is reported rather than
silently dropped. Enumeration is validated against an independent exhaustive
subset oracle that agrees set-for-set without budget pressure.

Those five are region formation's own bounds, not the compilation's. Other stages carry their own, and two of them differ in kind rather than degree. A *search* budget stops one growth path while both coverage extremes survive, so what it costs is an alternative. A region *shape* budget, as above, refuses a region rather than a search path, and can leave a program with no cover at all. And the [proof budget](optimizer.md#refinement-requires-discharged-index-domain-evidence) that bounds an index region's exhaustive access verification costs a *proof*: nothing disproves the subject's predicate, so the region stays valid analysis state with the predicate open, and the occurrence is refused rather than allowed into an executable frontier until the proof is discharged. All three are typed budget stops in the trace and they must not be read as the same finding.

Enumeration only proposes candidates. It selects no cover, chooses no
implementation, lowers no index region, plans nothing physical, and costs
nothing; those are separate authorities —
[complete-cover enumeration](../../tickets/prototype-region-cover-enumeration.md),
[physical-implementation frontier](../../tickets/prototype-physical-implementation-frontier.md),
and [complete physical-plan selection](../../tickets/prototype-complete-physical-plan-selection.md).
Producer duplication stays disabled in the first profile while the
exhaustive tiny-DAG oracle retains it as a completeness witness. This stage is
a candidate enumerator, not a cover selector or a public fusion API.

All three of those tickets are `done`, and the general DAG partition search over them landed 2026-08-04 under [`implement-general-dag-partitioning`](../../tickets/implement-general-dag-partitioning.md) — fan-out, ordered multi-result outputs, per-edge materialization, and budgeted memoized enumeration checked against exhaustive small-graph oracles.

**Fact — producer duplication is a stated legality contract now, and it is still off on the compile path for a reason that is not about legality.** It is no longer only an oracle-held completeness witness: the cover authority carries two admissions as distinct legality contracts — an exact partition, and a pure-recomputation admission whose per-member condition is purity, no named result, and a contract granting no realization freedom, each with its own typed refusal — and one enumeration and every verification of its covers run under the same stated policy so that a verification cannot silently apply the weaker rule. The compile path states the exact-partition admission, and the derivation for that is downstream: a duplicating cover assigns one occurrence to several region subjects, each of which needs a physical implementation, and [the physical layer proposes for only the partitions the request boundary pre-computed](optimizer.md#what-each-stage-is-general-over-today). Every duplicating cover of a governed program would therefore be enumerated, found unimplementable, and rejected. [`activate-shared-work-duplication-on-the-compile-path`](../../tickets/activate-shared-work-duplication-on-the-compile-path.md) is the `deferred` owner, and its activation trigger is a physical provider and a program assembly that can realize one — not a change to this contract.

Index-region lowering is not one of the later stages in that list. It runs *between* candidate enumeration and cover enumeration, per recognized occurrence rather than per candidate region: the compile path resolves each occurrence's index/access lowering capability and refines the region that capability's provider emits before the first cover is enumerated, because a cover grouping occurrences the installed authority cannot lower would be a claim about plans nothing could realize. Cover enumeration is therefore an independent legality authority that nonetheless runs downstream of a successful resolution. [The optimizer contract](optimizer.md#lowering-capability-resolution-and-index-region-refinement) owns that stage.

Every prototype candidate recomputes its stable identity from its exact member
occurrences and boundaries. Numerical evidence is bound to that candidate, the
canonical request subject, full numerical contract, and exact materialized
reference provider. A candidate kind or copied stable string is not evidence.
Schedules retain the canonical request subject—including full target facts and
provider provenance—and revalidate that subject before KIR refinement.

## Ownership boundary

This document owns fusion-region formation and schedule candidate generation,
legality queries, and split-plan retention. The IR contract owns the normalized
schedule fields and verifier; target backends own realization of accepted
requirements on concrete devices.

## Fusion is a plan choice

Fusion removes intermediate storage and dispatches by evaluating producer work
inside a consumer kernel. It can also increase live ranges, indexing work,
source size, synchronization, and register pressure. Therefore:

```text
can fuse != should fuse
```

The optimizer must retain a split implementation wherever a fused
implementation is considered.

Logical operation boundaries do not imply materialization. Named `Multiply`,
`Add`, `Gelu`, `Broadcast`, and `Reduce` nodes remain visible in the semantic
graph until region exploration chooses which operations to compose into an
iteration/scalar expression.

## Region representation

A candidate is more than a set of operation IDs:

```text
RegionCandidate {
    member_operations,
    boundary_inputs,
    retained_outputs,
    allowed_duplication,
    region_content_identity,
    region_occurrence_identity,
}
```

This is a conceptual sketch, not the literal Rust definition. In
`crates/tiler-compiler/src/region.rs` the two identity fields are the
region-content identity, which folds the numerical-contract key into its
canonical bytes, and the region-occurrence identity, which additionally pins the
exact graph site; there is no standalone numerical-contract-id field.

The initial region is also connected and convex. If `a -> b -> d` and
`a -> c -> d`, the set `{a, b, d}` is illegal because another path between its
members leaves through `c` and re-enters at `d`. Contracting such a set would
hide required interleaving. Duplication creates distinct, explicitly costed
occurrences; it is not an exception to convexity.

Overlapping candidates may be indexed as hyperedges during search. That
hypergraph is an optimizer data structure, not the semantic graph or selected
physical program. Two candidates with the same member operations can differ in
retained outputs or allowed producer duplication and therefore have different
feasible implementations. Boundary contracts and actual materializations
belong to region implementations and complete kernel programs.

For each candidate and target profile, iteration/access lowering plus local
scheduling returns a bounded `ImplementationFrontier`. The mature model has an
additive sum-typed body (`ScheduledKernel`, `KernelSubprogram`, `OpaqueCall`, or
`View`), boundary requirements/guarantees, applicability predicates, target
requirements, exact/proven resource requirements, resource estimates, and a
cost estimate. The bounded frontier admits checked `ScheduledKernel` and
`KernelSubprogram` proposals and explicitly rejects `View` while preserving that
additive seam; `OpaqueCall` is admitted through its own registration and
binding path. A `KernelSubprogram` is what makes one region subject realizable
by several dispatches — a `ScheduledKernel` is one region and therefore one
dispatch — and its stages are an ordered chain whose internal handoffs never
reach a cover edge, so a subprogram's boundary contract is indistinguishable
from a single kernel's at the join. A frontier additionally records the
strategies a provider *considered and withheld*, with a typed reason, because an
enumeration that cannot say why an alternative is absent cannot be audited for
completeness. Program selection chooses a compatible covering set only after
these frontiers are available.

Complete-cover enumeration is an independent legality authority over region
candidates. It neither waits for nor proves a local schedule. Conversely,
checked schedules and target-feasible local frontiers are per-region
authorities and do not depend on one globally chosen cover. A search may lazily
explore frontiers only for regions retained by viable covers, or pass bounds
between the two searches, but complete physical-plan selection is the first
authority allowed to join a complete cover with compatible implementations.
It emits a checked selected-plan or portfolio receipt; structured KIR
refinement follows that selection.

For a shared producer `p` with consumers `left` and `right`, legal alternatives
include one materialized `p`, a multi-output region `{p,left,right}`, or—only
with explicit duplication capability—two occurrences in `{p,left}` and
`{p,right}`. The first implementation keeps duplication disabled while the
exhaustive tiny-DAG oracle retains it as a completeness witness.

All three are now enumerated by the landed partition search under the admission that permits them, and the compile path states the exact-partition admission for the derivation above. One correction that generalizes came out of that oracle: an anchored search that admits a candidate only when it covers the branch's minimum uncovered operation can never choose a region all of whose operations are already covered, and `{p}` beside `{p,left}` is exactly such a region — so the anchored base is enumerated together with every augmentation by one, which is complete because running the anchor rule over any legal cover selects a base and leaves exactly that remainder.

## Legality

A proposed fusion region is legal only when all of the following hold.

### Iteration and indexing

- Every output coordinate maps to valid input coordinates.
- Reindex composition is representable and in bounds.
- Broadcast reads may alias but output writes do not overlap.
- Rank-changing view operations remain metadata unless a physical reorder is
  deliberately chosen.
- Zero-sized domains cause no memory access or illegal dispatch.
- Reduction axes are unique, in range, and canonically ordered before any
  contributor calculation; a zero extent is detected before multiplication so
  a late zero cannot be hidden by an earlier overflow.

### Dependencies and effects

- The region is acyclic.
- Internal values have a defined ownership and lifetime.
- Reduction dependencies are properly nested.
- No cross-threadgroup dependency exists without an explicitly supported
  atomic or multi-pass protocol.
- Barriers are reached uniformly by all participating threads.
- Alias and in-place behavior are explicit; the initial design is out-of-place.

### Target capabilities

- Required dtypes, operations, execution scopes, memory spaces, barriers, and
  collectives are supported by typed capability facts.
- Index arithmetic cannot overflow under guards.
- Selected execution-scope/group dimensions, local memory, bindings, and
  generated resources fit target limits.
- Vector access satisfies alignment and tail requirements.
- Any unresolved hard fact is deferred to a named safe preflight phase with an
  equivalent packaged alternative; estimates never establish legality.

### Numerical semantics

- Reduction identity and accumulator type are defined.
- Operation and reduction order satisfy the selected policy.
- A concrete reduction topology is a physical-plan decision proven to satisfy
  the semantic reduction's allowed evaluation-order or result class.
- Reduction scheduling proves reassociation and operand-permutation legality
  independently; a permission or algebraic capability for one is not evidence
  for the other.
- NaN, signed-zero, empty-domain, cast, and overflow semantics are preserved.
- Every fused scalar realization and opaque intrinsic refines each
  transcendental operation's effective reference, domain, accuracy, special-
  value, and subnormal contract.

Legality failure produces a structured split or fallback reason. An accuracy
failure is hard infeasibility, never a cost penalty.

## Profitability

Benefits of fusion include:

- avoided launches and intermediate allocation;
- eliminated global-memory writes and reads;
- register-local producer/consumer reuse;
- joint index simplification and layout planning.

Costs include:

- duplicated work at fan-out;
- larger live ranges and reduced occupancy;
- additional local memory and barriers;
- loss of parallelism around reductions;
- worse memory coalescing;
- index div/mod overhead;
- divergence and masked lanes;
- loss of tuned library kernels;
- integration-supplied compilation, artifact, and delivery-size costs; for the
  proposed Rust/Metal path these include cold macro expansion and embedded
  metallib/binary growth.

Fan-out greater than one is evidence for materialization, not a categorical
boundary. Cheap reindex or scalar work may be worth duplicating; a large
reduction usually is not.

## Pointwise and reindex schedules

Candidate schedules include:

- one thread per logical output;
- grid-stride loops;
- collapsed contiguous iteration;
- rank-aware dynamic-stride indexing;
- fixed vector widths such as 1, 2, 4, or 8 as backend/profile-specific search
  candidates, plus scalable vector shapes where the target model admits them;
- alternate axis orders for coalescing;
- masked vector or scalar tails.

Vector legality is queried for the complete operation, dtype, fixed/scalable
shape, mask/tail, address space, access width, and alignment contract. A
preferred width is a cost fact, not proof that the operation is legal.

A logical transpose need not be materialized. It becomes a different access
map in the fused consumer. Materialization remains a candidate when it improves
several downstream consumers enough to repay its write/read cost.

## Schedule representation

A `ScheduledRegion` is one canonical `IndexRegion` plus one normalized
`KernelSchedule`. The schedule owns execution axes, work assignment, output
ownership, loop/vector/tail organization, staging lifetimes, reduction
topology/result visibility, synchronization phases, launch formulas, and
specialization bindings. The index region continues to own scalar meaning and
logical access maps. Derived hard resource requirements are checked from the
schedule; target facts and cost estimates remain separate inputs.

A selected schedule explicitly represents loop/tile hierarchy, coordinate
mapping to grid/threadgroup/SIMD/lane/vector axes, vectorization, memory
placement, staging, synchronization, reduction topology, tails, and launch
formulas. It is canonical physical IR rather than a bag of node annotations.

Scheduling transformations and their rejection reasons are recorded in a
separate trace for explanation and replay. The normalized result is the
identity-bearing executable intent and is independently verified; successful
application of a transform sequence is not a legality proof.

## Reduction implementations and schedules

Reduction remains semantic until region implementation and scheduling select an
explicit strategy:

1. **Serial per output:** one thread loops over the reduction domain.
2. **Multiple outputs per thread:** amortizes indexing for small reductions.
3. **SIMD-group reduction:** lanes cooperatively reduce one or more outputs.
4. **Threadgroup reduction:** several SIMD groups combine through local memory.
5. **Multi-pass reduction:** a `KernelSubprogram` materializes partial outputs
   for a later scheduled kernel.

Each implementation declares valid extents, lane-result visibility, tail masking,
barrier requirements, accumulator type, and target capabilities. There is no
underspecified portable “block reduce” operation in final scheduled IR.

The threadgroup form's dataflow *and* its ordering are implemented and
verifier-owned. `ReductionTopology::CooperativeWorkgroup` states the participant
set, local coordinates, workgroup staging with declared lifetimes, phased writes
and reads, uniform phase reachability, and the single committing participant; the
ordering between a staged write and a later staged read is derived as an explicit
visibility edge, and the tile declares the synchronization points that discharge
those edges. The intrinsic schedule verifier proves all of it, including that
exactly one point discharges each edge — no edge left unordered, and no second
point stating the same ordering twice.

A point is a schedule-owned authority, not a barrier's field. It names its
operation kind, the phase boundary it occupies, the participants that must
arrive, and the complete realization it requires — arrival scope, publication
scope, fenced memory domains, ordering — together with the evidence class its
convergence rests on. Only the control barrier is admitted; asynchronous copies,
split-phase barriers, collectives, atomics, and inter-dispatch dependencies are
statable and refused by name, because each carries a contract this vocabulary
does not define and a target fact for one must never satisfy a requirement for
another. Convergence is refused outright when a producer merely asserts it: the
admitted evidence class is the derivation over the tile's own per-phase
participation, which makes "every participant reaches this point" a checked claim
rather than a caller's word.

A point lives inside the tile rather than beside the topology, and the
consequence is the elimination this authority exists for: a barrier in the
pointwise, global-linear program cannot be *stated* at the schedule layer at all,
because that schedule has no phases to place one between. The structured-kernel
verifier still refuses one there by name, which checks the redundant barrier at
the layer where a barrier can actually be written.

Feasibility then composes one atomic realization subject against a target's
declaration, by equality over the whole value. A subject nothing declares is
`Unknown`; a subject a profile declares unrealizable is a typed rejection; and a
profile whose facts each realize one dimension of the required subject — subgroup
arrival, device-wide publication, a wider fence, a stronger ordering — satisfies
nothing, because their conjunction is a statement about none of them. A schedule
with no synchronization derives no requirement at all, so it consults no fact,
produces no explain row, and stays feasible against a target that declares
nothing about synchronization.

### The single-workgroup tree

The threadgroup form's *strategy* is the depth-two tree, and every key it needs is stated rather than left to be read off an emitted body. One workgroup takes each output position and `participants` invocations occupy it. At level 0 every participant serially folds the contiguous contributor range its partition owns, in the region's declared contributor order, and stages the partial in its own slot of workgroup memory. The synchronization point separates the levels. At level 1 the one committing participant folds the `participants` staged slots in ascending participant order and performs the region's owning write. Active lanes are therefore every participant and then one; the storage is one `f32` slot per participant; the accumulation width is the resolved contract's arithmetic type, carried on the topology rather than inherited from the element type; and the contributor order is original-axis lexicographic within a partition and ascending participant across them.

The depth is two and not `log2(participants)`, and the reason is a property of the dataflow vocabulary rather than a shortcut. A staged span is addressed by *every* participant of the tile, so a write phase writes one slot per participant whatever round it belongs to and the writing lanes never narrow; rewriting one slot across rounds is separately refused by the one-writer-per-slot rule. A logarithmic tree therefore needs a per-access active-participant subset — distinct from a phase's participation, which is *arrival* and must stay uniform for the point between rounds to be convergent — and that subset is absent rather than reserved.

Tail handling is exact or nothing. The split must cover the contributor sequence exactly once each, so an extent admitting no balanced split is declined with its contributor count rather than padded with identity elements or given a masked lane. A masked lane would also break the emitted body's soundness argument, which rests on every launched invocation reaching the staged store.

The strategy consumes **reassociation and nothing else**, and the topology records why in a field rather than leaving it to be inferred. `ContributorArrival` names the order in which the staged partials reach the combining participant: the admitted ascending-participant arrival is fixed by the program, so the tree regroups the declared sequence without moving any contributor across a group boundary. An arrival the program does not fix — a nondeterministic one, or an atomic accumulation into a shared location — reorders the contributors themselves and requires contributor permutation *in addition*; both are statable and both are refused, first for the permission when the contract withholds it and then for the construct, because only the control barrier is an admitted synchronization kind.

Four failures reject before executable-frontier admission, each with its own reason and none as a cost. A contract forbidding reassociation withholds the strategy from the contract alone, before any region exists. Insufficient workgroup memory is a capability rejection naming the axis and both quantities. A profile that declares the realization unrealizable is a typed rejection carrying the whole subject and the refusing authority. A profile that declares nothing about it is a *separate* rejection carrying the subject and no authority — silence and refusal are different answers, and reporting one as the other would either invent a refusing profile or hide that a target was never measured. A tile whose phases are not uniformly reached never reaches a target at all: the schedule verifier refuses the divergence first.

The serial reduction, the multi-pass split, and the single-workgroup tree are retained together for one subject with distinct identities and one shared boundary contract. Enumerating the tree does not make it *win*: under the structural cost model it shares the serial alternative's dispatch count, launches strictly more threads, and materializes nothing, so it can never win by pruning. What it trades those threads for is a shorter critical path per output, which that model does not measure, and preference belongs to measured calibration.

The bounded prototype target profile declares zero threadgroup memory and declares nothing about synchronization, so it rejects the tree on both counts. That is the profile being truthful rather than a gap in the strategy: a prototype authority with no evidence for a threadgroup-memory budget or a barrier realization must not declare one, and what the baseline should guarantee is a separate decision from what the strategy is.

A multi-pass reduction is a `KernelSubprogram`: an initial scheduled kernel
fully defines a typed partials temporary in declared scratch, a typed `Data`
dependency on that materialized value makes those bits visible to its reader,
and a later scheduled kernel produces the result. `StorageHandoff` is the
separate allocation-reuse ordering edge that would place the partials
temporary's final users before a new writer of the same allocation; it is not
the producer-to-consumer visibility mechanism. Scratch preserves accumulator
bits unless the semantic contract explicitly admits a conversion. Canonical
stream/list order alone proves neither dependency.

The implemented split is the bounded profile's balanced two-pass form: a partial
pass staging one value per partition and a final pass combining them, with the
partial pass claiming the reduction occurrence and the final pass claiming none,
because that occurrence is already covered and claiming it twice would
double-cover the graph. It consumes **reassociation and nothing else** — the
contributor order within and across partitions is preserved, so permutation
neither grants nor substitutes for the permission — and it is withheld with a
typed reason when the resolved contract forbids reassociation or the contributor
extent admits no exact partition into at least two parts of at least two
contributors. A ragged final partition is not implemented: it needs a second
constant trip count the structured-kernel loop vocabulary does not carry, so a
prime or sub-four extent retains only the serial alternative rather than being
approximated. Enumerating the split does not make it *win*; the structural cost
model prices its extra dispatch and staged bytes, and preference belongs to
measured calibration.

### That measured calibration is taken, and it says the current behaviour costs a factor

Both paragraphs above defer preference to measured calibration.

**Measurement, 2026-08-02 — the state this section used to describe.** [The retained sweep](../../spikes/program-planning/reduction-crossover/README.md) compiled the reduction program family across 36 shapes against the authoritative profile under `NumericalContract::FLUSH_AND_REASSOCIATE_F32`, on a host matching the ledger's execution-environment row in every field. Exactly one shape retained all three strategies at once: one row of four contributors. Every wider shape was refused by hard feasibility on the grid axis, `event=feasibility:grid-axis:rejected:target-infeasible:threads=<required>:4`, at the pointwise prologue. A crossover needs at least two shapes on which the alternatives coexist and can be timed, so one point admitted no crossover, no calibration, and no held-out prediction.

That single point was forced by arithmetic rather than found by sampling. Both parallel strategies are withheld below four contributors — by `governed_partition` for the split and, since the partition calibration below landed, by `capped_tree_partition` for the tree, two rules that choose different participant counts but admit exactly the same extents — and the grid-axis row caps the prologue's one-invocation-per-element launch, so `4 <= contributors <= rows * contributors <= grid_axis_bound`; at a bound of four that chain closes on one shape.

**Measurement, 2026-08-04 — the row moved, and the domain opened.** [`establish-an-upper-bound-authority-for-the-metal-grid-axis-row`](../../tickets/establish-an-upper-bound-authority-for-the-metal-grid-axis-row.md) found that no normative source could fill the row at all: the row is consumed as a guarantee and therefore needs an authority stating a floor, while every available source states a ceiling on the space. It is now a bounded measurement at 268,435,456, from [the retained extent ladder](../../spikes/target-profiles/metal-grid-axis-extent/README.md). Rerunning the sweep unchanged against it reports **24 of 36 shapes retaining all three strategies and no grid-axis refusal at any shape**, where it previously reported one and twenty-three. The remaining twelve are the contributor counts admitting no balanced exact partition, which is a different question with its own ticket.

**Measurement, 2026-08-07 — the strategies were dispatched, and the serial fold is the wrong default over a large part of the domain the moved row opened.** [The retained dispatch sweep](../../spikes/program-planning/reduction-dispatch-crossover/README.md) timed all three alternatives across a 92-cell matrix on a host matching the ledger's rows in every field. The crossover is a diagonal contour, not a point, and both sides of it are far outside the noise: at four rows of 8,192 contributors the serial fold costs **50.7 times** the best parallel plan, and at 16,384 rows of four it costs **0.56 times** it. Parallel plans win wherever the row count alone cannot saturate the device; the serial fold wins wherever it can. **The two parallel strategies sit inside each other's noise almost everywhere**, so the consequential decision on this program family is whether to parallelize, not which parallel plan to use.

**A three-parameter analytical model reproduces that verdict, and one of its three parameters does all the work.** Cost is `sum over stages of ( encoder + max(work / parallel_threads, depth) * step )`, where `work` is a stage's fold steps summed over its invocations and `depth` is its longest sequential path — the classical work-span bound, in which no parameter names a strategy and no strategy has a term of its own. Fitted on the perfect-square contributor counts and scored on the rest, it agrees with the measured serial-or-parallel verdict on **24 of the 26 held-out cells whose verdict is separated from the noise**, and following it costs at most **1.81x** on a held-out cell. Rescaling the per-encoder or per-step parameter leaves *every* predicted winner unchanged; only `parallel_threads` — the fold steps the device retires at once — moves a decision.

**So the current behaviour is now a known cost rather than an open question, and it is unchanged for a boundary reason rather than an evidence one.** All three strategies are still enumerated and retained, the structural model still prunes none of them wrongly, and selection still takes the serial fold. Consulting the measurement would mean a target profile *declaring* `parallel_threads` and physical selection reading it — a public `TargetProfileBuilder` surface, a canonical-descriptor change, and therefore an identity move across every pinned artifact identity and cache subject. That is [`activate-measured-reduction-selection-from-a-target-cost-row`](../../tickets/activate-measured-reduction-selection-from-a-target-cost-row.md), and it is Tom's to accept. Encoding the preference any other way — widening the structural dominance dimensions, or biasing a constant — would be choosing a constant until the desired plan won, which is the failure the strategy tickets exist to prevent.

`tiler_build::metal_plan::tests::the_measured_grid_axis_admits_more_than_one_three_strategy_shape` reports the domain, so the paragraphs above cannot go stale without something saying so. It lives in `tiler-build` because that is the crate that can see the profile calibration measures against; the earlier trigger named here read the *target-neutral prototype baseline* instead, whose row is unmoved and cannot be moved by a single-target measurement.

**Boundary.** One profile, one contract, one program family, `f32`, one host row, wall-clock rather than GPU-busy time. The fitted parameters are quantities of that machine and the contour's position is determined to about a factor of four in `parallel_threads`. The domain counts above are statements about which plans exist; the timings are statements about one host.

**Measurement, 2026-08-07 — the partition within a strategy is measured too.** [The retained partition calibration](../../spikes/program-planning/reduction-partition-calibration/README.md) swept every admissible partition at seven of the crossover sweep's separated shapes and refuted the balanced exact split as *optimal* for both strategies (beaten on 10 of 14 cells, worth up to 5.05× between best and worst admissible value) while confirming it as a defensible middle (never worse than 1.413×). **The tree took its replacement and the split did not.** Under [`cap-the-tree-reduction-participants-at-the-measured-256`](../../tickets/cap-the-tree-reduction-participants-at-the-measured-256.md) `single_workgroup_tree_region` reads `capped_tree_partition` — the largest admissible participant count not exceeding 256, worst held-out regret 1.008 against the balanced choice's 1.216 — while `split_reduction_regions` keeps `governed_partition`. The split has no constant to take, because its optimum moves with the same saturation quantity the strategy contour turns on, so its calibration belongs to the target cost row above rather than to a constant. This is a *width* change and not a selection change: which alternative wins is still the boundary question of the paragraph above. It also parts the two strategies' declared groupings for the first time — they agree at four contributors, **twelve is the smallest count at which they differ**, and 2,561 of the 3,530 admitting counts below 4,096 differ while the remaining 964 still agree. The numerical consequence is recorded in [correctness and testing](../correctness-and-testing.md).

## Rearrangement schedules

Alternatives include:

- direct loads/stores from a composed logical tensor-access map;
- collapsed contiguous copy;
- tiled threadgroup-memory transpose;
- materialize once for multiple consumers.

A no-kernel alias/view result is another global physical alternative, not a
kernel schedule, and is deferred from the initial Candle custom-op path.

## Future contraction schedules

Einsum adds global contraction-order choices and local implementations:

- direct scalar or tiled contraction;
- GEMM canonicalization;
- optimized library matmul;
- layout conversion enforcers;
- batching and split-reduction strategies;
- fusible pointwise prologues and epilogues.

Contraction planning should follow, not precede, the boundary-contract and cost
infrastructure.

The global contraction-order choice in that sentence is not one of the local implementations listed under it. It is the tensor-contraction association rewrite that [the optimizer](optimizer.md#logical-exploration) admits only under effective distributivity, reassociation, and operand-permutation permissions, so a schedule may assume a contraction order only after that rewrite has been authorized. No contract Tiler can currently express authorizes it, because [numerical semantics](../numerical-semantics.md#distributivity-is-outside-the-order-contract) defines the distributivity dimension without admitting any permission that grants it. A schedule therefore may not select a contraction order at all today, and that is a legality boundary rather than an unimplemented search. The local implementations remain subject to the numerical-semantics legality rules above, which prove reassociation and operand permutation separately. "Contraction" is the tensor sense throughout this section, not ADR 0015's fused-multiply-add permission.

## Search control

Schedule exploration must be bounded using target limits, ranked candidate
sizes, dominance pruning, and explicit compile-time budgets. Deterministic
heuristics are preferred initially. Offline empirical calibration may improve
candidate ranking later without introducing runtime JIT compilation.
