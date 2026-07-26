---
id: exercise-the-expansion-cache-under-cargo-and-rust-analyzer
title: Exercise the expansion cache under Cargo and rust-analyzer
status: in-progress
priority: p2
dependencies: [port-the-cache-harness-to-the-production-bundle]
related: [implement-the-expansion-cache-protocol]
scopes: [research/cache]
shared_scopes: []
paths: []
tags: [cache, concurrency, frontend]
claimed_from: todo
assignee: agent-cache-exercise
lease_expires_at: 1785046812
---
The research note's seventh follow-up gate: run the harness under Cargo and rust-analyzer process patterns once the proc-macro spike exists.

ADR 0050's context is that "Cargo and rust-analyzer may run equivalent proc-macro expansions concurrently", and that is the workload the whole protocol was designed against. Everything measured so far uses a harness that spawns its own workers, which is a model of that workload rather than the workload.

## What this ticket owes

- The real process pattern: how many expansions run at once, whether they share a working directory, and whether rust-analyzer's and Cargo's expansions overlap in time on one key.
- Whether the per-key lock behaves as measured when the holder is a proc-macro server that may be killed and restarted by its editor.
- Whether the default cache root is reachable and private in both contexts, and what a sandboxed or CI environment overrides it to.

Blocked until there is a proc-macro layer to run under; `port-the-cache-harness-to-the-production-bundle` is the prerequisite that makes the harness exercise the real bundle.
