---
id: declare-metal-subgroup-realization-facts-in-the-target-profile
title: Declare Metal subgroup realization facts as atomic target facts
status: in-progress
priority: p2
dependencies: [accept-adr-0094-subgroup-execution-tier, admit-an-atomic-subgroup-realization-subject-to-target-profiles, decide-the-prepared-subgroup-width-equality-gate, carry-subgroup-width-through-exact-prepared-entry-equality, measure-metal-thread-execution-width-across-prepared-pipelines]
related: [design-the-subgroup-execution-tier, declare-cpu-vector-realization-facts-in-the-target-profile, correct-the-subgroup-threads-route-dimension-meaning, correct-the-metal-profile-authority-ledgers-stale-identity-pins, make-prepared-entry-observations-typed-and-key-dispatched, generalize-deferred-target-provenance-beyond-capability-axes, bind-prepared-pipeline-caches-to-loader-derived-route-identity]
scopes: [implementation/compiler, implementation/metal, implementation/build, research/target-profiles, contracts/decisions, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, metal, subgroup, execution-hierarchy, feasibility, public-boundary, decision, needs-tom]
claimed_from: todo
assignee: worker-metal-subgroup-facts
lease_expires_at: 1787066399
---
## User-visible outcome

A target profile states what a Metal device's subgroups actually do — as atomic declared facts a feasibility predicate reads — so a subgroup schedule is admitted or refused against declared target properties instead of against an assumption compiled into the backend.

## Why now

**Fact — the acceptance node released no implementation ticket when written.** [`accept-adr-0094-subgroup-execution-tier`](accept-adr-0094-subgroup-execution-tier.md), anchors `is what releases the implementation tickets gated behind it` and `releases the implementation tickets`, makes that claim; [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md), anchor `Nine public-boundary items are enumerated`, records the design-era surface. This is the target-profile third of the implementation population that now makes the release claim concrete.

**Resolved 2026-08-01 — the node closed and this ticket is what it released.** [ADR 0094](../docs/decisions/0094-bind-a-subgroup-combine-to-a-register-transfer-tree.md) landed `accepted` and the acceptance node is `done` under its final id, which is why the link above no longer reads `accept-the-subgroup-execution-tier-adr`. The paragraph above is preserved rather than rewritten because it is the reason this ticket exists.

**Fact — the record enumerates a `SubgroupRealization` subject and its builder method among nine public-boundary items for Tom.** [The subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md), heading `Public-boundary items, enumerated for Tom and not self-accepted`, opens that enumeration; [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md), anchor `a \`SubgroupRealization\` subject`, names the subject and builder method explicitly. This ticket drafts them and accepts none.

**Fact — one route dimension in the landed vocabulary is already wrong for these routes, and its correction is independent.** [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md), defect paragraph opening `A live defect was found in landed public vocabulary` and anchor `threads one subgroup must execute in lockstep`, records that `RouteResourceDimension::SubgroupThreads` was a floor over lockstep threads, and that lockstep within a subgroup is not a property current GPU families guarantee — the CUDA guide's independent-thread-scheduling text withdraws it explicitly. [`correct-the-subgroup-threads-route-dimension-meaning`](correct-the-subgroup-threads-route-dimension-meaning.md) owns that and is independent of whether this design is accepted; do not absorb it here, and do not declare a fact whose meaning that ticket is still fixing. **That ticket is `done` as of `77c36d5`**, so the meaning is now fixed rather than in flight: the dimension is compared by equality and carries no lockstep claim, and [the artifact ABI](../docs/artifact-abi.md) states it citing [ADR 0094](../docs/decisions/0094-bind-a-subgroup-combine-to-a-register-transfer-tree.md) item 7. Declare against the equality relation, not the floor this paragraph was written under.

**Inference — declaring these as facts rather than as backend code is what ADR 0090 item 1 decided.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), anchor `Target profiles declare what a target *can do*`, records the accepted split: profiles declare what a target can do, providers propose what to do with it, and the host performs every comparison. A subgroup width or a shuffle capability hardcoded in `tiler-metal` would put a target fact on the provider side of that split.

## Implementation keys

- **One `SubgroupRealization` subject matched by equality**, and one `declare_subgroup_realization` builder method — the shape public-boundary item 5 and [`declare-cpu-vector-realization-facts-in-the-target-profile`](declare-cpu-vector-realization-facts-in-the-target-profile.md) already state for the sibling tier. Dimensions (width, shuffle kind, out-of-range shuffle behaviour, and any others Tom freezes at acceptance) live *inside* the subject; they are not independently declarable, and there is no per-dimension accessor or setter. A bundled "supports subgroups" boolean is the shape to avoid: it cannot explain a refusal. Refusal names the unmatched or `Unrealizable` subject (and its explain row), not a free-standing dimension fact as if dimensions were separate profile declarations. Equality-based subgroup width is a dimension inside the subject, in addition to the already-landed route-dimension equality on `RouteResourceDimension::SubgroupThreads`.
- Hard feasibility stays separate from estimated cost. A subgroup schedule a target cannot realize is rejected with an explainable reason, never priced at an infinite or arbitrary cost.
- Where a subject is not observed on the tested hosts, the profile stays silent and feasibility resolves `Unknown` and refuses, not defaulted. Facts about a tested host stay distinct from portable guarantees. Two-valued support (`Realized` / `Unrealizable`, or the synchronization spelling) so "unsupported" and "unmeasured" are not one state.
- Mirror [`declare-cpu-vector-realization-facts-in-the-target-profile`](declare-cpu-vector-realization-facts-in-the-target-profile.md)'s shape; the two tiers' profile declarations should read as one system.

## Required failure-path evidence

A profile declaring a subject differing in exactly one dimension from the required one, once per dimension, each resolving to `Unknown` rather than satisfying it — the check that the subject is matched whole. A profile declaring `Unrealizable` for the required subject, resolving to `Rejected` with the reason named. A profile silent on the subject, resolving to `Unknown`. A profile declaring one subject both `Realized` and `Unrealizable` at one phase, refused at construction. A profile declaring a subject the target family cannot support (or `Unrealizable` on a wrong-family path), refused with a named reason rather than admitted by default.

## Non-goals

Schedule bindings and kernel-IR constructs (their own tickets). Emission. The `RouteResourceDimension::SubgroupThreads` correction. Any measured subgroup performance claim — this ticket declares facts, it does not benchmark them. Public-boundary item 6 (`PreparedKernelPreflight` / routing-commit discipline for prepared-pipeline `threadExecutionWidth`) is not self-accepted here; research ties the measurement deferral "is `threadExecutionWidth` ever knowable earlier than preflight" to this first Metal subgroup realization ticket, so implementers must not silently absorb the preflight stage into this CompileProfile subject *or* drop the three-stage width resolution the research requires — report residual ownership to Tom rather than invent a second ticket id.

## Decision packet — 2026-08-09

The research record explicitly reserved the `SubgroupRealization` subject and builder method for Tom. Recommendation: accept **one** `SubgroupRealization` atomic subject matched by whole-subject equality, with one `declare_subgroup_realization` builder method and no per-dimension setters; two-valued support (`Realized` / `Unrealizable` or the synchronization spelling); `Unknown` when the profile is silent; equality-based width as a dimension *inside* the subject, plus the already-landed route-dimension equality — not a free-standing width- or shuffle-fact vocabulary and not a bundled `supports subgroups` flag. This decision accepts only the target-profile declaration surface, not schedule bindings, kernel constructs, measurements, or the `PreparedKernelPreflight` stage (public-boundary item 6).

**Correction — 2026-08-10.** Earlier packet and Implementation-keys wording ("separate atomic declared facts" / "each atomic and separately declared" / independent width- and shuffle-declaration failure paths) could be read as independently declarable dimension rows — the shape research §3 and public-boundary item 5 eliminate in favour of whole-subject equality. Restated above to match the research record and the CPU vector realization mirror; the anti-boolean clause is preserved.

## Closes when

The subject and its declaration path exist and are read by feasibility under whole-subject equality, every refusal above is checked by a check observed failing, no unobserved subject is defaulted, and the profile subject and its builder method have gone to Tom rather than been self-accepted. Residual ownership of public-boundary item 6 and the `threadExecutionWidth` measurement deferral is reported rather than absorbed or dropped.

## Source-first decision correction — 2026-08-11

The packet's field list was incomplete. Width and shuffle kind without arithmetic type could let f32 evidence admit a future BF16/F16/F64 schedule. Conversely, out-of-range behavior is irrelevant to the only admitted XOR butterfly: for power-of-two width, every `lane ^ mask` remains in range. Making it compulsory would turn irrelevant absent evidence into a false `Unknown` refusal.

The generic `TargetProfileBuilder` cannot implement the packet's “wrong target family” refusal because it carries no backend-family authority. Metal-specific correspondence belongs to `BoundMetalCompileDeclaration` or another Metal-owned factory. The generic builder validates the typed row, its evidence, duplicates, contradictions, and descriptor bounds.

No current authority licenses a standard Metal width row. MSL fixes no numeric SIMD-group width, the Apple9 ledger contains no subgroup realization, and ADR 0094 requires a prepared-pipeline `threadExecutionWidth` equality before routing commit. A public declaration surface may land inert; a production `Realized` row may not.

## Accepted decision — 2026-08-11

Tom accepted a required atomic whole-use subject:

```text
SubgroupRealizationSubject {
    width: SubgroupWidth,
    arithmetic: ArithmeticType,
    transfer: SubgroupTransfer::InRangeXorShuffle,
}
```

- Match the whole subject by exact equality. Independent dimension rows and `supports_subgroups` are forbidden.
- The support vocabulary is `Realized` / `Unrealizable`; silence and every neighbouring subject are `Unknown`. No default, inherited family row, or deferred-success arm exists.
- Normative and measured declarations use distinct required methods and existing distinct source types.
- Result lane, combine order/arithmetic, coordinate mapping, contributor coverage, activity, and padding identity remain schedule/intrinsic obligations and are not duplicated into this target subject.
- The generic builder does not guess a target family. Metal-owned binding validates that any Metal row matches the Metal compilation/profile evidence.
- The feasibility rule-set identity advances for the new predicate. Every row field, verdict, phase, authority, validity, and source is canonical identity. A conditional domain-separated row family preserves silent-profile descriptor bytes where the existing grammar permits it; exact domain and pin effects are rederived at implementation.
- This carrier may land before a production row, but no subgroup route becomes executable until the measurement and prepared-width gate below are complete.

Delivery is split into `admit-an-atomic-subgroup-realization-subject-to-target-profiles`, the completed `decide-the-prepared-subgroup-width-equality-gate`, its implementation ticket `carry-subgroup-width-through-exact-prepared-entry-equality`, and `measure-metal-thread-execution-width-across-prepared-pipelines`. This ticket now owns only the eventual evidence-backed standard Metal declaration and remains blocked on the unfinished delivery tickets.
