---
id: stop-recomputing-pure-derivations-in-the-codec
title: Stop recomputing pure derivations in the artifact codec
status: in-progress
priority: p1
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [performance, artifact]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785180316
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

## Outcome

Partially done, and the part that landed is the largest of the four. **Decode fell from 662 µs to 501 µs — 24% — on a 26,126-byte envelope, paid back on every artifact load and every cache hit.**

**What landed: the identity is derived once per decode.** `decode` derived the canonical identity to compare against the manifest's, then ran its canonicity re-encode, which derived the *same identity from the same value* a second time. `encode` now splits: the public entry derives and delegates to `encode_with_identity`, which takes the identity as a parameter. `decode` passes the one it already has.

The parameter is documented at the site as being there because deriving it is not cheap and the one caller that needs it already holds the value — so a future caller has to pass it deliberately rather than get a second derivation for free.

**Nothing about the canonicity guarantee changed.** The re-encode still runs and still compares byte-for-byte; it simply stops re-deriving one of its inputs. Whether the backstop itself should exist is `decide-whether-the-canonicity-re-encode-is-redundant`, and this reduction stands whichever way that goes.

## Not done, and left explicitly

Three items from this ticket remain, each independent of the one above:

- **`expression_keys` runs four times per decode** — `decode.rs:590`, `validate.rs:80`, and twice through `encode_identity`. Worth attacking together with `encode-abi-expression-identity-in-linear-space`, since that ticket changes what the keys cost in the first place and doing them in the other order means measuring twice.
- **`decode_metadata` runs `2 + E` times**, each re-allocating a `PayloadMetadata` including a `source.to_vec()` bounded at 16 MB. The per-entry call at `validate.rs:342` carries a sound comment about not depending on having been reached, so the fix is to hoist the decode rather than to skip it — a real change to that function's shape and not a one-liner.
- **`DecodedExpr::value_type()`** rebuilds the type vector from node 0 on every call, though `validate.rs:103-117` already computes the full table and discards it.

Reopening rather than closing would misstate the state: the identity half is done and measured, the rest is untouched.

Gate: `make full` green (982 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck). Every existing codec test passes unchanged, which is what shows the encoding is byte-identical.
