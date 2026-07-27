---
id: stop-recomputing-pure-derivations-in-the-codec
title: Stop recomputing pure derivations in the artifact codec
status: todo
priority: p1
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [performance, artifact]
---
Duplicate work inside `decode`, each item a pure function of the same value computed more than once. No semantic change.

## Facts

**`canonical_identity()` twice per decode.** `decode.rs:102` derives it, then the canonicity re-encode at `decode.rs:113` reaches `encode.rs:194` and derives it again — same pure function, same value. The re-encode is **50% of decode time** (274 µs of 548 µs, measured), and this is a share of it available without touching the canonicity guarantee at all.

**`expression_keys` four times per decode** — `decode.rs:590`, `validate.rs:80`, and twice via `encode_identity` (`program/model.rs:1510`).

**`decode_metadata` `2 + E` times** where `E` is entries realized by the payload — `validate.rs:252`, `validate.rs:342` (per entry, deliberately, with a sound comment), and `view.rs:152`. Each call re-allocates a full `PayloadMetadata` including `source.to_vec()` at `payload.rs:403`, bounded at 16 MB.

**Every hashed byte is hashed twice.** Manifest SHA at `decode.rs:83` and again at `encode.rs:107`; per-section digest at `decode.rs:211` and again at `encode.rs:357`.

**`DecodedExpr::value_type()`** (`view.rs:849`) rebuilds the type vector from node 0 on every call, though `validate.rs:103-117` already computed the full table and discarded it.

## Scope

Derive each once and reuse. Where a re-derivation exists to *verify* rather than to produce, keep the verification and remove only the repetition.

This ticket does **not** decide whether the canonicity re-encode should exist — `decide-whether-the-canonicity-re-encode-is-redundant` owns that, and this work is worth doing whichever way it goes.

## Closes when

One decode derives the canonical identity once and the expression keys once, pinned by work-count guards; decode time is measured before and after; every existing codec test still passes unchanged; `make full` passes.
