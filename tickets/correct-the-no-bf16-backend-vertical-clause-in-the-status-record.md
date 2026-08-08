---
id: correct-the-no-bf16-backend-vertical-clause-in-the-status-record
title: Correct the no-BF16-backend-vertical clause in the status record
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786178177
---
## The clause the 2026-08-05 correction did not reach

`docs/status.md`'s authoritative-profile bullet ends its original sentence with "Device address width, a workgroup fact, F16, F64, and every iOS family remain absent and therefore unknown; no BF16 backend vertical is implied."

**Fact repaired 2026-08-08 (worker, at base `c81f9257`).** This quote carried `**` emphasis around the final clause that the source does not have, so the ticket's own rendering was unsearchable as written: `grep -c 'unknown; \*\*no BF16 backend vertical is implied\.\*\*' docs/status.md` returns 0 while `grep -c 'no BF16 backend vertical is implied' docs/status.md` returns 1. The emphasis is removed above so the anchor greps.

A dated correction follows it — but that correction addresses **the BF16 numerical-contract half** of the clause, saying a caller can now state a BF16 contract. **It does not reach the backend-vertical half**, which still stands as written and is now false: BF16 lowering exists, `bfloat` MSL is emitted against the authoritative macOS Apple9 declaration, and one dispatched device run lives under `crates/tiler-conformance`.

**Fact repaired 2026-08-08 (worker).** This read "one dispatched device run is **retained** under `crates/tiler-conformance`", which names the wrong evidence shape. Nothing is retained there: unlike the `spikes/**/results/` records this repository does retain, the BF16 run has no checked-in result file, and `grep -rn bf16 crates/tiler-conformance/src/retained_record.rs` is empty. What exists is a live harness — `the_bf16_vertical_agrees_with_the_oracle_on_the_measured_row` in `crates/tiler-conformance/src/bf16_vertical/tests.rs` — which dispatches when a device and toolchain are present and otherwise reports an unavailable measurement boundary through `require_or_report` without claiming a device result. The run is re-executed rather than recalled, so the status correction must say "dispatched", not "retained".

So the paragraph currently corrects one half of a two-half clause and leaves the other reading as current. **Verify both halves at your base before editing** — the correction's own wording is the evidence for what it did and did not cover.

## Bound the replacement precisely

The device run is real, and it is narrow: one macOS Apple9 row, one contract, three operations, fifteen corpus elements. State the extent rather than replacing one over-broad clause with another — the recurring defect here is a correction that overshoots in the opposite direction from the text it replaces.

Note also what a BF16 program still cannot do, so the correction does not imply reachability it lacks: `first_macos_apple9` declares **no BF16 contraction row**, so a flush-accepting BF16 contract meets `Unknown` at numerical resolution and the outcome is `NoFeasiblePlan`. Recognition admits it and fusion legality derives it; the **target profile** is what refuses.

Follow the file's own dated-correction convention, and cite by searchable anchor rather than line number.

## Closes when

Both halves of the clause read true, the replacement names the device run's exact extent and the profile refusal that still stands, and `make citations` passes.
