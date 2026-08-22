---
id: repair-the-artifact-identity-prose-the-v22-run-falsified
title: Repair the artifact identity prose the v22 run falsified
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, artifact, identity]
claimed_from: todo
assignee: worker-abiprose
lease_expires_at: 1787441390
---
## User-visible outcome

The normative descriptions of what an artifact manifest contains and what its canonical identity folds both name the physical-selection run, so a reader deriving either from prose gets the same answer the encoder gives.

## Why this exists

Found 2026-08-22 by the post-chain multi-lens audit, immediately after the `v21`→`v22` step landed. Both sites are present-tense normative claims that the step falsified, and the audit's wider drift census over `v21`/`v22`/`(21,0)`/`91,945` found **everything else current** — the live source, the ABI identity ledger, the Metal profile authority ledger, and all six spike harnesses. These two are the exception, not a class.

**Fact — the ABI's ordered manifest enumeration omits exactly one element.** `docs/artifact-abi.md`, anchor `deferred predicates, live-device route requirements, executable entries, execution order`. This undated present-tense **Fact** paragraph is *the* ordered enumeration of manifest contents and the physical-selection run is missing from it. The same document's own v22 paragraph places the run "between the feasibility-rule revision and the deferred-predicate run", which the audit confirmed against `push_variant` directly: profile → feasibility key → feasibility revision → the run → deferred.

**Fact — the module doc's identity enumeration omits it too, while its own doctest was updated.** `crates/tiler-artifact/src/program/mod.rs`, anchor `and their entry mappings, and the capability providers`. It lists what canonical identity folds and stops at the selected capability providers; the run **is** folded. That the doctest below it moved and the prose did not is the tell.

**Fact — two sibling sentences are now role-unqualified.** The same ABI paragraph says "the selected capability providers" without saying *lowering*, and `program/mod.rs` at anchor `construction-time authority used to prove` still describes `CompilationEnvironment` in its pre-split single-role framing. Both were exact before the role separation and are ambiguous after it.

## Required work

- Re-audit all three Facts at your base with a per-Fact verdict; each anchor was grepped against the file it names and returns exactly 1.
- Repair each against the **encoder**, not against another document — derive the ordering from `push_variant` and the identity contents from the fold, and say which you read.
- Qualify the role-ambiguous sentences.
- Check the siblings of both sites: any other ordered enumeration of manifest contents, or of what identity folds, in either file or in the identity ledger. Report findings **and** clean results.

## Non-goals

Changing any encoded byte, ordering, or domain; re-deriving the v22 step, which landed gated; and repairing prose outside the two named files unless the sibling scan finds it.

## Closes when

Both enumerations name the physical-selection run in the position the encoder writes it, the role-ambiguous sentences say which role, the sibling scan is reported with its clean results, and `make citations` is green.
