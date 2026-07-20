---
id: verify-candle-metal-post-wait-error-checking
title: Verify Candle Metal post-wait error checking
status: review
priority: p1
dependencies: []
related: [spike-runtime-semantic-validation-enforcement]
scopes: [research/runtime, contracts/integrations]
shared_scopes: []
paths: []
tags: [tiler-research, candle, metal, spike]
claimed_from: todo
assignee: pauli
lease_expires_at: 1784554431
---
Verify by source test or intentional GPU fault whether Candle Metal Commands::ensure_completed can return success when a committed/scheduled command buffer transitions to Error during wait_until_completed. Record the exact affected paths and required post-wait status/error check before Tiler relies on synchronous validation readback. Do not assume this local-source inference is confirmed until measured.
Verified at local Candle commit 31f35b147389700ed2a178ee66a91c3cc25cc80d. A structural source audit confirms one pre-wait status read, two wait sites, and zero post-wait status reads in Commands::ensure_completed. A nine-test transition harness reproduces success after Committed/Scheduled transitions to Error and validates the required final Completed/Error check. Exact affected kernel/core readback paths and the real-GPU measurement boundary are documented.
