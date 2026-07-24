---
id: repoint-adr-0076-evidence-at-the-numerical-record
title: Repoint ADR 0076 evidence at the Apple numerical record
status: in-progress
priority: p1
dependencies: []
related: [check-in-apple-numerical-behaviour-probe]
scopes: [contracts/decisions]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [docs, numerics, adr]
claimed_from: todo
assignee: agent-repoint-adr-0076-evidence-at-the-numerical-record
lease_expires_at: 1784923183
---
`check-in-apple-numerical-behaviour-probe` created `docs/research/apple-targets/numerical-behaviour.md` (id `tiler.research.apple-targets.numerical-behaviour`), which owns the Apple GPU `f32` measurements ADR 0076 rests on, links the checked-in harness, and is re-established by the repository gate. That ticket holds `research/apple-targets` and cannot edit the ADR.

Three edits are required in `docs/decisions/0076-declare-target-honourable-numerical-realizations.md`.

First, `evidence` must become `["tiler.research.apple-targets.numerical-behaviour", "tiler.research.numerics.operation-conformance-matrix", "tiler.research.target-profiles.physical-feasibility-model", "tiler.research.apple-targets.compatibility"]`. The numerical record is added as the primary measured evidence; the compatibility probe stays, because the ADR still cites it for the flag-acceptance row and for its own disclaimer, and the Traceability prose already scopes that citation correctly.

Second, the Traceability section's "Measured evidence" line currently reads "the on-device and compile-side measurements in `tickets/prototype-metal-numerical-realization.md`, independently re-verified below". It should name the research record and the harness instead, since a ticket outcome is not an evidence authority.

Third, the fifth open question — "Where the Apple numerical measurement should durably live" — is answered and should be removed, with its answer stated in the Traceability section: `spikes/apple-targets/numerical_probe.py` owns the harness, `docs/research/apple-targets/numerical-behaviour.md` owns the record, and `scripts/check_repository.py` re-establishes both.

Two corrections the numerical record raises should also be reflected. The ADR's re-verification of the flag spellings ("under `relaxed` each carries `reassoc nsz arcp contract afn`; under `fast` each carries `fast`") is contraction-dependent and holds only at `-ffp-contract=fast`. And the ADR's claim that "counting floating-point operations in the emitted LLVM IR explains it" is correct at `-O2` and incomplete at `-O0`, where both operations survive into the emitted IR and still do not execute; the measured guard therefore needs two layers, not one. Neither changes a conclusion, and the second strengthens the ADR's central inference that honourability must be a stated target fact.
