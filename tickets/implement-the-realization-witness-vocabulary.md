---
id: implement-the-realization-witness-vocabulary
title: Implement the realization witness vocabulary
status: done
priority: p2
dependencies: []
related: [accept-the-realization-witness-surface, enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle, compose-a-declared-reduction-topology-into-a-semantic-program-evaluation]
scopes: [implementation/ir, research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, numerics, witness]
---

## The accepted surface this implements

**Tom decided 2026-08-06 (recorded on [`accept-the-realization-witness-surface`](accept-the-realization-witness-surface.md)):** Item A — `RealizationWitness` in `tiler_ir::schedule`, aggregated by `RealizationWitness::of(&VerifiedScheduledRegion)` — and Item B — `UnpinnedFreedomSite`, a refusal enum with no `Conforms`-shaped arm — are accepted as drafted in [the freedom-sites record](../docs/research/reference/plan-freedom-sites.md) Part 7.2. Item C is **redirected to the plain-scalar form**: the reference's evaluation entry points keep taking plain scalars (`strict_partitioned_sum_under` is the pattern) and the aggregation lives in `tiler_ir` alone. `tiler-reference` must not gain a dependency on any plan structure — that constraint is part of the decision, not a style preference.

## The work

- Implement A and B against the record's Part 7.2 drafts, checking each drafted field against the enumeration's Part 2/Part 3 site list at the current tree rather than inheriting (the record was written at `c335bb5b` and lines have moved).
- The witness covers the evaluable-witness sites the enumeration classifies (the six a reference path can evaluate), refuses the unevaluable and undeclared sites through B by name, and does not paper over the mirror sites — a contract mirror is not a witness, per the record's five-way split.
- Determinism note from the record's Part 5: the builder-canonicalization claim ("the canonical form is a function of the program rather than the spelling") is stated there as UNTESTED — this implementation must test it before the witness's converse property (bit-identical plans agree on the witness) is asserted.
- No reference-side edits beyond what plain-scalar threading requires at existing call sites; if a reference entry point seems to need a new signature, stop and report (that would reopen C).

## Closes when

A and B exist as accepted (unlabelled) surfaces matching the record's drafts with drift corrected against source, the determinism claim is tested, the refusal enum's arms are each watched firing, and the freedom-sites record's Part 7 is updated to state which items landed and where.

## Outcome — A and B landed; C stayed excluded; five drift corrections

**Worker `agent-witness`, base `61414b91`, branch `tkt/implement-the-realization-witness-vocabulary`.**

**What shipped.** `crates/tiler-ir/src/schedule/witness.rs`, re-exported from `tiler_ir::schedule`. Item A is `RealizationWitness<'a>`, aggregated by `RealizationWitness::of(&VerifiedScheduledRegion)`; it borrows the region's realization, scalar program, and reduction topology, so "a witness cannot disagree with the plan it describes" is structural rather than a constructor property. Item B is `UnpinnedFreedomSite` with the three drafted arms and no `Conforms`-shaped arm; its two elided payloads are `UnrecordedFoldContraction` and `UnevaluableRealization`. **Item C did not land and no file under `crates/tiler-reference/` changed** — `grep -rn "tiler_ir::schedule" crates/tiler-reference/src --include='*.rs'` returns five lines, all behaviour vocabularies, no plan structure.

The redirection left the refusal without a producer, so one is sited beside the aggregation: `RealizationWitness::unpinned_freedom_site() -> Option<UnpinnedFreedomSite>`. An `Option` rather than a `Result` deliberately — `None` states that the enumeration found no site this contract grants and this plan leaves open, which is a claim about the table and never about values.

**Public boundary.** 7.2's accepted list — `of`, `realization`, `order`, `accumulation`, `contributor_partition`, `arrival`, `rounds`, `pointwise_f32` — is unlabelled. Five items beyond it carry `**Draft surface, not yet accepted.**` in their rustdoc, following `crates/tiler-ir/src/index/sourced.rs`: `reduced_axes`, `contracted_shape`, `pass`, `fold_epilogue`, `unpinned_freedom_site`. The first three exist because the record's Part 2 names those fields in the site sets of 4.1, 4.4, and 4.2 while 7.2 drafts no accessor for them.

**Five corrections to the record, each read at source at this base**, written up in its new [Part 7.5](../docs/research/reference/plan-freedom-sites.md):

1. `order` is `Option<ContributorOrder>`, not the total accessor 7.2 drafts: a total one would return the vocabulary's single variant for a non-reducing region, which is Part 1's mirror class exactly.
2. **Site 4.8's spend population is empty.** `builder.rs:1516` and `:1640` refuse a declared accumulation differing from the region's own arithmetic type, so the class moves from *Witness, unevaluable* to *Witness*, beside 4.4.
3. **Site 3.1's stated ground is wrong.** There is no realization cross-check on `FusedMultiplyAddSerialSum`'s `contraction`; the verifier admits the variant only when it is `false` (`builder.rs:1029`, `:1332`, `:1421`), while `physical.rs:1975-1976` still derives it from the contract. No verified region carries `true`. The conclusion — not a witness — is unchanged.
4. **Site 4.9 is new.** `ScalarProgram::SquaredSerialSumThenEpilogue` (`model.rs:628-646`) pins a second `PointwiseF32Expression`; class *Witness, unevaluable*. The current split is twenty-five sites, seven evaluable witnesses and three unevaluable.
5. **Site 3.3's refusal is narrower than refutation 2 predicts.** `-ffp-contract` governs `a * b + c`, so a fold whose step is `accumulator + contributor` has nothing to contract; `BackendOrderUndeclared` is raised only for a stated pointwise multiply-add adjacency, and the fold adjacencies are named more precisely ahead of it.

**Measurement — Part 5's canonical-form claim is bounded, not general.** It holds for the two mitigations the record names (`the_two_named_canonicalization_mitigations_hold`) and fails for a third spelling neither reaches: nothing shares an identical constant, so `x * 2.0 + 2.0` spelled with one constant value and with two gives two expressions, two witnesses, and two canonical schedule identities for one binary32 function (`a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse`). The failure is of the *converse* — the witness is too fine, never too coarse — so nothing unsound follows and `RealizationWitness` derives no `PartialEq`, with the reason in its rustdoc. Whether the compiler can mint the duplicated spelling is not established; filed as [`share-identical-constants-in-the-pointwise-expression-canonical-form`](share-identical-constants-in-the-pointwise-expression-canonical-form.md).

**Evidence.** Nine tests in `crates/tiler-ir/src/schedule/witness/tests.rs` over six verified-region fixtures (pointwise `f32`, pointwise `bf16`, serial fold, both split passes, cooperative tile at one and two rounds, contraction). Each of the three refusal arms was watched failing before restoration, and each payload variant is separately named and counted. No identity domain moved: `tiler.schedule.v5` is unchanged and no pinned artifact, cache, or region identity in the workspace moved.

**Scope added.** `research/reference`, required by the Part 7 update the ticket's Closes-when demands. No live sibling claim declares it.

## Outcome — delivered 2026-08-07 at `1f764e00`

`RealizationWitness<'a>` and `UnpinnedFreedomSite` landed in `crates/tiler-ir/src/schedule/witness.rs`, re-exported from `tiler_ir::schedule`. The witness **borrows** the region's realization, scalar program, and reduction topology rather than copying, so "a witness cannot disagree with the plan it describes" is structural rather than asserted. `UnpinnedFreedomSite` carries the three drafted arms and **no `Conforms`-shaped arm**, which was the accepted item B's whole content. No `crates/tiler-reference/` or `crates/tiler-compiler/` file was touched.

**Mirrors are never read.** Nothing consults `permits_reassociation`/`permits_permutation` off a topology, and `FusedMultiplyAddSerialSum`'s `contraction` field is named as an unrecorded adjacency rather than treated as an answer.

**The determinism claim was tested before anything was built on it, and it is refuted in the general form the record states.** Part 5's canonical-form claim holds for the two mitigations it names and fails for a third neither reaches: nothing shares an identical constant, so `x * 2.0 + 2.0` spelled with one constant and with two produces four-node and five-node expressions, two witnesses, and two canonical schedule identities for one binary32 function. The failure is of the **converse** — the witness is too fine, never too coarse — so a conformance oracle built on it stays fail-closed. `RealizationWitness` therefore derives no `PartialEq`, with the reason at its definition. Filed as [`share-identical-constants-in-the-pointwise-expression-canonical-form`](share-identical-constants-in-the-pointwise-expression-canonical-form.md), which must first settle whether the compiler can even mint the duplicated spelling.

**Five drift corrections against the record, each read at source**, landed as Part 7.5 with forward pointers at the six places the record now makes a superseded claim. The sharpest: site 3.1's stated ground is simply wrong — the record says hard-coding `false` "would fail the schedule verifier's realization cross-check", and there is no such cross-check; the verifier admits `FusedMultiplyAddSerialSum` only when the field is `false`, while `physical.rs` still derives it from the contract, so a permitting contract yields a region the verifier *refuses*. Site 4.8's spend population turns out to be empty; site 4.9 is new; site 3.3's refusal is narrower than the record predicts. Current split: 25 sites, 7 evaluable witnesses, 3 unevaluable.

**Watched failing on all three refusal arms**, each restored green: `ContractionUnrecorded`, `BackendOrderUndeclared`, and `RealizationNotEvaluable`. Populations counted — 9 new tests over 8 verified-region fixtures, with `population.len() == 6`, `cases.len() == 4` twice, and `named.len() == 3` twice asserted, so a loop over nothing cannot look green.

**No identity moved.** `tiler.schedule.v5` unchanged; the three standard Metal pins verified identical before any edit and at the final commit. `make full` exit 0 on the branch and again on the merged tree: 2,959 workspace tests, 1,033 release numerical.

**Two things released rather than absorbed.** The built surface differs from the drafted one — one accepted signature narrowed (`order` returns `Option<ContributorOrder>`, because a total accessor would hand back a value for a contributor sequence that does not exist) and seven items exist that Part 7.2 drafted no accessor for, landed as labelled drafts. Both park on [`accept-the-realization-witness-surface-as-built`](accept-the-realization-witness-surface-as-built.md) for Tom.

**One judgement call the coordinator let stand.** The worker set the record's `implementation_status` to `partial` but left `disposition: pending`, because changing it would oblige a catalog edit in `docs/research/README.md` under `contracts/navigation` — a scope it did not hold and declined to add unilaterally. That is the right call: the catalog and the disposition must move together, and doing half of it would be the drift this repository keeps finding.

## Correction — 2026-08-08, label population

The first Outcome's five-marker census is the accurate one. Seven public items beyond Part 7.2's accessor list landed, but only the five accessors `reduced_axes`, `contracted_shape`, `pass`, `fold_epilogue`, and `unpinned_freedom_site` carried markers beginning `**Draft surface, not yet accepted`. The later shorthand that all seven “landed as labelled drafts” is false for the payload enums `UnrecordedFoldContraction` and `UnevaluableRealization`, which landed unlabelled. This correction changes neither the shipped surface nor the implementation evidence above.
