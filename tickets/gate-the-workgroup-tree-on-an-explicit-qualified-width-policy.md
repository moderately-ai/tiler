---
id: gate-the-workgroup-tree-on-an-explicit-qualified-width-policy
title: Gate the workgroup tree on an explicit qualified width policy
status: in-progress
priority: p1
dependencies: []
related: [carry-the-tree-participant-cap-as-a-target-profile-row, cap-the-tree-reduction-participants-at-the-measured-256, pin-the-local-memory-refusal-band-the-tree-cap-opened]
scopes: [implementation/compiler, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, correctness, public-boundary]
claimed_from: todo
assignee: worker-gate-workgroup-tree
lease_expires_at: 1786636350
---
## Decision accepted 2026-08-11

Tom delegated the choice to the coordinator in this conversation after reviewing the ranked correctness, strictness, Tiler-runtime-performance, and maintainability trade-offs. The accepted narrow first pass is an **explicit closed tree-width policy**, not a configurable numeric target cost row.

The accepted semantic surface is one required policy choice for every target profile that offers the single-workgroup tree strategy:

- `MeasuredNearestCap256V1`: the existing nearest-admissible-width rule around the fixed internal value `256`, with the existing narrower tie break;
- no omitted/default case and no arbitrary numeric cap;
- a profile that does not declare an accepted policy makes the tree alternative unavailable with a typed, explainable reason;
- the optimizer may still choose another alternative it considered in the same target-independent portfolio. That is ordinary plan selection, not a retry, backend switch, or silent fallback;
- no clamp, balanced substitution, inherited `256`, or retry after prepared-kernel refusal is permitted.

The Rust spelling may follow the owning target-profile vocabulary, but it must preserve that exact closed one-variant meaning. If source-first implementation reading exposes two materially different public carriers, stop for review rather than choosing by convenience.

## Source-first decision audit at `e33811e4`

**Fact — the current rule is global.** `crates/tiler-compiler/src/physical.rs`, symbols `MEASURED_TREE_PARTICIPANT_CAP`, `capped_tree_partition`, and `single_workgroup_tree_region`: the selector takes only `contributors`; its production caller does not obtain a target-owned width policy.

**Fact — the fixed value is part of the present safety argument.** The `capped_tree_partition` documentation anchors `That ceiling is what keeps the rule a preference rather than a feasibility decision`; the literal `256` bounds its above-cap result to `509`, below `MAX_COOPERATIVE_PARTICIPANTS`.

**Measurement — an arbitrary cap breaks that argument.** The superseded ticket's exhaustive temporary parameterization over contributor counts `0..40_000` found that cap `0` produces one-participant results and cap `4_096` withdraws `9_936` tree cases through the schedule limit. A target's authoritative workgroup width is a `PreparedKernelPreflight` query, so it cannot validate a numeric cost row when the compile profile is declared.

**Measurement — no replacement policy is supported.** The retained shape-aware, interaction, and target-private-table/signature studies all rejected their frozen support bars and left production unchanged. The cross-record audit reports only `stable-named-subsets-only`, not a portable width policy.

**Inference — exact qualification is the honest narrow boundary.** One bounded policy has evidence on the current Apple9 measurement row. Allowing only an explicit declaration preserves that evidence class and makes a future target fail closed instead of silently inheriting it.

## Required implementation

1. Add the closed, typed target-profile policy declaration and reader. The policy affects planning and must be represented in canonical target-profile identity with measured-source provenance; rederive the owning domain/version and every transitive pin rather than assuming which must move.
2. Declare `MeasuredNearestCap256V1` on `BoundMetalCompileDeclaration::first_macos_apple9` from the retained 2026-08-07 measurement source.
3. Require an explicit accepted policy before `single_workgroup_tree_region` offers the tree. Add a typed `WorkgroupTreeUnavailable` case and preserve it in frontier/explanation evidence.
4. Update every test profile intentionally exercising the tree to declare the policy. Add a negative profile that omits it and proves no tree is offered, with no substitution of `governed_partition` or the global cap.
5. Keep `256` private and fixed. Do not add a public `u64`, `Option`, default, clamp, or retry path.
6. Correct the core documentation anchor `second target profile should carry its own row`: a future profile must carry an explicitly qualified policy, not necessarily a numeric row.
7. Re-read all target-profile builders, canonical encoders and length mirrors, selection explanations, construction sites, and consumers. Add any scopes the real population requires before editing them.

## Load-bearing evidence

Perturb the policy declaration, not the assertion:

- remove the Apple9 declaration and show its tree subject fails with the typed policy-unavailable reason;
- omit the policy from a second profile and show neither `256` nor balanced selection appears;
- change the declared policy identity/tag and show the canonical profile identity/pins move;
- bypass the gate and show the unchanged negative assertion fails;
- prove the existing qualified Apple9 partition census remains byte-for-byte/identity-equivalent wherever the chosen schedule is unchanged.

Run the affected package checks, nextest plus doctests, Clippy and rustdoc with warnings denied, citations, ticket lint, exact-base diff check and guard, then exact-tip `make full` because production crates change.

## Non-goals

No retuning, new timing, configurable cap, balanced fallback, cross-backend retry, runtime JIT choice, or general multi-width enumeration. [`enumerate-tree-width-alternatives-before-target-cost-ranking`](enumerate-tree-width-alternatives-before-target-cost-ranking.md) owns the general architecture and its planning-cost evidence.

## Closes when

The current Apple9 profile explicitly carries the fixed qualified policy; omission is a typed tree-strategy decline; all profile, identity, explanation, and test consumers are complete; no silent selection path remains; and independent review confirms the policy gate is load-bearing.
