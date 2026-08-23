---
id: thread-resolved-lowering-into-the-governed-spelling-path
title: Thread resolved lowering into the governed spelling path
status: in-progress
priority: p1
dependencies: [bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence]
related: [decide-whether-refinement-evidence-may-reach-a-physical-provider, emit-the-indirect-gather-on-metal]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, gather, frontier]
claimed_from: todo
assignee: worker-thread
lease_expires_at: 1787467109
---
## User-visible outcome

Physical planning spells a gather region using the proof the occurrence already carries, so `RegionVocabularyWall::GatherProofUnavailable` retires because the argument arrived — not because the check was relaxed.

## Why this exists

Filed 2026-08-22 from the refinement-seam packet. **The wall is a missing argument, not a missing boundary.** `spell_region` takes only a `&VerifiedTargetRequest` while the proof sits in `ResolvedLowering` in the same `plan_target` scope — the value exists at the right time and no seam carries it.

**Fact — this adds no public surface.** The packet verified item by item that every element it touches is `pub(crate)` or private: `enumerate_frontier`, `govern_spelling`, `spell_region`, `spell_output`, `verify_schedule_with_feasibility`, `verify_region_output_binding`, `gather_accesses_match`, `ResolvedLowering`. So it does not meet ADR 0075's reservation bar and is **not a Tom decision**. Re-derive that list yourself before relying on it.

**Fact — this strengthens verifier independence rather than costing it.** `verify_portfolio` in `crates/tiler-compiler/src/pipeline/verify.rs` already calls `resolve_lowering` itself. Threading planning's value in makes the verifier compare planning's retained proof against its **own** re-derivation. **Nothing borrows** — which matters, because the deliberate independence of `pipeline/verify.rs` is the property that makes verification meaningful, and a seam that let it borrow would retire that quietly.

**Three routes were eliminated before ranking, and the grounds are worth keeping.** Re-deriving the proof during planning conflates identities — the wall's own stated reason. A public proof constructor inverts deriver privacy. Dropping the proof and re-deriving at the schedule layer is tempting, since both current kinds *are* relation-derivable, but it depends on the kind set never growing, and the invocation-validation resolution is exactly a third kind that is **not** relation-derivable.

## Required work

- **Do not start until [`bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence`](bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence.md) lands.** Threading the value before the occupancy check exists would carry an unchecked proof further, which is the wrong order.
- Re-audit both Facts at your base with a per-Fact verdict, re-deriving the visibility list rather than inheriting it. **If any item turns out `pub`, stop and report** — that would make this a public-boundary change and therefore Tom's.
- Thread the already-derived `ResolvedLowering` into the spelling path **and** the proposal-verification path; land `physical::gather_region`; retire the wall.
- Perturb the subject separately for each behaviour and quote the failure text, including a control that a proof failing the occupancy check still refuses after the threading.
- State every identity domain that steps and every one that does not, derived on the merged tree.

## Non-goals

Widening `ImplementationContext`; the Metal emission, which depends on this; and re-opening the accepted data-dependent index surface.

## Closes when

A gather region is spelled from its own occurrence's proof, the wall is retired because the argument arrived, the verifier still re-derives independently, each behaviour is watched failing on its own subject, and the workspace gate is green.
