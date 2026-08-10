---
id: catalog-the-kani-push-slice-framing-research-and-spike
title: Catalog the Kani push_slice framing research and spike
status: todo
priority: p3
dependencies: [spike-kani-push-slice-framing-over-a-symbolic-byte-run]
related: [catalog-the-kani-verification-research-and-spike]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---

## User-visible outcome

The manually maintained research and experiment catalogs route readers to the bounded `push_slice` framing result and state the same metadata as the records themselves.

## Facts read before filing — 2026-08-10, base `49d38237`

- `docs/research/README.md` owns its hand-maintained rows and lives in `contracts/navigation`; the Kani rows sort in **Artifacts, build, and toolchains**.
- `spikes/README.md` owns the parallel experiment catalog in that same scope and already explains the authorized Kani 0.67.0 host prerequisite for the predecessor spike.
- The dependency's branch adds `docs/research/verification/kani-push-slice-framing.md` with `disposition: "pending"`, evidence classes `executable-model`, `bounded-measurement`, `primary-source-synthesis`, and `informs: ["tiler.contract.correctness-and-testing"]`.
- It also adds `spikes/verification/kani-push-slice-framing/README.md` with `experiment_status: "reproducible"`, evidence classes `executable-model`, `bounded-measurement`, and support for the new research record.

## Verbatim-landable rows

Research catalog, beside the existing Kani record:

```md
- [Kani bounded verification of `push_slice` framing](verification/kani-push-slice-framing.md) — pending; executable-model, bounded-measurement, primary-source-synthesis; informs: [Correctness and testing](../correctness-and-testing.md); experiments: [Kani bounded verification of `push_slice` framing](../../spikes/verification/kani-push-slice-framing/README.md)
```

Experiment catalog, beside the existing Kani spike:

```md
- [Kani bounded verification of `push_slice` framing](verification/kani-push-slice-framing/README.md) — reproducible; executable-model, bounded-measurement; supports: [Kani bounded verification of `push_slice` framing](../docs/research/verification/kani-push-slice-framing.md)
```

## Closes when

Both rows are present, their links resolve after the dependency lands, and the surrounding Kani host-prerequisite prose still applies to both spikes without claiming that either follows the repository `rust-toolchain.toml` pin.
