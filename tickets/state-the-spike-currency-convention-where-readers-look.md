---
id: state-the-spike-currency-convention-where-readers-look
title: State the spike currency convention where readers look
status: todo
priority: p3
dependencies: []
related: [keep-the-ungated-spikes-compiling-against-the-workspace-api, keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, spikes, navigation]
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

## Required work

- Re-audit each Fact above at your actual base and report a per-Fact verdict before editing; re-run the counts rather than trusting them.
- Add the convention to `spikes/README.md` in that document's own voice and convention: spikes sit outside every gate by design, are repaired on demand, and each carries a dated currency claim; a spike is evidence about the base its record names, not about `main`.
- Say what a reader should **do** — run the spike's documented command and read its dated claim — rather than only what is true. An entry-point statement a reader cannot act on is decoration.
- Check whether `docs/README.md` or `docs/research/README.md` (both `contracts/navigation`) route readers at spikes in a way that now needs the same caveat. Report what you found **and what you found clean**.

## Non-goals

Editing AGENTS.md (`implementation/workspace`) — if you conclude the rule belongs there instead of, or as well as, the spikes entry point, **stop and report**: that is canonical repository guidance and its placement is a decision, not a repair. Adding any gate, target, or census over `spikes/` — all three were eliminated on evidence recorded in the originating ticket, and re-litigating them is out of scope. Editing individual spike READMEs, which already carry their dated claims.

## Closes when

`spikes/README.md` states the convention and what a reader should do with it, the sibling navigation documents are checked with both findings and clean results reported, and `make citations` is green.
