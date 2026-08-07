---
id: audit-the-ingestion-records-no-measurements-header-claim
title: Audit the ingestion record's no-measurements header claim
status: done
priority: p3
dependencies: []
related: [widen-the-identity-growth-ladder-to-the-governed-operation-budget]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## User-visible outcome

`docs/research/program-planning/complete-model-ingestion-and-execution.md`'s header describes what the record actually contains, so its evidence-class framing can be trusted.

## The finding, from the widened-ladder worker

**Fact.** The record's header states "This record contains no measurements and takes none", and that was already false before the 2026-08-06 ladder edits: the body carries dated **Measurement** paragraphs relayed from other landings (the identity-growth paragraphs among them). The ladder worker deliberately left the header alone because correcting it honestly requires reading the whole record and restating what mixture of Fact, Inference, Proposal, and Measurement it now carries — a full-record audit, not a one-line patch.

## The work

Read the record in full. Either restate the header to name the evidence classes actually present, or (if the measurements genuinely belong elsewhere) move them to their owning records and keep the header true. Do not delete relayed measurements to rescue the sentence.

## Closes when

The header claim and the record's contents agree, verified by a full read.

## Outcome — 2026-08-06

**Fact.** `docs/research/program-planning/complete-model-ingestion-and-execution.md` was read in full (342 lines, top to bottom) before any edit. The header claim was false in the way the finding states, and the same falsity ran through two further framing sites.

### Inventory: what the record actually carries

Sectional evidence classes, by the record's own labels: **Proposal** carries the four boundaries, all four candidate-elimination grids (M-A/B/C, W-A/B/C, I-A/B/C, IN-A/B), the logits contract, the whole-model partition, the ownership-table delta, the fallback classes, the failure classes, and the typed refusals it owes. **Fact** carries every claim traced to inspected source at base `d862c2b` or to a merged record — `bind_region`'s validation order, `check_program_budgets`' five derivations, `select_supported_strategy`'s rules, `AotRefusal::SymbolicExtent`, `select_embedded_route`, the Candle adapter's arity, `compute_graph_identity`, `BroadcastAxisMapping::new`'s two refusals, `StorageScalar`'s two variants. **Inference** carries every byte figure derived by arithmetic over L1, L4, and L5 — the 62,923,776-byte layer, the 2,384,199,680-byte total, the permutation count, the peak-residency table, the C1 worked example.

**Six paragraphs are Measurement**, every one relayed from a landing or spike outside this record, and none of them taken by it:

| Site | Date | Taken by | What it bounds here |
| --- | --- | --- | --- |
| I-B row (`BF16 ingestion`) | undated in its own source | L7's control, [first-quantized-lm-profile](../docs/research/numerics/first-quantized-lm-profile.md) — max logit deviation `0.000000e+00` over 18 C1 positions | I-B's elimination of I-C |
| Whole-model composition, property (c) | 2026-08-05 | [`decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode`](decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode.md), at `crates/tiler-reference/tests/decoder_layer.rs` — 58 and 62 occurrences | the conditional-three artifact-identity claim |
| Same correction, "what does not move" | 2026-08-05 | same test file — 0 differing elements on all three outputs | that the divergence is identity-only |
| Identity-growth paragraph | 2026-08-06 | [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](widen-the-identity-growth-ladder-to-the-governed-operation-budget.md), [identity-growth spike](../spikes/program-planning/identity-growth/README.md) — `program_bytes(n) = 3525n + 727`, P3 at 7,777, the 39,502-byte eleventh point | the ×372 margin and the 7.2× envelope ceiling |
| `region_expansions` wall | 2026-08-06 | same sweep — 12..=62 refuse `NoFeasiblePlan`, 63 refuses `BudgetExhausted` | that every P1/P2/P3 identity figure is an extrapolation |
| Attribution-surface paragraph | 2026-08-01 | [`retain-the-c1-model-attribution-fixture`](retain-the-c1-model-attribution-fixture.md), [C1 fixture spike](../spikes/program-planning/qwen3-conformance-fixture/README.md) — 2,064,384 / 4,128,768 / 400 bytes, 310 tensors bit-exact | that the attribution figures were reproduced, not only computed |

**No measurement was moved.** Each of the six bounds a claim this record makes and is labelled with the source that owns it, so none is a duplicate whose home is elsewhere; the ticket's alternative disposition does not apply.

### What changed

**The header statement.** It read: *"Claims are labelled Fact when traced to inspected source at that commit or to a merged record, Inference when derived from stated facts, and Proposal when not yet accepted or tested. **This record contains no measurements and takes none.** Every byte figure is arithmetic over quantities L1, L4, and L5 already state, and is labelled as an inference for that reason."* Three things were wrong: the label list omitted **Measurement**, which the body uses six times; the bolded sentence was false; and "every byte figure is arithmetic" was false of P3's measured 7,777 bytes, the 39,502-byte compiled program, and the fixture's four retained byte counts. It now names all four labels, states that the record takes no measurement of its own and relays six, enumerates them with their dates and sources, and narrows the byte-figure claim to the figures it is true of — with the measured ones named. It follows L1's and L8's existing legend form, which already list **Measurement** among their labels.

**Two adjacent framing sites carried the same falsity and were fixed in the same pass.** The frontmatter read `evidence_classes: ["primary-source-synthesis"]` and now reads `["primary-source-synthesis", "bounded-measurement"]` — honest under [document metadata](../docs/document-metadata.md)'s definition ("an observation holds only for the recorded inputs, environment, and procedure") and matching what L4 and L8, which relay the same measurements, already carry. *What this record does not decide* read "and takes no measurement", stating of the whole record what is true only of that bullet's three subjects; it now reads "measures no tolerance, latency, or throughput itself", which is true and is what the bullet meant.

**Two undated Measurement labels were dated in place**, so the header's "each naming the landing or spike that took it" is checkable without leaving the paragraph: the cross-row evaluation is now `Measurement, 2026-08-05, at the same crates/tiler-reference/tests/decoder_layer.rs`, and the attribution-surface paragraph is now `Measurement, 2026-08-01` — sourced from the retained result directory `2026-08-01-c1-conformance-attribution-qwen3-0.6b-base-da87bfb6-...`. L7's inherited control was left undated because its own record dates it no more precisely, and the header says so rather than inventing one.

### Flagged, not edited

- **`docs/research/README.md:84`** still catalogs the record as `pending; primary-source-synthesis` and now disagrees with the frontmatter. That path is `contracts/navigation`, not this ticket's scope. Filed as [`carry-the-l6-bounded-measurement-class-into-the-research-catalog`](carry-the-l6-bounded-measurement-class-into-the-research-catalog.md).
- **`docs/research/program-planning/first-metal-lm-workload.md`** has the same frontmatter gap — `["primary-source-synthesis"]` against three relayed Measurement paragraphs at lines 196, 202, and 241 — and is in this ticket's path scope but is a different record, whose class list can only be restated after its own full read. Filed as [`audit-the-l1-workload-records-evidence-classes`](audit-the-l1-workload-records-evidence-classes.md) rather than patched blind. Every other `program-planning` record was checked: `flash-class-capability-set.md`'s single `**Measurement` hit is its label legend, not a measurement, and L4 and L8 already carry `bounded-measurement`.
- The 2026-08-04 status-line clause "every byte figure stands" is scoped to that correction's own supersession and was left as written; it is a dated statement about what that landing moved, not a live claim about the record.

### Checks

`tkt lint` clean; `git diff --check` clean; `tkt guard` reports the branch inside `research/program-planning` and `project/tickets`. **No `crates/` path is touched**, and neither is `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh`, or `prototypes/` — so this branch leaves the workspace gate untouched and carries the latest green gate.
