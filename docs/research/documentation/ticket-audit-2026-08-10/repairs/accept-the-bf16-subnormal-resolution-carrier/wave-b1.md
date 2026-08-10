Ticket: accept-the-bf16-subnormal-resolution-carrier
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-bf16-subnormal-resolution-carrier/27209ffd0a1a_c99ac54950f2.md
Pre-edit content hash (from ledger): 27209ffd0a1af43870265933f987b1b28803a2dfcb3dc2e7d99af4c0156bd2f4
Post-edit content hash: 976f7fc165af9e710d85c8b3db922bf7a7e05e81ac53be2b836e1c301b9937d5

Changes applied:
  - related frontmatter: added wire-the-bf16-reference-to-the-realization-it-is-told, subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types, give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject
  - Opening Fact preserving()/no-flush: dated Correction — 2026-08-10 (arm A evaluate via conformance_for + Bf16SubnormalRealization::new; no preserving() shorthand)
  - Graph maintenance: past-tense rewrite of blocked carry claim + dated correction that carry is status done
  - Decision §1 no-caller claim: dated correction that from_realization has production callers (decision-time only)
  - Decision "Where the guard goes" discard/`_`/binary32-only registry: dated correction (from_realization reads nan bits, returns ConformanceSubject::Arithmetic; registry three cases)
  - Decision window "currently unreachable": dated correction (subject check + bridge landed; Unstated residual named in correctness-and-testing)
  - Stale line pins retired in retouched paragraphs (metal_declaration, session, region_arithmetic_type, numerics nan-bits, verify_pointwise_bf16, from_realization, registry) → symbol/phrase anchors
  - Fact audit — 2026-08-10 summary section; status done left unchanged

Optional items skipped (with reason):
  - none (optional related edges applied as cheap graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none required; report listed no docs/crates edits and no new remainder tickets (arm B deferred + Unstated residual already owned elsewhere)

Verification:
  - files read: audit report; full ticket; greps on crates/tiler-reference/src/bf16.rs (preserving, evaluate/conformance_for), conformance.rs (from_realization + nan bits), registry.rs (host binary32 / three cases), carry ticket frontmatter status done, docs/correctness-and-testing.md Unstated residual, crates/ from_realization callers
  - checks: shasum -a 256 on post-edit ticket; required present-tense false claims each have a 2026-08-10 dated correction; decision text preserved as historical narrative

Recommended next ledger state:
  integrated
