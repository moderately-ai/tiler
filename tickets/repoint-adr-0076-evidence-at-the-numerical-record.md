---
id: repoint-adr-0076-evidence-at-the-numerical-record
title: Repoint ADR 0076 evidence at the Apple numerical record
status: done
priority: p1
dependencies: []
related: [check-in-apple-numerical-behaviour-probe]
scopes: [contracts/decisions]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [docs, numerics, adr]
---
`check-in-apple-numerical-behaviour-probe` created `docs/research/apple-targets/numerical-behaviour.md` (id `tiler.research.apple-targets.numerical-behaviour`), which owns the Apple GPU `f32` measurements ADR 0076 rests on, links the checked-in harness, and is re-established by the repository gate. That ticket holds `research/apple-targets` and cannot edit the ADR.

Three edits are required in `docs/decisions/0076-declare-target-honourable-numerical-realizations.md`.

First, `evidence` must become `["tiler.research.apple-targets.numerical-behaviour", "tiler.research.numerics.operation-conformance-matrix", "tiler.research.target-profiles.physical-feasibility-model", "tiler.research.apple-targets.compatibility"]`. The numerical record is added as the primary measured evidence; the compatibility probe stays, because the ADR still cites it for the flag-acceptance row and for its own disclaimer, and the Traceability prose already scopes that citation correctly.

Second, the Traceability section's "Measured evidence" line currently reads "the on-device and compile-side measurements in `tickets/prototype-metal-numerical-realization.md`, independently re-verified below". It should name the research record and the harness instead, since a ticket outcome is not an evidence authority.

Third, the fifth open question — "Where the Apple numerical measurement should durably live" — is answered and should be removed, with its answer stated in the Traceability section: `spikes/apple-targets/numerical_probe.py` owns the harness, `docs/research/apple-targets/numerical-behaviour.md` owns the record, and `scripts/check_repository.py` re-establishes both.

Two corrections the numerical record raises should also be reflected. The ADR's re-verification of the flag spellings ("under `relaxed` each carries `reassoc nsz arcp contract afn`; under `fast` each carries `fast`") is contraction-dependent and holds only at `-ffp-contract=fast`. And the ADR's claim that "counting floating-point operations in the emitted LLVM IR explains it" is correct at `-O2` and incomplete at `-O0`, where both operations survive into the emitted IR and still do not execute; the measured guard therefore needs two layers, not one. Neither changes a conclusion, and the second strengthens the ADR's central inference that honourability must be a stated target fact.

## Outcome

All three edits and both corrections landed in `docs/decisions/0076-declare-target-honourable-numerical-realizations.md`. `decision_status` is untouched; acceptance remains `accept-adr-0076-numerical-realizations`.

### The three edits

**`evidence`.** Now `["tiler.research.apple-targets.numerical-behaviour", "tiler.research.numerics.operation-conformance-matrix", "tiler.research.target-profiles.physical-feasibility-model", "tiler.research.apple-targets.compatibility"]`, exactly as specified. `scripts/docs.py` types `evidence` to `kind: research` and the new target satisfies it. Rendering regenerated one line of `docs/decisions/README.md`, the ADR 0076 catalog entry's evidence list; no generated item was hand-edited and `docs/research/README.md` did not move, because the research record already declared `adopted_by: ["ADR-0076"]` at the base commit.

**Traceability "Measured evidence".** Rewritten to name the research record and the checked-in harness rather than the `prototype-metal-numerical-realization` ticket outcome, and to state the answer to the removed open question in place: the harness is `spikes/apple-targets/numerical_probe.py`, the record is `docs/research/apple-targets/numerical-behaviour.md`, and `scripts/check_repository.py` collects its assertions so a toolchain change that alters a measured value fails the gate. The line also carries the two refinements forward and names the one gap below.

**The answered open question.** Removed rather than retained-with-resolution; the reasoning is in "Removal, and why the sibling precedent does not transfer" below. The "Open questions" preamble was left exactly as written — "These are recorded unresolved on purpose. None is settled by this record." — which stays true of the five that remain, so no amendment to it was needed.

### The two corrections

**Contraction-dependent flag spellings — two sites, not one.** The ticket names the re-verification measurement; the same contraction-dependent spelling also appears in item 4's "Measurement — why flags cannot substitute", which quotes `reassoc nsz arcp contract afn` for `relaxed`. Both were corrected. The re-verification now states all three modes at `off`/`on` and at `=fast` from finding 1's nine-cell table, and carries a "Refinement" paragraph recording what the record said before, that finding 1 is the authority, and that nothing in the argument depends on the spelling — the load-bearing fact is that `relaxed` and `fast` apply licences at all while the module declares fast math disabled, which holds in every cell. Item 4's measurement was made contraction-independent, since its argument never needed a specific setting: it now names both spellings and adds that only `fast` emits `air.compile.fast_math_enable`, so the divergence it relies on is `relaxed`'s and is present at every contraction setting.

**The `-O0` incompleteness, recorded as strengthening.** The trap measurement is now scoped to `-O2` where its operation counts hold, and a new sibling measurement records the `-O0` refinement: two floating-point operations survive into the emitted IR under `relaxed` and `fast` and the GPU still returns every operand unchanged, so a stage below the readable IR removed them and the guard needs two layers — emitted arithmetic *and* a dispatched execution witness. The inference it feeds was amended to carry the consequence rather than merely cite it: honourability cannot be probed from a kernel because a relaxation can delete the arithmetic *below any IR the toolchain will show*, so even a correct operation count does not establish that the operation executed. This is recorded as removing the last readable artifact from which honourability might have been inferred, not as a defect.

### Removal, and why the sibling precedent does not transfer

`record-presentation-label-naming-resolution` retained ADR 0074's answered naming question in place and amended that record's open-questions preamble to accommodate it. That treatment was not adopted here, on a distinction in what the two questions are about rather than on the ADR's `proposed` status.

ADR 0074's question is a *decision* question — which spelling a convention should use — and its resolution is a durable fact about the codebase that a reader of convention 2 benefits from having attached to the question that produced it; ADR 0074's preamble states that retention rule explicitly. This question is not a decision question at all. It asks where the record's own evidence should live, which is provenance, and ADR 0076 has a section that owns provenance. Moving the answer into Traceability puts it where a reader looks for it instead of in an inventory of what acceptance does not settle, and it keeps the remaining five entries a true list of live design questions. Nothing is lost: the history the question recorded — that the measurements lived only in a ticket outcome and a hand-built host nothing re-established — is now owned at length by the research record's own opening, which the ADR links from three places.

The reciprocal check was made rather than assumed: had the entry been retained, the preamble would have needed the same amendment ADR 0074's did, because "None is settled by this record" would have become false.

### Corrections to the ticket, and one gap it did not name

**The ordinal is wrong; the title is right.** The ticket calls this "the fifth open question". It is the **second** of six in the file. The title it quotes is unambiguous and was used to locate it.

**One re-verified measurement is not re-established by the harness.** The ADR's re-verification states an additive-path input flush — "An emitted `x + 0x00800000` returns `0x00800000` for the operand `0x80400000`" — and no probe kernel reproduces it. Reading `KERNELS` in full: every kernel that adds does so after a multiply, because `Kernel.source` emits `x * scale` then `+ bias` and no kernel sets `scale_bits=None` with a `bias_bits`, and no `case.*` key in the retained record is an add-only kernel. Writing the "Measured evidence" line accurately required knowing this, so the boundary is stated there and marked inline at the measurement. `extend-the-numerical-probe-to-an-additive-path-kernel` (p3, `research/apple-targets`) owns closing it and records that the obvious witness `3f800000` is inadmissible — `1.0 + 2**-126` rounds back to `3f800000`, making an executed add indistinguishable from a deleted one — while `00800000` works, since `2**-126 + 2**-126` is exactly `01000000`. This is narrower than `broaden-the-apple-numerical-probe-matrix`, which owns widening *past* multiply and add; the gap here is inside the vocabulary already claimed as measured.

### Staleness of the remaining open questions

Every remaining entry was checked against the research record. None is stale and none has a false premise.

The `applies_to` question's claim about `docs/backends/metal.md` still holds verbatim — that contract still reads "did not observe the numerical behavior these flags request" at line 165 — and the unsettled part, whether the profile declaration mechanism belongs there or in the architecture contract, is untouched by the measurement.

The arithmetic-gate question is the one the `-O0` refinement reaches, and it gained a sentence rather than a resolution: it already scoped its operation count to `-O2` and remains true, but the refinement sharpens what evidence class post-optimization reasoning would need — a dispatched execution witness on the exact toolchain row, which a plan-time authority cannot obtain by asking the compiler. That makes the question harder, not closer to settled, and it stays unowned. The remaining three entries are untouched by any measurement in the record.

### Gate

`uv run --locked python scripts/docs.py render` passed (179 records). `uv run --locked python scripts/check_repository.py` passed complete, including 179 Python tests with all 23 of `test_numerical_probe.py` passing and none skipped, and the Rust sub-gate. `git diff --check` clean. `ticketsplease lint` reports no problems.
