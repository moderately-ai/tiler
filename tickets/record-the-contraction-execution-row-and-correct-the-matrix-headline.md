---
id: record-the-contraction-execution-row-and-correct-the-matrix-headline
title: Record the contraction execution row and correct the matrix headline
status: review
priority: p2
dependencies: []
related: [decide-whether-the-l3-ladder-rung-moves-on-the-dispatched-contraction-cell, integrate-the-contraction-vertical-into-the-runtime, publish-an-l3-contraction-cell-through-the-accepted-route, raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-navigation-2
lease_expires_at: 1786030628
---
## The work (maturity audit 2026-08-06; integrate-the-contraction-vertical-into-the-runtime verified done since 2026-08-02)

`docs/roadmap.md:482`'s rung cell gains the execution row in the two sibling rows' exact idiom ("R7 bounded to checked target-neutral layers and one prototype execution row"), replacing the "no execution row / nothing dispatched" span with the 2026-08-02 measurement and its boundary (a 2×3×3 toy, not an L3 cell; the six L3 cells now compile after the grid-axis move but have not dispatched). `roadmap.md:435`'s "only backend-executed profile" sentence rewrites to cover the contraction AND the second executed contract (the FLUSH_AND_REASSOCIATE_F32 run correctness-and-testing.md:209 records). `roadmap.md:406/:408`'s ladder clauses — which contradict the same document's own row — repair. Explicit non-goal: do not promote L3.

## Closes when

The rung, headline, and ladder agree with each other and with the ticket outcomes they cite.

## Outcome (2026-08-06)

**The audit's own dispatch premise was refuted before any edit was written, and that is the finding rather than a detail.** This ticket's body states that "the six L3 cells now compile after the grid-axis move but have not dispatched", and instructs that nothing written may claim an L3 cell dispatched. One had. [`publish-an-l3-contraction-cell-through-the-accepted-route`](publish-an-l3-contraction-cell-through-the-accepted-route.md) is `done`: on 2026-08-05 `w_decode_kv` at `1 x 1024 x 1024` was published as a second contraction member, dispatched through the accepted route, and the SHA-256 of its **executed** result bytes — `79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f` — matched the retained `direct` value in `spikes/scheduling/metal_contraction_vertical/results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/workload.tsv`. Verified three ways rather than taken from the ticket body: the retained digest is line 2 of that `workload.tsv`; `prototypes/serial-sum-run/src/proof.rs` pins it as `L3_CELL_RESULT_SHA256` and compares it against `result_digest(&observed)`, the digest of *readback* bits, separately from the embedded reference expectation; and `crates/tiler-build/src/metal_plan.rs:1716`'s `the_measured_grid_axis_admits_every_l3_contraction_cell` holds the compile-phase half against the row moved to `268_435_456` at `crates/tiler-build/src/metal_declaration.rs:226`.

**No navigation ledger recorded that run anywhere.** `rg -l publish-an-l3-contraction-cell-through-the-accepted-route tickets/ docs/` returns four paths, none of them under a navigation scope. This is the `AGENTS.md` failure mode where delivered capability reaches no row because the landing's worker could not edit it — and it made the audit's premise stale in the same direction as the text the audit was correcting.

**What was written, therefore, is neither side of the contradiction: the verified fact, with the rung judgement left open.** The delivery is recorded in the ladder's prose and in the matrix row; the L3 row's own `Maturity today` cell is **untouched**, and [`decide-whether-the-l3-ladder-rung-moves-on-the-dispatched-contraction-cell`](decide-whether-the-l3-ladder-rung-moves-on-the-dispatched-contraction-cell.md) is filed at `todo` in `contracts/navigation` carrying the three arguments against a naive promotion (one cell of six under the unselected `direct` realization; every prior rung fired on the design-rung reading, so moving L3 on capability creates two readings of one column; the `Maturity today` column is what a reader consults before claiming the ladder's state).

### Sites edited

- **`docs/roadmap.md`, contraction row rung cell.** `no execution row exists` replaced by the two sibling rows' idiom, at the count the evidence supports: `R7 bounded to checked target-neutral layers and two prototype execution rows`.
- **`docs/roadmap.md`, contraction row evidence cell.** The `no execution row / nothing here dispatched on a device` span is superseded in tense with its derivation quoted, and both measurements land beneath it with their boundaries: the 2026-08-02 `td,od->to` toy at `activations[2, 3] x weights[2, 3] -> projected[2, 2]` under `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` (verified at `prototypes/serial-sum-compile/src/main.rs:309`, the one `compile_under` both contraction members use), five operand cases bit-compared, two of them discriminating, with the `SectionDigestMismatch { section: 2 }` refusal; and the 2026-08-05 L3 cell with its executed-bytes digest, its `environment.tsv` host check, and its watched-refusing perturbation. Both state extent, host row, realization, and what they do not cover.
- **`docs/roadmap.md`, contraction row trigger cell.** The stale `R7 needs a dispatched device comparison, which integrate-... owns` is replaced by what actually widens the bound — the remaining five cells, a second host row, the `tiled` realization — with the first two named as **unowned gaps** rather than schedule.
- **`docs/roadmap.md:435`, matrix headline.** `the only backend-executed profile` becomes two profiles under two contracts: the four-operation F32 prototype and the strict tensor contraction under `FLUSH_SUBNORMALS_TO_ZERO_F32`, and the three reduction strategies at `1x4` under `FLUSH_AND_REASSOCIATE_F32`, citing [correctness and testing](../docs/correctness-and-testing.md) (line 209 read, not edited — `contracts/numerics` was held by a live claim). The clause listing `Reindex` and `Broadcast` under semantic/reference admission was corrected in the same sentence: both are at R6, translation-bounded, and leaving that in a sentence being rewritten would have restated a claim the structural row already refutes.
- **`docs/roadmap.md:406`, ladder clause.** The R4 / no-fusion-role / no-lowering / only-the-residual-add-is-executable claims are superseded in tense against this document's own contraction row, and the paragraph's surviving point is sharpened to the composed-capability claim it was reaching for.
- **`docs/roadmap.md:408`, ladder clauses.** L3 is removed from the record-only rung list without being promoted; its clause states both deliveries and says explicitly that the cell is unmoved and why; and the closing `no part of the workload compiles, dispatches, or executes, no operation family moved a rung` is corrected with the narrower claim that survives.
- **`docs/status.md:25`, device-execution bullet.** `three runs rather than one` becomes four, and a fourth sub-bullet records the L3 cell. The contraction sub-bullet's closing `the cross-check was not taken and is recorded as unavailable` is corrected in tense — it was true of that run and stopped being true three days later — because the brief required the roadmap edits to agree with these sub-bullets, and agreeing with a stale one would have propagated it.

### Scope declaration added

`project/tickets` was added to this ticket's `shared_scopes`, which it had left empty. It is required and not optional: the work writes this ticket's own body and files a sibling under `tickets/**`, and `tkt guard --base 0bbb4bae` failed with `branch touched scope(s) it did not declare` until it was declared. It is bookkeeping rather than product scope — a shared scope collides as a non-failing WARN by config — and the same declaration was written into the filed ticket for the same reason.

### Checks

`tkt lint` (ok), `git diff --check` (clean), table integrity on the edited row verified as 5 pipes matching siblings 463/464, every added link target verified to exist, and each cited symbol (`the_measured_grid_axis_admits_every_l3_contraction_cell`, `RIGHT_SEED_MASK`, `268_435_456`) grepped against `crates/` and `prototypes/` beside a control string that returned zero files. No gate input was touched.
