---
id: carry-the-contraction-tile-width-policy-as-a-target-profile-row
title: Carry the contraction tile-width policy as a target-profile row
status: todo
priority: p2
dependencies: [calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol]
related: [decide-the-contraction-tile-width-authority]
scopes: [implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, public-boundary, identity, needs-tom]
---
## User-visible outcome

A target profile declares *which contraction tile-width rule* applies, in the shape the workgroup-tree precedent already established — the profile names the rule, the repository owns the number — so planning can offer a tiled alternative without defaulting a width on an unmeasured target.

## Why this exists

Filed 2026-08-22 by the coordinator from the tile-width authority packet. **It exists as a graph node so the width question is never treated as implementation detail of the offer ticket.**

**Fact — the precedent is a policy tag, not a numeric row, and the coordinator's earlier framing of this as "either a profile row carrying a width, or a named constant" was a false dichotomy.** `crates/tiler-compiler/src/target/rows.rs` declares `pub enum WorkgroupTreeWidthPolicy` with a single variant `MeasuredNearestCap256V1` and a stable governed key `workgroup-tree-width.measured-nearest-cap-256.v1` — it names a *rule*. The number stays private: `crates/tiler-compiler/src/physical.rs` documents `MEASURED_TREE_PARTICIPANT_CAP` with the anchor `A numeric row is not required`. Both verified by the coordinator at `f2c974a8`. The precedent is **both at once**, and a profile row carrying a `u64` width was explicitly declined by it.

**Fact — the explain vocabulary needs nothing new.** `crates/tiler-compiler/src/frontier.rs` already carries `TargetPolicyUndeclared { policy }` on the public `#[non_exhaustive] enum StrategyDeclineCause`, keyed by policy family. Reported by the tile-width lane and re-read by it; re-derive at your base.

**Reported, unverified by the coordinator — the identity consequence is conditional.** The lane reports that a silence-as-absence family keeps `declaration.v11`/`descriptor.v11`, but that *declaring* on `first_macos_apple9` moves that profile's bytes and every downstream pin, and needs its own `PopulationRows`. **Re-derive this on the merged tree rather than inheriting it.**

## The one question that may be Tom's

Whether adding a **new policy family** to a target profile falls inside the delegation Tom gave on 2026-08-11 for the identical tree decision, or is a fresh public-boundary decision. The tile-width lane's judgement was that ADR 0075 hits no always-ask category here — `pub mod target` exists, no new trait, no breaking signature, no visibility promotion — but it flagged that judgement as **its own inference, not a recorded rule**, and the parent ticket carried a `needs-tom` tag. Settle this by reading the 2026-08-11 provenance before landing anything public. If the provenance does not settle it, **stop and ask Tom one concrete question** rather than assuming either way.

## Required work

- Do not start until [`calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol`](calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol.md) reports. Without it there is no evidence a rule can cite, and ADR 0113 bars the existing record from ever supplying one.
- Re-audit every Fact at your base and report a per-Fact verdict.
- Follow the precedent's shape exactly: a closed policy tag with no omitted or default case, the number private. Silence must not become a default.
- Derive every identity, pin, and descriptor consequence on the merged tree and state them; do not copy the conditional above.
- Perturb each new refusal separately with quoted failure text, including the undeclared-policy path.

## Non-goals

Choosing the width — that is the calibration ticket's measurement; offering the alternative in planning; and any change that lets an unmeasured target infer a width.

## Closes when

A profile declares which contraction tile-width rule applies, an undeclared profile is refused by name rather than defaulted, every identity consequence is derived on the merged tree, the Tom-authority question is settled by provenance or asked, and each refusal has been watched firing.
