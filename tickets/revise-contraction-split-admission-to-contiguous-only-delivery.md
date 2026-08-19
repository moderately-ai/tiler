---
id: revise-contraction-split-admission-to-contiguous-only-delivery
title: Revise contraction split admission to contiguous-only delivery
status: done
priority: p2
dependencies: [decide-the-algebraic-capability-authority-for-contraction-splits]
related: [admit-reassociated-contraction-schedule-alternatives, decide-the-semantic-order-contract-for-relaxed-contractions]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

[`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) is revised to match the accepted successor contract: contiguous membership is the only reachable delivery, the lane-strided alternative moves behind its future-generation trigger, and the required refusal vocabulary names the algebraic and numerical causes separately.

## Why this exists

The 2026-08-18 acceptance in [`decide-the-semantic-order-contract-for-relaxed-contractions`](decide-the-semantic-order-contract-for-relaxed-contractions.md) chose the reassociation-only successor and its downstream item (5) requires the admission ticket to be revised so contiguous membership is the only reachable delivery, but no ticket carried that revision. The reopened algebraic-authority packet additionally fixed the refusal contract the revision must inherit. The admission ticket's current `User-visible outcome`, `Required delivery`, and `Closes when` still demand both alternatives ("Both alternatives exist"), which is unsatisfiable under the accepted contract because the successor descriptor's permutation maximum is `unsupported`.

## Required revision

- Reduce the deliverable to the contiguous split; move lane-strided admission behind its trigger — an accepted fold-commutativity declaration in a future successor key generation plus independently resolved permutation permission — and record that trigger in the admission ticket, which does not yet state it (*corrected by independent review 2026-08-18: the trigger is recorded in the accepted semantic packet's downstream item (6) and the reopened authority packet, not in the admission ticket*). Keep the preserved attempt and the membership-vocabulary decision as evidence.
- Carry the reopened authority packet's verifier contract: descriptor decode plus effective-profile resolution as the two-fact join; a new appended algebraic `StrategyDeclineCause` variant with a stable reason key naming the missing dimension, distinct from `NumericalPermissionRefused`; lane-strided refused algebraically, not numerically; provider-output recheck before frontier admission.
- Update `Closes when` so it no longer requires the lane-strided plan, while retaining the contiguous-plan bit-reproduction obligation on the eight-case corpus and the witness/explanation membership-projection repair from the 2026-08-17 review stop.

## Obligations inherited from the replacement migration's independent review — 2026-08-19

The ADR 0112 landing (`e61fbc60`, merged) shipped reserved vocabulary this revision's implementation graph must make live and tested, per review findings 4 and 5 at that commit; name these in the revised admission ticket so they are not rediscovered:

- **`StrategyDeclineCause::AlgebraicCapabilityUnsupported` has no construction site at the merged base.** Its 0x06 encoding, `algebraic-capability-unsupported` reason, and `CapabilityResolution` explain routing are landed but unreachable. The split-admission implementation is what constructs it (lane-strided membership refused algebraically); its perturbations — including proof that the algebraic and numerical sources report distinctly and never collapse — belong to that implementation, with failure text quoted.
- **The witness's regular-split branch is implemented but unreachable and untested** (`partitioned_chain_nodes`, `MalformedPartition`, `AmbiguousRealization`, and the split-combiner staging refusals in `crates/tiler-ir/src/program/contraction_witness.rs`). A construction bug there would surface only as a *different legal tree* — exactly the class the witness exists to pin — so split-path witness tests, including each named refusal, must land before any split plan consumes that code.
- Minor, same file: the evaluator's witness-`K` revalidation reuses `Tree(RootCoverage)` off-label for a contributor-count disagreement; when the split path gains its tests, consider a typed mismatch variant rather than the borrowed spelling.

## Closes when

The admission ticket's outcome, delivery, refusal vocabulary, and closing conditions are consistent with the accepted reassociation-only contract and the accepted algebraic authority, with the lane-strided remainder explicitly parked behind its trigger rather than silently dropped, and with the three review-inherited obligations above carried into the revised ticket's own delivery requirements.

## Outcome — 2026-08-19

Delivered as `a673ef9e` on `tkt/revise-contraction-split-admission-to-contiguous-only-delivery` from exact base `01e4ececd4e0b4064c8eddb48fc1acbc2e78e3ff`, a ticket-only change touching `tickets/admit-reassociated-contraction-schedule-alternatives.md` alone. No crate, doc, or spike file moved.

### Per-Fact verdict, delivered first

The admission ticket's eighteen load-bearing claims were re-read at this base before any edit. Twelve verified, six were false or stale, and the table with each file read and the evidence is in the revised ticket's `Contract revision — 2026-08-19 at exact base 01e4ecec` section. The false ones: the two-plan `User-visible outcome`, the two-alternative `Required delivery` bullet, the "Both alternatives exist" `Closes when` (all three unsatisfiable because the successor's permutation maximum is operation-owned `unsupported`); the `tiler.schedule.v6\0` live-domain claim (now `tiler.schedule.v7`); the "eight-case production corpus belongs in `tiler-conformance`" placement claim (it is a `tiler-reference` integration test); and the "`tiler::strict-tensor-contraction-f32@1` currently declares… both order-permission facts as `false`" claim (the key is retired and the two fact constants no longer exist under `crates/`). The frontmatter title was false in the same way as the outcome and was changed. Each is repaired by dated correction with its replacement stated, never silently restated.

Two verified Facts were also *sharpened* rather than left as found. The witness-census finding said the hand-sized aggregation "remains at six and omits the new topology"; at this base `ReductionTopology` already states seven variants, so the census under-covers the vocabulary before `CooperativeContractionSplit` is added at all. And the `split_family` Fact survives the key migration precisely because `ScalarProgram::StrictTensorContraction` is schedule vocabulary, not the semantic `OpKey` ADR 0112 moved — a distinction a reader could otherwise mistake for staleness and "repair" wrongly.

### What the revision changed, and why

- `User-visible outcome`, `Required delivery`, `Non-goals`, and `Closes when` rewritten to the single contiguous split. `Closes when` is now six numbered conditions; it retains the contiguous-plan bit-reproduction obligation on the eight-case corpus verbatim and the 2026-08-17 review stop's witness/explanation membership-projection repair, and it states explicitly that lane-strided delivery is not a closing condition.
- The lane-strided trigger is now stated **in** the admission ticket — a future successor key generation spelling `permutation: permission-gated`, *plus* independently resolved permutation permission, neither sufficient alone — with its reversal evidence. The preserved attempt `648a372f8cbb306df43a4edfc4e14a6211cac7b1` and the accepted membership vocabulary are referenced as retained evidence and explicitly not deleted: `LaneStrided` stays the named spelling of the refused population, which is what lets the refusal name a dimension.
- The accepted six-step verifier contract is carried: descriptor decode plus effective-profile resolution as the two-fact join, the algebraic decline cause with its stable reason key and distinct explain stage, lane-strided refused algebraically before any permutation-permission check, and the provider-output recheck before frontier admission.
- The three review-inherited obligations are folded in as obligations A, B, and C, each tied to a closing condition.
- Scheduling metadata: `status` `blocked` → `todo` (all six dependencies are `done`; nothing blocks it), `replace-the-standard-contraction-key-with-the-accepted-successor` added as a dependency, and `implementation/reference` added to `scopes` because both the eight-case corpus and obligation C's site live in `crates/tiler-reference`.

### Correction to this ticket's own framing

Obligation 3 above records the `Tree(RootCoverage)` off-label reuse as "Minor, same file" — that is, in `crates/tiler-ir/src/program/contraction_witness.rs` alongside obligation 2. **It is not there.** `RootCoverage` occurs at four sites in the tree and none is that file: the variant is declared in `crates/tiler-ir/src/schedule/contraction_topology.rs "The root does not cover the contributor interval"` and constructed off-label in the reference evaluator at `crates/tiler-reference/src/contraction/topology.rs "Witness revalidation against this occurrence"`. Reproduce with `rg -n 'RootCoverage' crates`. The revised admission ticket carries the corrected location as obligation C with the mislocation named, so a worker following the brief does not search the wrong file and conclude the finding was already fixed.

### Gates

Run in this worktree at the committed tree: `git diff --check` clean; `./check-citations.sh` green (1207 pinned citations and 6554 local links resolve, with every per-form population floor met); `tkt lint` reports `ok: no problems found`; `tkt guard tkt/revise-contraction-split-admission-to-contiguous-only-delivery --format json` reports no scope escape. No cargo command was run or needed — the delta touches `tickets/` only, which is outside the gated set in AGENTS.md's delta rule, so the previous green gate carries and `tkt lint` plus `make citations` are the required reruns.

All 44 anchor citations written into the revised ticket were resolved against the file each names before the commit, and the resolver was perturbed with a deliberately absent anchor (`pub const fn contributor_membership` against `crates/tiler-ir/src/schedule/witness.rs`) to confirm it reports `ANCHOR FAILS` rather than passing everything.

### Not verified

Whether the `implementation/conformance` scope is still owed. The eight-case corpus is a `tiler-reference` test, and `crates/tiler-conformance` holds no contraction corpus at this base; the scope is retained on the possibility that a device-executed conformance row is owed, and the revised ticket tells a worker to drop it rather than invent work for it if none is.
