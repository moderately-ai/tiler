---
id: accept-adr-0102-conversion-pair-decomposition
title: Accept or reject the conversion-pair decomposition ADR
status: awaiting-decision
priority: p2
dependencies: [land-the-conversion-pair-decomposition-adr]
related: [test-the-directional-conversion-pair-generalization]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, numerics, conversion, decision, needs-tom]
---
## User-visible outcome

[ADR 0102](../docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md) moves from `proposed` to `accepted`, or is rejected.

**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. Its permanent status is `awaiting-decision` — a parked state `tkt ready` excludes and that never satisfies a dependent; an agent that finds it in `todo` should set it back and do nothing else. It is filed in the shape [`accept-adr-0099-elementary-family-projection`](accept-adr-0099-elementary-family-projection.md) established, because the same asymmetry applies: landing a proposed record is a completed outcome, so the carrier ticket goes terminal the moment the file exists and cannot distinguish "written" from "decided".

**This node carries the scopes its own acceptance sweep needs, and no more.** The sweep touches `docs/decisions/[0-9]*.md` (`contracts/decisions`) and both catalog views in `docs/decisions/README.md` (`contracts/navigation`). Two sweep items reach further and are named under "What acceptance does" rather than silently scoped for here, because a node that pre-declares a scope for work Tom has not authorized is a claim on the board rather than a plan.

## What is being decided

**The *shape* a conversion family takes, over the whole class, and no specific pair's contract.** ADR 0102's five clauses say a family is keyed by the ordered `(source, destination)` pair together with a mode; that its owed field set is *derived* from containment predicates over the pair and never declared on it, with a contract carrying an unowed field or missing an owed one refused at construction; that "widening" and "narrowing" are not the discriminant and are not family names; that one keyed family parameterized by source, destination, and mode as free attributes is refused; and that two candidate families merge only when both constructibility and legibility hold, with field-set disjointness demoted from criterion to symptom.

**Why it is worth deciding before anything is registered.** No conversion key exists in any direction. [The delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md) records that "whichever shape the first registered conversion takes becomes the precedent every later pair is read against", so the shape gets fixed by the first registration whether or not it was decided — and identities are cheap to mint and expensive to move. Accepting makes the derived-field rule the standing answer; rejecting returns the question to per-pair judgment and leaves the `bf16`/`f16` finding as an observation rather than a rule.

**Three items are worth Tom's attention specifically.**

1. **The record refutes the closure test its own question fixed, rather than reporting its verdict.** `RQ-OP-04` said a non-disjoint second pair means "the parameterized form wins" — and the parameterized form is named and rejected by ADRs 0010, 0041, and 0091. The record's response is to replace the test with constructibility-and-legibility. That is a defensible move and it is also the record's most contestable one, because it declines a stated test rather than passing it.
2. **The population is five layouts, not the recognized catalog.** The `FN` and `FNUZ` float rows are held out behind two acquisition requests (IEEE Std 754-2019; OCP OFP8 v1.0 and OCP MX v1.0), and the record argues they would add infinity-mapping and signed-zero-mapping field classes that strengthen the finding. That argument is an **Inference** from the taxonomy's layout column, not a count.
3. **Clause 2 says what a *validator* must do before any validator exists.** "Refused at construction" is a normative obligation on a typed conversion-contract vocabulary that has no Rust spelling, no key, and no evaluator. Accepting the model does not approve any of those; it fixes what they will have to mean.

## Provenance

**No acceptance has been relayed at any point, and this node is the first place one can be.** [`test-the-directional-conversion-pair-generalization`](test-the-directional-conversion-pair-generalization.md) derived the answer and wrote the body as a draft, labelling it **Proposal** throughout; [`land-the-conversion-pair-decomposition-adr`](land-the-conversion-pair-decomposition-adr.md) transferred it byte-identically and relayed none. **Nothing has been released on the record and no contract sentence has been rewritten under it**, which is what keeps the rollback below cheap.

**It releases nothing, and that is checked against the board rather than asserted.** No ticket other than this one depends on the carrier: `grep -rn "^dependencies:.*land-the-conversion-pair-decomposition-adr" tickets/ | grep -v accept-adr-0102-conversion-pair-decomposition` reports no match over a `tickets/` population of 1033 files. The excluding form is the one to use — without it this node's own `dependencies:` line is the single hit, a check that cannot say no.

## What acceptance does and does not do

Acceptance flips `decision_status` to `accepted` on ADR 0102 and updates both catalog views in [the decisions index](../docs/decisions/README.md) — the theme row under "Numerical operations" and the chronology row — from `proposed` to `accepted`.

**Two further items belong to the acceptance and reach scopes this node does not hold.** They are named here so the sweep is a plan rather than a discovery, and each is a separate claim when the time comes.

- **[Numerical semantics](../docs/numerical-semantics.md) (`contracts/numerics`).** Its widening-and-narrowing section is correctly scoped today and needs no correction *while the record is proposed*; on acceptance the derived-field rule becomes normative and the section gains it, because under [the metadata contract](../docs/document-metadata.md)'s `mixed`-contract rule only accepted-ADR-derived invariants are normative. The contract's `evidence` array also does not yet list `tiler.research.numerics.conversion-family-decomposition-across-pairs`, and ADR 0101's acceptance is the precedent for adding an evidence record's id there in the same sweep.
- **[The minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) (`research/program-planning`).** It says "`RQ-OP-04` leaves conversion's family decomposition open". That is defensible while ADR 0102 is proposed and plainly wrong once it is accepted; the fix is one clause and the route half of the sentence — that the family classification does not move — stays true either way.

**It moves no research record's frontmatter.** [Conversion family decomposition across pairs](../docs/research/numerics/conversion-family-decomposition-across-pairs.md) carries `disposition: pending` and no `adopted_by` field at all — `grep -c '^adopted_by' docs/research/numerics/conversion-family-decomposition-across-pairs.md` reports `0`, against `grep -c '^informs'` on the same file reporting `1`. On acceptance those two fields *do* move together, to `adopted` and `["ADR-0102"]`, which is `research/numerics` and also not held here.

**It implements nothing and changes no public surface.** It registers no conversion key, chooses no Rust spelling or version, moves no encoding and no pinned identity, and moves no `docs/dtype-support.md` cell. The `Cast and convert` row of the [roadmap matrix](../docs/roadmap.md#operation-family-support-matrix) stays where it is and its cell already says the rung moves on neither outcome.

## Rollback, kept cheap on purpose

If the record is rejected after being accepted, the repair is one field and two catalog rows: `decision_status` back to `proposed`, both catalog rows back to `proposed`, and this node back to `awaiting-decision` — plus reverting whichever of the two out-of-scope sweep items had landed. Nothing else moves.

Rejecting the record outright is a deletion of one file, its two catalog rows, this node, and the reciprocal prose annotation on the research record's drafted-body heading. It leaves the derivation exactly where it is, as an answer to `RQ-OP-04` that no decision stands on.

## Closes when

Tom accepts or rejects it.
