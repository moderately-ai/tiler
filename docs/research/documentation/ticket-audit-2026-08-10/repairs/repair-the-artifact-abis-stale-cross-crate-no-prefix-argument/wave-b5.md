Ticket: repair-the-artifact-abis-stale-cross-crate-no-prefix-argument
Wave: B5
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/repair-the-artifact-abis-stale-cross-crate-no-prefix-argument/0109a0998ec4_c99ac54950f2.md
Pre-edit content hash (from ledger): 0109a0998ec4f9fe5ebddcd170ef4f630306a3d601fda2b31f538b71b4333884
Post-edit content hash: 53b98a55a1216d934bad5e5cae2153244a0d8cc75a6dd11d65ff1b83fede4c69

Changes applied:
  - Expanded Facts to name the "discharged by construction" label imprecision and the follow-on `It is still disjoint from `tiler.ir.`` scaffolding as part of the same live defect surface.
  - Rewrote Work as seven explicit contract-edit obligations aligned with the audit (retire both namespace premises; correct dependency-direction obstacle; name private IR enumeration + digest-knows-no-domains; terminator argument; preserve local obligation; rewrite follow-on; dated correction with quoted retired fragments; no counts; architecture/digest notes out of scope).
  - Added `## Fact audit — 2026-08-10` / `**Correction — 2026-08-10.**` re-verifying live contract premises, EXPR_DOMAIN counterexample, domains.rs source authority, test-only PINNED_IDENTITY_DOMAINS, Cargo edge, predecessor mapping, and unmet close condition.
  - Metadata left unchanged (status todo; dependency; empty related; scopes contracts/artifacts + shared project/tickets) per report.

Optional items skipped (with reason):
  - optional related edges to cover-the-fifth-envelope / reconcile hashing-site tickets: report says empty related is acceptable; chain already named from those Outcomes.

Residuals not applied (docs/crates/new tickets/authority):
  - Product residual (Class C ticket-only wave): edit `docs/artifact-abi.md` under "The governed digest" — replace the live cross-crate Fact (namespace premises, reversed dependency explanation, construction wording) with the source-true terminator / private-enumeration argument from `crates/tiler-artifact/src/domains.rs`, add dated correction quoting retired wording, rewrite follow-on disjoint scaffolding; then close when citations/lint/guard pass. Not edited this wave.
  - No new remainder ticket required (report: none if the paragraph including follow-on is repaired whole).
  - No crates/ edits.

Verification:
  - files read:
    - tickets/repair-the-artifact-abis-stale-cross-crate-no-prefix-argument.md (full pre/post)
    - audit report 0109a0998ec4_c99ac54950f2.md (full)
    - docs/artifact-abi.md governed-digest cross-crate Fact and follow-on (anchors for false premises)
    - crates/tiler-artifact/src/domains.rs no_governed_domain_of_this_crate_prefixes_another doc comment
    - crates/tiler-ir/src/program/abi.rs EXPR_DOMAIN
    - crates/tiler-ir/src/lib.rs #[cfg(test)] mod domains
    - crates/tiler-ir/src/domains.rs PINNED_IDENTITY_DOMAINS
    - crates/tiler-artifact/Cargo.toml tiler-ir.workspace edge
    - predecessor ticket correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places (frontmatter done + Outcome remainder mapping)
  - checks:
    - rg live anchors in docs/artifact-abi.md still hit false premises
    - rg EXPR_DOMAIN / retired claim / terminator anchors in source still match
    - shasum -a 256 tickets/repair-the-artifact-abis-stale-cross-crate-no-prefix-argument.md → 53b98a55a1216d934bad5e5cae2153244a0d8cc75a6dd11d65ff1b83fede4c69

Recommended next ledger state:
  integrated
