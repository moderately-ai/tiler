---
id: decide-the-invocation-gather-artifact-obligation-and-old-reader-fence
title: Decide the invocation-gather artifact obligation and old-reader fence
status: todo
priority: p1
dependencies: [decide-the-conditional-coverage-authority-for-invocation-gather-validation]
related: [admit-an-invocation-scoped-gather-index-validation-receipt, accept-the-invocation-scoped-gather-validation-public-surface]
scopes: [contracts/decisions, contracts/artifacts, research/artifacts, research/cache]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [decision, needs-tom, artifact, gather, validation, fail-closed, identity, schema]
---
## User-visible outcome

The conditional gather requirement has one exact artifact-owned record and compatibility fence. New readers reconstruct and validate its complete identity-bearing obligation; old readers refuse before interpreting or dispatching it; absence never defaults into satisfied validation authority.

## Exact-base Facts — `6e713e12`

- **Fact — no artifact carrier exists.** `VariantData` in `crates/tiler-artifact/src/program/model.rs` carries selected physical implementations, deferred predicates, live-device route requirements, entries, and scope cells, but no invocation-validation obligation.
- **Fact — the route-requirement family is the wrong owner.** `What belongs here, and the test that decides it` in `crates/tiler-artifact/src/program/requirement.rs` defines `RouteRequirement` as additional live-device evidence not derivable from the verified program. Gather validation is a semantic input obligation derived from the packaged conditional program, not a device property.
- **Fact — the exact compatibility consequence is undecided.** `MANIFEST_SCHEMA` in `crates/tiler-artifact/src/program/codec/encode.rs` is `(22, 0)`, `ARTIFACT_DOMAIN` in `crates/tiler-artifact/src/program/model.rs` is `tiler.artifact-program.v22\0`, and the derived required-feature set has no gather key. ADR 0108 says a fresh tagged row plus required feature may establish the old-reader fence and otherwise requires the major schema/domain step; it chooses neither, assigns no row placement/tag/key/version/domain, and says no Rust spelling.

## Decision packet

Starting from the exact artifact-facing conditional subject accepted by the dependency, compare:

- a conditional, length-framed obligation run whose presence derives a governed required-feature key and preserves the existing schema/domain where exact old-reader bytes prove early refusal;
- the appropriate manifest component/major and artifact-identity-domain step, with no legacy default, when the row changes framing an older reader can misinterpret; and
- typed deferral if neither can preserve all accepted identity and reader guarantees.

Do not reuse live-device route requirements, prepared-entry predicates, backend feature payloads, arbitrary opaque callbacks, or a generic validation registry. Specify the exact artifact model/view/builder records, fields and canonical order; run and row tags; limits; manifest placement; decoder and structural checks; identity preimage; required-feature derivation; component/manifest/domain versions; cache and explain consequences; old-artifact policy; and typed construction/codec errors.

Apply the Pareto-complete decision gate and obtain Tom's decision. A feature name without an exact byte-position/reader trace, or a schema bump without its identity/cache derivation, is incomplete.

## Closing checks and negative controls

- Pin reader commit `6e713e12`, the exact new artifact bytes supplied to that reader, the reproducing command, and the first typed refusal; prove it precedes variant interpretation and dispatch. Then remove only the proposed fence while retaining the new row and pin the distinct first result. At implementation time repeat both controls against the immediate pre-change parent reader, pin that reader commit and the exact bytes it receives, and record its first failure. If either reader can misparse or reach routing, the design must take the major step.
- Independently perturb every obligation field, field order, row tag, run tag/count, conditional-program identity and occurrence/binding coordinate; each must move artifact identity or be refused by a named check.
- Drop the obligation while retaining conditional program coverage, and retain an obligation on a proof-only program, independently. Artifact construction or decoding must reject each contradiction; zero rows is legal only when the packaged program proves it owes zero.
- Duplicate, reorder, truncate, over-limit and unknown-tag rows separately; pin canonical ordering, complete type-derived censuses, parser budgets and exact diagnostics.
- Prove an old artifact cannot acquire an implicit empty/satisfied obligation under the new reader, and a cache key cannot compare equal across different obligation subjects or across any required domain step.
- Record exact accepted names/tags/domains/versions/errors, old-reader bytes and failure text, acceptance provenance and landed contract hashes in the runtime-surface and implementation tickets.

## Non-goals

Runtime validation, receipt creation or consumption, storage ownership, public facade call sites, Metal emission, device or mutable validation, inline checks, callbacks, assertions, fallback, or implementing the chosen artifact record.

## Closes when

Tom has accepted one exact artifact obligation and compatibility stance; every field, tag, version, identity/cache effect, parser bound and typed error is assigned; both fence perturbations have reproduced pinned first-failure text against the exact `6e713e12` reader/bytes and are specified for repetition against the implementation's immediate pre-change reader/bytes; and the runtime ticket consumes the exact decoded view rather than inventing one.
