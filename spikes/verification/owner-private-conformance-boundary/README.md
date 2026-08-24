# Owner-private conformance boundary fixture

This fixture tests Rust visibility properties that constrain Tiler's owner-inventory design. It does not propose a production API.

Run:

```sh
sh spikes/verification/owner-private-conformance-boundary/check.sh
```

The checker demonstrates five distinct outcomes:

1. A dependency's `#[cfg(test)]` item is absent when a consuming crate is tested.
2. A dependency's `pub(crate)` item remains inaccessible.
3. A Cargo feature can make the item reachable only by exposing a conditional public API.
4. An owner-local unit test can read private state and emit a bounded manifest through an explicit output path.
5. Gating a subject, its declaration row, and its implementation behind the same Cargo feature changes the reported population while source and toolchain remain unchanged, so reporter configuration must participate in identity or applicability must be target-independent data.

The private-emitter result proves transport feasibility only. It does not select a schema, process orchestrator, receipt identity, configuration matrix, or new workspace crate, and it does not show that parsing test-harness stdout is acceptable. The retained design requires a dedicated bounded output channel and source/configuration-bound validation receipt instead.
