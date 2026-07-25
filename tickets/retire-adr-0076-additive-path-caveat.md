---
id: retire-adr-0076-additive-path-caveat
title: Retire ADR 0076's additive-path re-establishment caveat
status: done
priority: p3
dependencies: []
related: [extend-the-numerical-probe-to-an-additive-path-kernel, broaden-the-apple-numerical-probe-matrix, repoint-adr-0076-evidence-at-the-numerical-record]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, metal]
---
`docs/decisions/0076-declare-target-honourable-numerical-realizations.md` states, in two places, that one re-verified observation is not reproduced by the checked-in harness.

Its `Measured evidence` bullet: "One re-verified observation is outside the harness's kernel set and is therefore *not* re-established: the additive-path input flush, since every probe kernel that adds does so after a multiply. `extend-the-numerical-probe-to-an-additive-path-kernel` owns closing that gap."

Its subnormal-flush Measurement: "An emitted `x + 0x00800000` returns `0x00800000` for the operand `0x80400000` … confirming input flushing on the additive path — this last observation alone is not reproduced by the harness, whose every adding kernel adds after a multiply, and `extend-the-numerical-probe-to-an-additive-path-kernel` owns closing it."

Both are now stale in the direction that matters: the observation reproduced. `extend-the-numerical-probe-to-an-additive-path-kernel` added the `add_smallest_normal` kernel — a single `x + 2**-126` with no multiply before it — and `docs/research/apple-targets/numerical-behaviour.md` finding 20 records it returning `00800000` for `80400000` at `-O0` and `-O2`, under `safe`, `relaxed`, and `fast`, on both compilation paths, for both dispatchable families, with an execution witness (`00800000 → 01000000`) reporting `executed` in every configuration. The kernel is admissible under the relaxed modes where the `scale 1.0, bias +0.0` kernel is not, because adding a nonzero constant is an identity on no operand.

Do not delete the caveat's history. It recorded a real gap and the reason it existed; record that it closed, where, and on which environment row, in the way the ADR already handles an evidence refresh elsewhere.

Two further consequences of `broaden-the-apple-numerical-probe-matrix` may belong in the same pass, and both are conclusions the ADR supports rather than contradicts. Finding 16 measures a source-level `fma` fusing at every `-ffp-contract` setting including `off`, so contraction control is a constraint on what the emitter may write and not something the flag enforces on its behalf. Finding 17 measures a two-add chain reassociated under `relaxed` and `fast`, so a target profile admitting those modes cannot promise a reduction order on this row; `qualify-contraction-association-reassociation-permission` may be the better home for that one.

Closes when the ADR no longer claims the additive-path observation is unreproduced and `uv run --locked python scripts/check_repository.py` passes.

## Outcome

Both stale sites in `docs/decisions/0076-declare-target-honourable-numerical-realizations.md` are retired and the two further consequences landed in the same pass. `decision_status` is untouched at `accepted`; nothing in the ADR's conclusions moved.

### What was verified against the retained record rather than taken from prose

Every value below was read out of `spikes/apple-targets/results/2026-07-25-numerics-{covering,exhaustive}-xcode26.6-metal32023.883/record.tsv` and the kernel definitions in `spikes/apple-targets/numerical_probe.py`, not from finding 20's summary or this ticket's description.

The kernel is `add_smallest_normal`, whose `steps` is the single `Step(0x00800000, "+")` — one add of `2**-126`, with no multiply before it, confirming the ticket's premise at the source rather than by report. `probe.operands` is `00000001 00400000 007fffff 00800000 80400000 80000000 3eb97ef9 3f800000`, so `80400000` is the fifth position and `00800000` the fourth. All twenty-four dispatched `case.*.add_smallest_normal.*.results` rows — two families (`macos`, `ios-simulator`) times six offline configurations (`safe`/`relaxed`/`fast` at `-O0` and `-O2`, `contract-off`) plus six runtime ones (three math modes at `opt-default` and `opt-size`) — carry the identical vector `00800000 00800000 00800000 01000000 00800000 00800000 3eb97ef9 3f800000`. Position five is `00800000`, which is the claim. The exhaustive record's rows reduce to the same single unique vector.

The witness is real and not merely declared: position four is the operand `00800000` and returns `01000000`, which is the kernel's `Witness(operand=0x00800000, executed=0x01000000, deleted=0x00800000)` reporting `executed` rather than `deleted`. `ADDITIVE_INPUT_FLUSH` is `SubnormalProbe(operand=0x80400000, preserving=0x00400000, flushing=0x00800000)`, so the flushing candidate is a normal value derived by exact arithmetic and not a zero — the point the ADR refresh now carries. The `fadd` survival was checked directly: `case.macos.add_smallest_normal.{safe,relaxed,fast}.{O0,O2}.contract-off.float_operations` is `fadd` under `safe` and `fadd` carrying the mode's licence set under `relaxed` and `fast`, six of six, so nothing is deleted where the trap kernel's arithmetic vanishes.

Guard layering was read from `probe.guard_layers` rather than assumed uniform: offline-with-device carries both `emitted-arithmetic` and `execution-witness`, offline-without-device carries only the first, and runtime carries only the second, because the runtime path emits no readable module. The refresh is worded to that distinction.

The three counterexamples were verified the same way. Finding 16: `case.macos.fused_pair.safe.O2.contract-{off,on,fast}.float_operations` is `air.fma.f32` in all three and the results are `3fc58f9d` in all three, against `contraction_pair`'s `fmul fadd` and `3fc58f9e` at `off` and `on`; the runtime rows agree. Finding 17: `reassociation_chain` emits `fadd fadd` and returns `3f800000` under `safe`, and one `fadd+reassoc` returning `3f800001` under `relaxed` and `fast`, runtime included. Finding 15: `divide_by_two` and `divide_by_half` both emit a single `fmul` at `safe`/`-O2`/`contract-off`.

### The two retirements

The `Measured evidence` bullet no longer states the observation is unreproduced. It records instead that the bullet carried the gap until 2026-07-25, why the gap existed — every adding kernel added after a multiply, so no add took a subnormal operand from the buffer — that it closed, which ticket closed it, and where the refresh states the evidence. The history is preserved as history rather than as a false present-tense claim, which is what the ticket asked for and what the ADR's own evidence-refresh paragraphs already do.

The re-verification Measurement keeps its measured content and loses only the trailing caveat, which now points at a new `Evidence refresh 2026-07-25` paragraph placed immediately after it. That refresh states what the caveat said and why, what re-establishes it, and the exact coverage. Two `Fact` paragraphs follow it: that the flushed subnormal here is an addend rather than the whole result, so a flush need not surface as a returned zero and the third outcome `00000000` stays classified distinctly; and that this is the only witnessed additive observation available under the relaxed modes, satisfying both guard layers rather than being admitted despite one.

### What the counterexamples falsified, and the retraction

One sentence in the accepted decision was wrong as written, and it is corrected rather than merely annotated. Item 0's "What is deliberately *not* general" attributed the honourability of contraction and reassociation to a stated compiler selection with no further condition. Finding 16 refutes that for contraction: a source-level `fma` carrying this record's own constants is emitted `air.fma.f32` and returns the fused value at `-ffp-contract=off` as at `=fast`, so no contraction setting the driver offers unfuses what the emitter wrote fused. Contraction honourability on this row is jointly a compiler selection *and* an emission discipline, which makes the per-statement emission rule a correctness requirement on the Metal emitter and not a stylistic one. The sentence now says a selection is necessary and not sufficient, and a `Refinement 2026-07-25` pair states each dimension's added condition, explicitly recording what the record previously claimed.

Finding 17 narrows the same sentence more weakly and is recorded as a bound rather than a refutation: reassociation *is* honourable by selecting `safe`, so the claim survives, but it is measurably unhonourable under `relaxed` and `fast`, which "a stated compiler selection" left implicit. The consequence is written as a bound on what item 3's per-dimension declaration may report — a profile admitting either relaxed mode cannot declare a reduction order honoured on this row.

Neither refinement disturbs the surrounding claim that only the subnormal dimension has a measured target that cannot honour the strict reading *at all*: `safe` honours both contraction and reassociation, and no selection makes the subnormal flush go away.

Finding 15 does not falsify any ADR sentence — the record makes no division claim — but it sharpens the fourth open question, which reasoned only about the compiler *folding arithmetic away*. Substitution is a second way the executed operation departs from the written one and it is not fail-closed in the same direction: a written power-of-two division reaches the device as an `fmul` under the strictest offline selection. For the subnormal dimension the over-reporting direction survives, since a substituted operation still flushes; for an operation-specific obligation it does not. The question gains that paragraph and remains unowned and unsettled.

### Deliberately not done

The ticket suggests `qualify-contraction-association-reassociation-permission` as the better home for finding 17. It is not: that ticket is `done`, is scoped to `contracts/optimizer`, and is about contraction in the *tensor* sense — regrouping `(AB)C` to `A(BC)` — not the FMA sense finding 16 measures nor the arithmetic reassociation finding 17 measures. Its own outcome flags that the word carries two unrelated senses in this corpus. Both consequences are therefore recorded here, where the ADR's own scope sentence is what they narrow, and the contract-side statements they imply are attributed to `docs/numerical-semantics.md` and `docs/backends/metal.md`, which this scope does not hold.

`docs/research/apple-targets/numerical-behaviour.md` still describes ADR 0076 as `proposed` in two places, its "what ADR 0076 should cite" Proposal and its Traceability line, both stale since acceptance on 2026-07-24. That file is `research/apple-targets` and outside this scope, so it is reported rather than edited.

### Gate

`uv run --locked python scripts/docs.py render` passed (183 records) and regenerated no catalog line, since no frontmatter changed. `uv run --locked python scripts/check_repository.py` passed complete, including the Rust sub-gate. `git diff --check` clean, `tkt lint` reports no problems. The `validate_quotations` phase did not fire: the new paragraphs use indirect speech for the wording they retire rather than quoting it against a linked document, which is the known gap `let-a-correcting-document-quote-the-text-it-corrects` owns.
