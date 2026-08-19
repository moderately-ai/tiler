---
id: repoint-the-optimizer-contract-s-request-module-citations
title: Repoint the optimizer contract's request-module citations
status: todo
priority: p3
dependencies: []
related: [repair-the-navigation-and-contract-docs-the-audit-falsified, re-derive-the-contraction-fusion-role-rationale-after-the-key-replacement]
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, optimizer, citations]
---
## User-visible outcome

Every symbol `docs/compiler/optimizer.md` cites at `crates/tiler-compiler/src/request.rs` resolves to the module that actually defines it, so a reader following a citation lands on the code the sentence describes.

## Why this exists

Filed 2026-08-19 as a mapped remainder of `repair-the-navigation-and-contract-docs-the-audit-falsified`. That lane found the sites but could not repair them: `docs/compiler/**` is `contracts/optimizer`, which it did not hold — **the ticket that dispatched it listed this file as if it were in scope, which was the coordinator's error, not the worker's.**

**Fact — the path survives but defines nothing, so a path check cannot catch this.** `crates/tiler-compiler/src/request.rs` still exists beside its new `request/` submodules (`authority.rs`, `budget.rs`, `contract.rs`, `elementwise.rs`, `folded.rs`, `graph.rs`, `normal_form.rs`, `recognize.rs`, `refusal.rs`, `structural.rs`, `subject_budget.rs`, `subject.rs`, `tests.rs`, `verified.rs`, `verify.rs`), but it is a module spine that **defines no function at all**. So every citation naming a *symbol* at that path is stale while the path itself resolves — which is why `make citations` stays green over it and why this needs reading rather than a check.

**Fact — the population is eight lines in this file.** `grep -c "src/request\.rs" docs/compiler/optimizer.md` returns 8 at this base. The sibling lane's census over its own scopes found 17 occurrences across 6 documents and repaired 12, leaving 5 as dated history or correct module-root links — so expect a similar split here rather than eight mechanical repointings.

**Verified relocations, offered so they are not re-derived:** `a_compiled_plan_does_not_fold_a_bound_extent_value` → `crates/tiler-compiler/src/request/tests.rs`; `normalize_contraction` → `crates/tiler-compiler/src/request/folded.rs`.

## Coordination

Exclusive `contracts/optimizer`. At filing time that scope is held by `replace-the-serial-sum-contributor-fields-with-the-exhaustive-source`, which edits `docs/compiler/optimizer.md` to rename the `reduction-contributor-materialization` reason key to `reduction-contributor-depth`. **Re-audit at your actual base**: that landing rewrites part of this file, and some citations here may already have moved with it.

## Required work

- Re-audit the Facts above at your actual base and report a per-Fact verdict before editing; re-run the count rather than trusting the eight.
- For each citation, locate the named **symbol** and repoint at its defining submodule. Where the module root genuinely *is* the subject (a link to the module as a module, rather than to a symbol in it), leave it and say so — that distinction is the whole judgement in this ticket.
- Leave dated historical citations alone; repointing a `git show <sha>:path` style reference at a current file destroys the check it offers.
- Cite by searchable anchor rather than by path alone, and run each anchor's grep against the file its citation names before writing it.
- Report the count before and after, plus which sites were deliberately left and why.

## Non-goals

Any source change, any other document, and re-litigating the module split. The stale `reassociation-permitted: false` reasoning in `crates/tiler-compiler/src/fusion_legality.rs` is a separate ticket's subject.

## Closes when

Every symbol citation in `docs/compiler/optimizer.md` resolves to its defining module or is deliberately left with a stated reason, the before/after counts are quoted, and `make citations` is green.
