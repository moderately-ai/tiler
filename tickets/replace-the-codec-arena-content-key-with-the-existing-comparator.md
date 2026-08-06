---
id: replace-the-codec-arena-content-key-with-the-existing-comparator
title: Replace the codec arena content key with the existing comparator
status: in-progress
priority: p1
dependencies: []
related: [measure-artifact-decoder-allocation-amplification]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: []
paths: []
tags: [artifact, codec, performance, security]
claimed_from: todo
assignee: agent-codec-key
lease_expires_at: 1786048130
---
`decode_artifact` allocates a peak of **1,569,620,906 bytes** while validating a
**226,214-byte** envelope, and a forged envelope that will be *rejected*
allocates exactly the same. That is 6,939x amplification from bytes a producer —
or an attacker — chooses. Every consumer that decodes artifact bytes it did not
produce is exposed, the expansion cache validating a stored bundle among them.

Measured in
[`spikes/artifacts/decoder-allocation/`](../spikes/artifacts/decoder-allocation/README.md)
and reported in
[the research note](../docs/research/artifacts/decoder-allocation-amplification.md);
`measure-artifact-decoder-allocation-amplification` found it and could not fix
it inside its scopes.

## The cause, exactly

`super::model::expression_keys` derives one canonical content key per ABI arena
node with `tiler_ir::program::abi::expr_key`, which frames each operand's
**whole key** inside its node's key. A chain of depth `d` carries a key linear in
`d`, so an arena of `d` such nodes carries key bytes quadratic in `d`. Peak live
during a decode, no object carried, measured:

| Chain nodes | Envelope bytes | Peak live | x envelope |
| --- | --- | --- | --- |
| 0 | 114,083 | 283,005 | 2.5 |
| 128 | 117,798 | 1,770,867 | 15.0 |
| 512 | 128,550 | 25,997,811 | 202.2 |
| 1,024 | 142,886 | 103,258,099 | 722.7 |
| 2,048 | 171,558 | 411,919,347 | 2,401.1 |
| 4,000 | 226,214 | 1,569,620,906 | 6,938.7 |

Each doubling of the chain multiplies the peak by four. The last row sits at
`MAX_ABI_EXPRESSIONS`, so it is the governed bound measured rather than
extrapolated to.

## The fix already exists and is public

`tiler_ir::program::abi::compare_expr_nodes` is a total, content-derived order
that needs no numbering and is exactly injective, and its own documentation
gives this exact reason for existing: "Materializing a key per node embeds that
node's whole subtree, which is quadratic on a chain ... A comparison walks both
subtrees and stops at the first difference, so it never materializes one."

The identity encoder already uses it, through `canonical_arena_traversal`. The
codec does not, so the crate carries **two definitions of canonical arena order**
that only happen to agree. The three key-based sites are
`codec::model::canonical_expression_order`, the precondition sort in
`codec::model::project_entries`, and the duplicate check in
`codec::decode::parse_expressions`; `codec::validate` reads the same table for
its launch-precondition order check.

## Why it is a schema step rather than a refactor

The two orders are **not** the same relation. `expr_key` frames each operand with
an eight-byte length prefix, so comparing two keys compares operand *lengths*
before operand content, while `compare_expr_nodes` compares structure directly.
Switching therefore changes which byte string is *the* canonical encoding of a
given artifact:

- `MANIFEST_SCHEMA` takes a **major** step, for the reason every earlier one did.
- Every pinned or golden envelope byte in the workspace is rebaselined on the
  merged tree — `tiler-cache`, `tiler-macros`, and the prototypes hold some.
- **Artifact identity does not move.** `encode_identity` numbers the arena
  through `canonical_arena_traversal`, which is invariant to arena permutation,
  and `variant_order` already derives precondition order with the comparator.
  Confirm this on the merged tree rather than taking it from here.

Deleting `codec::model::expression_keys` is the check that the change is
complete: nothing may re-derive a key table on the decode path.

## Closes when

The codec orders and deduplicates arena nodes through `compare_expr_nodes`,
`expression_keys` is gone from the decode path, `MANIFEST_SCHEMA` states the
step with its reason, every rebaselined pin is recomputed on the merged tree, the
spike's arena rows are re-run and recorded beside the retained ones, artifact
identity is confirmed unmoved, and `make full` passes.

## Confirm the forger reach, or record that it is not reachable

The retained rows prove a *producer* can impose the cost and that a decode which
ends in rejection pays it in full. What they do not prove is that a manifest
carrying the chain can be forged from bytes alone. `parse_expressions` runs
inside `parse_manifest`, before any identity check, so the reading is that it
can; confirming it needs one hand-built manifest with a repaired manifest digest.
Do that before sizing the fix, because a producer-only cost and an
attacker-reachable one are different priorities.
