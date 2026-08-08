---
id: correct-the-flat-consumer-crate-comment-in-the-inline-dispatch-spike
title: Correct the flat consumer-crate comment in the inline dispatch spike
status: done
priority: p3
dependencies: []
related: [correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand, accept-the-public-route-requirement-answer-boundary]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, spikes, contracts]
---

The last surviving site of the retired flat claim. Reported by two workers independently, neither able to edit it in scope.

## Facts

**Verified at `cc667626` by the worker.** `spikes/runtime/inline-dispatch/Cargo.toml` carried a comment reading "The one crate a Tiler consumer declares". `docs/architecture.md` says "the one crate an inline-frontend consumer names", and the same paragraph explicitly carves out consumers that construct and compile arbitrary semantic programs. The flat form states a monopoly the contract refuses.

**Corrected 2026-08-08: this ticket previously rendered the `architecture.md` anchor as `an **inline-frontend** consumer`, with emphasis markers the source does not carry.** `grep -c 'the one crate an \*\*inline-frontend\*\* consumer names' docs/architecture.md` returns 0 while `grep -c 'the one crate an inline-frontend consumer names' docs/architecture.md` returns 1 — the exact fail-as-absence shape `AGENTS.md` warns about. Use the unemphasised form.

**Verified: the correct form already exists nearby.** `crates/tiler/src/lib.rs` reads "This crate is the only one an inline-frontend consumer declares" — `grep -c 'the only one an inline-frontend consumer declares' crates/tiler/src/lib.rs` returns 1. The repair follows that house model rather than inventing a second phrasing.

**Fact — this spike is the reason the distinction matters.** `spikes/runtime/inline-dispatch` delivers through `tiler::tensor!` and then drives Metal by hand. It is therefore an inline-frontend consumer that also dispatches, which places it inside the sentence's population and squarely on the open boundary question. A comment here asserting the flat monopoly is wrong in the one file best positioned to show why.

## Why p3

It is a comment in a spike manifest, reachable by no gate and read by few. It is filed rather than fixed because a correct claim about a contract belongs in the corpus even where the cost of being wrong is low — and because leaving one known-false copy behind is how a retired claim gets re-adopted later by someone grepping for prior art.

## What closes this

The comment restated to match `docs/architecture.md`, cited by **searchable anchor** rather than line number. Check the rest of the spike for the same flat form before closing, and **name the count either way** so a clean result is distinguishable from an unexamined one.

Do not expand this into a review of what the spike does; the manifest comment is the whole scope. If reading it surfaces a substantive claim about dispatch that is also wrong, file that separately rather than folding it in.

## Census of the same flat form elsewhere in the spike — 2026-08-08

**Fact — two sites, both corrected.** The population is `spikes/runtime/inline-dispatch/{Cargo.toml,README.md,src/*.rs}` (`Cargo.lock` excluded; it carries no prose). Counted from that directory after the corrections: `grep -rn consumer Cargo.toml README.md src | wc -l` = 85 lines (95 occurrences by `grep -roh`), `grep -rn declares Cargo.toml README.md src | wc -l` = 20 lines, of which 2 are the corrected facade sites. Each was read in place, together with every hit for `one crate`, `only crate`, `only .*Tiler`, and `sole`.

1. `Cargo.toml`, the ticket's target: "The one crate a Tiler consumer declares" → "The one crate an inline-frontend consumer declares", plus a clause naming the contract's carve-out so the qualification is not re-flattened by a later reader.
2. `src/adapter.rs` module header: "the property the facade exists to have: a consumer declares one dependency" → "an inline-frontend consumer declares one dependency". Same defect, inverted phrasing — an unqualified quantifier over the whole consumer population asserting a one-dependency monopoly the contract refuses. Weaker than the manifest's, because the file's own first line scopes it with "written against `tiler` alone", but it is the same claim and it is inside `research/runtime`.

Non-instances checked and rejected: `README.md`'s "Two consumers, one crate, one adapter" (the spike's own two binaries sharing one crate, not a facade claim); the remaining 18 `declares` lines, every one about what an artifact, region, route, or plan declares.

**The flat form stays greppable tree-wide, deliberately.** After this change `grep -rn "one crate a Tiler consumer" .` still returns hits, and every one of them is a ticket quoting the retired wording rather than asserting it: this file, `tickets/clarify-the-inline-frontend-facades-consumer-scope.md` (`status: done`, recording the already-repaired `crates/tiler/src/lib.rs`), and `tickets/correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand.md`, which quotes it while directing this work. No hit is under `crates/`, `docs/`, or `spikes/`. This is the hazard `AGENTS.md` names — a hit is evidence the string is present, not that the claim stands — so a later reader must check whether a hit sits inside a correction before "restoring" anything. Deliberately not counted here: any count would have to include this sentence and would go stale the moment another correction quotes the phrase. Filter on path instead.

Neither correction touches the **dispatch** axis. The contract qualifies by *frontend*; whether an inline-frontend consumer that also dispatches may reach a backend answer surface is the open public-boundary question reserved to Tom, recorded in `spikes/runtime/inline-dispatch/README.md` under "That surface is a public-boundary question for Tom rather than something to work around locally". Nothing here settles it.
