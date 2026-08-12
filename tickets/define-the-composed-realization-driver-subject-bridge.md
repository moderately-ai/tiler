---
id: define-the-composed-realization-driver-subject-bridge
title: Define the composed realization driver's subject bridge
status: awaiting-decision
priority: p2
dependencies: [retain-the-selected-semantic-candidate-for-the-conformance-oracle, decide-the-safe-cross-crate-composed-reference-boundary]
related: [retain-each-plan-alternative-s-verified-semantic-candidate, implement-the-composed-realization-evaluation-driver, accept-the-composed-realization-evaluation-surface, accept-the-realization-witness-surface]
scopes: [implementation/compiler, implementation/conformance, implementation/ir, implementation/reference, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, conformance, reference, correctness]
---
## User-visible outcome

The already-accepted composed conformance driver receives one compiler-minted, inseparable subject for the semantic candidate and its ordered physical realization, rather than asking callers to assemble a program and witness sequence that could come from different alternatives.

## Fixed constraints — corrected by the accepted prerequisite on 2026-08-12

- The supported plan-binding wrapper lives in `tiler-conformance`, remains `pub(crate)` and test-only, and accepts a complete `PlanAlternative` plus declared inputs and explicit reference registry/work authority. It never accepts a free `(SemanticProgram, witnesses)` pair.
- The exact retained candidate stays mandatory and private inside `ProgramAlternative` until this bridge and the driver land atomically.
- `tiler-reference` never names a scheduled plan. Its raw tensor-taking `ValueId` pin/observe primitive remains genuinely crate-private; its separate safe cross-crate session accepts no caller-provided internal tensor and owns every reference-produced intermediate.
- No caller may reconstruct a missing candidate, use the baseline, substitute another alternative's witness, or silently select a standard registry, strict contract, unsupported topology, or other default.
- The compiler and reference plumbing must be named honestly as language-public `#[doc(hidden)]` SPI where sibling-crate access requires it. Rust visibility is not presented as a safety boundary.
- No artifact, cache, canonical identity, or schema change is authorized.

## Source-first Fact audit — exact main `6b26ead4`

- **Verified — the prerequisite contradiction is resolved.** [`decide-the-safe-cross-crate-composed-reference-boundary`](decide-the-safe-cross-crate-composed-reference-boundary.md), anchor `Decision — accepted 2026-08-12`, keeps raw tensor pinning private, accepts a safe reference-owned no-tensor session, requires explicit registry/work authority, and keeps the conformance wrapper `pub(crate)` and test-only. The old fixed sentence calling that wrapper public and the old close condition promising a first non-test conformance API were false after that acceptance.
- **Verified — `PlanAlternative` already carries the load-bearing owner link.** `pub struct PlanAlternative<'a>` in `crates/tiler-compiler/src/session.rs` contains both `&'a Compilation` and `&'a ProgramAlternative`; `Compilation::{alternatives, selected}` mint the pair. `verify_global_portfolio`, under rules `semantic-portfolio-owner-set` and `semantic-portfolio-owner-binding`, re-derives each alternative from its semantic owner and private plan. The supported wrapper should therefore continue to accept `PlanAlternative`, not a detached bridge object.
- **Verified — current public evidence is insufficient but deliberately bounded.** `impl PlanAlternative` exposes kernels, ABI, selected providers/capabilities, delivered realization, and prepared-entry requirements. It exposes neither retained `P'`, private selected plan/cover, scheduled regions, realization witnesses, nor semantic handoffs.
- **Verified — the candidate retention dependency is mandatory, direct, and still unimplemented.** `SemanticCandidate.proposal` is transient today; the accepted child retains one Arc-backed `SemanticProgram` directly on every internal alternative and rechecks its complete semantic identity. No optional candidate or lookup is available to this bridge.
- **Verified — cover order and execution order are different subjects.** `RegionCover::{regions, materializations}` in `crates/tiler-compiler/src/cover.rs` are in canonical identity order. `CoverAssembly::from_plan`, anchor `let order = execution_order(cover)?`, derives producer-before-consumer region order, flattens selected multi-stage realizations, and produces the `scheduled_regions` order retained by `ProgramAlternative`. The ticket's old phrase `ordered stage cover` was imprecise; zipping cover rows with scheduled rows is wrong.
- **Verified — `CoverAssembly` is the one construction authority to reuse.** It already derives stage spans, internal bindings, actual producer/consumer dependencies, multi-pass `AssemblySplit`s, publishing copies, staged-realization handoffs, output attribution, and execution order. Final portfolio verification already re-runs it and compares the reassembled regions to the retained schedule vector. A bridge should factor that derivation once and reuse it, not restate its joins in `session.rs` or in conformance.
- **Verified — the producer of a materialization is not recoverable from cover folklore.** A region that both materializes and publishes writes the edge in its first dispatch and publishes in its second. Assuming the producer is the region's last stage produces the wrong dependency. The bridge must consume `CoverAssembly`'s actual bindings/dependencies.
- **Verified — public KIR evidence cannot replace this bridge.** `VerifiedKernelProgram` exposes execution order, dependencies, partial reductions, and staged realizations, but `MaterializedOrigin` explicitly does not claim which semantic value an internal temporary realizes. Artifact/KIR reconstruction would therefore be a lossy new authority.
- **Verified — `RealizationWitness::of` is derived borrowed evidence, not an ownership link.** It reads one exact retained `VerifiedScheduledRegion`'s numerical realization, scalar program, and reduction topology. It carries no semantic owner, cover order, materialization association, or whole-witness equality.
- **Verified — not every assembly value is a semantic `ValueId`.** A real `MaterializationEdge::value()` is a graph-local semantic ordinal and can be translated against the exact retained `P'` with owner/range checks. `RegionGraph::with_realizations` can also append synthetic handed values for staged realizations, and `AssemblySplit` partials/results and publishing-copy internals are assembly values rather than semantic values. The first composed-oracle slice must name and refuse staged/synthetic populations rather than coercing their ordinals into `ValueId`.
- **Verified — the plain-scalar redirection still governs reference.** `RealizationWitness` stays in shared IR and the safe reference session receives typed scalar/value/fold descriptors; `tiler-reference` does not learn compiler plan types.
- **Verified — a visitor is sequencing, not access control.** A callback can clone Arc-backed `SemanticProgram` and copy owned `ValueId`/witness scalars. Higher-ranked lifetimes prevent borrowed scratch state from escaping, but cannot make deliberate recombination impossible. No-mixing at the supported boundary comes from accepting one `PlanAlternative`, deriving one complete projection internally, validating all descriptors against the same `P'`, and accepting no caller recipe.
- **Verified — an ephemeral projection owes no identity step.** `ProgramAlternativeIdentity` domain `tiler.program-alternative.v2` already folds the complete semantic identity of `P'` and selected-plan identity; the retention child rechecks that binding. Graph-local ordinals remain navigation and the bridge is neither serialized nor cached.

## Audited decision packet

### Recommended exact boundary

Add one language-public, `#[doc(hidden)]`, one-shot method on `PlanAlternative`, using a closure rather than a public visitor trait. Names remain implementation-level, but the accepted shape is:

```rust
#[doc(hidden)]
pub fn visit_composed_realization<E>(
    self,
    visitor: impl for<'event> FnMut(
        ComposedRealizationEvent<'event>,
    ) -> Result<(), E>,
) -> Result<(), ComposedRealizationVisitError<E>>;
```

`ComposedRealizationEvent` is a closed, exhaustive language-public SPI enum with private fields, no public constructors, and borrowed accessors. A new event must stop the sibling conformance adapter at compile time rather than fall through a wildcard. Its initial population is:

1. `Begin`: the exact borrowed retained `P'` and exact stage/materialization/split census.
2. `Stage`: zero-based execution ordinal, `RealizationWitness::of` for that exact retained scheduled region, and the ordered covered semantic atoms as compiler-minted views of `(OperationId, realization-stage ordinal)`.
3. `Materialization`: one checked semantic `ValueId`, its actual producing dispatch, and its complete consumer dispatch population. It is emitted once after its producer and before every consumer. A synthetic/staged value refuses before `Begin` in the first slice.
4. `Split`: the semantic fold occurrence plus the producer/combiner dispatches and the exact `AssemblySplit` partition association. Multi-pass stages are paired through `AssemblySplit`, never adjacency guessing; a cooperative single-stage fold remains stated by its `Stage` witness.
5. `Complete`: closes all stated censuses. The conformance adapter finalizes the safe reference session only here.

Do not expose `SelectedPlan`, `RegionCover`, `MaterializationEdge`, `SemanticMemberId`, `SemanticValueId`, assembly-internal ordinals, parallel candidate/stage/edge slices, or a caller-mintable recipe. Publishing-copy administration is validated by assembly; it never acquires a fabricated semantic pin.

The test-only supported wrapper has the following ownership shape:

```rust
pub(crate) fn evaluate_composed(
    alternative: PlanAlternative<'_>,
    registry: &FrozenReferenceRegistry,
    allowance: IterationStepAllowance,
    inputs: &[InputBinding<'_>],
) -> Result<ReferenceOutputs, ComposedConformanceError>;
```

The exact retained plan lifetime is independent of input tensor borrows; outputs and errors are owned. There is no evaluator/registry/default parameter omission and no free bridge/recipe parameter.

### Atomic validation and refusal boundary

Before emitting `Begin`, the compiler must:

1. rederive the retained candidate's complete semantic identity and the alternative owner/identity binding;
2. invoke the same factored `CoverAssembly::from_plan(P', selected_plan)` authority used by construction;
3. compare its execution-ordered regions elementwise with the alternative's retained scheduled regions and the verified program stages/canonical identities;
4. translate every semantic member/value ordinal through exact `P'` ownership and bounds checks;
5. derive producers, consumers, split pairs, and publishing copies from assembly bindings/dependencies and prove exact censuses with no duplicate or omitted relation;
6. refuse the first-slice staged/synthetic population by a named typed cause; and
7. build all temporary descriptors before calling user code.

A compiler-subject failure therefore emits no event and causes no partial reference work. A visitor failure stops immediately and is returned distinctly as `Visitor(E)`; it emits no later event, no output, and no fallback. `Complete` is required for success. The safe reference session independently validates every handle and descriptor against the same `P'`, exact arithmetic subject and subnormal modes, topology, type/shape/reachability, reference registry, and work allowance. Unsupported or incompletely witnessed freedom refuses by its own typed rule.

`ComposedRealizationVisitError<E>` should be `#[non_exhaustive]` and preserve concrete compiler-subject failures separately from `Visitor(E)`; no string erasure. The bridge implementation belongs beside `CoverAssembly` (or a factored compiler-private module that owns it), not as a duplicate mapper in `session.rs`.

### Why this is the sole current survivor

The event visitor does **not** claim to make data extraction impossible. Its advantage is narrower and real: the supported wrapper receives one complete, compiler-ordered, prevalidated stream and cannot accidentally zip or omit independently fetched slices. The plan-binding claim remains exclusively at that wrapper. A direct user of the doc-hidden compiler SPI and safe reference session can compute a caller-stated reference value, but cannot thereby manufacture the supported conformance verdict.

An opaque borrowed subject with `candidate()/stages()/materializations()` accessors performs the same reconstruction while exposing independently repeatable streams and inviting omission or wrong joins. A public visitor trait adds an unnecessary participation surface and cannot be genuinely sealed for a sibling crate. A returned iterator permits silent prefix consumption unless it recreates a mandatory finish protocol. The one-shot fallible closure makes sequencing and completion explicit with the least surface.

### Performance, maintenance, and compatibility

- **Ordinary compilation:** no bridge work. The only persistent cost is the already-accepted one Arc bump per alternative for `P'` retention.
- **One conformance call:** re-run the existing assembly derivation plus bounded scans and temporary maps over program values, stages, and materialization edges; temporary memory is dropped after the call. Reference evaluation remains the dominant work. No device/kernel runtime changes.
- **Maintenance:** one construction authority and a closed event census prevent compiler/conformance traversal drift. Unsupported staged/synthetic values are visible refusals rather than a prematurely universal mapping.
- **Compatibility:** the SPI is a Rust public-boundary commitment despite `#[doc(hidden)]`, but it changes no artifact, cache, request, schedule, KIR, semantic, or canonical identity bytes.

Retaining an eager private recipe beside every alternative can match correctness and strictness, but charges ordinary compilation and every retained alternative for a test-only consumer. A lazy cache adds synchronization, clone, and invalidation complexity. Both are dominated until a repeated-conformance measurement shows on-demand assembly materially matters. A detached owned snapshot becomes top-tier only if a real asynchronous/cross-thread persistence consumer appears; artifact-only or cross-process use remains the separate source-bundle decision.

### Strongest counterpoint and reversal evidence

The callback and closed event enum are more API machinery than a scoped opaque subject, and they cannot prevent malicious owned extraction. Reverse to a scoped opaque subject only if the safe reference session necessarily consumes one single complete iterator with a mandatory `finish()` that independently refuses every omitted suffix, making compiler-driven visitation redundant. Reverse to retained or cached projection only if a defined repeated-conformance workload measures on-demand assembly as material. Neither fact exists at this base.

### Required negative controls

- Substitute baseline `P`, swap retained `P'`, private plan, or retained scheduled regions independently, and observe distinct identity/assembly/stage refusals.
- Reverse, omit, or duplicate one edge; separately perturb a materializing-and-publishing producer from the first to the last dispatch.
- Use a foreign or correct-shape/wrong-graph `OperationId` and `ValueId`; ownership rather than shape must refuse it.
- Introduce a staged synthetic handoff and observe its named first-slice refusal before `Begin`.
- Make the callback fail after its first event and prove there is no `Complete` or output.
- Omit an event in the conformance adapter or skip finalization and observe incomplete-coverage refusal.
- Attempt to pass a device-produced tensor into the safe reference session and retain compile-fail evidence that no such parameter exists.
- Pin nonzero multi-stage, multi-edge, and split populations plus a zero-edge control; assert producer-before-materialization-before-consumer ordering.

## Decision still required

Tom must accept or reject the exact one-shot prevalidated event SPI, its language-public/doc-hidden visibility, the first-slice synthetic/staged refusal, and the test-only wrapper signature. No equal-correctness alternative remains nondominated under the present no-consumer/no-measurement constraints.

## Closes when

Tom has accepted the exact bridge and wrapper shape, the ticket records the decision provenance, and the implementation ticket is corrected from its stale `one public driver` wording to the accepted `pub(crate)` test-only supported wrapper with the event SPI and first-slice refusal population.
