---
id: repair-research-evidence-residuals
title: Repair research evidence residuals
status: todo
priority: p2
dependencies: []
related: []
scopes: [research/embedding, research/numerics, research/target-profiles, research/region-search, research/runtime, research/transfers, contracts/artifacts]
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
