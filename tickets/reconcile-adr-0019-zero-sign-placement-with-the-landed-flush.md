---
id: reconcile-adr-0019-zero-sign-placement-with-the-landed-flush
title: Reconcile ADR 0019's zero-sign placement with the landed flush behaviour
status: todo
priority: p2
dependencies: []
related: [widen-numerical-vocabulary-and-complete-identity, reconcile-adr-records-with-the-widened-numerical-vocabulary]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, numerics]
---
ADR 0019 (accepted) decides that "Each dimension initially supports preservation or an explicit flush-to-zero behavior; zero-sign behavior is resolved with the signed-zero contract." ADR 0076 item 1 refines ADR 0019 and deliberately reopened that placement: "Whether the sign is carried as a field of the flush behaviour or resolved from the contract's signed-zero dimension is an implementation choice for the IR ticket; leaving it unstated is not."

**Fact — the implementation took the other option.** `widen-numerical-vocabulary-and-complete-identity` (`1f78223`, 2026-07-24) landed `tiler_ir::schedule::SubnormalMode` as `Preserve | FlushToZero { zero_sign }` over `FlushedZeroSign::{PreservesSign, AlwaysPositive}` in `crates/tiler-ir/src/schedule/numerics.rs`. The sign is a field of the flush behaviour, not a resolution against a signed-zero dimension. Its stated reasoning is that a permission may leave a zero's sign unspecified, and an unspecified flush result is exactly the under-specification ADR 0076 item 1 forbids, so every `SubnormalMode` value must answer "which zero" on its own.

**Fact — ADR 0019 has not been updated.** Read `docs/decisions/0019-split-subnormal-handling.md` in full: its Decision still states the signed-zero-contract resolution, it has no Amendments section, and nothing in it records that an accepted refining ADR reopened the question or which way the implementation went. A reader of ADR 0019 alone gets the wrong answer about where the sign lives.

**What closes this.** Decide whether ADR 0019's sentence is amended (the placement changed and ADR 0019 records it), or whether the two statements are compatible and the record should say how — a flush that names its own zero could still be *constrained* by the signed-zero dimension rather than resolved from it, and that reading has not been checked against `docs/numerical-semantics.md`, which says "The zero sign follows the resolved signed-zero and subnormal contract rather than an ambient target mode." Do not change `decision_status` on either record. If the answer is an amendment, ADR 0019 has no Amendments section and would gain its first, so follow the form ADR 0074 documents.

Also check `docs/numerical-semantics.md`'s `SubnormalContract` sketch, which still spells `inputs: Preserve | FlushToZero` with no zero sign. It is marked descriptive rather than a committed API, so it may be correct as written; decide rather than assume. That file is `contracts/numerics`, so declare the scope before touching it.
