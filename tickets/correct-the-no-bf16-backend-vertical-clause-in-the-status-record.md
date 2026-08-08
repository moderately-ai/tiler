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

`docs/status.md`'s authoritative-profile bullet ends its original sentence with "Device address width, a workgroup fact, F16, F64, and every iOS family remain absent and therefore unknown; **no BF16 backend vertical is implied.**"

A dated correction follows it — but that correction addresses **the BF16 numerical-contract half** of the clause, saying a caller can now state a BF16 contract. **It does not reach the backend-vertical half**, which still stands as written and is now false: BF16 lowering exists, `bfloat` MSL is emitted against the authoritative macOS Apple9 declaration, and one dispatched device run is retained under `crates/tiler-conformance`.

So the paragraph currently corrects one half of a two-half clause and leaves the other reading as current. **Verify both halves at your base before editing** — the correction's own wording is the evidence for what it did and did not cover.

## Bound the replacement precisely

The device run is real, and it is narrow: one macOS Apple9 row, one contract, three operations, fifteen corpus elements. State the extent rather than replacing one over-broad clause with another — the recurring defect here is a correction that overshoots in the opposite direction from the text it replaces.

Note also what a BF16 program still cannot do, so the correction does not imply reachability it lacks: `first_macos_apple9` declares **no BF16 contraction row**, so a flush-accepting BF16 contract meets `Unknown` at numerical resolution and the outcome is `NoFeasiblePlan`. Recognition admits it and fusion legality derives it; the **target profile** is what refuses.

Follow the file's own dated-correction convention, and cite by searchable anchor rather than line number.

## Closes when

Both halves of the clause read true, the replacement names the device run's exact extent and the profile refusal that still stands, and `make citations` passes.
