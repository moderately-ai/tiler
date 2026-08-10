Ticket: floor-the-trybuild-compile-fail-fixture-populations
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/floor-the-trybuild-compile-fail-fixture-populations/2a87a3669932_c99ac54950f2.md
Pre-edit content hash (from ledger): 2a87a36699322b3355b48360e364da855ed887d43779a372f158a4500c6600c3
Post-edit content hash: 351f71eac77186454a5b8562dc6f171ce43ae9de730bdfa6f7841d005f2240e6

Changes applied:
  - related frontmatter: linked close-the-fmt-blind-spot-over-the-trybuild-facade-fixtures and close-the-fmt-blind-spot-over-the-tiler-ir-trybuild-fixtures (graph hygiene).
  - Reframed live present-tense "are unfloored" / harnesses with "no floor" as historical at-open / gate-level absence; harnesses still register compile_fail without in-harness floors; counts kept as live sizes.
  - Softened ADR 0051 / "exactly what these globs carry" to Makefile's Preflight::commit doc-test vs trybuild-glob distinction.
  - Corrected pin claim: two tiler-macros named fail fixtures; seven more in TENSOR_FIXTURE_INVOCATION_PINS (invocation counts); population floor is Makefile count.
  - **Correction — 2026-08-10.** dated block summarizing the obsolete gate-absence and pin/ADR claims.
  - Added ## Outcome: delivery ab174f67…; seven Makefile test floors + facade pass in fmt; tiler-ir pass expansion; independent wrong-count failure; missing historical quote residual; sibling fmt tickets stay separate.

Optional items skipped (with reason):
  - none (optional related applied).

Residuals not applied (docs/crates/new tickets/authority):
  - none for floors (delivered). No Makefile/harness edit required. Historical per-floor failure quotes absent from closing record; live independent eq N−1 exits remain. Scratch zero-match trybuild crate not re-run (trybuild 1.0.118 source still supports the silence mechanism).

Verification:
  - files read:
    - tickets/floor-the-trybuild-compile-fail-fixture-populations.md (full, pre- and post-edit)
    - audit report 2a87a3669932_c99ac54950f2.md (full)
    - Makefile test/fmt floor lines and comment block on Preflight::commit vs trybuild globs
    - crates/tiler/tests/workspace_unsafe_sites.rs TENSOR_FIXTURE_INVOCATION_PINS
    - crates/tiler-macros delivery fixture(...) paths for family_cfg fail fixtures
    - on-disk fail/pass counts (9/4/7/6 and pass 10/1/2/1); ab174f67 ancestor of HEAD
  - checks:
    - shasum -a 256 tickets/floor-the-trybuild-compile-fail-fixture-populations.md → 351f71eac77186454a5b8562dc6f171ce43ae9de730bdfa6f7841d005f2240e6

Recommended next ledger state:
  integrated
