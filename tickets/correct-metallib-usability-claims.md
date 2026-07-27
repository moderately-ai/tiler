---
id: correct-metallib-usability-claims
title: Stop treating a linked metallib as proven runtime-usable
status: todo
priority: p3
dependencies: []
related: [prototype-metal-aot-slice, declare-a-required-gpu-family-in-the-artifact]
scopes: [implementation/metal-aot, contracts/artifacts]
shared_scopes: []
paths: []
tags: [metal, aot, documentation, correctness]
---
Keep offline compilation evidence separate from runtime compatibility evidence.

`tiler-metal-aot` currently describes one successful-output condition as a
“usable Metal library.” A successful `metallib` link proves that the offline
toolchain produced a library for the requested compilation target. It does not
prove that every device or deployment target named by surrounding metadata can
load or execute it.

## Outcome

Use “produced” or “linked” artifact language at the AOT boundary. Reserve
“runtime-compatible” or “usable on device” for evidence that includes the
declared family/profile checks and successful runtime preparation required by
the runtime contract.

Correct code documentation, diagnostics, backend contracts, and examples that
currently cross that evidence boundary. Do not weaken a genuine
output-validation failure merely to change its wording.

## Closes when

No offline-only result claims runtime usability, runtime compatibility claims
name their evidence boundary, diagnostics remain actionable, and the
documentation corpus agrees.
