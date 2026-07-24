---
id: re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening
title: Re-audit ADR 0011 and ADR 0019 implementation status after the vocabulary widening
status: todo
priority: p2
dependencies: []
related: [close-remaining-adr-status-drift, widen-numerical-vocabulary-and-complete-identity, reconcile-adr-records-with-the-widened-numerical-vocabulary]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions]
---
`close-remaining-adr-status-drift` audited all 71 accepted ADRs against `crates/` and deliberately left ADR 0011 and ADR 0019 at `not-started`, recording the exact reason: "the decision's central mechanism does not exist and a bump would misreport a type-system reservation or an architectural seam as implemented support: ADR 0011 and ADR 0019 (single-variant permission and subnormal enums that can never resolve anything)".

**Fact — that reason no longer holds.** `widen-numerical-vocabulary-and-complete-identity` (`1f78223`, 2026-07-24) made both enums multi-variant. `crates/tiler-ir/src/schedule/numerics.rs` defines `SubnormalMode` as `Preserve | FlushToZero { zero_sign }` and `NumericalPermission` as `Forbidden | Permitted`. Both subnormal dimensions of `NumericalRealization` now resolve independently and can differ; both permissions can be granted or withheld. The enums resolve something.

**What this ticket does not assert.** That either ADR is therefore `partial`. The falsified premise is the stated *reason for exclusion*, not the conclusion, and the two records differ:

- ADR 0019's decided behaviour is close to the landed shape — independent input and result dimensions, both behaviours expressible, a backend that must emulate, consume an authorized relaxation, or reject. `crates/tiler-metal/src/emit.rs` and `crates/tiler-metal/src/record.rs` now realize the reject branch per declared dimension with three typed gap variants. Its Consequences also claim reference evaluation can distinguish input flushing from result flushing; `crates/tiler-reference/` matches neither `SubnormalMode` nor `NumericalPermission` anywhere, so that clause is unrealized.
- ADR 0011's decided behaviour is a *program ceiling* intersected with per-operation restrictions and operation capabilities, resolved before semantic optimization, with every rewrite declaring which effective permission it consumes. None of that machinery exists; what exists is a region-level `NumericalRealization` and `FusionNumericalProof::forbidden_transforms`. A bump on the strength of the widened enum alone would misreport exactly what the audit ticket warned about.

**What closes this.** Read both decisions and their implementations in full — not this ticket's summary — and either bump or restate the exclusion reason so the corpus records a live justification rather than a dead one. Apply the rule `close-remaining-adr-status-drift` established and `docs/document-metadata.md` states: `implementation_status` is the highest maturity the record's own decided behaviour has reached, a retained high-water mark rather than a live mirror. Do not change `decision_status`. `reconcile-adr-0019-zero-sign-placement-with-the-landed-flush` is a separate question about ADR 0019's body and should not be folded in here.
