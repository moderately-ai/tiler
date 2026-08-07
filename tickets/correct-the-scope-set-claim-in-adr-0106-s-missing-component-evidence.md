---
id: correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence
title: Correct the scope-set claim in ADR 0106's missing-component evidence
status: done
priority: p3
dependencies: []
related: [survey-what-belongs-in-the-conformance-crate, record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, conformance, correction]
---
## User-visible outcome

[ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md)'s Context states its evidence for the missing-component claim correctly, so a reader checking the argument finds it holds rather than finds it self-contradicting in the same sentence.

## The defect

Filed 2026-08-07 by [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md), which was asked to test the claim and read the tickets.

ADR 0106's Context reads:

> **Fact — the evidence that a component is missing rather than a file being homeless is that the work does not share a scope set.** [...] counts five open conformance tickets and no two share one: three are `implementation/compiler`, one adds `implementation/reference`, `contracts/numerics`, and `research/scheduling`, and one adds `implementation/runtime`.

**The clause "no two share one" is falsified by the clause after it, and by the tickets.** Read from the tickets on `main`:

- `route-the-contraction-conformance-through-the-staged-oracle` — `scopes: implementation/compiler`, `shared: project/tickets`
- `route-the-index-region-conformance-through-the-staged-oracle` — `scopes: implementation/compiler`, `shared: project/tickets`
- `retain-the-selected-semantic-candidate-for-the-conformance-oracle` — `scopes: implementation/compiler`, `shared: project/tickets`

Three of the five carry **identical** scope sets. Reproduce with `tkt show <id>` on each.

## Why it is worth correcting rather than ignoring

The sentence is the ADR's stated ground for admitting a workspace member. A reader auditing the admission finds the evidence self-contradicting at the point it is offered, which is exactly the failure `AGENTS.md` names when it says comments and examples are claims about current behaviour.

## The correction is a rewording, not a retraction — the conclusion survives

What is true, and what the surrounding sentences already say, is that the five tickets **span** five distinct scopes across three crates and two documentation contracts with no scope common to all of them, and that the three sharing a set are three tickets about **one compiler-resident file** rather than three independent pieces of scattered work. The survey found the underlying claim understated rather than overstated: `grep -ril conformance tickets/` returns 289 files of which 283 are tickets, 76 of those are non-terminal, and the crates holding conformance-named source are `tiler-ir`, `tiler-reference`, `tiler-compiler`, and `tiler-conformance`.

## Closes when

The Context's scope-set sentence states what the five tickets' scopes actually are, the three-identical-sets fact is stated rather than contradicted, and the conclusion it supports is preserved with its rationale intact per `AGENTS.md`'s supersession rule.

## Outcome — delivered 2026-08-07 at `68a607b3`

**The brief's numbers did not reproduce, and that is itself the finding.** The population read **289 tickets, 80 non-terminal** rather than the survey's 283/76 — the tree moved seven in a few hours. The three identical scope sets reproduced exactly, byte-identical across the three tickets. The worker wrote the counts into the record **pinned to a commit and with the exact command**, and recorded both the old and the new pair, so the number reads as something that moves rather than as a threshold. It also caught that "five *open*" is now stale, since the BF16 vertical is `done`.

**It corrected a claim the coordinator had repeated.** "Reached only by `cargo run`" is imprecise: `proof.rs` holds 44 `#[test]`s under `#[cfg(test)]` and its `[[bin]]` carries `test = true`, so those **are** in `make full` — ADR 0106's own Context says as much two paragraphs up. What is ungated is the device-reaching `run()` narrative, and the test module's own header states it "reaches no device". Transcribing the compressed claim would have made the record contradict itself.

**Verified versus trusted was separated in the record itself**, which is the discipline this repair existed to restore. Verified by reading: the 8,159 line count; that `proof.rs`'s declared dependency row is **identical** to `tiler-conformance`'s, read from both manifests; the two dispatch paths and the bit-exact reference comparison; and that the whole `Makefile` contains exactly one `prototype-run` mention, two Clippy excludes, and no target invoking the binary. Taken on the survey's word and **labelled as such**: that this run holds the corpus's only device observation of a permitted reassociated answer and its only executed match against a retained device digest — both universal claims over the corpus, not exhaustively checked.

**The repair shape was chosen deliberately.** The false clause is **substituted** rather than dated-beside, because unlike ADR 0077/0088 — which hold what was true at acceptance — this clause was **never true at any commit**. That follows ADR 0079's precedent for a wrong stated reason with a surviving conclusion, and the dated `Correction — 2026-08-07` note quotes the original and says why.

**Catalogs checked and correctly not moved:** both generated blocks in `docs/decisions/README.md` carry only title, status and links, and the diff touches no frontmatter key or heading — confirmed by grepping the diff rather than by inspection. All nine ADR-0106 references in `docs/architecture.md` describe what the record *decides* rather than restating its evidence.

**Delta rule confirmed by the coordinator against the merge's own file list:** exactly one file, `docs/decisions/0106-…md`, touching none of the build-configuration set, so it carries the latest green gate with `tkt lint` rerun.

### Reported, not fixed — and one is larger than this ticket

Three tickets carry the same false scope-set claim, one of them with a *differently* wrong attribution. Left for [`refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew`](refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew.md).

**And ADR 0106 is materially stale beyond this ticket**, which the worker found while reading and correctly did not touch. The record describes the empty crate in unpinned present tense — "holds no items at all", "contains no code at all", "inherits the workspace lint table unchanged" with `forbid` standing — and the crate now has 13 source files, device dispatch, two named unsafe sites and `unsafe_code = "deny"`. `docs/architecture.md` carries the same at three lines. **Independently verified by the coordinator** before filing. The repair there is the *opposite* of this one: those statements were true at acceptance, so they are dated rather than substituted.
