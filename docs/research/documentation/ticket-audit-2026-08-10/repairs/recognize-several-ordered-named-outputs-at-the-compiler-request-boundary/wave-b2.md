Ticket: recognize-several-ordered-named-outputs-at-the-compiler-request-boundary
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/recognize-several-ordered-named-outputs-at-the-compiler-request-boundary/d045f1a41e97_c99ac54950f2.md
Pre-edit content hash (from ledger): d045f1a41e974f84302f91f716a17f301f4e50a986a18ebecc50181dfccb5191
Post-edit content hash: d73c06e0aad11ca247980cbf24b5afa0cf0333e0a29ba8b3709e2116223bbefb

Changes applied:
  - Why this exists: prefixed **Correction — 2026-08-10.** that Facts/Inference are pre-landing only; struck first Fact, second Fact, and Inference to historical past tense so they cannot be read as live single-output / only-remaining-wall claims.
  - Identity Fact: labeled as this ticket's step only; added **Correction — 2026-08-10.** that domain step was v4→v5, live domain is `tiler.compiler.request-subject.v6`, live explain pin is `request=7ba3d77a66f04638`, and hex pair `c91fc7c9…`→`45467875…` is intermediate.
  - Measurement: optional rename note — `two_outputs_sharing_one_walk_refuse_rather_than_publish_twice` → `an_output_key_pair_naming_one_value_still_refuses_by_name`.
  - Closing condition 3: kept reassignment to parent; replaced live claim that the named test asserts `output-arity` refusal with historical-at-close vs after-parent (test now asserts admission); noted parent is done.
  - Closing condition 4: pointed at identity Correction for live v6/pin supersession.
  - Added terminal **Fact audit — 2026-08-10** summarizing the four repairs and confirming board metadata stands.

Optional items skipped (with reason):
  - none (optional test-rename note applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none — report required ticket prose only; no docs/crates edits, no new remainder tickets, no metadata changes.

Verification:
  - files read:
    - audit report d045f1a41e97_c99ac54950f2.md (full)
    - tickets/recognize-several-ordered-named-outputs-at-the-compiler-request-boundary.md (full, pre- and post-edit)
    - crates/tiler-compiler/src/request.rs (select_supported_strategy / recognize_program_outputs; tests recognizing_several_ordered_named_outputs_names_one_partition_each, an_output_key_pair_naming_one_value_still_refuses_by_name; request-subject.v6 encode site)
    - crates/tiler-compiler/src/domains.rs (PINNED_IDENTITY_DOMAINS request-subject.v6)
    - crates/tiler-compiler/src/explain.rs (request=7ba3d77a66f04638 pin)
  - checks:
    - re-verified live recognition path, test admission semantics, renamed test name, domain v6, and explain pin against current tree before writing corrections.
    - post-edit sha256: d73c06e0aad11ca247980cbf24b5afa0cf0333e0a29ba8b3709e2116223bbefb

Recommended next ledger state:
  integrated
