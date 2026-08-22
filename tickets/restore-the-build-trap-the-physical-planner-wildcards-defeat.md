---
id: restore-the-build-trap-the-physical-planner-wildcards-defeat
title: Restore the build trap the physical-planner wildcards defeat
status: todo
priority: p3
dependencies: []
related: [offer-the-tiled-contraction-alternative-in-physical-planning]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, exhaustiveness, maintainability]
---
## User-visible outcome

Adding a variant to `RegionProgram` or `ScalarProgram` fails the `tiler-compiler` build, as those enums' own documentation says it will, instead of reaching a wildcard that answers for a program nobody checked.

## Why this exists

Found 2026-08-22 by the post-chain multi-lens audit's wildcard census.

**Fact — two enums deliberately decline `#[non_exhaustive]` for the express purpose of breaking this consumer's build.** `crates/tiler-ir/src/schedule/model.rs` documents `RegionProgram` at the anchor `a wildcard that answers for a program it was never checked against`, saying **"Do not add `#[non_exhaustive]`"** because `physical.rs` and `frontier.rs` map the program totally from outside the crate and a third computation class must stop those builds. `ScalarProgram` carries the sibling claim at the anchor `marking it would force a wildcard arm there`, and records that this was **verified by marking it and watching that consumer fail to compile** — so the guarantee was perturbation-tested when written.

**Fact — three wildcards in that exact consumer defeat it.** `crates/tiler-compiler/src/physical.rs` matches those two types with `_ => None` in `declared_input_for_verified_access` and `_ => false` twice in `verify_region_output_binding`. A new variant reaches a wildcard rather than a build error, which is precisely the outcome both doc blocks exist to prevent.

**This is not a correctness hazard today, and the ranking says so.** The audit traced all three dispositions and corrected its own census's severity claim: both `_ => false` arms are verification *predicates*, so `false` means "does not carry" and the binding is refused; the `_ => None` reaches `frontier.rs` and becomes `.ok_or(WorkResolutionError::UnknownParameter(name))`, a typed refusal. Every path is fail-closed. **What is lost is a deliberately engineered compile-time trap, degraded to a runtime refusal** — a design-intent erosion, not a wrong answer. Ranked p3 accordingly.

**Pre-existing and outside the audited span, recorded so it is not mistaken for regression.** `physical.rs` does not appear in `git diff e20ed09e..09474993`.

## Correction — 2026-08-22: there is a fourth site, and this ticket's closing condition would not have caught it

The post-chain audit found a **fourth** wildcard defeating the same build trap, and it is not spelled `_ =>`. In `crates/tiler-compiler/src/physical.rs`, `verify_cooperative_contraction_subject_binding` writes `matches!(&region.index.program, RegionProgram::Numerical { scalar: ScalarProgram::StrictTensorContraction { .. }, .. } if ...)`. **`matches!` carries an implicit false arm**, so it is exhaustiveness-equivalent to `_ => false` and defeats the trap exactly as the three named sites do.

**This matters more than one extra site.** The Facts above enumerate exactly three `_ =>` sites and the closing condition reads "No wildcard … matches `RegionProgram` or `ScalarProgram`". A worker repairing the three, watching the perturbation redden there, would close this ticket **with the fourth still standing** — a green close over a live gap. Amended here rather than left for that worker to discover.

**Search for the pattern, not the spelling.** `matches!`, `if let` without an else arm, and `is_some_and` over a match all carry implicit false arms. Enumerate them at your base and say which forms you searched for.

## Required work

- Re-audit both Facts at your base with a per-Fact verdict. Read all three wildcard sites in full; the audit's own count of the wider census failed to reconcile, so trust nothing here without reading.
- Replace each wildcard with an exhaustive match over the current variants. Where a group genuinely shares a disposition, name the variants explicitly rather than collapsing them — the point is that adding a variant must be a build error at this site.
- **Perturb the subject the way those doc blocks did**: add a variant to each enum on a scratch tree and show `tiler-compiler` failing to compile, quoting the error. Do this separately for `RegionProgram` and `ScalarProgram`; one perturbation reddening both cannot show which site is load-bearing.
- State explicitly whether any behaviour changes. **Expected: none** — every current variant should keep its present disposition — but derive that rather than assume it, and if a variant's disposition turns out to differ from what the wildcard gave it, **stop and report**, because that is a live defect rather than a maintainability repair.

## Non-goals

Adding `#[non_exhaustive]` to either enum — the documentation forbids it for a stated reason. Changing any refusal's runtime behaviour. The `work_span` arm and its own wildcard, which belong to [`offer-the-tiled-contraction-alternative-in-physical-planning`](offer-the-tiled-contraction-alternative-in-physical-planning.md).

## Closes when

No wildcard in `crates/tiler-compiler/src/physical.rs` matches `RegionProgram` or `ScalarProgram`, a variant added to either is watched failing the build with its error quoted, each enum perturbed separately, and no current variant's disposition has changed.
