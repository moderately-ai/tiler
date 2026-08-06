---
id: record-or-validate-the-fast-honor-pragmas-selection
title: Record or validate the fast-honor-pragmas selection the measured toolchain rejects
status: in-progress
priority: p3
dependencies: []
related: [compile-an-elementary-function-golden-through-the-metal-toolchain, emit-the-contraction-pragma-as-a-declared-metal-realization]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [metal-aot, toolchain, fail-closed, doc-claim]
claimed_from: todo
assignee: agent-fp-contract
lease_expires_at: 1786038040
---

## The observation (elementary-golden work, 2026-08-06; coordinator-verified at source)

`FpContract::FastHonorPragmas` (`crates/tiler-metal-aot/src/input.rs:527`, spelled `fast-honor-pragmas` at `:538`) is rejected by the measured toolchain: `metal: error: unsupported argument 'fast-honor-pragmas' to option '-ffp-contract='` on the Xcode 27.0 / Metal 32023.921 row. The failure is closed and typed (`ToolFailure`), so nothing is silently wrong — but the enum offers a selection the measured row cannot deliver, and nothing records that anywhere a caller or a reader would find it.

## The question

Whether this is a doc fix (the variant's doc states the measured-row rejection with its boundary — the value may be valid on other toolchain rows, and clang's own `-ffp-contract` accepts `fast-honor-pragmas`, so the enum may be honestly wider than one row) or a validation gap (the input layer should refuse the selection against a target whose measured row rejects it, before the tool run). The fail-closed posture makes the doc fix the likely floor; a validation route would need a per-row capability fact the aot layer may not want to own. Note the emitted-pragma realization ticket is related: if the pragma route lands, `fast-honor-pragmas` is the one `-ffp-contract` value whose semantics interact with source-level pragmas.

## Closes when

The variant's doc states what the measured row does with it (with the boundary and the reproducing invocation), and either a validation is added with a test watched refusing, or the derivation for why tool-time failure is the right layer is recorded at the variant.
