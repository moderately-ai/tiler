---
id: restore-the-build-trap-the-physical-planner-wildcards-defeat
title: Restore the build trap the physical-planner wildcards defeat
status: done
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

## Coordinator re-audit at `5fd9e1a5`, 2026-08-23 — the original Fact holds exactly; the fourth-site Correction is now stale

`crates/tiler-compiler/src/physical.rs` was rewritten substantially by the gather chain since this ticket was filed. Re-derived here rather than relayed, with each wildcard attributed to its enclosing function **and** to the enum it actually matches on — the attribution matters, because the file carries six `_ => None` arms and only one of them is against these types.

**The original Fact is verified, and the population is exactly three:**

- `_ => None` at `physical.rs:1421`, inside `declared_input_for_verified_access`, matching **`RegionProgram` and `ScalarProgram`**.
- `_ => false` at `physical.rs:4305`, inside `verify_region_output_binding`, matching **`ScalarProgram`**.
- `_ => false` at `physical.rs:4443`, inside `verify_region_output_binding`, matching **`ScalarProgram`**.

**A near-miss worth naming so the next reader does not over-repair:** `_ => None` at `physical.rs:1372` sits inside the *same* function as the first site but matches **neither** enum. A census that greps `_ => None` and attributes by enclosing function — rather than by what is being matched — would report four sites and send a worker to widen an arm that has nothing to do with this trap.

**The `## Correction — 2026-08-22` above is stale and must not be acted on.** It names a fourth site, a `matches!` inside `verify_cooperative_contraction_subject_binding`. **That site no longer exists.** `grep -n "matches!" physical.rs | grep -E "RegionProgram|ScalarProgram"` returns **nothing** at this base, and the function has been rewritten to destructure `ReductionTopology::CooperativeContraction` and `ExecutionBinding::BlockedWorkgroup` through `let … else`, refusing via `intrinsic("request-binding", region.index.id)`. It matches neither enum now. The correction was true when written and is history; the retired wording is left above rather than deleted, and this note is where a reader learns it no longer applies.

**Its underlying point survives and should be kept.** `matches!` carries an implicit false arm and is exhaustiveness-equivalent to `_ => false`, so any census for this trap must look for `matches!` as well as `_ =>`. That it currently finds none is a fact about this base, not a reason to drop the check from the closing condition.

**Both doc anchors still resolve**, so the trap's stated intent is intact: `a wildcard that answers for a program it was never checked against` → 1, and `marking it would force a wildcard arm there` → 1, both in `crates/tiler-ir/src/schedule/model.rs`.

## Worker audit at `92d9a980`, 2026-08-23 — the three `_ =>` Facts hold; the re-audit's `matches!` finding is false and the 2026-08-22 Correction still stands

Per-Fact verdict, each re-derived at `92d9a9803721e6c5e29ce4904f5d1549853e0df3` by reading the enclosing function and attributing the arm **by the type being matched**, not by the enclosing function:

- **Verified.** `_ => None` at `physical.rs:1421`, in the `NormalizedOutputSubject::SerialSum` arm of `declared_input_for_verified_access`, over `match &region.index.program` — so `RegionProgram` **and** the nested `ScalarProgram`.
- **Verified.** `_ => false` at `physical.rs:4305`, in `verify_region_output_binding`, over `match (&normalized.expression, scalar)` — `RecognizedPointwise` and **`ScalarProgram`**.
- **Verified.** `_ => false` at `physical.rs:4443`, in `verify_region_output_binding`, over `match (staged_plan(occurrence), scalar)` — `Option` and **`ScalarProgram`**.
- **Verified.** The near-miss the re-audit names: `_ => None` at `physical.rs:1372` is in the same function but matches `position: usize`. Not a site.
- **Verified.** The `_ =>` form has exactly these three sites. The file carries fifteen `_ =>` arms, counted as `grep -o '_ =>' | wc -l` over occurrences; besides the three above and the `usize` near-miss at 1372, the remaining eleven match `IndexRealizationLaw` (273), `CanonicalValueView` (456), `SourcedExtent` (1669), `LogicalAccess` (2027 and 2699), `ReductionTopology` (3995), `ReductionPass` (4955), `TailPolicy` (5078), and `OperationView` (6261, 6269, 6279) — each read at its `match` head, none over either enum. **All eleven of those subject types are `#[non_exhaustive]`**, checked one by one at their definitions, so every one of those wildcards is required by the attribute rather than chosen. `RegionProgram` and `ScalarProgram` are the only two types this file wildcards that decline the attribute, which is what makes these three arms the anomaly rather than the house style.

**False — `matches!` over these enums is not empty at this base, it holds six, and the check that reported zero cannot find any of them.** The re-audit's command is `grep -n "matches!" physical.rs | grep -E "RegionProgram|ScalarProgram"`. Every one of these six `matches!` invocations wraps, so the enum name never shares a line with the macro name and a line-oriented grep returns 0 on a population of six. Walking each `matches!` span to its balanced closing paren finds them at lines 1317, 4208, 4912, 4934, 4984, and 5083.

**Consequently the `## Correction — 2026-08-22` is not stale.** Its named site — the `matches!` in `verify_cooperative_contraction_subject_binding` — is alive at `physical.rs:5083`. The re-audit is right that the function was rewritten to destructure `ReductionTopology::CooperativeContraction` and `ExecutionBinding::BlockedWorkgroup` through `let … else`; it is wrong that this left the function matching neither enum, because the *program* is still classified by `matches!(&region.index.program, RegionProgram::Numerical { scalar: ScalarProgram::StrictTensorContraction { .. }, .. } if …)` a few lines further down. This is the "anchor fails as absence" hazard `AGENTS.md` names, reached through a line-oriented matcher rather than a rotted line number.

**Perturbation settles it, and the numbers are the record.** With a probe variant added to each enum and `tiler-ir`'s in-crate total maps patched so the consumer is reached at all, at the ticket's base `physical.rs` raised **zero** errors for a `RegionProgram` variant and **one** for a `ScalarProgram` variant — that one being the `SerialSum` binding arm, which was already exhaustive. Every site named above absorbed the new variant silently.

**Residual implicit-arm population, enumerated and deliberately not converted.** Eight refutable `let … else` / `if let` bindings over these enums survive, counted at the branch tip. Three are production code — `physical.rs:4384` (`RegionProgram::Numerical`, else refuse under `request-binding`), and `4666` and `4710` (`ScalarProgram::PointwiseF32`, else delegate to the producer's subject). Five are fixture setup inside the `#[cfg(test)]` module that begins at line 6019: 6630, 6654, 6693, 7032, and 7081. These carry an implicit arm exactly as `matches!` does, but they are refutable bindings rather than wildcards, so the closing condition above — "No wildcard … matches `RegionProgram` or `ScalarProgram`" — does not reach them. Named here so the next reader does not have to rediscover them, and so a decision to convert them is taken rather than assumed. Searched forms: `_ =>`, `_` in a tuple pattern position typed as either enum, bare-identifier bindings in such a position, `matches!` (walked to the balanced close, not line-matched), `if let`, `let … else`, and `is_some_and`.

**No disposition changed.** Every restored arm answers exactly what its wildcard answered; the `None` still reaches `frontier.rs` as `WorkResolutionError::UnknownParameter` and both `false` answers still refuse under `request-binding`.

## Coordinator correction — 2026-08-23: my re-audit was WRONG, and the `## Correction — 2026-08-22` was never stale

**Retract the claim I added above.** My re-audit asserted *"That site no longer exists"* of the `matches!` in `verify_cooperative_contraction_subject_binding`, and retired the 2026-08-22 Correction on that basis. Retired wording preserved. **It was alive the whole time**, at `crates/tiler-compiler/src/physical.rs:5083`, and `worker-buildtrap` caught me.

**How I got it wrong, which is the part worth keeping.** I ran `grep -n "matches!" physical.rs | grep -E "RegionProgram|ScalarProgram"` and got **0**. That grep is line-oriented, and **every one of these `matches!` invocations wraps** — the macro name and the enum name never share a line. A span-aware scan that walks each `matches!` to its balanced closing paren finds **six**, at lines `1317, 4208, 4912, 4934, 4984, 5083`. I reproduced both results before writing this: the line grep still returns 0, the span scan still returns 6.

This is the *exact* hazard AGENTS.md records and that I have briefed on repeatedly this session — a matcher failing on a multi-line construct and **reading as absence**. I committed it while performing a re-audit whose whole purpose was to catch stale claims, and the failure direction was the dangerous one: I retired a true correction. **Had the worker trusted me, six live defeating sites would have survived and this ticket would have closed green over them.**

**What I did get right, and it is narrow:** `verify_cooperative_contraction_subject_binding` *was* rewritten to destructure `ReductionTopology::CooperativeContraction` and `ExecutionBinding::BlockedWorkgroup` through `let … else`. That rewrite simply left the program classification a few lines below untouched. A true observation about one part of a function is not evidence about another part of it.

**The durable lesson for any census over Rust source:** `grep` answers about *lines*, and `matches!`, `let … else`, wrapped `match` scrutinees, and `///` comments are all multi-line. A census whose unit is the line cannot see them, and its silence is indistinguishable from absence. Use a span-aware scan, or state that the count is line-scoped and therefore a floor.

**The expanded scope is accepted.** The lane fixed the three briefed `_ =>` arms in `243c265f` and the six `matches!` plus one tuple-position `_` in `bf779a83`, deliberately separated so I could take the narrower commit. I am taking both: this ticket's Closes-when says *no* wildcard matches either enum, the 2026-08-22 Correction names `matches!` as exactly such a wildcard, and a second lane over the same functions would be pure overhead. Verified at the merge: a span-aware scan of `physical.rs` now finds **0** `matches!` spanning either enum, and `git grep BuildTrapProbe` returns nothing.

**Its perturbation evidence is what a build trap should look like.** With a probe variant added to `RegionProgram`, the base tree errored **only** at `frontier.rs:916` — `physical.rs` was completely silent, which is the defect stated as a measurement. After the fix it errors at four `physical.rs` sites. For `ScalarProgram`, base gave one `physical.rs` site; now eight. It also reproduced the docs' own recorded method — marking `ScalarProgram` `#[non_exhaustive]` — and watched the same eight sites redden.

**Both enum docs were overclaiming before this landed, not after.** `RegionProgram`'s doc says `physical.rs` maps the program totally; that was **false** there (zero errors). The docs described a guarantee the code had stopped providing, which is precisely what this ticket existed to restore.
