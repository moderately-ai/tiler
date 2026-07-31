---
id: exercise-opaque-admissions-downstream-of-the-frontier
title: Exercise opaque admissions downstream of the frontier
status: done
priority: p2
dependencies: []
related: [integrate-opaque-calls-into-the-physical-frontier, implement-boundary-property-enforcers]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, testing]
---
## User-visible outcome

The claim "an opaque call is a first-class alternative to a scheduled kernel" becomes *tested* below the frontier instead of asserted: selection composes or refuses it correctly, plan identity distinguishes it, cost arms report `Unknown` rather than zero, and an unlowerable plan refuses instead of silently omitting the call's work. Today all of that is unexercised — green tests stop at admission.

The frontier admits a registered, well-bound, feasible opaque call, and everything downstream of that admission is untested with an opaque body: selection composition, plan identity encoding, the component-cost `Unknown` arms (`scheduled()?`), the lowering refusal (`unlowerable-opaque-body`), and `pipeline/verify.rs`'s `None` arm. The last two are additionally unreachable in the compile path because no production provider proposes an opaque call (`pipeline/planning.rs` hardcodes the one governed provider and passes an empty registry).

The audit's framing: untested-but-claimed levels are findings. The integrate ticket's closing section now states what is tested at which level; this ticket closes the gap.

## What to drive, at selection level and below

- A cover whose region has both a scheduled admission and an opaque admission: selection must treat them as alternatives, and the plan identities must differ.
- An opaque producer with `Aliasing::MayAliasInputs` composing against a scheduled consumer requiring `MaterializedBuffer`: the handoff must be refused by `unsatisfied_properties` naming `Materialization` — this is the first *reachable* case of the mismatch the enforcers ticket waits for, so `implement-boundary-property-enforcers`'s startable condition should be re-evaluated when this lands (its trigger note names this).
- A selected plan containing an opaque body: `Indexing`/`RedundantWork`/`MemoryTraffic` report `Unknown` (not zero), lowering refuses with `unlowerable-opaque-body`, and verify's `None` arm fires rather than a filtered region list reaching `build_plan_program`.

## Constraint that must survive

Do not fabricate a production provider to make paths reachable in the compile path — test-level providers are the honest subject until caller-supplied physical providers exist. The compile-path unreachability is a stated boundary, not a bug to hide.

## Closes when

- Each bullet above has a test that can fail (perturb once and watch it fire), and the integrate ticket's tested-at-which-level table is updated to match.

## Graph maintenance

- **The AliasView-vs-MaterializedBuffer refusal test is the important one**: it is the first *reachable* instance of the mismatch `implement-boundary-property-enforcers` is deferred on. When it exists and refuses correctly, append that fact to the enforcers ticket's trigger section — its startable condition explicitly waits on this evidence — and re-evaluate whether the deferral still holds. Do not un-defer it yourself; record the evidence and leave the status to the coordinator.
- **When each bullet lands**: update the tested-at-which-level table in `integrate-opaque-calls-into-the-physical-frontier`'s closing corrections in the same commit — that table is the honest boundary this ticket exists to move.
- **If a downstream layer turns out to mishandle an opaque body** (wrong result rather than typed refusal): that is a p1 fix ticket, filed immediately with the failing test attached, not a note here.
- **Keep the compile-path boundary honest**: test-level providers only. If you need a production provider to reach a path, the path is out of this ticket's scope — it belongs to the caller-supplied-providers work, and forcing it here would hide that boundary.

## Implemented outcome

Test-level opaque providers now reach every named downstream authority without changing the production provider set. Selection retains scheduled and opaque alternatives for the same fused region and their plan identities differ because the selected implementation identity is folded. A `MayAliasInputs` opaque producer is refused when the scheduled consumer requires `MaterializedBuffer`, with the disagreement naming `Materialization`. The three schedule-dependent analytical components return `Unknown`. Lowering returns `unlowerable-opaque-body`, and verification independently returns `portfolio-schedule-binding` from the absent scheduled body.

Each new check was fault-injected once: omitting the implementation identity collapsed the two plans; substituting `Exact(0)` failed the cost assertion; changing the aliasing declaration to `Distinct` admitted the handoff; and restoring schedule filtering failed both lowering and verification checks.

## Evidence added by the L3 contraction spike — 2026-07-31

Recorded here rather than turned into a separate integration path, because an opaque/library contraction realization *is* an opaque physical call and the admission machinery this ticket exercised already exists. [`spike-first-metal-contraction-vertical`](spike-first-metal-contraction-vertical.md) measured `MPSMatrixMultiplication` as one of six realization candidates for the workload's projection contraction; the [realization record](../docs/research/scheduling/first-metal-contraction-realizations.md) holds the derivation and the [probe](../spikes/scheduling/metal_contraction_vertical/README.md) the retained bits.

**Measurement — the library route is the fastest thing measured at prefill and the only candidate with no attributable reduction topology.** On the M3 Pro bench host it ran the `512x3072x1024` prefill cell in 839 microseconds against the surviving strict kernel's 6,390 — 7.6x. Over an eight-case corpus it is consistent with **none** of twenty-two named reduction topologies, each refuted with the case that refutes it. Its evaluation is also *shape-dependent* on one device, one dtype, one driver: at three cells with `M = 128` it is byte-identical to a hand-written `simdgroup_float8x8` kernel, at `M = 1` and `M = 10` it differs from everything, and at `16x16x16` it differs from the simdgroup kernel again.

**Inference — two consequences for caller-supplied providers, neither of which this ticket's test-level providers exposed.**

- A provider's numerical guarantee cannot be keyed on the provider alone. The measured shape-dependence means a single declaration would be unsound, and a `(provider, shape class)` declaration needs a shape partition the provider does not publish. That is **decision D-7** of the realization record, and it is a question for whoever admits caller-supplied providers rather than something to resolve by measuring more shapes.
- The refusal is a *numerical-guarantee* refusal, not a feasibility or cost one. [Optimizer](../docs/compiler/optimizer.md) already requires a candidate to advertise "a machine-checkable numerical guarantee, realization/provider identity, and scoped evidence", admitted "only when that guarantee refines every effective operation contract", and checks that before the dominance relation. A provider that publishes no accumulation order refines nothing and is inadmissible before it is ever costed. The L3 elimination is an instance of that existing rule and needs no new mechanism — but nothing currently *exercises* the rule with a provider whose guarantee is absent rather than merely `Unknown`-costed, and that gap is worth a test when caller-supplied providers land.

Neither point reopens this ticket. Both belong to [`implement-opaque-physical-call-providers`](implement-opaque-physical-call-providers.md) and to whatever admits a caller-supplied provider; they are recorded here because this is the ticket a reader reaches when asking what an opaque admission has actually been shown to do.
