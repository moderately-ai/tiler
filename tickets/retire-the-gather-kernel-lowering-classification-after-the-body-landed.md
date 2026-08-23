---
id: retire-the-gather-kernel-lowering-classification-after-the-body-landed
title: Retire the gather kernel-lowering classification after the body landed
status: in-progress
priority: p1
dependencies: []
related: [emit-the-indirect-gather-on-metal, lower-the-indirect-gather-read-through-the-structured-kernel-body, route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type, restate-the-gather-standing-after-the-kernel-body-and-classifier-landed]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, gather, compiler]
claimed_from: todo
assignee: worker-retireclass
lease_expires_at: 1787480886
---
## User-visible outcome

The compiler stops classifying a kernel-lowering refusal it can no longer take, and the vacuously proved gather fixture asserts the wall it actually reaches instead of one that has been retired.

## Why this exists

Filed 2026-08-23 by `worker-gatherbody` from [`lower-the-indirect-gather-read-through-the-structured-kernel-body`](lower-the-indirect-gather-read-through-the-structured-kernel-body.md), which landed the kernel body and therefore retired the refusal that lane's classifier was written for. The parent ticket asked for this removal in the same landing; it could not be done there, because `implementation/compiler` was held by a live exclusive claim (`re-derive-the-contraction-fusion-role-rationale-after-the-key-replacement`, `worker-fusionrole`) and the parent's brief declared `tiler-compiler` a non-goal. Splitting it is the boundary that keeps the IR landing coherent rather than half-merged.

**Fact — the workspace gate is red on exactly one test until this lands.** Measured on the parent lane's branch: `cargo nextest run --workspace` reports `4064 tests run: 4063 passed, 1 failed`, and the failure is `tiler-compiler request::tests::a_statically_proved_gather_is_declined_for_its_missing_kernel_body`, reporting `left: None  right: Some(("kernel-lowering", "gather-kernel-body"))`. `planning_capability_rule` answers `None` because the refusal is no longer a kernel-lowering one.

**Fact — the wall the fixture now reaches is one layer further down, and it is a different crate's.** Probed on the parent's branch by making the same test print its refusal: `InvalidCompilerOutput(Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 })))`. The kernel declares its address operand at `KernelType::U32`, and `crates/tiler-compiler/src/program.rs`'s `BoundedCarrier::of` materializes every boundary value at the *program's* arithmetic carrier — `ArithmeticType::F32` yields `KernelType::F32` — so the U32 index input is declared as `f32` and `KernelProgramBuilder` refuses the stage. `tiler_ir::program`'s `StorageScalar::U32` already exists and its own documentation names `KernelType::U32` as its natural access type, so the missing half is the compiler's per-input carrier selection, not an IR carrier.

## Required work

- Re-audit every Fact above at your own base before editing.
- Remove `kernel_lowering_failure`'s gather arm and `GATHER_KERNEL_BODY_RULE` from `crates/tiler-compiler/src/pipeline/planning.rs`. The classifier never took a refusal of its own, so nothing changes class except the case that no longer occurs; check whether the function still earns its existence once the arm is gone.
- Decide what `a_statically_proved_gather_is_declined_for_its_missing_kernel_body` should assert now, and say which of the two it is: the fixture either pins the *new* wall by name — which requires deciding whether `InvalidCompilerOutput` is the truthful class for a U32 operand the compiler declines to materialize, or whether that too is a missing capability — or it is deleted because a differently owned ticket pins that wall. Do not leave it asserting a retired rule, and do not rename the assertion to whatever the run prints.
- Route a program input's storage carrier from its own resolved value type rather than from the program's arithmetic type, so a `tiler::u32@1` index operand materializes at `StorageScalar::U32` / `KernelType::U32`. If that turns out to be a public-boundary or identity question rather than a local fix, stop, split it, and say so.
- Perturb each behaviour on its own subject and quote the failure text.

## Non-goals

The kernel body, which landed. The Metal emission, which is [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) and refuses `KernelType::U32` at `msl_type` today.

## Coordinator graph correction — 2026-08-23: this is a co-landing constraint, not a sequencing one

This ticket was filed with `depends_on: [lower-the-indirect-gather-read-through-the-structured-kernel-body]`. **That edge was semantically wrong and has been changed to `related`.**

`depends_on` asserts *that must land first*. These two must land **together**, and neither ordering is buildable alone:

- The body alone turns `main` red — `a_statically_proved_gather_is_declined_for_its_missing_kernel_body` asserts a classification whose arm the body makes unreachable.
- The retirement alone would make `main` stop classifying a gather it still cannot lower, replacing a correct report with an absent one.

The `depends_on` edge also made the work **unreachable**: `tkt claim` refused it with `has unfinished dependencies`, because the body ticket sits in `review` — complete work, held for integration — and `review` is not terminal. The body cannot reach `done` until it merges, and it must not merge until this lands. A dependency edge cannot express that; it expresses the opposite.

**So the constraint is recorded where it belongs — in prose and in the branch layout — rather than as an edge that misstates it.** This lane's worktree is branched from `7d1219ec`, the gather-body branch merged up to `main`, so the body is present in its tree and the classifier's arm is genuinely unreachable there. The combined result merges into `main` **once**.

I did **not** reach for `tkt claim --force`: that flag steals a live lease from another agent and has nothing to do with dependency ordering. Using it here would have been misusing a tool to silence a check that was correctly describing a real problem with my own graph edge.

## Fact audit at `7d1219ec` (worker-retireclass, 2026-08-23)

Every Fact above was re-read at the dispatched base before any edit, and every command was run rather than quoted. The ticket file on this branch was also **stale**: it carried the pre-`bc906d3f` frontmatter and lacked the `## Coordinator graph correction` section entirely, because the merge at `7d1219ec` predates that commit. It has been reconciled to `origin/main`'s content here, so the co-landing merge does not have to resolve a conflict in the record of why it is a co-landing.

**Fact 1 — the workspace gate is red on exactly one test until this lands: verified in substance, imprecise in its count.** `cargo nextest run -p tiler-compiler a_statically_proved_gather_is_declined_for_its_missing_kernel_body` at `7d1219ec` reports `1 test run: 0 passed, 1 failed` with exactly the stated message:

```
assertion `left == right` failed: a region this build spells but cannot emit is a missing capability, not malformed compiler output
  left: None
 right: Some(("kernel-lowering", "gather-kernel-body"))
```

The failing test, its assertion, and its cause are exactly as stated. **The count `4064 tests run` is not this base's**, and the ticket says so itself — it is measured "on the parent lane's branch", whose base `db8ae185` predates the two test-split commits `main` has since taken (`8f993234` / `267cae83` for the IR program tests and `40121c3c` for the compiler pipeline tests). It is a correct historical measurement at a base that is not this one, so it is not repaired, only bounded. The number this landing is accountable for is derived under **Outcome** below.

**Fact 2 — the wall the fixture now reaches is one layer further down: verified, and reproduced independently rather than taken from the probe.** With the classifier removed, the same fixture was made to print its innermost refusal, which reported:

```
PROBE-REFUSAL: "InvalidCompilerOutput(Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 })))"
```

byte-for-byte the outcome the parent lane predicted. Each supporting clause was read at its own anchor rather than inferred: `crates/tiler-compiler/src/program.rs` holds `BoundedCarrier::of` under `The carrier one recognized arithmetic type materializes through` and is a total map from `ArithmeticType` alone; the single carrier is chosen once at `let Some(carrier) = BoundedCarrier::of(request.numerical_contract().arithmetic)` and stamped into every value by `fn program_input(` and `fn internal(carrier: BoundedCarrier, role: ValueRole, shape: Shape)`; the refusal is `crates/tiler-ir/src/program/builder.rs`'s, at `if buffer.element_type != value.element_type {`. `StorageScalar::U32` exists in `crates/tiler-ir/src/program/model.rs` and its documentation does name its natural access type — **but the ticket's rendering of that sentence is not greppable**, which is the false-absence trap `AGENTS.md` records: the source wraps mid-sentence, so the shortest anchor that resolves is `natural access type is the exact-width`, and a reader greping the full clause would conclude the documentation had been removed.

**Fact 2's supporting clause — "the missing half is the compiler's per-input carrier selection, not an IR carrier": verified, and the reason it is not already owned was checked.** [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md) is `done` and landed the whole IR/artifact/frontend pair, but its own boundary paragraph states the limit at the anchor `This is a physical program-input carrier and exact access type only` and lists no compiler-side selection. So the remaining work is genuinely unowned rather than a duplicate of that ticket, which is why it is filed as [`route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type`](route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type.md) rather than reopened there.

**One brief claim could not be verified and is recorded rather than restated.** The dispatch brief instructed reading "the complete ticket … including its `## Coordinator graph correction`". At `7d1219ec` that section does not exist in this file — `grep -rn "Coordinator graph correction" tickets/` returns nothing at this base. It exists only on `origin/main` at `bc906d3f`, which this branch had not taken. It was read from `git show origin/main:` and is now present here.

## Outcome (worker-retireclass, 2026-08-23)

**What was retired.** `GATHER_KERNEL_BODY_RULE` and `kernel_lowering_failure` are gone from `crates/tiler-compiler/src/pipeline/planning.rs`. The function does **not** still earn its existence once the gather arm goes: its remaining body is `physical_error_stage` followed by `failure_at_source`, its `region` parameter becomes unused, and its name would claim a classification it no longer performs. That exact two-line form is already written inline at the sibling call site in `crates/tiler-compiler/src/pipeline/verify.rs`, so the call site in `build_alternative_for_origin` now matches it rather than keeping a wrapper around a duplicate. `grep -rn "GATHER_KERNEL_BODY_RULE\|kernel_lowering_failure\|gather-kernel-body" crates/` returns **one** line, not zero, and that is correct rather than a miss: it is the retired rule quoted verbatim inside the replacement test's own documentation, where the sentence says what the test *used* to assert. This is the case `AGENTS.md` names — a correction that quotes retired wording makes the withdrawn string searchable again, so **the count cannot shrink to zero across this repair**, and a closing condition demanding an empty grep here would be unsatisfiable. The claim that holds is narrower and was checked by reading the one hit: no *code* reference to the classifier, its rule constant, or its rule string survives.

**The fixture was replaced, not deleted, and the argument is the vacuous bounds proof.** Deleting it would have removed the only compiler-level witness that the *vacuous* closed bounds argument travels the spelling path. Its sibling `a_gathers_spelling_follows_its_own_occurrences_bounds_evidence` asserts its own proof kind is `U32RangeContainedBySourceExtent` and states in its own comment that its fixture "must rest on the inhabited argument, not on vacuity" — so the two arguments are deliberately split between the two tests, and only one of them was made stale by the body landing. The retired half was the *classification*; the reached half is new and worth pinning. `a_statically_proved_gather_clears_kernel_lowering_and_stops_at_the_program_carrier` keeps the spelling premise unchanged and replaces the rule assertion with the typed payload the compile now produces.

**What a proved gather does end to end, pinned.** `gather_program_over([4, 0], [2], 0)` is recognized, lowered through the governed gather capability, refined, statically proved, spelled as `RegionSpellingKind::Gather`, verified, admitted, and **lowered to a kernel body** — the wall that used to stop it. It then stops in `build_plan_program` with `InvalidCompilerOutput(Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 })))`. The test matches that payload structurally rather than comparing a `Debug` string.

**The asserted class is not endorsed as truthful, and that is stated in the test rather than fixed here.** `InvalidCompilerOutput` claims malformed output; nothing here is malformed, and the honest reading is the same one the retired classifier made one wall up. It is not reclassified, because installing a second stopgap of exactly the kind just retired would be the wrong fix twice — the right fix is to stop earning the refusal, which is [`route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type`](route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type.md). That ticket carries the measured refusal, the ABI consequence (`declare_host_abi` pushes **one** shared `element_bytes` literal for every input and internal, so a per-input carrier is a program-identity change and not a field swap), and the warning that `U32` and `F32` are both four bytes wide so the gather fixture cannot witness a wrong shared width.

**Three perturbations, one per independent property, each on its own subject.** The failure text of each is quoted in the test's own documentation and reproduced here:

- Removing the kernel body again — `return Err(KernelDiagnostic::BodyRefinement);` at the head of the `LogicalAccess::GatherSource` arm in `crates/tiler-ir/src/kernel/lower.rs` — reports `got Physical(Refinement { rule: "body-refinement", region: RegionId(0) })`. **This one corrected the test's own documentation:** the first draft predicted the refusal would stop being `InvalidCompilerOutput` at all, and it does not — the class is identical in both worlds and only the payload differs. The prediction was replaced with the observed text rather than left standing.
- Giving `ArithmeticType::F32` the `U32` carrier in `BoundedCarrier::of` reports `got Program(CoreConstruction(StageElementType { position: 0, expected: F32, actual: U32 }))`, so the assertion discriminates *which* operand disagrees rather than merely that a type check fired.
- Inverting the spelling proof gate to `gather_bounds_proof(lowering, normalized.member).is_none()` in `crates/tiler-compiler/src/physical.rs` reports `a vacuously proved gather is spelled by the governed vocabulary: GatherIndexBoundsUnproved` from the earlier `expect`, so the premise is load-bearing rather than incidental setup.

Each perturbation reddens a different assertion line, so no single one of them could stand in for the others. All three were reverted with `git checkout --` and the tree re-verified green.

**Test-count delta: zero, derived from the diff rather than measured after the fact.** `git diff -- crates/ | grep -c '^+.*#\[test\]'` and the `^-` form both return **0**: the diff renames one test function and rewrites its body, and the `#[test]` attribute lines are untouched. `crates/tiler-compiler/src/request/tests.rs` holds 87 `#[test]` occurrences at `7d1219ec` and 87 after. The new `compiler_output_refusal` is a helper, not a test. So this branch's workspace count equals its base's, and the base is `cd3d689a` merged to `main` — the body lane's +5 (all five in `crates/tiler-ir/src/kernel/tests/gather.rs`; `git show cd3d689a -- crates/tiler-ir/src/kernel/{lower,model,verify,builder}.rs | grep -c '^+.*#\[test\]'` is 0) plus whatever `main`'s two test-split commits changed. The absolute number is reported by the gate below rather than predicted from the stale 4060/4064 figures, neither of which was measured at this base.

**Follow-ups filed rather than absorbed.**

- [`route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type`](route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type.md) — `implementation/compiler`, the wall this landing exposes. Added as a hard `depends_on` of [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md), because no gather program is assembled at all until it lands, so there is nothing for a backend to emit. That is a genuine ordering, unlike the co-landing constraint this ticket's own graph correction had to downgrade.
- [`restate-the-gather-standing-after-the-kernel-body-and-classifier-landed`](restate-the-gather-standing-after-the-kernel-body-and-classifier-landed.md) — `contracts/optimizer` and `contracts/navigation`, neither held here. It is the third link in the chain `27fa3043` → `0b51531f` foretold, and it also carries `docs/roadmap.md`'s gather row, which claims at the anchor `has no indirect relation, so the family has no realization law` that the family has no lowering capability or executable plan — falsified by three landings, and flagged by the parent's hold note as needing a coordinator pass.
- [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) carries a dated correction withdrawing the present tense of its `kernel_lowering_failure` paragraph. The prediction it made was right; only its tense is retired, so the sentence is preserved and corrected rather than rewritten.

### Gates and the derived count

`DEVELOPER_DIR=/Applications/Xcode.app TILER_REQUIRE_METAL_TOOLCHAIN=1 make full` exits **0** on this branch: **4065 workspace tests run, 4065 passed, 8 skipped**, and **1350 release tests run, 1350 passed, 3 skipped**, with `make citations` reporting every pinned citation and local markdown link resolving, `tkt lint` `ok: no problems found`, and shellcheck clean. `cargo nextest run -p tiler -p tiler-compiler` — run as its own step because a two-package run is what catches a `workspace_unsafe_sites` regression a single-package run misses — reports `1076 tests run: 1076 passed, 1 skipped`. `git diff --check` is clean. A first `make full` failed only at `fmt`; `cargo fmt --all` was applied and the gate rerun end to end rather than resumed.

**The count derivation, and one figure that does not reconcile.** The stated baseline of 4060 workspace plus the body lane's +5 plus this lane's 0 gives exactly the observed 4065. Each term is measured rather than assumed: a tree-wide `git grep -o '#\[test\]' <rev> -- 'crates/*' 'prototypes/*' | wc -l` census returns **4070** at both `db8ae185` and `origin/main` — so `main`'s two test-split commits moved tests between files without changing the population — and **4075** at both `cd3d689a` and this branch's tip, which is the body's +5 and this lane's net zero. In the one file this lane touches, `crates/tiler-compiler/src/request/tests.rs`, the count is 87 before and 87 after; the diff renames one test function and rewrites its body while adding a non-test helper, so `git diff -- crates/ | grep -c '^+.*#\[test\]'` and its `^-` form both return 0.

**The parent lane's `4064 tests run` is not reconciled and is reported as such rather than explained away.** It was measured at `db8ae185` plus the body, a base this branch is not, and the one-test difference is *not* a population change: the `#[test]` census above is identical across `main`'s intervening commits. It is also not an environment partition — `cargo nextest run --workspace` without `DEVELOPER_DIR` or `TILER_REQUIRE_METAL_TOOLCHAIN` reports the same `4065 tests run … 8 skipped` as the gated run, so the run/skip split is not what differs. Nothing this lane changed accounts for it, and no claim is made about what does.

**Gate carry, recorded rather than assumed.** The green `make full` above was run before this Outcome section and the two follow-up tickets were written. That delta is `tickets/**` only — it touches none of `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh`, or `check-citations.sh` — so it carries the gate under the rule in `AGENTS.md`, and the two checks that rule names as still required were rerun on the final tree: `tkt lint` reports `ok: no problems found` and `make citations` exits 0 with every pinned citation and local markdown link resolving. `git diff --check` is clean and `tkt guard --base 7d1219ec` exits 0, reporting shared and reverse-dependency overlaps as WARN with no under-declared scope.
