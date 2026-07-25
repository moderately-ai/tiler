---
id: reconcile-adr-0019-zero-sign-placement-with-the-landed-flush
title: Reconcile ADR 0019's zero-sign placement with the landed flush behaviour
status: done
priority: p2
dependencies: []
related: [widen-numerical-vocabulary-and-complete-identity, reconcile-adr-records-with-the-widened-numerical-vocabulary]
scopes: [contracts/decisions]
shared_scopes: [project/tickets, contracts/numerics]
paths: []
tags: [documentation, decisions, numerics]
---
ADR 0019 (accepted) decides that "Each dimension initially supports preservation or an explicit flush-to-zero behavior; zero-sign behavior is resolved with the signed-zero contract." ADR 0076 item 1 refines ADR 0019 and deliberately reopened that placement: "Whether the sign is carried as a field of the flush behaviour or resolved from the contract's signed-zero dimension is an implementation choice for the IR ticket; leaving it unstated is not."

**Fact — the implementation took the other option.** `widen-numerical-vocabulary-and-complete-identity` (`1f78223`, 2026-07-24) landed `tiler_ir::schedule::SubnormalMode` as `Preserve | FlushToZero { zero_sign }` over `FlushedZeroSign::{PreservesSign, AlwaysPositive}` in `crates/tiler-ir/src/schedule/numerics.rs`. The sign is a field of the flush behaviour, not a resolution against a signed-zero dimension. Its stated reasoning is that a permission may leave a zero's sign unspecified, and an unspecified flush result is exactly the under-specification ADR 0076 item 1 forbids, so every `SubnormalMode` value must answer "which zero" on its own.

**Fact — ADR 0019 has not been updated.** Read `docs/decisions/0019-split-subnormal-handling.md` in full: its Decision still states the signed-zero-contract resolution, it has no Amendments section, and nothing in it records that an accepted refining ADR reopened the question or which way the implementation went. A reader of ADR 0019 alone gets the wrong answer about where the sign lives.

**What closes this.** Decide whether ADR 0019's sentence is amended (the placement changed and ADR 0019 records it), or whether the two statements are compatible and the record should say how — a flush that names its own zero could still be *constrained* by the signed-zero dimension rather than resolved from it, and that reading has not been checked against `docs/numerical-semantics.md`, which says "The zero sign follows the resolved signed-zero and subnormal contract rather than an ambient target mode." Do not change `decision_status` on either record. If the answer is an amendment, ADR 0019 has no Amendments section and would gain its first, so follow the form ADR 0074 documents.

Also check `docs/numerical-semantics.md`'s `SubnormalContract` sketch, which still spells `inputs: Preserve | FlushToZero` with no zero sign. It is marked descriptive rather than a committed API, so it may be correct as written; decide rather than assume. That file is `contracts/numerics`, so declare the scope before touching it.

## Outcome

**Decision — amendment, not compatible readings.** ADR 0019's Decision clause now reads "a flush-to-zero behavior states which zero it produces, as part of that behavior", replacing "zero-sign behavior is resolved with the signed-zero contract". The record gains its first `## Amendments` section, in the form ADR 0074 documents: what it said, what it says now, the evidence that moved it, and what a reader of the earlier text should un-learn. `decision_status` on both records is untouched, and so is `implementation_status` — the ADR 0019 status question is `re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening`'s and was deliberately not folded in.

**Why the compatible reading was rejected, having been tested rather than dismissed.** The ticket's suggested reading is that a flush naming its own zero could still be *constrained* by the signed-zero dimension rather than resolved from it. That reading fails on a fact neither this ticket nor ADR 0076 states: **there is no signed-zero dimension in the resolved contract at all.** `NumericalRealization` in `crates/tiler-ir/src/schedule/numerics.rs` carries `profile_key`, `canonical_arithmetic_nan_bits`, `input_subnormals`, `result_subnormals`, `contraction`, and `reassociation` — six fields, none of them signed zero. The exact check is `grep -rn 'SignedZero\|signed_zero' crates/`, which returns `MslOptimization::preserves_signed_zero` in `crates/tiler-metal-aot/src/input.rs` — a property of an Apple compiler math mode, not a contract dimension — plus prose in doc comments and two test bindings. So the constraint reading describes a relationship between one field that exists and one that does not, and recording it as the current state would leave a reader of ADR 0019 still looking for a dimension to consult. It is recorded instead as a stated **obligation** on whoever admits such a dimension: it constrains the flush's stated sign rather than supplying it, disagreement is a rejection rather than a precedence rule, and nothing checks this today because there is no second field to disagree with.

**Fact — the strongest evidence that the sentence needed amending is ADR 0076's own framing.** ADR 0076 `refines` ADR 0019 and its item 1 says the placement "is an implementation choice for the IR ticket". A refining record cannot reopen a question its predecessor had closed; that it reopened this one is evidence that ADR 0019's sentence was never read as settling the placement. Under either reading of "resolved *with*" — resolved *from*, or resolved jointly *alongside* — the sentence sends a reader to a separate contract, and that is the part that is wrong.

**Fact — checked against `docs/numerical-semantics.md`, which was never falsified.** Its sentence "The zero sign follows the resolved signed-zero **and subnormal** contract rather than an ambient target mode" already names the subnormal contract as a source, and its operative content is the negative clause about ambient target modes. It was imprecise about which of the two states a flushed zero's sign, not wrong, and is sharpened rather than corrected.

**Decision — the `SubnormalContract` sketch is changed, and being descriptive is not a defence here.** It spelled `inputs: Preserve | FlushToZero` with no sign. ADR 0076 item 1 makes "a flush behaviour must state its zero" operative and says in terms that leaving it unstated is not an available choice, so a sketch omitting the sign illustrates exactly the under-specification that item forbids — regardless of whether the sketch is a committed API. It now reads `FlushToZero { zero_sign }` on both dimensions, which is one token per line and keeps the sketch's actual job, showing the two dimensions are independent.

**Scope.** `contracts/numerics` was added as a shared scope for the two `docs/numerical-semantics.md` edits and was uncontended; no `in-progress` ticket held it. `docs/decisions/README.md` regenerated with no diff, since no catalog metadata moved.

**Measurement.** `uv run --locked python scripts/docs.py render` reported "documentation render passed (183 records)". `uv run --locked python scripts/check_repository.py` exited 0 with "complete repository validation passed". Host macOS arm64, toolchain `nightly-2026-07-19`.
