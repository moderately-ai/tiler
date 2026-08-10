Ticket: admit-reassociated-contraction-schedule-alternatives
Exact audit base: c99ac54950f242d88d8dfe8335332bef0cf75f2d
Ticket content hash: f0ea5ecdccfb8e788b02ff9d5e2c45597de79615c153c8e08067af3f4eda17c7
Assigned checkout: /Users/tsanterre/workspace/github.com/moderately-ai/.worktrees/tiler/ticket-audit-2026-08-10-ro
Initial repository status: todo
Worker: wave1-07

Files read in full:
  - tickets/admit-reassociated-contraction-schedule-alternatives.md
  - tickets/implement-parallel-reduction-strategies.md (frontmatter + outcome/criteria; status done)
  - tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md (full)
  - tickets/realize-the-contraction-through-the-appendable-direct-path.md (frontmatter + outcome; status done)
  - tickets/realize-the-strict-contraction-on-metal.md (frontmatter + non-goals; status closed, superseded by tiled)
  - tickets/reduction-semantics-contract.md (frontmatter + outcome; status done)
  - tickets/implement-analytical-component-cost-model.md (frontmatter; status done)
  - tickets/enumerate-the-split-reduction-on-the-planning-frontier.md (frontmatter + outcome; status done)
  - tickets/admit-a-reassociating-contract-without-contraction.md (frontmatter + outcome; status done)
  - tickets/spike-first-metal-contraction-vertical.md (outcome section)
  - docs/research/scheduling/first-metal-contraction-realizations.md (full realization/measurement sections used for claims; delivery table)
  - docs/research/numerics/reduction-semantics-and-legality.md (order permissions, partial/seed contract, contiguous vs strided examples)
  - spikes/scheduling/metal_contraction_vertical/contraction_probe.py (topology models + split_topology case)
  - spikes/scheduling/metal_contraction_vertical/kernels.metal (ksplit_contiguous / ksplit_strided)
  - spikes/scheduling/metal_contraction_vertical/results/2026-07-31-timing-apple9-f32-msl4-macos26-m3pro-metal32023.883/timing-summary.tsv
  - spikes/scheduling/metal_contraction_vertical/results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/semantics-observations.tsv (split_topology rows)
  - crates/tiler-reference/tests/contraction_conformance.rs (split separator comments + strict value)
  - crates/tiler-ir/src/schedule/builder.rs (verify_contraction; multi_pass_family / cooperative_family refuse StrictTensorContraction)
  - crates/tiler-compiler/src/physical.rs (direct Contraction region construction)
  - ticketsplease.toml (workflow states + scope map)
  - docs/research/documentation/ticket-audit-2026-08-10/MANIFEST.md (audit base pin)

Identity checks:
  - Assigned RO path is readable and serves the ticket and source tree named in the brief.
  - Ticket body matches the brief's subject; content-hash and HEAD were taken as the brief's fixed assignment parameters (this worker environment has no shell tool for `git rev-parse` / `shasum`).

Per-Fact verdicts:
  1. [VERIFIED] Two spike split kernels exist: `ksplit_contiguous` (contiguous contracted-axis intervals, ordered merge) attributed uniquely to `contiguous_split+ftz`, and `ksplit_strided` (strided subsets) attributed uniquely to `strided_split+ftz`.
     Evidence: L3 candidate table; probe host maps; Metal kernels; attribution TSV.
     Raw source anchor: `ksplit_contiguous` partitions the contracted axis into contiguous intervals
     Construction path: spikes/scheduling/metal_contraction_vertical/kernels.metal `contract_ksplit_contiguous` / `contract_ksplit_strided`; contraction_probe.py TOPOLOGIES.
     Consumption path: L3 elimination rows; this ticket's delivery ask.
     Reproduction: read L3 table row for `ksplit_contiguous` / `ksplit_strided`; semantics-attribution.tsv consistent rows.

  2. [VERIFIED] At the spike's `split_topology` case the two kernels return different bits: `0xbb1d0683` vs `0xbb1d0672`.
     Evidence: semantics-observations.tsv split_topology rows; L3 prose; reference test comment.
     Raw source anchor: `split_topology	ksplit_contiguous	ok	bb1d0683`
     Construction path: searched vector in contraction_probe.py `cases["split_topology"]`.
     Consumption path: ticket measurement claim; crates/tiler-reference/tests/contraction_conformance.rs comment.
     Reproduction: `rg 'split_topology	ksplit_' spikes/scheduling/metal_contraction_vertical/results/`

  3. [VERIFIED] Reduction semantics: reassociation without permutation may combine only contiguous contributor intervals in order; a lane-strided partition reorders leaves and needs permutation.
     Evidence: reduction-semantics-and-legality.md inferences 4, partial contract, normative example 7.
     Raw source anchor: Noncontiguous lane assignment therefore also needs permutation permission
     Construction path: research contract adopted by ADRs 0012/0014/0022/0025.
     Consumption path: L3 attribution grounds; this ticket's dual-permission delivery.
     Reproduction: `rg 'Noncontiguous lane assignment' docs/research/numerics/reduction-semantics-and-legality.md`

  4. [VERIFIED] At `t_vocab_full` (M=1, N=151936, K=1024) `ksplit_contiguous` was fastest among measured candidates: ~4,247 µs vs MPS ~4,418 and `direct` ~4,757; cell ~146 GB/s bandwidth-bound.
     Evidence: L3 timing table; timing-summary.tsv settled_min; weight-byte derivation in L3.
     Raw source anchor: `t_vocab_full	ksplit_contiguous	1	151936	1024	ok	4247.375`
     Construction path: M3 Pro timing harness results under spikes/scheduling/metal_contraction_vertical/results/.
     Consumption path: ticket measurement paragraph; first-quantized-lm-profile reuses the same cell.
     Reproduction: compare timing-summary.tsv `t_vocab_full` rows; 622329856 / 4247e-6 / 1e9 ≈ 146.5 GB/s.

  5. [VERIFIED] Prefill: same contiguous-split kernel is roughly 5×–7× slower than `tiled` at M ≥ 128 (ticket's measurement bound; not a strategy indictment).
     Evidence: timing-summary ratios vs tiled: mlp_in 7735/1602≈4.83; mlp_out 10707/1599≈6.70; o 5091/1051≈4.84; prefill_mlp_512 30894/6395≈4.83. L3 text "roughly 5× to 7×".
     Raw source anchor: roughly 5× to 7× slower than `tiled` at every prefill cell
     Construction path: same timing-summary.tsv.
     Consumption path: ticket "This implementation is not the strategy" bullet.
     Reproduction: compute ratios from timing-summary.tsv prefill rows with M≥128.

  6. [VERIFIED] Required delivery is not present in `crates/`: no contraction schedule alternative for contiguous/strided K-split; `StrictTensorContraction` is refused by both multi-pass and cooperative split family admissions.
     Evidence: multi_pass_family and cooperative_family match arms return None for StrictTensorContraction; only ReductionTopology::Contraction direct path in physical.rs; no ksplit emission in crates/.
     Raw source anchor: `| ScalarProgram::StrictTensorContraction { .. } => None,`
     Construction path: crates/tiler-ir/src/schedule/builder.rs multi_pass_family / cooperative_family.
     Consumption path: this ticket's open outcome.
     Reproduction: `rg 'StrictTensorContraction \{ \.\. \} => None' crates/tiler-ir/src/schedule/builder.rs`

  7. [VERIFIED] Seed attaches once at the root-facing boundary (never per lane); partials carry `has_value` unless nonempty or proven-neutral padding is established.
     Evidence: reduction-semantics-and-legality.md Physical partial and multi-pass contract.
     Raw source anchor: The seed is attached once at the root-facing boundary
     Construction path: research contract / ADR 0022 lineage.
     Consumption path: ticket Required delivery partial-state bullet.
     Reproduction: `rg 'seed is attached once at the root-facing' docs/research/numerics/reduction-semantics-and-legality.md`

  8. [VERIFIED] Split precondition `K` positive multiple of split width is the spike's structural precondition and L3 typed-refusal rule for pad-vs-refuse.
     Evidence: L3 candidate table `K ≡ 0 (mod split)`; typed refusals section.
     Raw source anchor: `K ≡ 0 (mod split)`
     Construction path: host probe refusals; L3 record.
     Consumption path: ticket Required delivery typed refusal.
     Reproduction: L3 table column Structural preconditions for ksplit rows.

  9. [VERIFIED] Non-goal: distributivity is a different dimension; no permission Tiler grants for regrouping a contraction chain (ADR 0095 decline; non-goal correctly excludes it).
     Evidence: ticket Non-goals; decide-whether-to-admit-a-distributivity-permission done / ADR 0095.
     Raw source anchor: Distributivity, in either direction
     Construction path: ADR 0080/0095.
     Consumption path: ticket non-goal and refusal-naming discipline.
     Reproduction: ticket Non-goals section.

  10. [VERIFIED] Close condition unmet: alternatives do not exist under permission-gated admission with watched refusals and eight-case contiguous_split bit reproduction in the compiler path.
     Evidence: Fact 6; close condition text; status still todo.
     Raw source anchor: Both alternatives exist, each is admitted only under its own permission
     Construction path: n/a (not delivered).
     Consumption path: board readiness for dependents that assume this capability (e.g. quantized block-map reopen).
     Reproduction: no compile-path ksplit strategy name under crates/tiler-compiler.

  11. [FALSE / IMPRECISE as hard dependency] Hard dependency on `realize-the-tiled-contraction-schedule-and-its-metal-emission` (status deferred) is not required by this ticket's stated outcome and parks it behind an unrelated public-boundary deferral.
     Evidence:
       - Tiled ticket Non-goals explicitly: "The split alternatives…".
       - Tiled is deferred behind `admit-a-cooperative-tile-over-shared-operands` (awaiting-decision) for free-index blocked write maps / ownership proof kinds — a vocabulary this K-split outcome does not name.
       - `realize-the-contraction-through-the-appendable-direct-path` is done and already lands the whole-program contraction compile path (`ReductionTopology::Contraction`) this ticket must extend beside.
       - Parallel multi-pass / cooperative machinery for *reductions* is done; both family admissions currently return None for `StrictTensorContraction` — that is this ticket's IR work, not tiled free-index tiling.
       - Original L3 delivery order placed this after the strict/tiled realization ticket; when strict was split into direct (done) + tiled (deferred), the edge appears to have followed the deferred half rather than the landed foundation.
     Raw source anchor: `dependencies: [implement-parallel-reduction-strategies, realize-the-tiled-contraction-schedule-and-its-metal-emission]`
     Construction path: ticket frontmatter.
     Consumption path: ready/dispatch graph — deferred dep never satisfies dependents (workflow: deferred is parked, only done satisfies_dependents).
     Reproduction: `rg '^status:' tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md` → deferred; `rg '^status:' tickets/realize-the-contraction-through-the-appendable-direct-path.md` → done.

  12. [VERIFIED] Dependency `implement-parallel-reduction-strategies` is done and is a legitimate prerequisite for independent reassociation vs permutation checks and split-portfolio patterns.
     Evidence: ticket status done; closes-when criterion 4 on independent permissions; multi-pass + single-workgroup strategies landed.
     Raw source anchor: `status: done` on implement-parallel-reduction-strategies.md
     Construction path: split execution graph under that rollup.
     Consumption path: this ticket's permission-gated dual alternatives.
     Reproduction: `rg -m1 '^status:' tickets/implement-parallel-reduction-strategies.md`

  13. [VERIFIED] Related tickets: `reduction-semantics-contract` done; `implement-analytical-component-cost-model` done. Related vs depends-on usage is correct for those two (evidence / cost-explain, not hard blockers).
     Evidence: status lines on both tickets.
     Raw source anchor: `related: [reduction-semantics-contract, implement-analytical-component-cost-model]`
     Construction path: ticket frontmatter.
     Consumption path: graph hygiene.
     Reproduction: status greps on both related ticket files.

  14. [IMPRECISE] Related set omits the closest landed construction/consumption sites that a worker will actually read: direct-path contraction, multi-pass split enumeration, and the reassociation-only contract preset.
     Evidence: realize-the-contraction-through-the-appendable-direct-path done; enumerate-the-split-reduction-on-the-planning-frontier done; admit-a-reassociating-contract-without-contraction done (NumericalContract::ReassociateF32 / FLUSH_AND_REASSOCIATE_F32).
     Raw source anchor: `related: [reduction-semantics-contract, implement-analytical-component-cost-model]`
     Construction path: ticket frontmatter only.
     Consumption path: worker discovery.
     Reproduction: status greps on the three omitted tickets.

  15. [VERIFIED] Scopes `implementation/compiler`, `implementation/ir`, `implementation/metal` and shared `project/tickets` are valid ticketsplease scope names mapping to the crates a schedule+Metal delivery would edit.
     Evidence: ticketsplease.toml [scopes] entries.
     Raw source anchor: `"implementation/metal" = ["crates/tiler-metal/**"]`
     Construction path: ticketsplease.toml.
     Consumption path: guard/scope declaration.
     Reproduction: open ticketsplease.toml scopes section.

  16. [HISTORICAL BUT ACCURATE] Measurement-bound caveat that spike split kernels idle lanes at large M and do not bound the general split strategy.
     Evidence: L3 inference paragraph; ticket Required delivery bullet.
     Raw source anchor: This implementation is not the strategy
     Construction path: L3 measurement record.
     Consumption path: planning honesty for cost/selection later.
     Reproduction: ticket Required delivery last bullet.

Current repository behavior:
  - Whole-program F32 contraction compiles as the `direct` realization (`ReductionTopology::Contraction`, one invocation per output, ascending fold from first product) via the appendable direct path.
  - Parallel reduction multi-pass split and single-workgroup cooperative strategies exist for serial-sum-class programs and are permission-gated; they do **not** admit `StrictTensorContraction`.
  - Contiguous/strided K-split contraction kernels exist only under `spikes/scheduling/metal_contraction_vertical/` with retained attribution and timing fixtures; nothing under `crates/` enumerates, verifies, emits, or selects them.
  - Contracts permitting reassociation without contraction exist (`ReassociateF32` / `FLUSH_AND_REASSOCIATE_F32`); the LM profile's governed contracts still forbid reassociation for the workload cases L3 measured.
  - Tiled free-index contraction schedule remains deferred behind a public-boundary decision on cooperative tile-over-shared-operands.

Board and graph verdict:
  status:
    todo is coherent with "not delivered" but the ticket is currently non-ready solely because of a deferred hard dependency that the outcome does not need. After dependency repair (Fact 11), both remaining hard deps are done and the ticket should surface as ready (or stay todo until the board's ready transition), not stay parked behind tiled.
  dependencies:
    Keep: implement-parallel-reduction-strategies (done) — correct.
    Repair: replace realize-the-tiled-contraction-schedule-and-its-metal-emission with realize-the-contraction-through-the-appendable-direct-path (done), and move tiled to related (its non-goals already exclude this outcome; shared lessons about contraction topology dispatch are related, not blocking).
    Optional hard dep if single-workgroup barrier form is chosen over multi-pass: nothing extra beyond already-landed synchronization authority — do not re-introduce the cooperative free-index tile ticket.
  related work:
    Keep reduction-semantics-contract and implement-analytical-component-cost-model.
    Add related: realize-the-tiled-contraction-schedule-and-its-metal-emission, enumerate-the-split-reduction-on-the-planning-frontier, admit-a-reassociating-contract-without-contraction, realize-the-contraction-through-the-appendable-direct-path (if not promoted to depends-on).
  scopes:
    Correct for a schedule + IR + Metal emission delivery. No repair.
  trigger state:
    Not a deferred ticket; no trigger-check log required today. After dep repair it is ordinary ready work, not trigger-gated.
  closure state:
    Open and correctly unclosed — close condition not met. Not obsolete / already delivered.

Repair required:
  - exact metadata changes:
      dependencies: [implement-parallel-reduction-strategies, realize-the-contraction-through-the-appendable-direct-path]
      related: [reduction-semantics-contract, implement-analytical-component-cost-model, realize-the-tiled-contraction-schedule-and-its-metal-emission, enumerate-the-split-reduction-on-the-planning-frontier, admit-a-reassociating-contract-without-contraction]
      (If a coordinator insists on preserving a sequential relationship to free-index tiling, keep tiled as related only — never as depends-on while it is deferred for an orthogonal public boundary.)
  - exact prose correction: none required for measurement claims; optionally one dated note under Required delivery that the foundation is the landed direct contraction path plus parallel-reduction portfolio patterns, and that free-index tiled emission is a sibling alternative not a prerequisite.
  - exact dated correction: 2026-08-10 (audit) — hard dependency on `realize-the-tiled-contraction-schedule-and-its-metal-emission` re-pointed: tiled Non-goals exclude split alternatives; tiled parks on cooperative free-index tile public boundary; direct-path contraction is done; multi_pass_family/cooperative_family still return None for StrictTensorContraction (this ticket's work).
  - exact new or connected remainder: none — outcome remains one ticket. Do not split unless implementation discovers that multi-pass K-split and cooperative single-dispatch K-split are separately shippable with different identity domains; if so, file then, not preemptively.

Public/API/identity/architecture consequences:
  - Delivery will extend schedule topology admission for `StrictTensorContraction` (new or reused topology variants / family arms), frontier strategy offers, and Metal emission — identity-sensitive if new `ReductionTopology` tags or kernel-program fields are appended.
  - Must keep reassociation and permutation as independent permission checks (watch strided refuse under reassociation-only; contiguous refuse under neither).
  - Must not treat cost or fusion with per-block quantized maps as this ticket's outcome; consumers (first-quantized-lm-profile / fuse-quantized-weight-decode) only need the contiguous alternative to exist under a caller-granted reassociation permission.
  - No public-boundary self-acceptance: any new public contract preset or pub surface still needs Tom if it leaves crate-private `pub(crate)` seams.
  - Escalation only if design chooses a free-index cooperative tile for the split body (then it truly collides with the awaiting-decision tile relation) — current L3 spike and multi-pass reduction precedent do not require that.

Tests and checks:
  - Closing evidence already named: permission refusal watched for the wrong dimension; K-multiple typed refusal; contiguous plan bits match spike `contiguous_split` on the eight-case corpus (split_topology → 0xbb1d0683 under ftz attribution).
  - Perturb independently: forbid reassociation → both refuse; permit reassociation forbid permutation → contiguous admits, strided refuses naming permutation; permit both → both admit.
  - Do not compare reassociated results to the strict serial oracle under equality; use the order-specific oracle pattern from reduction split work (`strict_partitioned_sum` analogue for contraction products).
  - Seed-once / has_value: negative_zero_seed and empty-partition cases from the spike corpus.
  - Emission: no fused multiply-add on accumulation path (finding 16 discipline), same as direct/tiled.

Exact files expected to change (when implemented; not this audit):
  - tickets/admit-reassociated-contraction-schedule-alternatives.md (metadata repair only for this audit's graph fix)
  - crates/tiler-ir/src/schedule/builder.rs (family admission + verify arms)
  - crates/tiler-ir/src/schedule/model.rs (topology / partition encoding if extended)
  - crates/tiler-compiler/src/physical.rs and frontier.rs (strategy constructors + declines)
  - crates/tiler-metal/src/emit.rs (+ goldens if emission lands)
  - tests under tiler-compiler / tiler-ir / tiler-conformance / tiler-reference for permission and corpus bits

Residual uncertainty:
  - Whether the first production shape should be multi-dispatch multi-pass K-split (no threadgroup barrier; matches landed reduction split) or single-dispatch cooperative K-split (matches the spike's isolation kernels). Both satisfy the semantic permission distinction; only the second shares walls with cooperative staging. That is an implementation design choice inside this ticket, not a reason to keep the tiled free-index dependency.
  - Identity check commands (`git rev-parse`, `shasum -a 256`) were not re-executed in-process; base and content hash are accepted from the assignment after full ticket read at the assigned RO path.

Recommended audit_state:
  audited-repair-required
