---
id: record-explain-ownership-decision
title: Record the explain ownership decision as an ADR
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [contracts/decisions, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, explain]
claimed_from: todo
assignee: agent-record-explain-ownership-decision
lease_expires_at: 1784827223
---
Tom decided on 2026-07-23 that typed explain infrastructure stays a compiler-owned module rather than becoming a `tiler-explain` crate. File the next sequential ADR recording it, and represent the explain authority in docs/compiler/optimizer.md, which owns Explainability.

The decision and its supporting evidence, all verified against the merged implementation:

- `tiler_compiler::explain` remains a module of `tiler-compiler`. It is private today and may be promoted to `pub` when the public compiler API requires it; promotion is not itself a crate decision.
- Extraction today would invert the dependency graph. `explain.rs` imports `crate::fusion::FusionNumericalProof` and `crate::request::{VerifiedTargetRequest, LoweringProviderIdentity}`, so a `tiler-explain` crate would depend on `tiler-compiler` while `tiler-compiler` depends on it to emit records. Extraction would first require relocating those subject types.
- There is no second consumer. `tiler-artifact` depends only on `tiler-ir`, docs/artifact-abi.md never contemplates embedding explain traces, and only 2 of `explain.rs`'s 3,047 lines reference `tiler_ir`.
- ADR 0070 had just consolidated shared compiler IR into `tiler-ir` and dropped the compiler-to-artifact edge; minting a new crate immediately afterward would cut against that consolidation.

Record the reconsideration trigger explicitly, because it is the durable part: if a second crate must ever READ canonical traces, the record/subject/disposition vocabulary moves into `tiler-ir` following the `AbiExpr` co-location precedent of ADRs 0068 and 0070, with emission staying compiler-owned. A new crate is not the expansion path. Cite the merged typed-explain implementation as evidence and cross-reference ADRs 0068 and 0070.

This ticket records an accepted decision; it does not reopen it. Do not move code.
