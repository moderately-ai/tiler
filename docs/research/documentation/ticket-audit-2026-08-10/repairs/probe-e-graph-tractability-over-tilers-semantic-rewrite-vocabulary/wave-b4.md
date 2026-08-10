Ticket: probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary/ecb8a3bd7485_c99ac54950f2.md
Pre-edit content hash (from ledger): ecb8a3bd7485d85af12b1c7fae6ac31b06cf76c39d36a3f3e8dbbff3c44a0a80
Post-edit content hash: 446b804829d21d7e41d603c0929f1089435ee99f6e46e19120c27f43032a4e41

Changes applied:
  - Rewrote "Why this is deferred" Fact: no unqualified "input rewrite set does not exist"; no present-tense Part 7 causal clause that flash capability derivation has not run; aligned with refined log (axis 5 Proposal exists; four production tiler identities / two algebraic; inventing egg encoding still hits stop (a); Part 7 scoping and (b)/(c) remain authoritative).
  - Added 2026-08-10 Trigger check log line: **not fired** — reconfirmed four production identities, five Proposal rows, no egg, no e-graph spike; noted Part 7 stop-(a) prose drift without reopening the probe.
  - Metadata unchanged (status deferred, deps [], related, scopes, tags).

Optional items skipped (with reason):
  - none (report listed no optional ticket edits; optional Part 7 docs repair is residual product/doc debt).

Residuals not applied (docs/crates/new tickets/authority):
  - docs/research/region-search/rewrite-search-formalism.md Part 7 stop-(a) causal wording still stale ("because the flash capability derivation has not run") — out of wave B ticket-only scope; ticket no longer quotes that clause as current Fact.

Verification:
  - files read:
    - tickets/probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary.md (full)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary/ecb8a3bd7485_c99ac54950f2.md (full)
    - docs/research/region-search/rewrite-search-formalism.md (Part 7 anchors via grep)
  - checks:
    - `grep -rn 'RewriteRuleIdentity::new("tiler' crates/ --include='*.rs'` → 4 lines
    - `grep -c '^| R[0-9]' docs/research/program-planning/flash-class-capability-set.md` → 5
    - Cargo.lock: no `name = "egg"`
    - spikes/region-search/: exhaustive_oracle.py, phase_ordering_witness.py, README.md only
    - shasum -a 256 of ticket after edit → 446b804829d21d7e41d603c0929f1089435ee99f6e46e19120c27f43032a4e41

Recommended next ledger state:
  integrated
