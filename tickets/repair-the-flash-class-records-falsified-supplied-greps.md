---
id: repair-the-flash-class-records-falsified-supplied-greps
title: Repair the flash-class record's falsified supplied greps
status: done
priority: p2
dependencies: []
related: [reconcile-the-l4-records-self-contradicting-softmax-elimination-row, re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, falsified-evidence]
---
## User-visible outcome

`docs/research/program-planning/flash-class-capability-set.md` hands a reader commands whose output supports the claim beside them, so re-running one confirms the record rather than appearing to reverse it.

## Why this exists

Found 2026-08-22 by `worker-l4row2` during a sibling scan. **Two of the five are falsified supplied greps, and both fail in the direction that reads as reversal** — the record says a search returns nothing or little, and it now returns a great deal, so a reader re-running it concludes the finding is dead when the conclusion is in fact intact.

Verified by the coordinator at `123f1b02`:

- The record states `grep -rn 'SubgroupWidth\|lane_identity\|SubgroupThenWorkgroup' crates/` **"returns nothing"**. It returns **69 lines**.
- It states `grep -rni 'simdgroup' crates/` **"returns five lines"**. It returns **21**.

**The conclusions survive.** `MetalTargetFacts` still has exactly five fields, none of them a subgroup width — the record is right about what it concludes and wrong about what it offers as proof. So this is a **re-evidencing**, not a withdrawal.

**Three further line pins have drifted** (reported by that lane, unverified by the coordinator): `feasibility.rs:211`→241, and the item is `pub(crate)` rather than the `pub` the record states; `target.rs:755`→871; `component_cost.rs:619`→629.

## Required work

- Re-audit all five at your base with a per-Fact verdict, **running each command yourself** and reporting its actual output.
- **Re-evidence rather than withdraw.** Give each surviving conclusion a reproduction that supports it — prefer a claim about structure that stays true (`MetalTargetFacts` has five fields, none a width) over a bare emptiness assertion, which is the shape that rots into apparent reversal.
- Replace drifted line pins with **anchors**; a line number rots silently while an anchor fails loudly. Correct the `pub`/`pub(crate)` mis-statement.
- **Preserve retired wording in dated corrections**; grep counts cannot shrink.
- Check this record's siblings for the same shape — a supplied command whose stated output no longer matches. Report findings **and** clean results.

## Non-goals

Re-deciding any flash-class conclusion; editing `crates/`; and the wider zero-synchronization retirement, which is its own ticket.

## Closes when

Every supplied command's stated output matches what it produces, each surviving conclusion carries a reproduction that supports it, drifted pins are anchors, and the sibling scan is reported with its clean results.

## Coordinator re-audit at `39fdb54c`, 2026-08-22 — all five confirmed, and **two further pins the ticket did not name**

Every command below was run by the coordinator at this base before dispatch, not relayed.

- **Falsified grep 1 — confirmed.** `grep -rn 'SubgroupWidth\|lane_identity\|SubgroupThenWorkgroup' crates/ | wc -l` returns **69**. The record says "returns nothing".
- **Falsified grep 2 — confirmed.** `grep -rni 'simdgroup' crates/ | wc -l` returns **21**. The record says "returns five lines".
- **`feasibility.rs:211` — confirmed drifted, and the visibility mis-statement is real.** `grep -rn "enum CapabilityAxis" crates/` returns `crates/tiler-compiler/src/target/feasibility.rs:241:pub(crate) enum CapabilityAxis {`. Pin moved 211 → 241, and it is `pub(crate)`.
- **`component_cost.rs:619` — confirmed drifted to `:629`.** The shared arm is intact: `CostComponent::ResourcePressure | CostComponent::CompileTime => CostValue::Unknown`.
- **`target.rs:755` — confirmed drifted to `:871`, and the ticket's bare filename is ambiguous.** The record cites **two different `target.rs` files**: `crates/tiler-metal/src/target.rs` for `MetalTargetFacts` and `crates/tiler-compiler/src/target.rs` at line 229. Always name the crate. `MetalTargetFacts` still has exactly five fields — language, platform, deployment_minimum, subnormal_arithmetic, and the buffer binding limit — so that conclusion survives, as the ticket says.

**Two pins the ticket does not name, and they fail in the dangerous direction.** Line 229 of the record cites `declare_local_memory_bytes` and `declare_measured_local_memory_bytes` at `crates/tiler-compiler/src/target.rs:1937`, `:1950`. Both functions now live in `crates/tiler-compiler/src/target/builder.rs:634` and `:647`. **The named file still exists** — 3,818 lines — and line 1937 lands inside an unrelated test assertion, `Err(TargetProfileBuildError::DuplicateSynchronizationRealization)`. So a reader following the pin does not get an obvious miss; they land in plausible, compiling, entirely unrelated code. This is the module-split false-absence hazard AGENTS.md records under the anchor `the named file usually still exists`. Audit the record for **every** citation of this shape, not only the seven now named — the sibling scan the Required work asks for should treat a stale path as a distinct failure class from a stale line.

**Carried from `retire-the-l4-zero-synchronization-ground-where-other-records-restate-it`, landed as `624b863c`.** That lane verified that no production target profile declares a subgroup width — the sole declaration is the `pub(crate)` fixture `BoundMetalSubgroupDeclaration` in `crates/tiler-build/src/metal_subgroup_declaration.rs`, not re-exported from its crate root. Its finding **does not** contradict this record's surviving conclusion; it supports it, and is a better-shaped reproduction than the emptiness grep this ticket exists to retire. That lane also withdrew the L4 zero-synchronization ground wherever other records restated it. If this record restates that ground too, **it is in scope here only where it is one of the supplied-command repairs**; the retirement itself is that ticket's and it is done.

## Coordinator correction, 2026-08-22 — one claim in my own re-audit above was false, and the worker was right to refuse it

My re-audit section states, of the `feasibility.rs:211` pin: *"confirmed drifted, and the visibility mis-statement is real"*. The retired wording is quoted so the count of it cannot shrink. **There is no visibility mis-statement.** `worker-flash` checked both the working tree and the base with `git show`, found no `pub` claim anywhere near `CapabilityAxis`, and declined to invent a correction for it; the coordinator has since confirmed the same at `2c312826` — a grep for a `pub` claim adjacent to `CapabilityAxis` in that record returns **0**. The record only ever said "seven variants" and made no visibility claim at all.

**How the error was made, because the shape matters more than the instance.** The original ticket Fact read *"the item is `pub(crate)` rather than the `pub` the record states"*. I verified the first half — `CapabilityAxis` genuinely is `pub(crate)` — and carried the second half through into a re-audit as though verifying one half had verified both. That is precisely what AGENTS.md means by replacing a false Fact with a different false claim in new words, committed by the coordinator in the document whose purpose is to stop workers doing it. The worker contradicted the brief with evidence and was right, which is now the standing pattern rather than the exception.

The `pub(crate)` annotation the lane added anyway is free precision and correct; it is not a repair of a mis-statement, because there was none.
