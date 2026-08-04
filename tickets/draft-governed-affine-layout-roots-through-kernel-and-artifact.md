---
id: draft-governed-affine-layout-roots-through-kernel-and-artifact
title: Draft governed affine layout roots through kernel and artifact
status: todo
priority: p1
dependencies: [establish-a-dynamic-kv-physical-layout-authority, admit-live-extent-operands-to-payload-indexing]
related: [bind-the-kv-cache-through-the-artifact-and-runtime-interface]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, layout, abi, artifact, metal, identity, public-boundary]
---
## User-visible outcome

One compiled payload can address a governed positive-stride physical tensor whose stride is supplied at execution time, without treating the live value as semantic meaning or pipeline specialization.

## Authority and boundary

Consume [the dynamic KV physical-layout record](../docs/research/runtime/dynamic-kv-physical-layout.md). Reuse the accepted governed dispatch-parameter transport from `admit-live-extent-operands-to-payload-indexing`, but add a separately typed physical layout-root subject whose authority is a runtime storage descriptor rather than a semantic interface extent. Draft, document, and test the smallest bounded affine surface that realizes the initial rank-three F32 head-major KV recipe. This changes consequential public structured-kernel and artifact types plus the artifact schema; the exact tested surface is Tom's to accept. Do not self-accept it and do not implement a KV-specific duplicate beside a general bounded physical seam.

## Required work

- Add a typed layout-root declaration and address-use representation distinct from semantic roots, target properties, tensor buffers, launch builtins, and specialization values. Bound counts, ranks, arithmetic, root types, associations, and supported positive-stride recipes; arbitrary code, pointers, negative/zero strides, overlap, broadcast, raggedness, and caller-authored use sites refuse.
- Make structured-kernel verification prove every dynamic address operand is declared, typed, used by the binding it belongs to, and covered by the same bounds/ownership evidence as the resulting load or store. The old implicit-dense lowering is removed at migrated sites rather than retained as a second authority.
- Emit the governed root through a read-only Metal dispatch-parameter mechanism. Keep parameter transport separate from tensor-buffer roles; preserve positional ABI identity and fail when target capability or reflection cannot realize the exact signature.
- Carry the complete declaration/use/transport population through artifact construction, canonical ordering, codec, decode, validation, and views. The binding's reachable-span expression consumes the same governed root/use authority as payload indexing; it may not retain dense logical byte count as a second, smaller range. A missing, duplicate, extra, reordered, wrong-type, wrong-binding, or unreachable row is a typed refusal.
- Execute the identity step whole: enumerate every moved kernel/program/payload/artifact identity domain and pinned value, bump the owning manifest major schema and ledger, and recompute pins on the merged tree. Live root values never enter canonical bytes.
- Preserve one payload subject across `capacity = 18` and `capacity = 8,320`; neither capacity nor `C`/`S` may appear in artifact or pipeline specialization identity.

## Required evidence

- An executable rank-three fixture evaluates `base + head × head_stride + sequence × 128 + component` for `head_stride = 2,304` elements and proves `(1,0,0)` addresses byte 9,216 for both logical `C = 14` and `S = 15`. It also proves exact reached spans 71,680 and 72,192 bytes inside the 73,728-byte allocation.
- Deliberate omission, reordering, wrong association, overflow, unsupported recipe, and transport/reflection perturbations each make the exact responsible layer fail.
- Replacing the governed root with logical dense strides 1,792 or 1,920 elements makes the fixture fail rather than return bytes 7,168 or 7,680; replacing the reached spans with dense logical payload counts 57,344 or 61,440 fails independently.
- Targeted IR/compiler/artifact/Metal/build tests, identity blast-radius comparison, `tkt lint`, `git diff --check`, guard, and the full gate pass.

## Explicit non-goals

No KV state object, runtime storage observation, state publication, batching, ragged layout, paging, negative stride, in-place update, or performance claim. No public or schema acceptance by an agent.

## Closes when

The exact tested public/schema draft is accepted by Tom; the whole identity step lands; the negative fixtures prove the check can reject; unsupported cases are documented; and `route-governed-layout-roots-from-runtime-state` can consume decoded root declarations without restating them.
