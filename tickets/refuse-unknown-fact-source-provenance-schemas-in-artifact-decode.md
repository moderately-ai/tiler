---
id: refuse-unknown-fact-source-provenance-schemas-in-artifact-decode
title: Refuse unknown fact-source provenance schemas in artifact decode
status: review
priority: p1
dependencies: []
related: [record-the-compilation-selection-in-target-measurement-provenance, carry-required-compilation-selection-identity-on-compile-profile-contexts]
scopes: [implementation/ir, implementation/artifact, contracts/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, provenance, identity, correctness, fail-closed]
claimed_from: todo
assignee: worker-refuse-unknown-schema
lease_expires_at: 1786631597
---
## User-visible outcome

An artifact carrying an unsupported fact-source provenance schema is rejected with a typed error before any field is interpreted. It is never normalized into the current schema.

## Source-first audit — 2026-08-13 at `cd1f76da`

**Fact — verified.** `decode_provenance` in `crates/tiler-artifact/src/program/realization/codec.rs` reads the incoming schema into `_schema`, discards it, decodes the remaining bytes as the current grammar, and reconstructs the value through `FactSourceProvenance::new`. Anchor: `let _schema = cursor.u32()?;`. The comment above it claims `is_valid` in `check_references` is the one place provenance validity is decided.

**Fact — verified.** `FactSourceProvenance::new` in `crates/tiler-ir/src/numerics.rs` always stamps `schema_version: FACT_SOURCE_PROVENANCE_SCHEMA_VERSION`. The constant is `pub const FACT_SOURCE_PROVENANCE_SCHEMA_VERSION: u32 = 3`. Changing that constructor's contract is out of scope.

**Fact — verified.** `FactSourceProvenance::is_valid` requires `self.schema_version == FACT_SOURCE_PROVENANCE_SCHEMA_VERSION`. After `new`, that comparison cannot refuse a foreign *incoming* schema, because the incoming number was discarded. The decode comment's justification is therefore false for the defect this ticket names: `is_valid` never sees the wire schema.

**Fact — verified.** Encode writes whatever `schema_version` the value holds (`bytes.extend_from_slice(&schema_version.to_be_bytes())` in `FactSourceProvenance::encode`). `DeliveredRealizationRecord::canonical_bytes` encodes each evidence row through `source.encode`. A decode that reconstructs via `new` therefore re-encodes schema 3 even when the incoming bytes named something else.

**Fact — verified.** `check_references` calls `row.source().is_valid()` and reports `IncompleteProvenance`. That is a completeness check on a reconstructed value, not a schema-dispatch. `DeliveredRealizationRecord::from_canonical_parts` stores the reconstructed source as given. The only production consumer of these bytes is `decode` in this codec, reached from `crates/tiler-artifact/src/program/codec/decode.rs` as `decode_realization(cursor.slice()?)`. The `decode_provenance` in `crates/tiler-artifact/src/program/codec/payload.rs` decodes a `ProvenanceDraft` for payload metadata, not `FactSourceProvenance`.

**Fact — verified, no older decoder.** `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` was introduced at 3 in `2ff6bd97` and has never been another number in production Rust. No other crate implements a version-specific `FactSourceProvenance` body decoder. Schemas 1 and 2 have no decoder to preserve; refusing them does not drop a supported artifact. The spike at `spikes/numerics/delivered-realization-record/src/codec.rs` copies the same `_schema` discard and is not a production decoder.

**Coordinator inference — confirmed.** Because `new` stamps the current schema, a foreign schema whose remaining bytes happen to parse as the current grammar is silently normalized. The repair is explicit dispatch before the body is read, not a second `is_valid` check after reconstruction.

## Fact

`decode_provenance` in `crates/tiler-artifact/src/program/realization/codec.rs` reads the incoming schema into `_schema`, discards it, decodes the remaining bytes as the current grammar, and reconstructs the value through `FactSourceProvenance::new`. That constructor emits the current schema, so a foreign schema can be silently normalized if its remaining bytes happen to parse. The decode comment that `is_valid` in `check_references` is the one place provenance validity is decided is false for this defect: `new` has already overwritten the incoming schema.

## Required delivery

- Make provenance-schema dispatch explicit and exhaustive before decoding the body.
- Support only schemas with an actual decoder. Unknown, newer, and retired schemas have typed, distinguishable refusal; there is no current-schema fallback.
- Preserve the incoming schema through any accepted version-specific decode so canonical re-encoding cannot silently change it.
- Re-read every artifact and compiler consumer before choosing whether an older schema is supported or refused.
- Perturb only the schema bytes of an otherwise valid delivered-realization record and quote the exact refusal. Also prove current-schema round trip and canonical-byte equality.

## Non-goals

This ticket does not add compilation-selection provenance, migrate a target profile, or reinterpret old bytes under a new grammar.

## Closes when

Every admitted provenance schema has an explicit decoder and every other schema fails closed before its body is consumed.

## Outcome

Decode now matches the incoming fact-source provenance schema before any body field is read. Schema 3 has the only explicit decoder (`decode_provenance_v3`). Unknown, newer, and retired schemas are distinct `RealizationCodecError` variants under the rule `unsupported-provenance-schema`. This generation lists no retired number: `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` was introduced at 3 and no predecessor wire grammar was implemented. `FactSourceProvenance::new` is unchanged and still stamps schema 3; the v3 arm is reached only after that exact incoming schema was matched, so canonical re-encoding cannot change it.

The three new variants are **labelled drafts** on the existing public `RealizationCodecError` enum (already `pub` and `#[non_exhaustive]`). No new public type was added. No identity domain or pin moved. Current-schema bytes are byte-identical through encode → decode → encode.

**Support-matrix / dtype-maturity:** this work advances none. It is a decode fail-closed repair.

### Perturbation refusal text

Only the four schema bytes of an otherwise valid delivered-realization record were rewritten.

- schema 1: `unsupported-provenance-schema: UnknownProvenanceSchema { version: 1 }`
- schema 4: `unsupported-provenance-schema: NewerProvenanceSchema { version: 4 }`

A damaged body byte after schema 4 still reports `NewerProvenanceSchema { version: 4 }`, so the body is not interpreted. Current-schema round trip and canonical-byte equality hold.

### Commands

```
cargo test -p tiler-artifact -p tiler-ir
```

Passed: tiler-artifact 268 lib tests (1 ignored), tiler-ir unit and integration tests including trybuild, plus crate doc-tests.

```
cargo test -p tiler-artifact --lib an_unsupported_provenance_schema_is_refused_before_the_body_is_read -- --nocapture
```

Passed.

```
cargo clippy -p tiler-artifact -p tiler-ir --all-targets -- -D warnings
```

Passed.

```
RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-artifact -p tiler-ir --no-deps
```

Passed.

```
tkt lint
```

`ok: no problems found`

```
git diff --check
```

Clean.

```
tkt guard --base main --format json tkt/refuse-unknown-fact-source-provenance-schemas-in-artifact-decode
```

`severity: ok`, `under_declared: []`. Re-run after the review commit so the guard sees the files.

### Unsupported cases

- Schema 1, 2, and 0: unknown; no decoder.
- Schema > 3: newer; body not read.
- Retired: typed variant exists; this generation emits none.
- Incomplete current-schema provenance still fails as `incomplete-provenance` after the v3 body is decoded.

### Identity / pin blast radius

None. Current-schema encode bytes are unchanged. `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` stays 3. `DELIVERED_REALIZATION_DOMAIN` is untouched.
