---
id: correct-artifact-abi-reconstruction-ownership
title: Correct the artifact ABI's stale reconstruction-ownership sentences
status: in-progress
priority: p2
dependencies: []
related: [carry-reconstructable-kernel-programs-in-the-neutral-envelope, correct-adr-0078-public-module-and-entry-point-facts]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, artifacts]
claimed_from: todo
assignee: agent-abi-reconstruction
lease_expires_at: 1785043640
---
Two sentences in `docs/artifact-abi.md` describe the reconstruction question as open. [`carry-reconstructable-kernel-programs-in-the-neutral-envelope`](carry-reconstructable-kernel-programs-in-the-neutral-envelope.md) is `done`, so it has decided rather than owns deciding, and the contract has not caught up.

Found while correcting [`correct-adr-0078-public-module-and-entry-point-facts`](correct-adr-0078-public-module-and-entry-point-facts.md), which declares `contracts/decisions` and could not reach `contracts/artifacts`. ADR 0078 was checked for the same staleness and does not carry it: `grep -n -i "artifact\|decode\|envelope" docs/decisions/0078-name-the-intended-public-extension-seams.md` returns only uses of "artifact plan", the compiler-side plan record, and no claim about the artifact ABI.

**Fact — both sites, reproducible in one line each at base `0f62737`.**

- `grep -n "owns deciding what a decoded envelope must reconstruct" docs/artifact-abi.md` returns the last sentence of the "Deliberate exclusions" paragraph on reconstructable kernel programs. It names a `done` ticket as the present owner of an undecided question.
- Item 3 of "Where the implemented profile is narrower than this contract" records the reconstruction gap as one of the two items that are "open, each with a stated trigger" — `grep -n "Items 1 and 4 have since been closed" docs/artifact-abi.md` locates the sentence that classifies it. If the question is decided, item 3 is a closed item retained as the record of what closed it, in the shape items 1 and 4 already use, and the count sentence above it moves with it.

**What must not change.** The structural fact itself is not stale and is not in question: `tiler_ir::program::KernelProgramBuilder::new` takes a `&SemanticProgram` requiring a frozen semantic registry holding live inferencer implementations, so a decoded envelope proves which program an artifact names and cannot resurrect it. What is stale is the *status* attached to that fact — that a live ticket still owes a decision about it. Read the closing ticket's recorded outcome and represent exactly what it decided; do not restate the decision from the contract's own text, and do not widen a narrowing the contract deliberately states.

Check the same pair of failures elsewhere in the file before finishing: every other sentence naming a ticket as a present owner, and every other item in "Where the implemented profile is narrower than this contract", against that ticket's current status. A `done` ticket cited in the present tense is the pattern, and finding one instance is reason to check its siblings.

Run `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` before completion.
