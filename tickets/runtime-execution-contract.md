---
id: runtime-execution-contract
title: Define the consumer-neutral runtime execution contract
status: todo
priority: p1
dependencies: [artifact-envelope-model, kernel-program-buffer-plan, spike-runtime-semantic-validation-enforcement, verify-candle-metal-post-wait-error-checking]
related: []
scopes: [research/runtime]
shared_scopes: []
paths: []
tags: [tiler-research, research, runtime, correctness]
---
Define the runtime boundary independently of Candle: live-device identity, artifact validation, library and pipeline preflight, ABI binding, allocation, scratch lifetime, ordered multi-dispatch execution, named outputs, pipeline caching, guards, and fallback before partial work.

Deliver a state-machine or equivalent contract identifying failure stages and transactional boundaries, plus the minimum adapter responsibilities a consumer framework must provide. Use Candle only as one evidence source, not as the core abstraction.
