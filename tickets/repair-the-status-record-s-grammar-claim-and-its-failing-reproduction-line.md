---
id: repair-the-status-record-s-grammar-claim-and-its-failing-reproduction-line
title: Repair the status record's grammar claim and its failing reproduction line
status: done
priority: p1
dependencies: []
related: [correct-the-roadmap-s-milestone-0b-inline-composition-claim]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## A record whose own reproduction block fails on the line that carries its evidence

**Historical problem statement (as filed; standing prose on `docs/status.md` was repaired 2026-08-07 — see Outcome).** At filing, `docs/status.md` claimed of the inline frontend: "**What they do not carry is a grammar**… region syntax, expansion, symbol binding, runtime value adaptation, and the complete cold/warm inline AOT and embedding workflow **all remain open**."

`crates/tiler-macros/src/` holds `grammar.rs`, `region.rs`, `binding.rs`, `delivery.rs`, `numerics.rs`, `aot.rs`, `retention.rs`, `eviction.rs`, `preflight.rs`, `family_cfg.rs`, and `cache_root.rs`. Nine `crates/tiler/tests/facade/fail/*.stderr` goldens exist; not all are grammar diagnostics (region/statement, expansion-span, family-selection, and `#[cfg]`-retained toolchain fixtures also sit in that directory).

**At filing the record shipped a five-line reproduction block whose third line failed:** it tested for `crates/tiler/tests/facade/fail/undefined_grammar.stderr`, a compile-fail golden that no longer existed. And `status.md` itself called that line "the checked-in compile-fail golden behind '`tensor!` has no grammar': it is the **evidence that rejecting undefined input is a tested behaviour rather than a description of one**."

So the sentence's stated evidence was a file that was absent, and **the conclusion fell with its ground**.

## Read before repairing, because the correction is easy to overshoot

The claim is not simply inverted. Establish what the frontend *does* carry and at what maturity — `AGENTS.md` keeps reserved type, architectural seam, implemented support and tested guarantee distinct, and the recurring defect in this repository is a correction that overshoots in the opposite direction from the text it replaces.

Several of the listed items have moved independently since that sentence was written, so **check each of the five separately** rather than treating "all remain open" as one claim to flip. Note the retention read-back landed and is a **labelled draft awaiting Tom's acceptance** under ADR 0075 — do not describe it as accepted.

**Repair the reproduction block too, not only the prose.** A block whose lines are not all runnable is the same defect one layer down; run every line you leave in it and say so.

## Closes when

Every line of the reproduction block runs and passes on a clean tree; the grammar claim states what is carried and at what maturity, with each of the five items checked separately; and no sentence cites an absent file as its evidence.

## Outcome — 2026-08-07

Close conditions met on `docs/status.md`. Work landed at `5fadb801` (merge `886e9dd3`); ticket closed at `5087bda1` (status flip only).

**Standing grammar absence claim retired.** The wording "What they do not carry is a grammar" / "all remain open" is no longer a standing fact; it is quoted only inside a 2026-08-07 dated correction on the inline developer-experience bullet. The five items are recorded separately at distinct maturities: region syntax (implemented + tested; grammar not Tom-accepted), expansion and symbol binding (implemented + out-of-tree tested), runtime value adaptation (reviewed draft), cold/warm inline AOT (implemented E2E for one measured family; retention read-back held as labelled draft awaiting acceptance).

**Reproduction block repaired.** Third line is now `test -f crates/tiler-macros/src/grammar.rs && test -f crates/tiler/tests/facade/fail/region_syntax_diagnostics.stderr`. Retired third line named `undefined_grammar.stderr` (present at facade admission `b5e22ed4`, removed by grammar landing `5c261f9d` without updating status.md). All five current lines were run from the repository root under `set -e` and pass; `undefined_grammar.stderr` remains absent.

**Retention bound preserved.** Retention read-back is a labelled draft awaiting Tom's acceptance under ADR 0075; acceptance surface is `accept-the-retention-read-back-s-caller-visible-boundary` (`awaiting-decision`), not accepted public surface.

**Roadmap remainder filed and closed.** Residual Milestone 0B prose was split to `correct-the-roadmap-s-milestone-0b-inline-composition-claim` (same repair commit family); that sibling is `done` with its own Outcome.

## Fact audit — 2026-08-10

Ticket-record hygiene only (product close already delivered). Added `related: [correct-the-roadmap-s-milestone-0b-inline-composition-claim]`; marked problem body historical so "line 3 fails" is not read as current; recorded Outcome hashes and the five-item / four-maturity split. Re-verified at this tree: modules listed above present; nine fail `*.stderr` goldens present; five-line block form matches status.md; retention acceptance ticket still `awaiting-decision`; standing "all remain open" only inside dated-correction quote on status.md.
