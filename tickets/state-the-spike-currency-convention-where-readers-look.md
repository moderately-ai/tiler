---
id: state-the-spike-currency-convention-where-readers-look
title: State the spike currency convention where readers look
status: in-progress
priority: p3
dependencies: []
related: [keep-the-ungated-spikes-compiling-against-the-workspace-api, keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, spikes, navigation]
claimed_from: todo
assignee: worker-spikedoc
lease_expires_at: 1787419795
---
## User-visible outcome

`spikes/README.md` states the convention a reader needs before trusting any spike: spikes are outside every gate, they are repaired on demand rather than kept green, and each spike's own README carries a dated currency claim. A reader arriving at the spikes entry point learns this without having to open an individual spike and infer it.

## Why this exists

Filed 2026-08-19 by the coordinator, from `keep-the-ungated-spikes-compiling-against-the-workspace-api`. That lane recorded its decision in the two spike READMEs it owned and **correctly reported rather than reached** for the repository-wide statement, because `spikes/README.md` is `contracts/navigation` and was outside its scopes.

**Fact — the decision the record is missing.** After weighing a workspace census test and a compile-only `make spikes` target, both were eliminated on evidence and the recorded outcome is: spikes are **repaired on demand**, with each spike's README carrying a dated currency claim. Only the two repaired spikes' own READMEs currently say so.

**Fact — the census option was eliminated because a name-based check reports green on the exact defect.** Verified by the coordinator at `b37d2f2b`: `crates/tiler-artifact/src/program/codec/view.rs` carries `pub fn static_shape(self) -> Option<Shape>` at two sites **and `pub fn shape(self) -> &'a Shape`** at a third, `DecodedComponent`'s, in the same file. `fn shape` matches 43 sites across `crates/`. So a census asserting the name still resolves would have passed while both spikes were broken.

**Fact — the compile-only option was eliminated because one of the four breaks is invisible to compilation.** `scalar-cpu-vertical` compiled cleanly and exited 1 at run time with `TargetNumericalContractRefusal … disposition: Unknown`, because `declare_numerics` never declared `ReciprocalTransform`, `ApproximateIntrinsics`, or `MaterializationRounding`. The lane demonstrated this by subject perturbation: deleting `declare_reciprocal_transform` leaves `cargo check` clean and turns `cargo run` red.

**Fact — the breakage was wider than one accessor, which is why the convention matters.** The originating ticket named one API change; the real cause was **six changes across four landings** (`79dc05a1`, `c77aab39`, `bc0b7c0e`, plus the undeclared numerics), producing 5 and 9 compiler errors in the two spikes. A reader who assumes a spike still runs because it once did is making an unsupported assumption, and the entry point should say so.

**Fact — AGENTS.md already carries the adjacent rule and does not carry this one.** It states spikes are run "manually from documented commands so exploratory dependencies do not silently become repository gates". That is the *reason* spikes are ungated; it does not tell a reader what to expect of a spike's current state.

## Correction — 2026-08-22, `worker-spikedoc` at base `ba46f2b2`

Every Fact above was re-audited at this base. Three are verified as written; two are **imprecise in a way that would have put a false number or a false population into the entry point**, and both are repaired here rather than restated.

**Imprecise — "`fn shape` matches 43 sites across `crates/`".** The structural half is verified exactly: `crates/tiler-artifact/src/program/codec/view.rs` carries `pub fn static_shape(self) -> Option<Shape>` at two sites and `pub fn shape(self) -> &'a Shape` at a third, so a name-based census cannot distinguish them and the elimination stands. The number does not. `43` is the count of the **unbounded substring** `fn shape`, which sweeps in `fn shape_product`, `fn shape_environment`, `fn shape_of`, `fn shape_elements`, `fn shaped()`, and others that are not the accessor at all. The word-boundary population is **26**: `grep -roh "fn shape" crates/ | wc -l` returns 43 and `grep -roh "fn shape\b" crates/ | wc -l` returns 26, and both return the same values at `b37d2f2b`, so this is an imprecise pattern rather than drift. The spike's own record already states it correctly — `is still defined at more than twenty sites` in `spikes/target-profiles/scalar-cpu-vertical/README.md` — so the ticket's `43` was the restatement that lost precision, not the source. The entry point states the collision without a count.

**Imprecise — "each spike's README carries a dated currency claim ... Only the two repaired spikes' own READMEs currently say so".** The second clause is false of dated currency claims and true only of the *narrative convention statement*. A dated currency claim is already repository-wide and governed: `last_verified` is carried by **54 of the 62** spike READMEs (`grep -rl "last_verified" spikes --include=README.md | wc -l`), and [`docs/document-metadata.md`](../docs/document-metadata.md#required-common-fields) requires it, at `A reproducible experiment requires nonempty`. Four records additionally carry `verified_at_commit`, which is **not** a field the metadata contract defines (`grep -c verified_at_commit docs/document-metadata.md` returns 0). What only `scalar-cpu-vertical` and `backend-provider-portfolio` carry is the prose statement of the repaired-on-demand decision and its reasoning.

That distinction changed what the entry point had to say, and is the reason it says it. The 2026-08-22 repair moved **neither** spike's `last_verified` (`2026-08-05` and `2026-08-16` respectively) nor its `verified_at_commit`, because the repair deliberately did not rewrite fixtures bounded to their own bases; the currency claim for that repair is a dated body paragraph naming base `3cca5438`. So there are two dated mechanisms with different meanings, the frontmatter date is routinely **older** than the last known-good run, and an entry point telling a reader to "check the dated claim" without saying which one would have sent them to the wrong number. The written convention names both and orders them.

**Verified as written.** The compile-only elimination (`scalar-cpu-vertical` compiled clean and exited 1 with `TargetNumericalContractRefusal`, `declare_numerics` having omitted `ReciprocalTransform`, `ApproximateIntrinsics`, and `MaterializationRounding`; the perturbation is recorded at `Perturbing the subject rather than the assertion` in that README). The breadth Fact (six changes across four landings; `79dc05a1`, `c77aab39`, `bc0b7c0e` all exist and are ancestors of this base; five errors in `scalar-cpu-vertical` and nine in `backend-provider-portfolio`, both stated in the spikes' own records). The AGENTS.md Fact (`manually from documented commands so exploratory dependencies do not silently become repository gates` returns 1; no currency or repair rule is present).

**Not escalated, with the reasoning stated.** The Non-goals ask for a stop if the rule belongs in AGENTS.md as well. It does not, and the reason is the content of the decision rather than a scope preference: "repaired on demand" means there is no standing producer-side obligation to create — a worker who breaks a spike incurs no duty, because repair happens when someone next needs the evidence. Every remaining half of the convention is an instruction to a *reader* deciding whether to trust a number, which is what a portal is for. AGENTS.md already carries the one producer-side fact there is, that spikes are run manually and must not become gates. If Tom reads it otherwise, the AGENTS.md half is one sentence and a separate change.

**Two out-of-scope defects found, neither repaired here.** Both want their own ticket. (1) Nine spike READMEs are unreachable from the experiment catalog, including `spikes/runtime/backend-provider-portfolio/README.md` — one of the two records carrying this very convention — plus `extensions/forkless-physical-provider`, `program-planning/qwen3-checkpoint-f32-inputs`, and the three `apple-targets` sibling probes; the other three are a section portal, a sub-guide, and a results-provenance file, which may be correctly unlisted. Reproduce with the per-README link check over `spikes/README.md`. (2) `verified_at_commit` is used by four records and defined by no contract, and **six** spike READMEs carry no frontmatter at all and therefore no governed identity — all six are among the nine above, so the two defects are largely one population. (The other two of the eight READMEs lacking `last_verified` are `kind: portal` records, which are not required to carry it; that is correct rather than a gap.) Adding a typed field to the metadata contract obliges format and relationship rules across the governed corpus — 277 documents at `9476f4c0` by that document's own measurement — which is not this repair.

## Required work

- Re-audit each Fact above at your actual base and report a per-Fact verdict before editing; re-run the counts rather than trusting them.
- Add the convention to `spikes/README.md` in that document's own voice and convention: spikes sit outside every gate by design, are repaired on demand, and each carries a dated currency claim; a spike is evidence about the base its record names, not about `main`.
- Say what a reader should **do** — run the spike's documented command and read its dated claim — rather than only what is true. An entry-point statement a reader cannot act on is decoration.
- Check whether `docs/README.md` or `docs/research/README.md` (both `contracts/navigation`) route readers at spikes in a way that now needs the same caveat. Report what you found **and what you found clean**.

## Non-goals

Editing AGENTS.md (`implementation/workspace`) — if you conclude the rule belongs there instead of, or as well as, the spikes entry point, **stop and report**: that is canonical repository guidance and its placement is a decision, not a repair. Adding any gate, target, or census over `spikes/` — all three were eliminated on evidence recorded in the originating ticket, and re-litigating them is out of scope. Editing individual spike READMEs, which already carry their dated claims.

## Closes when

`spikes/README.md` states the convention and what a reader should do with it, the sibling navigation documents are checked with both findings and clean results reported, and `make citations` is green.
