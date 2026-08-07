---
id: audit-the-l1-workload-records-evidence-classes
title: Audit the L1 workload record's evidence classes against its relayed measurements
status: review
priority: p3
dependencies: []
related: [audit-the-ingestion-records-no-measurements-header-claim]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
claimed_from: todo
assignee: agent-l1-audit
lease_expires_at: 1786076416
---
## User-visible outcome

`docs/research/program-planning/first-metal-lm-workload.md`'s `evidence_classes` names the evidence it actually rests on, verified by a full read.

## The finding, from the L6 header audit

**Fact.** [`audit-the-ingestion-records-no-measurements-header-claim`](audit-the-ingestion-records-no-measurements-header-claim.md) swept every `docs/research/program-planning/` record for the same falsity it was correcting and found exactly one other: L1 carries `evidence_classes: ["primary-source-synthesis"]` while its body relays three **Measurement**-labelled paragraphs — the retained C1 fixture at line 196, the eighteen-position arithmetic at line 202, and the two-way F32 sensitivity envelope at line 241, the last measured on a named host row. Its own label legend at line 28 already lists **Measurement** as a class it uses, so the frontmatter and the record's stated vocabulary disagree.

**Fact.** `flash-class-capability-set.md` was checked and is clean: its single `**Measurement` hit is the label legend, not a measurement. No other `program-planning` record has the gap — L4 and L8 already carry `bounded-measurement`.

## The work

Read L1 in full — the same precondition the L6 audit worked under, because a class list can only be restated honestly against the whole record. Decide whether `bounded-measurement` (an observation holding only for its recorded inputs, environment, and procedure, per [document metadata](../docs/document-metadata.md)) is the honest addition, or whether the three paragraphs are relays that belong to their owning spike. Do not delete a relayed measurement to keep the field as it stands.

If the field moves, `docs/research/README.md`'s catalog line for the record moves with it, and that path is `contracts/navigation` rather than this scope — file or fold it into [`carry-the-l6-bounded-measurement-class-into-the-research-catalog`](carry-the-l6-bounded-measurement-class-into-the-research-catalog.md), which already owns the same drift for L6.

## Closes when

L1's `evidence_classes` and its body agree, verified by a full read, with any catalog consequence routed.

## Outcome — 2026-08-06

**Fact.** `docs/research/program-planning/first-metal-lm-workload.md` was read in full (289 lines, top to bottom) before any edit. The frontmatter gap is real, `bounded-measurement` is the honest addition, and no relayed measurement belongs to its owning spike instead. The ticket's enumeration of three Measurement paragraphs was **not exhaustive** — there are four.

### Inventory: every Measurement the record carries

| Site | What it is | Taken by | What it bounds here |
| --- | --- | --- | --- |
| Line 28 | the label legend, not a measurement | — | — |
| C1 fixture paragraph | the fixture exists and is retained, with three retained outcomes: the 18-token sequence, the unexercised tie branch, `logits_to_keep=0` keeping all | [C1 conformance fixture spike](../spikes/program-planning/qwen3-conformance-fixture/README.md), retained record `2026-08-01-c1-conformance-attribution-...-f32-eager-cpu-torch2.6.0-transformers4.51.0` | that C1's retained observables and its 10.43 MiB retention size are reproduced, and that the row exercises the budget arm of termination and not the EOS arm |
| Decode-step arithmetic paragraph | prefill 0–9, eight decode passes 10–17, argmax at 17 retained but not appended | same retained record | that the C1 table's 18 positions and 8 steps are consistent rather than an off-by-one |
| **Subnormal-flush sentence, mid-paragraph under *Effective numerical policy*** | **the fourth, which the finding missed** — both compilation paths flush F32 input and result subnormals to sign-preserving zero on the qualified Apple9/macOS/F32 row | [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md), its unified MSL 4 replay section | the whole "effective policy is subnormal-flushing, contraction-free, safe-math F32" derivation, and why the profile rejects strict subnormal-preserving F32 |
| Envelope paragraph | the F32 sensitivity envelope measured both ways on the fixture's host row | same retained record's `envelope.tsv` | the oracle's bound-derivation procedure — the smallest deviation any correct F32 realization could be required to fall inside |

It sits mid-paragraph after a **Fact** lead-in, which is why a paragraph-lead scan finds three and a full read finds four. Its content is also the one relay not sourced from the fixture, so a class list built from the fixture alone would have been under-evidenced in kind, not only in count.

**The four relayed figures were verified against the retained record, not taken on the prose's word.** `envelope.tsv` in the 2026-08-01 directory gives per-variant maxima of `max_abs_deviation` 2.048015e-4 (`f64_unmodified`) and 2.007484e-4 (`f64_promoted`), `top32_max_abs_deviation` 7.82013e-5 and 7.43866e-5, `top32_max_ulp_deviation` 78 for both, and `bit_identical_logits` spanning 483–3,863 unmodified and 507–3,723 promoted. Every figure the record states reproduces. The 7.44e-5 promoted top-32 value is stated in L1 but **not** in the fixture spike's own README, which gives only the unmodified 7.82e-5 and the shared 78 ULP; it is correct against the retained TSV, so this is a place where L1 carries a figure its source record does not restate rather than a discrepancy.

### The decision, and its ground

**`bounded-measurement` is added; nothing is moved.** Each of the four bounds a claim L1 itself makes — C1's retained observables, that table's internal consistency, the effective numerical policy the oracle compares against, and the envelope the bound derivation needs. None is a duplicate whose home is elsewhere, so the ticket's alternative disposition does not apply, and the instruction not to delete a relayed measurement was never in tension with the field. `["primary-source-synthesis", "bounded-measurement"]` is honest under [document metadata](../docs/document-metadata.md)'s definition — an observation holding only for the recorded inputs, environment, and procedure — and it is what L4 and L8, which relay the same fixture, already carry.

**The legend was true and was extended rather than corrected.** Unlike L6, L1's line-28 legend already listed **Measurement**, so there was no falsity there and none elsewhere: the record carries no "takes no measurement" sentence, and its two nearby absolute claims are both correctly narrow — "no measurement in this repository bounds the result" is about physical residency, and the B1 **Proposal** already says every one of its observables "is a `Measurement` bound to an exact host, toolchain, and procedure when it is taken, and none of them is a number this document supplies". What the legend could not tell a reader is that all four Measurements are relayed and none is taken here, which is exactly what the new frontmatter class needs in order to be legible. One paragraph was added after the legend stating that, naming all four with their owning records, and stating why the digests, safetensors header reads, and prompt token IDs this profile produced itself are **Fact** and not **Measurement** — they are reproducible from pinned immutable bytes and depend on no host. That distinction is the reason the record's own acquisition work does not widen the class list further.

### Catalog line owed — `contracts/navigation`, not this scope

`docs/research/README.md:90` still reads `pending; primary-source-synthesis` and now disagrees with the frontmatter. [`carry-the-l6-bounded-measurement-class-into-the-research-catalog`](carry-the-l6-bounded-measurement-class-into-the-research-catalog.md) is already closed, so this cannot fold there. **The exact one-line replacement**, verbatim, changing only `primary-source-synthesis` to `primary-source-synthesis, bounded-measurement`:

```text
- [First Metal language-model workload profile](program-planning/first-metal-lm-workload.md) — pending; primary-source-synthesis, bounded-measurement; informs: [Correctness and testing](../correctness-and-testing.md); experiments: [Qwen3-0.6B-Base C1 conformance and attribution reference fixture](../../spikes/program-planning/qwen3-conformance-fixture/README.md), [Qwen3-0.6B-Base conformance-corpus reachability probe](../../spikes/program-planning/qwen3-corpus-reachability/README.md)
```

### Flagged, not edited

- **L1's operation-family standing is materially stale**, found while reading the L2-handoff section against the roadmap it cites. The record says contraction "sits at R1 with no registered key" and that softmax, SiLU, `rsqrt`, reindex, broadcast, slice, and concatenate are "at R2 with no registered key"; the roadmap's family-state table now carries `tiler::strict-tensor-contraction-f32@1` at R6, `tiler::silu-f32@1` at R6, `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` at R6, `tiler::rms-norm-f32@1` and `tiler::softmax-f32@1` at R5, `Concatenate` at R5, and `Slice` at R4. Three sites carry it — the status line's "no rung of the ladder is built", the L2-handoff closing **Inference**, and the last bullet of *What remains open*. Filed as [`refresh-the-l1-operation-family-standing`](refresh-the-l1-operation-family-standing.md) rather than patched inside an evidence-class audit: every moved row is bounded (one measured toolchain row, prototype execution rows, R7 unmet), so the honest correction has to state each bound and not just a rung, which is a full read of the roadmap's family cells and a recheck of whether the two new reduction roles change L1's fusion-legality claim. It is a capability-standing change that L2 and L8 derive from, which is the case the repository routes to its own ticket.
- **The reductions and cast-and-convert claims survive that staleness** and were confirmed against the same table: reductions beyond the strict serial sum are still R2, and the cast-and-convert row is still R2, so L1's BF16-to-F32 ingestion paragraph stands unchanged.
- **The second bullet of *What remains open* relays measured quantities without a label** — the 2.2101e-4 joint band, 1.0872e-4 and 87 ULP over the top-32, the 1,204× margin — citing the fixture spike's `joint.tsv` and `perturbation.tsv`. It was left as written and is a considered non-change, not an omission: that section's four bullets use bold status lead-ins rather than the body's Fact/Inference/Measurement labels, and each bullet mixes what is measured with what remains open, so no single label classifies one. The frontmatter class now covers those figures either way. Relabelling the section is a convention change for the whole record and belongs to whoever makes it.
- **The prior sweep was re-run from this branch and confirmed.** All four remaining `program-planning` records carrying `["primary-source-synthesis"]` alone were checked: `flash-class-capability-set.md`'s single `Measurement` hit is its label legend, and `abi-expression-ownership.md`, `general-compilation-boundary.md`, and `minimum-correct-physical-realization-profile.md` have no `Measurement` hit at all. L1 was the last `program-planning` record with the gap, and the population is now named rather than asserted.

### Checks

`tkt lint` clean; `git diff --check` clean; `tkt guard` reports the branch inside `research/program-planning` and `project/tickets`. `project/tickets` was **not** in the frontmatter's shared scopes and was added with `tkt set --add-shared-scope project/tickets` in order to write this Outcome and file the follow-up ticket. **No `crates/` path is touched**, and neither is `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh`, or `prototypes/` — so this branch leaves the workspace gate untouched and carries the latest green gate.
