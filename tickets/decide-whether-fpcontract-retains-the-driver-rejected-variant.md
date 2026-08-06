---
id: decide-whether-fpcontract-retains-the-driver-rejected-variant
title: Decide whether FpContract retains the driver-rejected variant
status: awaiting-decision
priority: p3
dependencies: []
related: [record-or-validate-the-fast-honor-pragmas-selection]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [public-boundary, metal-aot, decision]
---

## The question, for Tom

`FpContract::FastHonorPragmas` is a public enum variant that no reachable toolchain accepts (the `metal` driver's admitted set is exactly `off`/`on`/`fast`, measured 2026-08-06 on the Xcode 27.0 / Metal 32023.921 row) and that is semantically redundant on that row (`fast` measurably honours source contraction pragmas, despite the driver's own help text). The measurement, the provenance (the value came from the driver's help text, which is inherited clang wording), the fail-closed behaviour, and a gate-time watcher that fires if a future toolchain starts accepting the value are all recorded at the variant (`crates/tiler-metal-aot/src/input.rs`) by [`record-or-validate-the-fast-honor-pragmas-selection`](record-or-validate-the-fast-honor-pragmas-selection.md).

**The options.** Keep the variant as the documented dead option (status quo — the semantic distinction is real in clang, the watcher guards the claim, and pre-alpha surface stability is not a constraint the repo honours); or remove it (the repo's own rule is that superseded internal paths go once replacements are complete, and a value no driver accepts arguably fails the "extensible does not mean unknown behaviour is optimizable" bar — but removal is an enum-surface change, which is why this is parked rather than done). Removing is a one-variant deletion plus the watcher retargeting to assert the admitted set's size.

## Closes when

Tom names keep-documented or remove; either way the variant's doc or its removal commit cites this decision with provenance.
