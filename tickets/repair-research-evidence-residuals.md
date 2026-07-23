---
id: repair-research-evidence-residuals
title: Repair research evidence residuals
status: todo
priority: p2
dependencies: []
related: []
scopes: [research/embedding, research/numerics, research/target-profiles, research/region-search, research/runtime, research/transfers, contracts/artifacts, contracts/navigation, research/documentation]
shared_scopes: [project/tickets]
paths: []
tags: [research, documentation, evidence]
---
Adversarially verified measurement and provenance defects that survived the evidence-provenance reconciliation (2026-07-23 audit):

- docs/research/embedding/embedded-artifact-costs.md misstates which table cells are six-run medians (the "same, release, 8 unique" row is verifiably six-run); recompute from the checked-in fixtures and correct the aggregation prose;
- docs/research/numerics/sound-region-analyzer-spike.md quotes two timing pairs ("13/1171 ms", "8/914 ms") absent from the measurements.json it cites; re-derive or remove them, and reconcile the spike's `informs` frontmatter with its own Traceability prose, which names numerical-semantics as a destination;
- docs/research/target-profiles/physical-feasibility-model.md's Candle source claims and docs/research/region-search/exhaustive-region-oracle.md's Burn OperationFuser claim lack the inspected commit or version AGENTS.md requires for source claims; pin exact revisions;
- docs/backends/metal.md widens the compile-only Apple probe into a "measured strict baseline" on a "qualified" toolchain, against the probe's explicit "not qualified for numerical conformance" boundary; restate the claim within the measured boundary;
- docs/research/runtime/candle-metal-post-wait-error-checking.md still states the separately downloadable Metal Toolchain is not installed although the repository records the authorized 17F109/32023.883 installation; date or update the measurement boundary; and
- docs/research/transfers/transfer-synchronization-and-resource-lifetime.md claims incorporation into physical and runtime contracts that contain none of its content; restate its disposition accurately or add the incorporating references.

Run the full documentation gate before completion.

## Outcome

Each defect was re-derived from the checked-in fixture or the inspected source
before the prose was changed.

- **Embedding aggregation.** Recomputing `results.json` shows 27 named cases of
  exactly three fresh builds each (81 rows). `count-8-identical` and
  `boundary-same-release-identical` are one configuration, as are
  `count-8-unique` and `boundary-same-release-unique`. The counts table's
  reported 0.28 s / 139 MiB and 0.27 s / 138 MiB match the six-run merged
  medians and match neither three-run submatrix alone, so **two** rows are
  six-run medians, not the single "central" cell the prose claimed. The crate
  boundary table's `same, CGU 16, LTO off` row reports the three-run
  boundary-submatrix median (0.26 s / 141 MiB) of the same configuration; that
  divergence is now stated. Every other cell in all four tables was recomputed
  and already agreed with the fixture.
- **Daisy timings.** `measurements.json` records timing pairs for exactly four
  profile invocations (16/1012, 8/1033, 320/1451, 192/1350 ms). Neither
  "13 / 1171 ms" nor "8 / 914 ms" appears, and no provenance for them exists.
  `relational_ratio` is one of the seven functions the runner's default profile
  requires, so its cell is now "included in batch"; `gradual_subnormal_add`
  lives in `boundary_regions.scala`, which `daisy_runner.py` never invokes, so
  its cell is now "not recorded". The Outcome section's dependent
  "roughly 0.9--1.5 seconds" total-time range became 1.0--1.5 s once the
  invented 914 ms was removed. `informs` gained
  `tiler.contract.numerical-semantics`, reconciling the frontmatter with the
  Traceability prose and with that contract's own `evidence` edge.
- **Source revisions.** Both claims were reverified against local checkouts
  before pinning. Candle `31f35b147389700ed2a178ee66a91c3cc25cc80d` (0.11.0)
  exposes only `max_total_threads_per_threadgroup`, caps `get_block_dims` at a
  1024-thread product, keys its pipeline cache on `(KernelName,
  Option<ConstantValues>)`, and forwards threadgroup bytes plus two dispatch
  modes. Burn `e5467f02c3cf88eb5d709f190c170005ce26038d` (0.22.0-pre.1) defines
  `FuserStatus::{Open, Closed}`, `FuserProperties { score, ready }`, and
  `Block::still_optimizing`, which keeps growing while any builder is open.
- **Metal compile probe.** `docs/backends/metal.md` called the toolchain
  "qualified" and the flags a "measured strict baseline". The probe measured
  only flag acceptance and compile success; the apple-targets research qualifies
  that row for bounded compile and same-host reproducibility and explicitly not
  for the runtime matrix or numerical conformance. The claim now states exactly
  that.
- **Metal Toolchain boundary.** The `xcrun metal -v` failure is a 2026-07-20
  observation that predates the user-authorized installation of component build
  17F109 / `com.apple.dt.toolchain.Metal.32023.883`, which the apple-targets
  research records and its 2026-07-21 rerun used. The boundary is dated, and
  the report now says compiler availability was never the reason no fault was
  injected.
- **Transfers disposition.** Only `docs/backends/cpu.md`, itself a proposed
  not-started contract, names this report as evidence, and it does so for a
  physical-resource boundary. `docs/artifact-abi.md`,
  `docs/integration/candle.md`, and `docs/backends/metal.md` contain none of its
  content, and no ADR cites it (ADR 0047 takes its evidence from the
  device/memory-domain research). `disposition` moved from `adopted` to
  `pending`, the incorporation claim was replaced, and Traceability now names
  the one real inbound reference and the still-open destinations.
- **Documentation spike freshness.** `scripts/docs.py` and
  `scripts/tests/test_docs.py` changed in `6c74a15` on 2026-07-23; the
  experiment record's `last_verified` was bumped to 2026-07-23 after its 18
  mutation tests, `docs.py validate`, and `docs.py render --check` were rerun.

`docs/research/README.md` was regenerated for the `informs` and `disposition`
changes.

**Known blocker outside this ticket:** `scripts/docs.py validate` reports
`tickets/prototype-typed-explain-infrastructure.md: done ticket requires ##
Outcome`. Base commit `e29fa193` flipped that ticket to `done` without an
Outcome, and validating a pristine archive of that commit reproduces the error
as the sole failure. It is left for its owning ticket rather than repaired with
an invented completion narrative.
