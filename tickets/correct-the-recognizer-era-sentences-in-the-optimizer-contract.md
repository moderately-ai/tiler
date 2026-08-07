---
id: correct-the-recognizer-era-sentences-in-the-optimizer-contract
title: Correct the recognizer era sentences in the optimizer contract
status: done
priority: p3
dependencies: []
related: [widen-the-strategy-recognizer-past-the-f32-wall]
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [docs, doc-drift]
---
## What is stale

`widen-the-strategy-recognizer-past-the-f32-wall` removed the `dtype-f32` gate on 2026-08-07: recognition now derives the program's arithmetic type from its values, a non-`f32` program reaches a selected `PlanAlternative`, and the refusal moved to the contract and the profile as their own typed causes.

Two sentences in `docs/compiler/optimizer.md` were falsified by it:

- **:197** — "two program-wide properties — at least one declared input, `f32` throughout".
- **:199** — "the program-wide `dtype-f32` check refuses it first".

Verify both at your own base before editing; line numbers move.

## Why it is a separate ticket

Found by the worker on [`establish-bf16-optimizer-legality`](establish-bf16-optimizer-legality.md), **in its own exclusive scope**, which could have edited them. It deliberately did not, on the ground that they are another ticket's debt and silently absorbing them would hide that the recognizer landing left a documentation obligation. That is the right call, and this ticket is the obligation made visible rather than a chore.

## What to state instead

Not merely deleting the clauses — say what the program-wide properties **now are**, and where the refusal now lives, so a reader learns the current shape rather than losing a sentence. The refusal did not disappear: a BF16 program under an `f32` contract is refused by the **contract**, program-scoped and before any target, under `compile.request.numerics.inapplicable`; a dtype the profile cannot dispatch is refused by the **profile**. The recognizer keeps two rules of its own — a width this build spells no body for, and two widths in one program.

## Check the rest of the document

Read `docs/compiler/optimizer.md` in full rather than patching two lines. The recognizer widening was a structural change and this document describes the stage it changed; two sentences being reported does not mean two are wrong. Report anything else you find, whether or not you fix it.

## Closes when

Both sentences state current truth, the document carries no other recognizer-era claim, and the refusal's new authorities are named rather than the old check merely removed.

## Outcome

**Both reported sentences were verified false at base `0ee647ee` by reading `crates/tiler-compiler/src/request.rs` rather than the summary, and both are corrected in place with a dated `*Corrected 2026-08-07*` note that states what the retired text said, in this document's existing convention.** `select_supported_strategy` (`:4566`) now checks `input-arity` and then calls `recognized_program_arithmetic` (`:4608`), which derives the program's one arithmetic type from its values and refuses under `dtype-recognized` or `dtype-uniform`; no `dtype-f32` rule exists.

The correction states, rather than deletes: the second program-wide property is now *one recognized arithmetic width throughout*, derived rather than fixed, over the two widths `recognized_arithmetic` (`:4641`) admits; the two width refusals join the refusal list as this stage's own; and the moved refusal's two new authorities are named with their public keys — `RequestError::NoApplicableNumericalContract` under `compile.request.numerics.inapplicable`, program-scoped and before any target, and `RequestError::DTypeNotDispatchable` under `compile.request.dtype.dispatch`, per target and resolving silence to disposition `Unknown` (`session.rs:1317`, `:1321`).

**Three further stale claims were found by reading the document in full, all in exclusive scope, all fixed.**

- The same paragraph at `:199` said `elementwise_family` "classifies exactly" the three `f32` keys. It is now keyed by the derived arithmetic and carries a `bf16` row of `tiler::add-bf16@1` and `tiler::multiply-bf16@1` (`request.rs:5147`). Corrected, with the vocabulary reason the `bf16` row is shorter.
- `:199` said reindex and broadcast are "among the nine `governed_index_access_capabilities` returns". The function returns twenty — thirteen named rows plus one per concatenate arity `2..=8` (`governed.rs:260`–`:473`). Corrected, with the derivation so the count is reproducible.
- The algebraic-portfolio paragraph's "Other dtypes … do not become executable merely because semantic IR can represent them" read as a present-tense claim that `f32` is the only width reaching a physical product. `bf16` has since built every layer that sentence requires, so a recognized `bf16` program is projected into `PointwiseBf16Expression` rather than refused. Amended to state the discharged instance and to keep the rule as a condition; the reassociation rules stay `f32`-only for the separate reason that no `bf16` family declares ordered associativity (`normalize.rs:1893`).

**One stale claim found is not this landing's debt and is corrected here only because it sits inside a paragraph this ticket rewrote; it needs its own follow-up.** `:199`'s "two-family list" of registered families the region vocabulary cannot spell is now three: `scope-the-concatenate-fusion-role-and-lowering` landed `tiler::concatenate-f32@1`'s fusion role and per-arity lowerings in `a86fddc2` on 2026-08-07, moving it into this class, and `UNPLANNED_OPERATIONS` (`policy.rs:1107`–`:1123`) records that the request boundary refuses the family under `operation-set` because no kernel construct writes a partitioned output. The count and the family's own gap are now stated; **no ticket owns admitting the concatenation into the region vocabulary**, and this document says so rather than implying the structural-families ticket covers it.

**Retained `dtype-f32` mentions are deliberate.** Three remain in `docs/compiler/optimizer.md`, each inside a dated correction describing what the retired gate was. `correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents` closes on `grep -rn 'dtype-f32' docs/` being empty, which that convention makes unreachable; the mechanical check needs replacing with one that distinguishes a live claim from a recorded correction.

## Outcome — delivered 2026-08-07 at `e16c5593`

All 738 lines read in full before editing, and every claim verified against the compiler source rather than the brief.

**A precision worth keeping: the count was not wrong.** "Two program-wide properties" is still exactly right — `select_supported_strategy` checks two. Only the *second property's content* was false. The correction says "at least one declared input, and **one recognized arithmetic width throughout**", names the two functions that derive it, and — the part that mattered — says **where the refusal went** rather than deleting the clause: `NoApplicableNumericalContract` under `compile.request.numerics.inapplicable`, program-scoped and before any target; `DTypeNotDispatchable` per target; and the two rules the stage kept for itself.

**Three further stale claims the full read caught**, none reported and none findable from the two named lines:

- The `elementwise_family` list said it "classifies exactly" three `f32` keys; it is now keyed by the derived arithmetic and has a `bf16` row. Corrected **with why that row is shorter** — no `silu-bf16` is registered and the BF16 node has no division or exponential.
- "the **nine** `governed_index_access_capabilities` returns" — it returns **twenty**, thirteen named rows plus one per concatenate arity. Corrected with the derivation so the count is reproducible rather than another number to go stale.
- "Other dtypes do not become executable merely because semantic IR can represent them" — false read as present tense, since `bf16` built every layer that sentence names as its condition. Amended to name the discharged instance while keeping the rule readable, and to record that the reassociation rules stay `f32`-only for a *separate* reason: no `bf16` family declares ordered associativity.

**One correction made deliberately outside its own debt, and flagged as such.** The "two registered families the region vocabulary cannot spell" is now **three** — `tiler::concatenate-f32@1` joined it when its fusion role and per-arity lowerings landed the same day. The worker corrected the count only because it framed a paragraph already being rewritten, on the ground that leaving a known-false count inside edited text would be worse, and named the cause rather than absorbing it. **No ticket owned admitting concatenate into the region vocabulary**; filed as [`admit-the-concatenate-family-into-the-scheduled-region-vocabulary`](admit-the-concatenate-family-into-the-scheduled-region-vocabulary.md). It also distinguished `tiler::slice-f32@1` as a different class — registered and unplanned but holding no governed lowering, an uninstalled-provider case — and made no claim about it.

**Four claims checked and found still true** were left alone and named, so the next reader knows they were examined rather than skipped.

### The finding that repaired another ticket

Three `dtype-f32` mentions remain in this document, each **inside a dated correction describing the retired gate** — which is the document's own established convention, followed by its 2026-08-04, -08-05 and -08-06 corrections. The sibling ticket [`correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents`](correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents.md) closed on `grep -rn 'dtype-f32' docs/` being **empty**, which that convention makes **unsatisfiable** — a closing condition demanding the repository forget what it corrected, and the mirror of the unfireable check: one that can never say *yes*.

The coordinator replaced it: every remaining mention must be **inside a dated correction or gone**, classified per hit rather than counted, because a count cannot tell a live claim from a recorded one — which is precisely how the original went wrong.

**Delta rule confirmed against the merge's own file list:** two files, `docs/compiler/optimizer.md` and this ticket, neither under the build-configuration set, so it carries the latest green gate with `tkt lint` rerun.
