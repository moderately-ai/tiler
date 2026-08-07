---
id: rename-the-workload-named-eps-constant-out-of-the-core-surface
title: Rename the workload-named eps constant out of the core surface
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/ir, implementation/reference, implementation/metal]
shared_scopes: []
paths: []
tags: []
claimed_from: todo
assignee: agent-eps-rename
lease_expires_at: 1786114012
---
## User-visible outcome

`tiler-ir`'s public surface names no vendor model, so the rule at `docs/roadmap.md:380` ("does the compiled public surface learn a transformer word") holds over compiled items and not only prose.

## Why this exists (leakage audit 2026-08-06, coordinator-verified: the constant at rms_norm.rs:94 + re-export at semantic.rs:176 is the only workload-named public item in crates/)

`RMS_NORM_F32_QWEN3_EPS_BITS` landed 2026-08-01, three days before the reclassification set the rule (2026-08-04); the sweep covered prose, not identifiers. It is inert — the schema has no default, every consumer is a test — so only the name leaks. Its siblings are already correctly named.

## The work

Rename to `RMS_NORM_F32_REFERENCE_EPS_BITS`; the doc-comment's Qwen3/`modeling_qwen3.py` provenance stays verbatim (provenance is evidence). No `pub use` alias — pre-alpha, remove the superseded name. Update the ~21 test/fixture sites and the two intra-doc links (`rms_norm.rs:24`, `standard_operations.rs:498`). Widen the roadmap rule's enumeration ("type, module, field, variant, or error") to cover consts as a delta in the same change. Pure rename: no identity, no value change. Verification: the audit's grep returns only the two PROPERTY false friends; rustdoc -D warnings on tiler-ir.

## Closes when

The rename lands with provenance intact, all sites updated, the rule enumeration widened, and the grep clean.
