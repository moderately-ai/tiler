---
id: verify-candle-metal-post-wait-error-checking
title: Verify Candle Metal post-wait error checking
status: todo
priority: p1
dependencies: []
related: [spike-runtime-semantic-validation-enforcement]
scopes: [research/runtime, contracts/integrations]
shared_scopes: []
paths: []
tags: [tiler-research, candle, metal, spike]
---
Verify by source test or intentional GPU fault whether Candle Metal Commands::ensure_completed can return success when a committed/scheduled command buffer transitions to Error during wait_until_completed. Record the exact affected paths and required post-wait status/error check before Tiler relies on synchronous validation readback. Do not assume this local-source inference is confirmed until measured.
