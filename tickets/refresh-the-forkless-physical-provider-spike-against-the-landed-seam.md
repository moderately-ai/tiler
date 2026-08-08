---
id: refresh-the-forkless-physical-provider-spike-against-the-landed-seam
title: Refresh the forkless physical-provider spike against the landed seam
status: todo
priority: p2
dependencies: [drive-an-external-physical-implementation-provider-through-compilation]
related: [prototype-a-forkless-custom-metal-physical-provider, record-the-landed-physical-provider-seam-in-adrs-0078-and-0090]
scopes: [research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, spike, evidence]
---
## User-visible outcome

The forkless physical-provider spike either compiles and drives the landed seam end to end from a genuinely out-of-tree crate, or its retained compile-fail goldens state the boundary that actually holds — so the spike stops asserting a blocker that was removed.

## Why this exists

`drive-an-external-physical-implementation-provider-through-compilation` landed the installable seam on 2026-08-08 and did not hold `research/extensions`, so the spike was not touched. The spike's probe takes `tiler-compiler` by `path` (`spikes/extensions/forkless-physical-provider/probe/Cargo.toml`), so it tracks the live tree rather than the commit its Measurement is pinned to.

**Fact, verified at `750b29e0`.** `spikes/extensions/forkless-physical-provider/probe/tests/ui/fail/no_physical_provider_installation_seam.stderr` pins `error[E0599]: no method named \`with_physical_providers\` found for struct \`CompileRequest<'a>\``. That method now exists — `grep -n "fn with_physical_providers" crates/tiler-compiler/src/session.rs` returns one line — so the golden cannot be reproduced. The fixture's own header comment also cites `crates/tiler-compiler/src/pipeline/planning.rs:171` and `request.rs:542` as the blockers, both stale.

**Fact.** Spikes are run manually and gate nothing (`AGENTS.md`, Research), so this is a stale artifact rather than a red gate. Nothing in `make full` exercises it.

## What this decides

[ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)'s Alternatives section rejects treating "installed" and "visible" as one obligation, and cites this fixture as the direct measurement behind that elimination. The elimination is unaffected — the landing needed two independent changes, which is exactly what the fixture claimed — but the record now carries a dated correction saying its present-tense evidence sentence no longer reproduces. Re-running the spike is what would let that correction be replaced by a measurement.

The upgrade this would also supply is the operation-extension contract's rung boundary: the landed evidence is an integration test inside the defining package, and a genuinely out-of-tree crate driving the seam is the measurement that contract names as stronger.

## Closes when

The spike builds at a recorded commit, its `README.md` and `results/` state what the landed seam does and does not admit, every retained compile-fail golden is reproduced or replaced with the boundary that actually holds, and ADR 0090's dated correction is updated to cite the re-run rather than the absence.

## Graph maintenance

- Do not delete a compile-fail fixture whose boundary still holds; four bypasses are pinned as `compile_fail` doctests in `crates/tiler-compiler/src/physical_provider.rs` and the spike should state the same boundary from outside the tree rather than a different one.
- If the spike finds the out-of-tree path blocked by something the integration fixture cannot see, that is a defect in the landed surface and belongs in its own ticket rather than in a spike note.
