---
id: narrow-the-contraction-witness-refusal-to-staging-it-cannot-read
title: Narrow the contraction witness refusal to staging it cannot read
status: in-progress
priority: p2
dependencies: []
related: [join-the-scheduled-region-into-the-contraction-witness, derive-staged-combine-structure-from-program-scope]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [ir, witness, fail-closed, correctness]
claimed_from: todo
assignee: worker-narrow
lease_expires_at: 1787443883
---
## User-visible outcome

The contraction plan witness refuses the kernels whose combine tree it genuinely cannot establish, rather than every kernel that declares any workgroup staging at all.

## Why this exists

Found 2026-08-22 by the staged-combine derivability spike while auditing a Fact the coordinator wrote, and recorded separately because it is a different defect from the join that spike motivated.

**Fact — the refusal is broader than its own documentation says.** `crates/tiler-ir/src/program/contraction_witness.rs` refuses at the predicate `staging().len() != 0` — two sites, one for the covering realization and one for a split's combiner. The **enum-level** doc of `ContractionF32PlanWitnessError` says the catch-all covers a realization *"whose exact binary combine tree cannot be derived from program scope — including any kernel that declares workgroup staging"*. **The code does not implement that.** It refuses on the presence of staging alone, including staging that carries no combine structure at all, so the documented "including" is in fact the whole predicate.

**Correction — 2026-08-22, the coordinator's original wording of this Fact was wrong and is repaired above.** I wrote that the prose "sits in the doc comment of the `TopologyUnsupported` variant, several lines above". Verified at `e7a2d0d4`: **three distinct pieces of prose were conflated.** The `// A kernel declaring workgroup staging combines inside the workgroup;` line is a plain `//` comment **two lines directly above** the refusal it introduces, not a doc comment and not distant from it. The `TopologyUnsupported` variant's own doc reads only *"The covering realization's exact binary combine tree cannot be derived from program scope"* and does not mention staging. The "including any kernel that declares workgroup staging" clause lives in the **enum-level** doc block, above `pub enum ContractionF32PlanWitnessError {`. Found by `worker-site411`, which was auditing a sibling ticket carrying the same conflation from the same source. **The defect this ticket names survives all three corrections** — the predicate is still broader than the enum doc's "including" implies — but the citation was wrong and a worker following it would have edited prose that says something else.

**Re-audit — 2026-08-22 by `worker-narrow`, at base `b6248f91`.** Every citation in the Fact and in the correction above is stale at this base. [`join-the-scheduled-region-into-the-contraction-witness`](join-the-scheduled-region-into-the-contraction-witness.md) landed as `8926b71b` and rewrote all three pieces of prose *and* the predicate itself. Per-Fact verdicts, each count from `grep -c` against `crates/tiler-ir/src/program/contraction_witness.rs`:

- **False — the predicate.** `staging().len() != 0` returns 0. It is now spelled `staging().len() == 0` (returns 1), inverted into an early return of `StagedRole::Unstaged`.
- **Imprecise — "two sites".** There are no longer two copies of a predicate. One classifier, `staged_role`, is called from two sites: `staged_role(covering.kernel(), join)` for the covering realization and `staged_role(split.combiner().kernel(), join)` for a split's combiner. The *sites* are still two; the *predicate* is one.
- **False — the `//` comment anchor.** `A kernel declaring workgroup staging combines inside the workgroup` returns 0. The line now reads `A kernel declaring workgroup staging may combine`.
- **False — the enum-doc quote.** `including any kernel that declares workgroup staging` returns 0. The enum doc now scopes the clause to the unjoined constructor: `that includes any kernel that declares workgroup staging` (returns 1).
- **Half-true — the defect.** "It refuses on the presence of staging alone" still holds for `from_program`. It no longer holds for `from_program_and_regions`, which reads the joined region's `ReductionTopology` and admits `CooperativeContraction` as `StagedRole::CarriedAccumulator`. The narrowing this ticket asked for has therefore *already landed* for every caller that supplies regions.

**Why this is worth its own ticket rather than folding into the join.** The two are separable and fail differently. The join adds a route for kernels whose region states the tree; this narrows the arm so kernels with *structurally irrelevant* staging stop being refused for a reason that does not apply to them. Landing only the join would leave the over-broad arm in place for every staged kernel the join does not reach, still reporting a cause that misdescribes it.

**This fails closed, so it is not a wrong answer.** It is an over-refusal and a false doc claim. Ranked accordingly.

## Required work

- Re-audit the Fact at your base and report a per-Fact verdict; read both refusal sites, not one.
- Decide **by reading** whether staging that carries no combine structure is distinguishable at this layer at all. **If it is not, the correct outcome is to repair the documentation to state the real predicate and record why the arm must stay broad** — that is a valid close, and better than a narrowing that guesses.
- If it is distinguishable, narrow the arm and keep the refusal fail-closed: an unrecognized staging shape must still refuse, never fall through to a derived tree.
- Perturb both sites separately with quoted failure text. Show a kernel that should now be admitted being admitted, and one that must still refuse still refusing.

## Non-goals

The scheduled-region join — that is [`join-the-scheduled-region-into-the-contraction-witness`](join-the-scheduled-region-into-the-contraction-witness.md); any new encoding or identity change; and recovering a combine tree from a kernel body.

## Closes when

Either the refusal names only what it cannot read, with both admitted and still-refused cases watched, or the arm is recorded as necessarily broad with its documentation repaired to state the real predicate and a reconsideration trigger attached.

## Outcome

**The arm is necessarily broad under `from_program`, and is already exactly as narrow as the records permit under `from_program_and_regions`.** The second disjunct of "Closes when" is the one taken: the documentation now states the real predicate and records why, with a reconsideration trigger. Landed in `crates/tiler-ir/src/program/contraction_witness.rs` on `staged_role`, under the heading `# Why the unjoined refusal must stay broad`.

**Fact — staging that carries no combine structure is not a population this layer has.** A verified kernel declares non-empty staging exactly when its region declares a cooperative tile: `verify_cooperative` in `crates/tiler-ir/src/kernel/verify.rs` refuses a kernel that stages without one at the anchor `must declare nothing, or the kernel`, and `cooperative_tile` in `crates/tiler-ir/src/schedule/model.rs` yields a tile only for `CooperativeWorkgroup` and `CooperativeContraction`. Those two are what staging *means* here, and only the second leaves the canonical left chain intact. The real question is therefore which of the two a staged kernel is, not whether its staging is structural.

**Fact — program scope does not answer that question.** A staging row carries an ordinal, an element type, an address space, and a slot count; `CooperativeTile::staging` is a plain `Vec<WorkgroupStaging>`, so neither the row count nor the slot count is tied to a topology. The execution binding is the one field that names the topology — `CooperativeContraction` requires `ExecutionBinding::BlockedWorkgroup` and never defaults it — and it does not reach the signature: `verify_signature` derives `GlobalInvocationIndex` for the blocked and global-linear bindings out of a single shared match arm, then appends `LocalInvocationIndex` for either cooperative tile. Measured: the two lowered kernels present `[GlobalInvocationIndex, LocalInvocationIndex]` alike, the contraction staging two `F32`/`Workgroup` rows of `256` slots and the reduction one of `4`. Watched by `a_staged_contraction_and_a_staged_reduction_agree_on_program_scope_builtins` in this crate's program tests.

**Inference — why the remaining basis for narrowing is rejected.** What is left is the reachability wall: no *compiler-built* program pairs a contraction occurrence with a cooperative-workgroup kernel, so on that population a staged contraction is always the carried accumulator. Narrowing on it is fail-open twice over. `from_program` is a public constructor over any `VerifiedKernelProgram`, so a hand-built program is not bound by what the frontier emits; and the wall is a property of the current fold gate rather than a record the program carries, so widening the frontier would silently turn the refusal into a left chain for a round-structured fold. That is a wrong tree, strictly worse than the over-refusal it would replace.

**Reconsideration trigger.** Reopen when program scope carries a *declarative* statement of the topology — a topology tag on the staging row, the scheduled region itself, or any other field a reader can check rather than infer. A newly discriminating *derived* field does not qualify on its own. Recorded at the `staged_role` anchor `Reconsideration trigger`; the builtin collision that makes the current answer "no" is watched by the test named above, so the trigger has a mechanical half as well as a prose one.

**Finding — the combiner site is unwatched, and it is reachable.** Perturbing the covering site reddens two tests; perturbing the combiner site reddens nothing across `-p tiler-ir -p tiler`. Replacing the split branch's `partitioned_chain_nodes(...)` with `unreachable!()` also leaves `1349 tests run: 1349 passed`, so the witness's whole split branch is untested. It is not dead, though: neither `push_partial_reduction` in `crates/tiler-ir/src/program/builder.rs` nor `verify_partial_reductions` in `crates/tiler-ir/src/program/verify.rs` reads an occurrence's operation family, and the program layer retains no op keys, so a hand-built program can declare a contraction split whose combiner stages. Recorded as a comment at the call site. Landing the tests is already obligation **B** of [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md), which names this refusal explicitly, so no new ticket is filed.

**Unrepaired, out of scope.** `spikes/reference/staged-combine-derivability/README.md` carries two citations into `contraction_witness.rs` that the join made stale the same day the spike was written: the anchor `A kernel declaring workgroup staging combines inside the workgroup` and the predicate `staging().len() != 0` (twice), all now zero hits at tip against a frontmatter `last_verified` of 2026-08-22. `make citations` declines spike citations by design, so the gate does not see them, and `spikes/` is outside this ticket's scopes.
