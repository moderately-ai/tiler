---
id: decide-the-contraction-tile-width-authority
title: Decide the contraction tile-width authority
status: in-progress
priority: p1
dependencies: []
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, offer-the-tiled-contraction-alternative-in-physical-planning]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, target-profiles, scheduling]
claimed_from: todo
assignee: worker-tilewidth
lease_expires_at: 1787428751
---
## User-visible outcome

Tom decides where a tiled contraction's tile width comes from, so the physical planner can offer the tiled alternative without either hard-coding a measured constant or inventing a target-profile row on a worker's authority.

## Why this exists

Filed 2026-08-22 by the coordinator, from the tiled-contraction lane's enumerated remainder. That lane landed the schedule, the lowering, and the Metal emission, and **stopped deliberately** before the compiler alternative because its first prerequisite is a decision rather than code.

**Fact — the repository already refused this exact shortcut once, and said why.** `crates/tiler-compiler/src/physical.rs` declines to offer the workgroup tree unless the target profile declares a closed width policy. Anchor: `Silence is` — which resolves exactly once in that file, at a `///` comment reading *"The tree is offered only under an explicit closed policy. Silence is / not a default, not a clamp onto the internal `256`, and not a / substitution of [`governed_partition`]."* Verified by the coordinator at `b3c07259`. **Note the anchor is deliberately short**: the rendered sentence "Silence is not a default" spans a line break in the source and greps to zero, which is the failure mode AGENTS.md records.

**Fact — the measured width is 16 and it is one host's measurement.** The tiled kernel the first-Metal-contraction record measures uses a 16-wide tile. A measured value is not a portable authority.

**Inference — there are two honest shapes and they are not equivalent.** Either the target profile grows a contraction-tile-width policy row — a target-profile public boundary, and so Tom's — or a named measured constant is accepted with the standing `MEASURED_TREE_PARTICIPANT_CAP` already has. The first makes the width a declared target property that a profile can refuse to state; the second makes it a repository constant that every profile inherits.

## Required work

- Re-audit both Facts and the Inference at your own base and report a per-Fact verdict before writing any packet prose.
- Apply AGENTS.md's decision-packet readiness gate in full. In particular, enumerate the status quo (no tiled alternative is offered at all, which is the current state and is honest) alongside both shapes above.
- For each survivor, state what it enables and prevents, its identity and public-surface consequence, its strongest counterargument, and the evidence that would reverse it.
- Do not present until the gate is satisfied. If one option dominates, recommend it rather than manufacturing a choice.

## Fact audit at `e7b6026f` — worker-tilewidth

Re-read at this base before any packet prose. Every verdict below rests on a full read of the file named, not on grep output.

| Ticket claim | Verdict | Evidence |
| --- | --- | --- |
| The repository refuses this shortcut once and says why; anchor `Silence is` resolves exactly once in `crates/tiler-compiler/src/physical.rs` | **verified** | `grep -c 'Silence is' crates/tiler-compiler/src/physical.rs` → `1`. The rendered `Silence is not a default` greps to `0`, as the ticket warns. The decline is `WorkgroupTreeUnavailable::QualifiedWidthPolicyUndeclared`, and the production gate reading it is the `workgroup_tree_width_policy(AvailabilityPhase::CompileProfile)` match in `single_workgroup_tree_region`. |
| The measured width is 16 | **imprecise, and understated in the direction that matters** | The number exists, but it is not a *measured* width in the sense the ticket's comparison needs. `spikes/scheduling/metal_contraction_vertical/kernels.metal` declares `constant uint TILE = 16;` — one compile-time constant, never a swept parameter. The spike named **four realization families** ahead of the run (`spikes/scheduling/metal_contraction_vertical/README.md`, `Four realization families were named ahead of the measurement`); it swept realizations, not widths. **No width other than 16 was ever executed.** |
| It is one host's measurement | **imprecise** | The retained experiment carries *two* records — `correctness on an Apple M4 Max and timing on the Apple M3 Pro bench host` (`docs/research/scheduling/first-metal-contraction-realizations.md`). The cost evidence that would bear on a width is M3 Pro timing only. The record is `research_status: "complete"` but `disposition: "pending"` and `implementation_status: "spike-only"`. |
| **Inference** — there are two honest shapes: a target-profile policy row, or a named constant with `MEASURED_TREE_PARTICIPANT_CAP`'s standing | **false as a disjunction** | The landed precedent is **both at once, and neither of the ticket's two**. `WorkgroupTreeWidthPolicy` (`crates/tiler-compiler/src/target/rows.rs`) is a closed *policy tag* naming a rule, with `deliberately no omitted/default case` and no public numeric cap; the number stays private in the compiler constant, whose own doc says `A numeric row is not required`. So the profile declares *which rule*, and the repository owns *which number*. A profile row carrying a `u64` width is not the precedent and was explicitly declined by it. |

**Repair.** The Inference above is replaced by the enumeration below rather than restated in new words. This does not change what the ticket is for — it still decides where a tile width's authority comes from — but it changes the answer, because the option the ticket did not name is the one the repository already accepted, and the evidence the ticket assumed exists does not.

### Two facts the ticket does not carry, both load-bearing

**Fact — the retained record measures the 16 losing, and attributes the loss to the 16.** `docs/research/scheduling/first-metal-contraction-realizations.md` timing table: at `w_vocab_slice` (1×8192×1024) `tiled` is 523.5 µs against `direct`'s 251.4 µs and `ksplit_contiguous`'s 234.0 µs — the **worst** of the candidates that run. At `t_vocab_full` (1×151936×1024) `tiled` is 9,669 µs against the best 4,247 µs, a factor of 2.28. The record states the cause: `one useful row and fifteen masked ones` when `M = 1`, `a schedule mismatch, not a bandwidth result`. So the only retained evidence about this width includes a measured regression *caused by* this width.

**Fact — the tile is one scalar, because the lowering says so.** `crates/tiler-ir/src/schedule/blocked.rs` admits a general `output_block: &Shape` and `contracted_tile: &Shape` (contracted extents must divide exactly on both paths — `exact-divisible on both paths`; output extents need not, because `admit_predicated_cooperative_contraction` ceilings the grid). But `crates/tiler-ir/src/kernel/lower.rs` carries `block: u64` under `Square tiles only`: `B_m == B_n == T_k`. So the authority under discussion is one `u64`, and `T = 16` means a 256-thread workgroup. The ticket's singular "tile width" is therefore right at the lowering layer and wrong at the schedule layer, and the restriction that makes it right is itself flagged as a defect by the parent lane's remainder item 4.

## Decision packet

### Option enumeration

**A — Status quo: offer no tiled alternative.** Current state. `crates/tiler-compiler/src/physical.rs` has `contraction_region` and no tiled sibling; there is no `TiledContractionUnavailable` enum; nothing in `crates/` hard-codes 16. Honest, and costs only that the landed IR, lowering, and emission stay unreachable from `compile()`.

**B — A named measured constant with `MEASURED_TREE_PARTICIPANT_CAP`'s standing.** *Eliminated before ranking*, on two independent grounds.

1. *It misstates its evidence class.* The cap's standing rests on a leave-one-out selection over every admissible partition of seven separated shapes with held-out worst regret 1.008. `TILE = 16` has a sweep of size one. Granting it the same standing is promotion by relocation — the failure `carry-the-tree-participant-cap-as-a-target-profile-row` already names.
2. *It defaults a width on every unmeasured target.* A repository constant is inherited by every profile, which is precisely what the cap's own doc forbids for itself: a second profile `must declare an explicitly qualified tree-width policy rather than inherit this one`. And what would be inherited is a value the retained record measures losing by 2.28x on a real workload cell.

**C — A numeric target-profile cost row carrying a `u64` tile width.** *Eliminated before ranking.* A cost row's silence is `Unknown` meaning *no preference*: `TargetCostRowResolution` documents that a consumer must treat it `never as a refusal, and never as a zero`. A width consumer reading `Unknown` must then either invent a width or withhold the strategy. If it withholds, the family is not a cost row; if it invents, it defaults a width on an unmeasured target. The precedent decided this exact point — `declare_workgroup_tree_width_policy` documents itself as `not a cost row and not a capability axis`.

**D — The precedent carrier: a closed named policy tag on the target profile, number private.** The exact `WorkgroupTreeWidthPolicy` template — a one-variant public enum, a `Declared`/`Deferred`/`Unknown` resolution whose `Unknown` is fail-closed, a general and a measured declaration constructor, a duplicate-per-phase error, a conditional silence-as-absence descriptor family, and a typed `…Unavailable::QualifiedTilePolicyUndeclared` decline. **This is the right carrier and it is not eliminated.** It is, however, **unpopulatable from the evidence that exists**, which is a separate matter — see below.

**E — Calibrate first under a frozen protocol, then land D populated from it.** Sweep the square block `T` over the admissible lattice at both halves of the workload, under a protocol that pre-names the beneficiary profile key before the run. *Survivor.*

**F — Derive the width from the shape, anchored on 16, as `capped_tree_partition` derives from `contributors`.** Not a separate option: it is one of the rules a sweep might select, and it cannot be chosen before the sweep exists. `measure-the-tree-width-excursion-past-the-cap` already found that nearest-to-anchor distance `is not a sufficient general width model` even for the tree, which had seven shapes behind it.

**G — Deferral with a reconsideration trigger.** Status quo plus bookkeeping. Applicable, and dominated by E only if E is affordable.

### The eliminating constraint: ADR 0113 component 3

[ADR 0113](../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md) is accepted and states when a measured row may enter a family-keyed profile. Condition (a) is that the producing measurement's frozen protocol `named that exact profile key as beneficiary` **before the run**. The contraction spike's protocol named four realization families and no profile key; the production key is `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`. The ADR is explicit that this is not repairable after the fact — a record whose protocol scoped it elsewhere `composes into nothing else, ever`.

**Consequence: option D cannot be populated from the retained record, today or ever.** A `MeasuredSquare16V1` variant declared from that record would state as measured what was never measured, on a key the protocol never named. That is not a close call and it is not a matter of taste; it is an accepted decision applied to its own case.

### The second eliminating finding: the width is not the gate

The parent lane recorded the width as the lane's first prerequisite. Re-derived at this base, it is not.

`crates/tiler-compiler/src/measured_cost.rs` `fn work_span` has arms for `None`, `Serial`, `Contraction`, `MultiPass`, and `CooperativeWorkgroup`, and ends in `_ => None,`. `CooperativeContraction` has no arm, so a tiled plan scores `None` and never reaches measured comparison. **A width authority granted today would therefore select a plan that still could not be chosen** — and, more sharply, *no tile width can be compared against another until the cost model can score the topology at all.* The work that unblocks the lane is the `work_span` arm and the `verify_region_output_binding` widening, and neither depends on a width. Both live on `offer-the-tiled-contraction-alternative-in-physical-planning`, which currently declares a dependency on this ticket.

### Nondominated frontier

After elimination, A/B/C/F are out and D is blocked on evidence, leaving **E** and **G**. E dominates G on every key dimension — correctness (it produces the evidence rather than declaring without it), strictness (it terminates in the fail-closed precedent carrier), maintainability (it reuses a landed, four-times-rederived template rather than inventing a family), and Tiler host runtime and memory (unchanged either way) — and its only cost over G is one measurement session, which is pre-authorized on the `m3` host.

**One option dominates, so no question is manufactured for Tom.** Per the readiness gate's step 5, the recommendation is taken rather than presented as a choice.

### Recommendation

**Decline to declare any tile-width authority now, and unblock the lane the other way.** Concretely:

1. **Break the false dependency.** `offer-the-tiled-contraction-alternative-in-physical-planning` should drop its dependency on this ticket for the two items that do not need a width — the `work_span` arm with the wildcard removed, and the widened output-binding check — and keep the width gate only on the final "offer the alternative" step.
2. **File the calibration** as its own ticket: a frozen protocol pre-naming `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` as beneficiary, sweeping square `T` over the admissible lattice at both a prefill cell and a decode cell, on the `m3` host, with the `M = 1` waste ratio reported as a first-class output because the retained record already identifies it as the dominant term.
3. **Then land D**, the precedent carrier, populated from that sweep, with a variant name that states what was actually fitted.

### Authority: this is not Tom's, and the derivation rather than the expectation

The ticket is tagged `needs-tom`. Re-derived at this base, the eventual carrier D does not meet [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md)'s always-ask list: it adds no new publicly reachable namespace (`pub mod target` already exists), no public trait, no breaking change to an existing public signature, and no `pub(crate)`-to-`pub` promotion. Tom explicitly **declined** the tighter variant that would have routed every new public type to him. The direct precedent agrees: the identical decision for the tree was put to Tom on 2026-08-11 and he **delegated it to the coordinator** after reviewing the ranked trade-offs.

What is genuinely Tom's is unchanged and untouched here: no width is declared, no profile row is added, no key is minted, and the support matrix does not move.

### Identity, schema, and public-surface consequences, derived

- **Today: none.** This packet declares nothing and edits no code.
- **When D eventually lands**, derived from `crates/tiler-compiler/src/target/descriptor.rs`: a new conditional family written last behind its own separator and only when non-empty is silence-as-absence, so every existing descriptor keeps its exact bytes and `tiler.target-profile.declaration.v11` and `descriptor.v11` do not move — the rule the encoder has now rederived for four consecutive families, each ending `The owning declaration domain therefore stays at` `v11`. One new pinned domain would be added to `crates/tiler-compiler/src/domains.rs`. **The exception, which must not be lost:** declaring the policy on `first_macos_apple9` changes *that profile's* descriptor bytes and every artifact and cache pin derived from it, exactly as the tree policy did — and it needs its own `PopulationRows` entry with its own execution environment, because ADR 0113 component 2 forbids folding measurement populations.
- **Support matrix:** unmoved. This is a fail-closed gate on an unoffered strategy, not a new capability.

### Strongest counterargument to the recommendation, and what would reverse it

**Counterargument.** The lane is p1 and this defers a working, landed realization behind a measurement session. One could argue that a fail-closed D declared with an honestly-named non-measured variant — say `UncalibratedSquare16V1`, whose name claims nothing — is strictly better than the status quo: it is opt-in, silence still withholds the strategy, and no profile inherits anything. That is a real argument and it is the closest thing to a surviving alternative.

**Why it still loses.** The only profile that would declare it is `first_macos_apple9`, and declaring it there is a claim *about that host* — the one the retained record measures the 16 losing on at two of eight cells. So the honest-name variant buys reachability by declaring, on the single host where contrary evidence exists, the width that evidence contradicts. It also spends the descriptor-and-pin move twice: once now and again after calibration.

**What would reverse the recommendation.** A sweep showing 16 is within noise of the best square `T` at both workload halves — in which case D lands immediately with a `Measured…V1` variant. Or a decision to widen the lowering past `Square tiles only`, which would change the object under discussion from one `u64` to a block shape and would need its own packet.

**Negative controls that would test it.** (i) Build a profile that omits the policy and prove no tiled alternative is offered, with no substitution of `contraction_region` and no clamp — the precedent's own negative-profile test, transposed. (ii) Perturb the `work_span` arm by deleting it and confirm the tiled plan silently scores `None` rather than failing the build, which is the hazard the wildcard creates; then remove the wildcard and confirm it becomes a build error. (iii) Drive the `M = 1` cell and confirm the waste ratio `grid_threads / work_items` is visible to the cost model, since `PredicatedCooperativeContraction` already carries both fields.

### Follow-up tickets required so nothing is left implicit

1. **Calibrate the contraction tile width under a beneficiary-named protocol** — the sweep in step 2 above. Blocks the width half of the offer ticket.
2. **Carry the contraction tile-width policy as a target-profile row** — the D carrier, dependent on 1. Must re-derive the descriptor and pin consequences on the merged tree rather than inheriting the derivation above.
3. **Amend `offer-the-tiled-contraction-alternative-in-physical-planning`** to split the width-independent work (the `work_span` arm, the wildcard removal, the output-binding widening) from the width-dependent offer, so the lane proceeds.
4. **Reconsider the `Square tiles only` lowering restriction** — the parent lane's remainder item 4, now with a measured motivation: the record attributes the 2.28x regression to a square block wasting fifteen of sixteen rows at `M = 1`.

### Corroboration and one surface finding, independently re-read

An independent location sweep was run and every claim below was then re-read in the file it names, at this base.

- **The explain vocabulary needs nothing new.** `crates/tiler-compiler/src/frontier.rs` already carries `TargetPolicyUndeclared { policy: &'static str }` on the public `#[non_exhaustive] enum StrategyDeclineCause`, documented with the wrapped `///` line `default width and not a substitution of another partition rule` (the full rendered sentence greps to zero there, so the short fragment is the anchor). It is keyed by a stable policy-family code rather than by the tree, so a contraction decline maps into the **existing** public variant. D's public cost is therefore the target-profile carrier alone, not a frontier surface as well.
- **The K-divisibility refusal is already landed and typed.** `crates/tiler-ir/src/schedule/error.rs` carries `ContractedTileNotDivisible { axis, contracted, tile }` with the stable code `cooperative-contraction-contracted-tile-not-divisible`. Whatever width is eventually chosen, a non-dividing contracted extent is already a typed refusal and never a pad.
- **Nothing in `tiler-compiler` names the tiled surface at all.** `grep -rn --include='*.rs' -E 'CooperativeContraction|BlockedWorkgroup|blocked_operand_tile|admit_exact_cooperative|admit_predicated_cooperative' crates/tiler-compiler/` returns zero lines, tests included. Option A is the true current state.
- **The evidence bound is narrower than even the record's prose suggests.** `spikes/scheduling/metal_contraction_vertical/README.md` states the retained claims reach no `contracted extent outside` `{16, 1024, 2048, 3072}`, and separates the hosts: `Correctness is measured on an Apple M4 Max`, performance on the M3 Pro.
- **`blocked_operand_tile(block, rounds)` is already a labelled draft public boundary** in `tiler-ir` and takes the width as a parameter. No production path in `crates/` hard-codes 16; the single production mention is the `Square tiles only` doc comment.

## Non-goals

Implementing the alternative — that is [`offer-the-tiled-contraction-alternative-in-physical-planning`](offer-the-tiled-contraction-alternative-in-physical-planning.md); changing the landed schedule, lowering, or emission; and declaring a width on any profile before the authority is decided.

## Closes when

Tom has accepted one authority for the tile width with provenance recorded, or has redirected the question, and the implementation ticket can name the accepted source without inventing one.
