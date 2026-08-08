---
id: complete-the-runtime-product-bullet-s-device-execution-retention-claim
title: Complete the runtime product bullet s device execution retention claim
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786188969
---

`docs/status.md`'s runtime-product bullet says device-execution code "is retained in `prototypes/serial-sum-run`". That was complete when written and is not now.

## Facts

**Reported by the worker that refreshed the execution-row census, not coordinator-verified — check before editing.** `crates/tiler-conformance` now holds device-dispatching entry points across three verticals, so the prototype is no longer the only place device execution lives. The bullet is **incomplete rather than false**: the prototype does still retain such code.

**Coordinator-verified:** that worker deliberately left this alone as outside its census, and named it for its own ticket rather than folding it in.

## What closes this

The bullet stating where device execution lives now, without implying the prototype was replaced — it was **retained alongside**, and a sibling established that one vertical *re-homes* the prototype's corpus (three reduction classes × two plan roles × five operand cases reproduces its thirty) while two others are independent. Getting that relation wrong in either direction is the failure.

**Prefer naming the construction over counting.** A sibling replaced a seven-row ledger restatement in this same file with a reference to its owner, on measured evidence: the owner was current on all three rows where the restatement was stale, same tree, same day, and a hand patch two days earlier had **held two days**. If an enumeration in `tiler-conformance` owns this, name it.

**Treatment:** true when written → dated beside. Decide with `git show <commit>:<file>`. Repository practice, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim **stays greppable**; say inline that a later hit lands inside your note.

**Preserve `git log -S` anchors.** A sibling achieved **14 insertions, 0 deletions** across three documents so every pre-existing byte was unchanged, then ran a ten-word overlap scan of its own inserted lines against the pre-edit file and found **eight** accidental near-quotations that would have created new collisions — including one reproducing its own ticket's anchor. Meet that standard and disclose any occurrence count that moves.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`** — anchors fail as absence four ways: a line break inside them, an emphasis or backtick marker the source lacks, unescaped brackets read as a character class, and a quoted sentence that never appeared contiguously. This file spells one crossing "between 50 and 51 operations", so an anchor written `50/51` returns 0.

**Do not edit `crates/**`** — read it to describe it correctly. Check the neighbouring claims and **name the count**; three sweeps of this file this week each found more than they were sent for, including a manifest schema version stale by a full step that was labelled coordinator-verified.

## Fact audit, worker, 2026-08-08 at base `aae3da24`

- **Verified.** `crates/tiler-conformance` holds device-dispatching entry points across three verticals. `serial_sum` dispatches at five `#[test]` entry points, `bf16_vertical` at two, `envelope` at three (one of them `#[ignore]`d, `the_prefill_cells_carry_their_retained_digests`).
- **Verified.** The bullet is incomplete rather than false: `prototypes/serial-sum-run/src/proof.rs` still encodes, commits, waits, and inspects the terminal status. Retained alongside, not replaced.
- **Verified.** One vertical re-homes the prototype's corpus and the arithmetic is exact. `envelope`'s `REDUCTION_CLASSES` (3) by `PLAN_ROLES` (2) by `publication::proof`'s `OPERAND_CASES` (5) is thirty; `prototypes/serial-sum-compile/src/sidecar.rs` carries an identical five-element `OPERAND_CASES` and `prototypes/serial-sum-run/src/proof.rs` an identical three and two. `serial_sum` and `bf16_vertical` share no corpus with it.
- **Imprecise — "ten dispatching entry points".** Ten is right for dispatching `#[test]` functions only when the `#[ignore]`d run is counted. There are also ten `require_or_report` call sites, but that is a *different* ten reached by different arithmetic: it includes `measured_offer`, which observes a device and dispatches nothing, and collapses `envelope`'s two contraction runs into the one call inside their shared `route_and_compare` helper. Six plus two plus two, against five plus two plus three. The two readings coinciding on ten is a coincidence, so the repair names the construction and carries no count.
- **False as an implied exhaustive census — a third device-execution home was named by neither the ticket nor the brief.** `git grep -l dispatch_threads HEAD -- crates/ prototypes/` returns three files, and the third is `prototypes/candle-metal-adapter/src/adapter.rs`, which encodes, commits, waits, and reads the terminal status. It was admitted 2026-08-01. The same command at `3e291474` returns one file, which is what makes the retired clause true when written.

## Outcome

`docs/status.md`'s runtime-product bullet keeps its original sentence byte-for-byte and gains a dated 2026-08-08 note beside it: two insertions, zero deletions. The note names `crate::dispatch` as the construction rather than restating a count, states the re-homing relation so the thirty are not double-counted, names all three homes, and records that the crate exports nothing and has no workspace dependents — so the five absence claims beside it survive and the first is strengthened. `prototypes/serial-sum-run`'s occurrence count in the page is unchanged.

**Neighbouring sweep, seven claims checked, one defect found — and it is in `crates/**`, outside this ticket's edit permission.** `crates/tiler-conformance/src/portability.rs`'s `DEVICE_FREE_TEST_FLOOR` doc states "The crate declares 76 tests and the macOS predicate removes three of them, in `dispatch`, so a non-Apple host runs 73". The crate declares **77** at this base and a non-Apple host runs **74**: `fe282f1e` added `the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor` on 2026-08-08 without moving the prose census. The floor itself is 72 and still passes, so nothing is red; only the narrative is stale by one. Needs its own ticket.

Also imprecise, and left alone deliberately: the device-execution Measurement bullet says `crates/tiler-conformance` "was admitted on 2026-08-07 and every one of its runs landed the same day". Still true of *runs* — `f519c695` and `fe282f1e` on 2026-08-08 added no `require_or_report` call site and no measured entry point — but the crate has been edited since, and `fe282f1e` did add a device-free test. Repairing it would put a second dated note on a bullet this ticket does not own.

The five reproduction commands in the page's workspace-member block were re-run from the repository root under `set -e` at this base; all five pass.
