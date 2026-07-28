---
id: carry-the-binding-offset-through-the-runtime-route
title: Carry a binding's accessible offset through the runtime route
status: done
priority: p2
dependencies: []
related: [carry-the-byte-offset-of-a-partial-binding-view, route-the-runtime-loader-through-the-dispatch-record, carry-the-stage-execution-order-in-the-envelope]
scopes: [implementation/runtime, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifact, abi]
---
## User-visible outcome

A host running a plan whose binding addresses part of a buffer — a slice, an offset view, a shared scratch region — gets its storage bound at the right byte, or a refusal that says why. Today the loader refuses every such plan outright; this ticket replaces that interim refusal with a published, honoured offset, which is what makes a partial-window artifact *executable* rather than merely packageable and decodable.

`carry-the-byte-offset-of-a-partial-binding-view` made a binding row carry the offset its accessible range starts at, so an ABI slot may address part of the value it names. `crates/tiler-runtime` was written when every slot addressed its value whole, and it publishes nothing else.

**Fact — the loader evaluates the offset and refuses a nonzero one, as an interim.** `place_bindings` (`crates/tiler-runtime/src/load.rs`; find it with `grep -n "fn place_bindings" crates/tiler-runtime/src/load.rs`) evaluates `DecodedBinding::accessible_offset` under `AbiSubject::AccessibleOffset` and rejects a nonzero result as `LoadRejection::UnpublishedBindingOffset`, because `RoutedBinding` (`crates/tiler-runtime/src/load/route.rs`) holds `binding`, `transport`, and `accessible_bytes` and no offset — a host given one binds storage at byte zero, whatever the artifact says. The refusal was added when the offset landed, precisely so no route could reach that silent misplacement.

**Fact — a nonzero offset is reachable through decode, which is what changed.** When this ticket was drafted, `decode_artifact` refused a two-stage envelope through `tiler.artifact.feature.multi-stage-program` and the loader was protected by another layer's refusal. `carry-the-stage-execution-order-in-the-envelope` lifted that: `codec::tests::a_partial_binding_window_survives_encode_and_decode` decodes a two-stage plan whose scratch bindings start at byte 24. What still stands between a decoded nonzero offset and a host is only the loader's own refusal above, which is this crate's stated posture rather than an inherited one — the standard `route-the-runtime-loader-through-the-dispatch-record` records.

## The work

Publish the evaluated offset on `RoutedBinding` beside the extent and thread it to wherever a host places storage — `prototypes/serial-sum-run` binds through the Metal argument table and needs the offset at that call. Then remove `LoadRejection::UnpublishedBindingOffset` and the `require_zero_offset` gate, whose reason ends when the value is published.

Publishing the value and letting a host ignore it is not an option, because a host that never learns the offset existed cannot tell that it defaulted; the interim refusal exists so that failure mode has no route today.

**Owed test.** `require_zero_offset`'s unit test records that no in-crate fixture can present a nonzero offset through a decoded artifact — the smallest such plan is two stages, and assembling one needs the `tiler-ir` builders this crate deliberately does not depend on. Whoever takes this ticket must build that fixture where the loader's end-to-end probes live (`prototypes/serial-sum-run/src/proof.rs` assembles envelopes through the live builders and records why that is the right home), and prove the honoured path: a two-stage partial-window artifact routes, and the scratch slot's published offset is the artifact's, not zero.

## Graph maintenance

- When this lands, correct `RoutedBinding`'s doc in `route.rs` (it names the interim refusal) and the "fails closed on both counts" sentence in `docs/artifact-abi.md`'s dispatch-record future-work item, which cites this ticket as owning the honouring.
- If `preflight-every-entry-of-a-multi-stage-route` lands first, coordinate: its multi-entry dispatch work touches the same `place_bindings` path, and whichever lands second should re-run the other's probes rather than assuming composition.

## Outcome

**Decision — ratified by Tom on 2026-07-28 and implemented.** `RoutedBinding` publishes the evaluated range start through the additive `accessible_offset()` accessor beside `accessible_bytes()`, `place_bindings` carries both values, and the interim `require_zero_offset` gate and `LoadRejection::UnpublishedBindingOffset` class are gone. A replacement `RoutedByteRange` public type was considered and rejected: replacing `accessible_bytes()` would break the established boundary merely to regroup two values, while retaining it beside a range accessor would publish the extent twice and create two paths that must remain consistent. Re-evaluation downstream was eliminated because it would retain `AbiFacts`, duplicate the artifact evaluator, and permit a new refusal after routing commit. The serial-sum Metal host carries the offset through preflight, sizes every allocation through `offset + extent` with checked arithmetic, and passes that same byte to `set_buffer`.

**Measurement — pinned nightly on the `fa8fabd` checkout, 2026-07-28.** `cargo nextest run -p tiler-runtime -p tiler-prototype-run` passed 24 tests; `cargo test -p tiler-runtime --doc` passed one positive and four compile-fail doc-tests; and `cargo clippy -p tiler-runtime --all-targets -- -D warnings` passed. The new two-stage fixture rebuilds the compiler's verified materialized program with both scratch bindings addressing the upper half of an enlarged value, encodes and decodes it, then proves the routed producer and consumer each publish the artifact's nonzero offset and that host placement reaches through the end of the window. Perturbing its expected offset from 16 to 17 made the targeted nextest case fail at that assertion, proving the check can say no; the correct assertion was restored and passed. The final `make full` passed 1,136 dev-profile tests, all workspace doc-tests and rustdoc with warnings denied, 405 release-profile numerical tests, ticket lint, and shellcheck.

**Graph maintenance applied.** `RoutedBinding`'s interim-refusal documentation now describes the published start and extent, and `docs/artifact-abi.md` records both the multi-entry route and binding-offset follow-ons as closed.

## Closes when

A nonzero binding offset is published to a host and honoured at the binding call, `LoadRejection::UnpublishedBindingOffset` and `require_zero_offset` are gone, the two-stage loader fixture proves the honoured path end to end, `crates/tiler-runtime/src/load/route.rs` no longer describes a binding record that is complete only while offsets are zero, and `make full` passes.
