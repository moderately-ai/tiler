---
id: correct-artifact-abi-reconstruction-ownership
title: Correct the artifact ABI's stale reconstruction-ownership sentences
status: done
priority: p2
dependencies: []
related: [carry-reconstructable-kernel-programs-in-the-neutral-envelope, correct-adr-0078-public-module-and-entry-point-facts, carry-the-stage-execution-order-in-the-envelope, correct-adr-0081-loader-gap-list]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, artifacts]
---
Two sentences in `docs/artifact-abi.md` describe the reconstruction question as open. [`carry-reconstructable-kernel-programs-in-the-neutral-envelope`](carry-reconstructable-kernel-programs-in-the-neutral-envelope.md) is `done`, so it has decided rather than owns deciding, and the contract has not caught up.

Found while correcting [`correct-adr-0078-public-module-and-entry-point-facts`](correct-adr-0078-public-module-and-entry-point-facts.md), which declares `contracts/decisions` and could not reach `contracts/artifacts`. ADR 0078 was checked for the same staleness and does not carry it: `grep -n -i "artifact\|decode\|envelope" docs/decisions/0078-name-the-intended-public-extension-seams.md` returns only uses of "artifact plan", the compiler-side plan record, and no claim about the artifact ABI.

**Fact — both sites, reproducible in one line each at base `0f62737`.**

- `grep -n "owns deciding what a decoded envelope must reconstruct" docs/artifact-abi.md` returns the last sentence of the "Deliberate exclusions" paragraph on reconstructable kernel programs. It names a `done` ticket as the present owner of an undecided question.
- Item 3 of "Where the implemented profile is narrower than this contract" records the reconstruction gap as one of the two items that are "open, each with a stated trigger" — `grep -n "Items 1 and 4 have since been closed" docs/artifact-abi.md` locates the sentence that classifies it. If the question is decided, item 3 is a closed item retained as the record of what closed it, in the shape items 1 and 4 already use, and the count sentence above it moves with it.

**What must not change.** The structural fact itself is not stale and is not in question: `tiler_ir::program::KernelProgramBuilder::new` takes a `&SemanticProgram` requiring a frozen semantic registry holding live inferencer implementations, so a decoded envelope proves which program an artifact names and cannot resurrect it. What is stale is the *status* attached to that fact — that a live ticket still owes a decision about it. Read the closing ticket's recorded outcome and represent exactly what it decided; do not restate the decision from the contract's own text, and do not widen a narrowing the contract deliberately states.

Check the same pair of failures elsewhere in the file before finishing: every other sentence naming a ticket as a present owner, and every other item in "Where the implemented profile is narrower than this contract", against that ticket's current status. A `done` ticket cited in the present tense is the pattern, and finding one instance is reason to check its siblings.

Run `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` before completion.

## Outcome

Landed at `83f15ac` on `tkt/correct-artifact-abi-reconstruction-ownership`, from base `2da6f1d`.

### The decision is stated *and* its implementation state is, because both had moved

Both named sites are corrected, and the correction is larger than the ticket anticipated for one reason it could not have known: the decision was not merely recorded, it was **carried out**. [`expose-the-dispatch-record-on-a-decoded-artifact`](expose-the-dispatch-record-on-a-decoded-artifact.md) is `done` and landed the whole projection — `DecodedArtifact` now publishes the named interface, each variant's guard, target profile, feasibility rules and deferred predicates, each entry's stage key, resources, numerical realization, launch contract and preconditions, each binding's `BindingTarget`, kind, element type, address space, access, alignment and accessible-byte expression, and each carried payload's compilation subject, backend symbol, transport slots and object bytes, with `DecodedExpr::evaluate` over any of those expressions. Verified by reading `crates/tiler-artifact/src/program/codec/view.rs` in full rather than from the ticket's report.

So writing item 3 as *decided* was not possible without also correcting the maturity section beside it, which still said a consumer could not reach a manifest row and listed the section-purpose vocabulary as untested against a real compilation. Those two paragraphs contradicted what the corrected item 3 had to say, four lines away.

- **Ownership boundary.** "A decoder must reconstruct shared IR through its checked builders" — a normative `must` that Tom's decision inverted — now states that a decoded envelope is a dispatch record that never reconstructs a verified kernel program, and keeps ADR 0071's *amended* conditional clause verbatim in force for any future path that does yield an IR value. This is the one edit that changes what a reader is **permitted** to do rather than only what they are told; it is Tom's decision of 2026-07-25 being applied, not a new one, and the closing ticket assigned this exact sentence ("must end up true or amended").
- **"Deliberate exclusions".** The `owns deciding` clause is replaced by the decision, its evidence-based elimination of reconstruction, and the accepted cost — with one correction to how the cost is usually stated: a binding's target is **not** asserted by a producer. `ArtifactProgramBuilder::check_bindings` derives it from the program's own stage access (`crates/tiler-artifact/src/program/builder.rs:797`, `binding_target` at `:1088`), and `encode_identity` folds it (`crates/tiler-artifact/src/program/model.rs:1589`). What is weaker than re-derivation is that the proof happened on the writing side.
- **Item 3 and the count sentence.** Items 1, 3 and 4 are closed and item 2 alone is open. Item 3 is marked as having closed *differently*: the other two were closed by widening the implementation to meet the contract, this one by a decision narrowing the contract to what the implementation can do.
- **Future work kept labelled as future work**, not folded into the closure: a multi-stage variant's execution order is still not carried, and a binding addressing part of a value is still unpackageable. Each names a live owner.

### The sibling sweep found one more, by an exhaustive check rather than a grep for a phrase

`tkt list | awk '{print $3}' | sort -u > ids.txt && grep -o -F -f ids.txt docs/artifact-abi.md | sort -u` matches all 345 ticket ids against the file and names its own population, so it cannot report clean by failing to match. Nine ids occur. Two were `done` and cited in the present tense; the rest are `todo` owners cited correctly, one frontmatter `ticket:` provenance field, and one coincidental match on a linked research filename.

The second stale citation is **`prototype-metal-bundle-assembly`**, cited as a `Proposal` whose "decision" about payload identity this contract "deliberately leaves the seam open" for. It decided: a carried payload is content-addressed over its compilation inputs, and the emitted object is opaque with a digest that is integrity rather than identity. Recorded with the cost it accepts — *equal identity implies equal bytes* now holds for the identity-bearing part of an envelope and not for object sections.

That same landing had left three further sentences un-applied, each corrected here because leaving them would have been the identical defect one paragraph away: the required-feature table said "four governed keys" where the build derives five (`tiler.artifact.feature.embedded-payload-code` was missing); "backend payload bytes never enter it" was false of the envelope, since object bytes now travel in a `BackendPayloadCode` section — true only of the neutral manifest and of identity, which is what it now says; and the rejection vocabulary was ten variants short of `ArtifactCodecError`, six from the payload/section work and four from the dispatch-record projection.

The four items of "Where the implemented profile is narrower than this contract" were each re-checked against source. Item 2 is genuinely still open: `crates/tiler-artifact/src/program/codec/model.rs:263-273` maps every governed purpose to `SectionDisposition::Required`, so no optional section exists and the skip mechanism still describes no implemented behaviour.

### Two follow-ups filed rather than absorbed

- [`carry-the-stage-execution-order-in-the-envelope`](carry-the-stage-execution-order-in-the-envelope.md) — the multi-stage gap had **no live owner**. `carry-reconstructable-kernel-programs-in-the-neutral-envelope` held it and closed; `grep -rln "multi-stage" tickets/` names seven files and no live owner.
- [`correct-adr-0081-loader-gap-list`](correct-adr-0081-loader-gap-list.md) — ADR 0081's `implementation_status` bullet states three capabilities the loader now has as absent, and names the same closed ticket as their owner; ADR 0071's boundary paragraph carries the second half of the same pattern. Both are `contracts/decisions`, which this ticket does not hold.

### Verification

`uv run --locked python scripts/docs.py render` (187 records, no catalog change), `uv run --locked python scripts/check_repository.py`, `tkt lint`, `git diff --check`, and `tkt guard` against `2da6f1d` all pass. The gate was chained to the commit with `&&`.
