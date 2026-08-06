---
id: move-the-structural-row-to-r6-and-retire-its-backend-residual
title: Move the structural row to R6 and retire its backend residual
status: review
priority: p2
dependencies: []
related: [emit-the-structural-region-on-metal, realize-parallel-reduction-strategies-on-metal, integrate-the-contraction-vertical-into-the-runtime, admit-elementwise-epilogues-over-a-materialized-intermediate, lift-the-four-published-and-consumed-walls-together, admit-a-partitioned-write-ownership-contract, re-read-the-bf16-and-elementary-support-rows-against-source, record-the-contraction-execution-row-and-correct-the-matrix-headline]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-navigation
lease_expires_at: 1786028995
---
## The work (maturity audit 2026-08-06, findings coordinator-verified: the IndexSubtract arm and both structural goldens exist)

`docs/roadmap.md:479`'s rung moves to `R6 for the two admitted families`; the "does not move to R6" span is corrected in tense (the derivation stays — it is what the widening was accepted on); the row's own reproduction command now returns hits under `crates/tiler-metal/` and must be restated; `roadmap.md:555`'s closing sentence and `status.md:32`'s "No backend has emitted a structural region" clause both correct. The new text carries the measurement boundary the old text lacks: emission measured on Xcode 27.0 / Metal 32023.921 — not the authoritative ledger row — and it is a translate-and-link fact, not a dispatch; R7 stays unmet and unowned.

## Closes when

All four sites agree, the boundary is stated, and the embedded reproduction commands return what they claim.

## Scope note

`project/tickets` was added to `shared_scopes`. Every claimed ticket declares it because the guard does not treat a ticket's own file as implicitly shared, and this branch edits `tickets/**`. Declaration and scheduling metadata for already-authorized work; no product scope moved.

## Outcome

**The structural row is at R6, and the batch of navigation corrections rode with it because every site contends on the one exclusive `contracts/navigation` scope.** Nothing here is a rung claim that source does not carry: each correction below was reproduced against source or against the cited ticket's own Outcome before it was written, and the two claims that did not reproduce as briefed are recorded as such rather than written.

### The four coordinated sites of the claimed ticket

**`docs/roadmap.md:479` — rung cell.** `R5 for the two admitted families, with the R6 residual now exactly one crate wide and named below` → `R6 for the two admitted families, bounded to offline translation on one measured toolchain row and with R7 unmet`. Views and bit-preserving copies stay R2, unchanged.

**`docs/roadmap.md:479` — the "does not move to R6" span**, corrected in tense with the derivation preserved verbatim in the past, because the one-arm bound is what [`emit-the-structural-region-on-metal`](emit-the-structural-region-on-metal.md) was scoped on. The successor states the arm (`binary_realization` in `crates/tiler-metal/src/emit.rs`, the mapping moved out of `emit_binary` so it can be exercised over the whole vocabulary), the non-wrap derivation as *attribution* rather than a recomputed check with both alternatives eliminated on fail-closed grounds, the wrap perturbation's real refusal site one layer up (`KernelDiagnostic::BodyRefinement`, so no compile-stage perturbation could carry the claim), the two goldens, and the `core::mem::variant_count::<BinaryOp>()` totality check that makes an omitted construct a length type error.

**`docs/roadmap.md:555`** — the closing sentence, which had itself been corrected earlier the same day to "what holds the structural row at R5 is the *backend*, one crate further out". Corrected again in tense; both quotations retained because the sequence is the row's evidence.

**`docs/status.md`** (the paragraph formerly at `:32`, now `:35`) — "No backend has emitted a structural region: the roadmap's structural row holds it at R5 for that reason alone and names the one missing `BinaryOp::IndexSubtract` arm" discharged, with both boundaries stated inline.

**The measurement boundary, stated at every site rather than once.** It is a *translate-and-link* fact, not a dispatch: a wrapped index compiles just as cleanly. And the toolchain row is **not** the [compile-profile authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)'s — the ledger sources `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` from Xcode 26.6 (17F113) with offline compiler and linker at `32023.883` at MSL 4.0 for macOS 26.0, and it excludes `metalfe-32023.921` **by name** under [ADR 0086](../docs/decisions/0086-require-attributable-or-attested-native-translation.md) item 4 as a runtime-compiler build no AOT route supplies source to. Verified by reading the ledger (`docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md:36-38` and `:61`), not inferred from the difference in version strings. **R7 is unmet and unowned**, and the trigger cell now says so in two parts: no dispatched device comparison, and no *compiler-derived* structural region put through `emit` — `tiler-metal` cannot depend on `tiler-compiler`, so both goldens are hand-restatements verified by shape. Neither is filed.

### Two stale cross-references this rung move created, corrected here

Not in the brief; found by grepping the navigation scope for every claim about the structural families' rung after the edit, and both would have left the corpus self-contradicting.

- **`docs/roadmap.md:469`** (SiLU) read that the unary-families wall "is the same one holding the structural families at R5". It identifies nothing now.
- **`docs/roadmap.md:481`** (sub-tensor selection) read that its own `operation-set` refusal stands "exactly as the two structural families above still are". Corrected to say that family is now alone in the refusal, with the structural row as the worked precedent for closing it.

`docs/roadmap.md:435`'s "only backend-executed profile" headline is **deliberately untouched**: it is assigned to [`record-the-contraction-execution-row-and-correct-the-matrix-headline`](record-the-contraction-execution-row-and-correct-the-matrix-headline.md), and this row's R6 is *emission*, not execution, so nothing here falsifies it.

### The inline batch, each verified before it was written

| Site | Claim | Check run | Verdict |
| --- | --- | --- | --- |
| `status.md:21` | The synchronization realization is declared | `crates/tiler-build/src/metal_declaration.rs:611` `declare_synchronization_realization`; row at `:263-273` = `ControlBarrier` / workgroup arrival / workgroup publication / `{workgroup: true, device: false}` / `AcquireRelease`, `SynchronizationSupport::Realized`; `git log -S declare_synchronization_realization` → `668e5b47`, 2026-08-01 | **Corrected.** Subject named in full, with its "stated literally so it can refuse" property and the four-quoted/one-eliminated dimension split |
| `status.md:21` | Both parallel strategies executed under `FLUSH_AND_REASSOCIATE_F32` | `realize-parallel-reduction-strategies-on-metal` `done`, executable-half Outcome dated 2026-08-02: `1x4`, three retained alternatives all `41700000`, strategy labels from device-reported evidence; `rg -n 'pub const FLUSH_AND_REASSOCIATE_F32' crates/tiler-compiler/src/session.rs` → `1490` | **Corrected**, with the boundary that makes it not a rounding claim: operands `1.0, 2.0, 4.0, 8.0` are exact under every grouping |
| `status.md:21` | Remaining work repoints at calibration | `calibrate-and-activate-parallel-reduction-selection` `todo`; `check-synchronization-realization-before-the-routing-commit` `todo` | **Corrected**; "backend qualification" replaced by the two live owners |
| `status.md:32` | The epilogue wall is gone, compiles and bit-compared | `admit-elementwise-epilogues-over-a-materialized-intermediate` `done`; Outcome table: `matmul(a,b)*2.0` two dispatches, `sum(x*x, axis 1)*2.0` three, all five contracts; `pipeline::tests::{a_contraction_epilogue_chain,a_reduction_epilogue_chain}_matches_the_reference_evaluator` present | **Corrected**; item removed from the refusal list and recorded as the third correction entry |
| `status.md:32` | `output-partition-overlap` narrows; `published_and_consumed_overlap` admits | Read `check_output_cover` and `published_and_consumed_overlap` in `crates/tiler-compiler/src/request.rs:4048-4161` in full — four conjuncts, each load-bearing; `pipeline::conformance::a_published_and_consumed_intermediate_compiles_and_agrees:648` asserts two cover regions, three dispatches, one publishing copy, both outputs bit-compared | **Corrected to a narrowing, not a removal** |
| `status.md:32` | `structural-operand` stands | `crates/tiler-compiler/src/request.rs:4804`/`:4807`; `composed_family_recognition::a_structural_occurrence_over_a_computed_value_refuses_by_name` refuses under all contracts | **Left unchanged** — see the unverified item below |
| `roadmap.md:480` | `admit-a-partitioned-write-ownership-contract` is done; only the lowering remains | That ticket `done`; `lower-the-concatenate-occurrence-through-partitioned-writes` `todo` with `dependencies: [admit-a-partitioned-write-ownership-contract, admit-sub-range-write-domains-for-unequal-partitions]`, the second also `done` | **Corrected**, and *widened* on evidence the brief did not carry: the contract discovered and filed a **second** prerequisite (only equal-share partitions were representable; this family's pinned `[8, 0, 128]`-beside-`[8, T, 128]` occurrence is maximally unequal). Both are done, so the lowering is dependency-satisfied |
| `roadmap.md:467` | Cite ADR 0100 | `docs/decisions/0100-*.md` `decision_status: accepted`, `implementation_status: not-started`; index rows at `README.md:94`/`:244` | **Cited**, no rung claim; two independent reasons it moves nothing (topology is physical, and the record is not-started with a total measurement boundary) |
| `roadmap.md:468` | Cite ADR 0101 | `0101-*.md` accepted 2026-08-06, `not-started`; item 5 admits no permission; the `exp(-1.0)²` vs `exp(-2.0)` divergence and 502/1681 grid disagreement read from the record | **Cited**, no rung claim |
| `roadmap.md:287` | Check the three-dimension framing | ADR 0101 states it "supersedes nothing: no accepted decision claims the dimension set is exhaustive"; `:287` names distributivity as the third and never claims closure | **Not falsified.** A citation added anyway, because a reader counting dimensions from that sentence would now undercount, and the record's own item 6 makes naming *all* missing dimensions load-bearing |
| `roadmap.md:469` | Cite ADR 0099 | `0099-*.md` accepted 2026-08-05, `implementation_status: partial`, one-family caveat accepted with it | **Cited**, no rung claim; the evidence-bar decision is explicitly left to `re-read-the-bf16-and-elementary-support-rows-against-source` and the deferral is stated in the cell so a reader does not mistake silence for a verdict |
| `status.md:25` | Device execution becomes one of three | `integrate-the-contraction-vertical-into-the-runtime` `done`, five bit-compared cases at `td,od->to`, `[2,3]×[2,3]→[2,2]`; the reassociating run as above | **Corrected**; three sub-bullets, each with its own boundary and an explicit non-generalization clause |

### One brief claim that did not survive checking, and what was written instead

**The brief attributed the fourth published-and-consumed wall's closure to [`admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary`](admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary.md); that ticket did not close it.** Its own body is struck through: it was filed as the `tiler-ir` widening that would deliver the row, and instead *measured* the four-deep stack, found the `tiler-ir` account to be the last and least reachable of the four, and filed [`lift-the-four-published-and-consumed-walls-together`](lift-the-four-published-and-consumed-walls-together.md), which is what lifted them. Caught by reading both ticket bodies after `git log -S 'PublishingCopy' -- crates/tiler-ir/src/program/model.rs` returned one commit (`a8842638`, 2026-08-06) that neither ticket's body cites by hash. The status text names both tickets and says which did what, because the inversion is the instructive part.

### One item left unchanged as unverified

**`structural-operand` may have narrowed and the evidence does not reach.** `recognize_structural_read`'s doc (`request.rs:4770-4779`) claims an epilogue's staged operand is admitted, and the check is `leaves.is_leaf(*operand)` — which an epilogue's producer value satisfies — so the code plausibly admits a structural occurrence over a staged value. **No test exercises that combination**: `rg -n 'fn .*epilogue' crates/tiler-compiler/src/pipeline/tests.rs crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs` lists five epilogue tests and none is structural, and the epilogue ticket's Outcome does not claim it. `status.md`'s wording ("a value the program *computes* rather than declares") is therefore left exactly as it was rather than narrowed on a doc comment. Flagged for the coordinator: if that admission is real it wants a test before a doc says so, and if it is not, the doc comment in `request.rs` is the defect.

### Two `contracts/numerics` items for the coordinator — out of scope here

`docs/correctness-and-testing.md` maps to `contracts/numerics`, which this ticket does not hold. Both items were read there and neither was edited.

1. **The facade-gate row's *stated cause* for the materialized-intermediate bound has moved, and the row itself is already correct.** `admit-elementwise-epilogues-over-a-materialized-intermediate`'s own Outcome flags this for whoever holds the scope: the bound read "no elementwise region this profile builds reads a materialized intermediate", and that is no longer true — `verify_pointwise_region` now admits reads naming strictly ascending declared inputs plus at most one `TensorRole::Intermediate`. Its sibling open bound, `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs`, has since landed and `:112` already records it, so a sweep should check whether the gate's remaining stated bounds are the current ones.
2. **`:112`'s attribution of the published-and-consumed row is right and `:209` is current, so the numerics edit is narrower than it may look.** `:112` already credits `lift-the-four-published-and-consumed-walls-together` and already says `admit-elementwise-epilogues-over-a-materialized-intermediate` "built the region without lifting the row" — which the ticket bodies confirm. `:209` already carries the grouping-sensitive `FLUSH_AND_REASSOCIATE_F32` measurement (operands `0x3f400000, 0x3e800000, 0x33400000, 0x33000000`, serial `0x3f800000` against both parallel strategies' `0x3f800001`) and already records that the grid-axis row is now a measured 268,435,456. What a numerics holder should check is whether `:112`'s epilogue sentence needs the *delivered* three shapes stated beside "built the region", since that ticket has since landed whole.

### Reproduction commands embedded in the docs, each run

- `rg -n 'IndexSubtract' crates/tiler-metal/` → **10 hits**, `emit.rs:1257` the realization (`BinaryOp::IndexSubtract | BinaryOp::I32Subtract => ("-", None)`). The superseded text's claim was four hits workspace-wide, none under `crates/tiler-metal/`; the workspace count is now 15.
- `ls crates/tiler-metal/goldens/structural_*.metal` → **two files**, `structural_mirrored_reindex.metal` and `structural_widening_broadcast.metal`.
- `TILER_REQUIRE_METAL_TOOLCHAIN=1 cargo nextest run -p tiler-metal -E 'test(golden_compilation)' --no-capture` → **11 tests run: 11 passed, 101 skipped**, `metal "Apple metal version 32023.921 (metalfe-32023.921)" / metallib "AIR-LLD 32023.921 ..." (SDK 27.0 build 26A5388f)`, and the nine link lines including `structural_mirrored_reindex.metal linked 3555 bytes` and `structural_widening_broadcast.metal linked 3555 bytes` — the exact byte counts the row now quotes.

The environment fields the row states were read from the host rather than copied from the emission ticket: `xcodebuild -version` → `Xcode 27.0 / Build version 27A5228h`; `sw_vers` → macOS `27.0` build `26A5388g`; `xcrun --sdk macosx --show-sdk-version`/`--show-sdk-build-version` → `27.0` / `26A5388f`; `sysctl -n machdep.cpu.brand_string` → `Apple M4 Max`. The compile flags and target triple were read from `crates/tiler-metal/src/golden_compilation.rs:83-84` and `:454` rather than from the ticket.

### Checks

`tkt lint` clean after every ticket edit; `git diff --check` clean. **No gate input was touched**: the diff is `docs/roadmap.md`, `docs/status.md`, and `tickets/*.md` only — nothing under `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh` — so no cargo gate is owed for this delta. The `tiler-metal` test above was run as a *reproduction of an embedded command*, not as a gate, and it passed. `tkt guard` run against the true branch base after committing.
