---
id: pair-the-elementwise-bounds-proof-with-the-parametric-broadcast-wall
title: Pair the elementwise bounds proof with the parametric-broadcast wall it depends on
status: todo
priority: p2
dependencies: []
related: [accept-the-parametric-broadcast-access-surface, admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, fail-closed, bounds-proof, latent]
---
## User-visible outcome

Lifting the physical planner's parametric-broadcast wall cannot silently emit a domain-sized bounds proof for a widening read. Either the proof derives the addressed range from the relation, or the wall's removal is a build error at the site that must change with it.

## Why this exists

Found 2026-08-23 by the coordinator during the post-chain read-only audit. **This is latent, not live** — no wrong proof is reachable today. It is filed because its safety rests entirely on a wall that a planned chain exists to remove, and the coupling is stated nowhere.

**Fact — `addressed_elements` names two relations and answers the region domain for every other.** In `crates/tiler-compiler/src/physical.rs`, anchor `fn addressed_elements`, the match names `ReindexBijection` and `BroadcastReplication` and closes with `_ => elements`. Its single caller, anchor `element_count: addressed_elements(map, elements)`, builds each read's `BoundsProofKind::LinearRange` in `elementwise_region`. `LogicalAccess` has **12** variants, sized from the type at `pub enum LogicalAccess` in `crates/tiler-ir/src/schedule/model.rs`.

**Fact — the wildcard is compiler-mandated, not a defeated build trap.** `LogicalAccess` carries `#[non_exhaustive]` and is defined in `tiler-ir`; `physical.rs` is in `tiler-compiler`, so an exhaustive match is impossible from there. This is the same reason the two `| _ =>` arms at anchors `fn access_domain_shape` (`crates/tiler-compiler/src/frontier.rs`) and `LogicalAccess::ParametricBroadcast { operand_shape, .. } => operand_shape` (`crates/tiler-compiler/src/request/normal_form.rs`) are correct as written. **Do not "fix" any of the three by making them exhaustive; it will not compile.** Recorded because the coordinator's first reading of this audit called them defeated traps and was wrong.

**Fact — today the dangerous variant cannot arrive.** `ParametricBroadcast` is refused before any pointwise or epilogue region is spelled: three sites in `spell_output` and its fold arms return `RegionVocabularyWall::ParametricBroadcast`, at the anchor `matches!(map, LogicalAccess::ParametricBroadcast { .. })`. So `addressed_elements` never sees it, and `_ => elements` is safe **by that wall alone**.

**Inference — when the wall lifts, the failure is silent and is exactly what the function exists to prevent.** `addressed_elements`'s own doc states that deriving the range from the relation is what stops a region from *"binding a widened read against a domain-sized proof"* — quoted as a line-safe fragment, because the source `///` comment wraps mid-sentence and the full sentence greps to zero. A parametric broadcast is a widening read whose operand is smaller than the domain, so it would take `_ => elements` and receive a domain-sized `LinearRange` — a proof that overstates the addressed range. Because the arm is a mandated wildcard, nothing fails to compile and no existing test covers a population that cannot yet be built.

**A correct derivation already exists to copy.** `crates/tiler-compiler/src/request/normal_form.rs` handles the same relation at the anchor `A sourced operand answers only when every extent is already a literal`, returning the operand's static element count and declining when any extent is unbound — deliberately not folding `ExtentSources::determined`, because that would size the read from a value the request must not specialize.

## Required work

- Re-audit every Fact above at your base with a per-Fact verdict before editing. The wall sites and the helper are in one file that moves often.
- Decide **by reading** between two acceptable outcomes. **Both are fail-closed; a domain-sized proof for a widening read is not.**
  - Give `addressed_elements` a `ParametricBroadcast` arm now, deriving the operand range as `normal_form.rs` already does, so the helper is correct before the wall moves.
  - Or bind the two together so the wall cannot be removed alone — for example by routing both through one function that must answer for the relation, so deleting the refusal is a build error at the proof.
- Whichever is chosen, leave a durable statement of the coupling at both anchors. The defect is that the helper's safety is invisible from the wall and the wall's necessity is invisible from the helper.
- **Do not lift the wall in this ticket.** Admitting parametric broadcast into physical spelling is a public-boundary question with its own owner; this ticket makes that lift safe, it does not perform it.

## Evidence

- Construct an elementwise region whose read carries `ParametricBroadcast` and show the proof it receives. Today that requires bypassing the wall; say exactly how the subject was built so the construction is reproducible.
- Perturb the subject, not the assertion: with the fix in place, show a widening parametric read receiving the operand range, and show a `LinearIdentity` read still receiving the domain — those are separate properties and must fail separately.
- State what it would take for any new check to say *no*, and confirm that case is reachable. A check that can only pass while the wall stands has not been demonstrated.
- **State whether any identity value moves.** A bounds proof is encoded into the scheduled-region identity, so a changed `element_count` on a reachable region **would** move pins. Expected today: none, because no such region is constructible. Re-derive rather than assume, and stop and report if one moves.

## Non-goals

Lifting the parametric-broadcast wall. Making any of the three `LogicalAccess` wildcards exhaustive, which cannot compile from `tiler-compiler`. Changing `normal_form.rs`'s derivation, which is the reference here rather than the subject.

## Closes when

A widening parametric-broadcast read cannot receive a domain-sized bounds proof — either because the helper derives its range, or because removing the wall fails the build at the helper — with the coupling stated at both anchors and no identity value moved.
