---
id: remove-the-workload-shapes-from-the-concatenate-normative-definition
title: Remove the workload shapes from the concatenate normative definition
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## A pinned-workload shape reaches canonical operation identity

`CONCATENATE_F32_NORMATIVE_DEFINITION` in `crates/tiler-ir/src/semantic/concatenate.rs` contains the sentence:

> `[8, 0, 128] with [8, T, 128] on axis 1 yields [8, T, 128] whose elements are the second operand's, `

Those are the pinned `Qwen3-0.6B` decoder's **8 KV heads and 128 head dimension**. The string is not documentation — it is encoded into identity: `encode_operation_definition` in `crates/tiler-ir/src/semantic/registry.rs` does `push_slice(output, definition.normative_definition().as_str().as_bytes())`.

**So a change to the pinned workload rewrites a registered operation's identity.** That is the leak `AGENTS.md` forbids when it says to use examples to exercise general machinery rather than specialize semantics around one case, and to keep the compiler core independent of any one consumer.

### It is the only family that does this — verified exhaustively

The coordinator scanned every `*_NORMATIVE_DEFINITION` in `crates/tiler-ir/src/semantic/`:

| family | concrete shapes in its normative definition |
| --- | --- |
| `broadcast` | none |
| `catalog` (complex) | none |
| **`concatenate`** | **`[8, 0, 128]`, `[8, T, 128]`** |
| `gather` | none |
| `reindex` | none |
| `slice` | none |

Every sibling states its zero-extent and empty-operand rule **abstractly**. Concatenate is the outlier, so the fix is to match the established pattern rather than to invent one.

## What to do

State the rule without the instance — something of the shape "a zero-extent operand is admitted and contributes no coordinate; the result takes the other operand's elements". **An identical illustration already lives in the doc comment further up the same file**, which is the correct home for it: doc comments are not encoded.

## The consequence that makes this more than a wording fix

Changing a normative definition **changes the operation's canonical identity**, which steps the semantic-graph identity domain and moves every pinned identity that folds it. So this is not a cosmetic edit:

- **Recompute every pinned identity on your own merged tree** and report which moved and which did not. Do not carry pin values from any ticket body — two branches moved shared pins from different bases on 2026-08-07 and neither's values survived.
- Coordinate with `carry-a-sourced-shape-on-semantic-values`, which is **live right now** and itself steps `tiler.semantic-graph.v2 → v3`. Two independent identity-domain steps in flight is exactly the shared-identity collision `AGENTS.md` says to serialize. **Check whether that ticket has landed before starting, and if it has not, say so and stop** rather than racing it.

## Closes when

No normative definition in `crates/tiler-ir/src/semantic/` contains a concrete tensor shape — re-run the scan above and report the table; the concatenate rule reads as generally as its five siblings; every pinned identity is recomputed on the merged tree with moved/unmoved stated per pin; and the identity-domain step is coherent with whatever `carry-a-sourced-shape-on-semantic-values` did.
