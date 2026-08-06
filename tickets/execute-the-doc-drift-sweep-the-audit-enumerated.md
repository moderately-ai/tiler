---
id: execute-the-doc-drift-sweep-the-audit-enumerated
title: Execute the doc drift sweep the audit enumerated
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference, implementation/build, implementation/metal-aot, implementation/frontend]
shared_scopes: []
paths: []
tags: []
claimed_from: todo
assignee: agent-doc-sweep
lease_expires_at: 1786029055
---
## The work (drift audit 2026-08-06; A4/A1/B spot-verified by the coordinator)

The ~35 mechanically-verifiable doc corrections the cross-cutting sweep enumerated, as one commit: section A's load-bearing claims (A1 two-account admission, A3 multi-output justification, A6 allowed-authority closed-set vs builtin bypass, A7 access-list shape, A14 eight-field count, A15 four destructure sites, the A12/A13 # Errors lists); B's four phantom-boundary sentences (private mods with zero pub items claiming reviewed-draft public boundaries); C's ten outgrown enumerations; D's link levels, byte counts, the orphaned CONTRACT_KEY_DOMAIN block + policy.rs:106 reference, the tensor_role_tag rename, the test-only import. Plus the vocabulary audit's two cooperative.rs corrections (the unreachable phase-0 example; the StagedProducer rationale narrowed for rounds>1 with the third broader-space bullet), the AxisDecode non-closure-under-composition paragraph (the einops '(a b)->(b a)' counterexample — verify the map before writing it), the caps audit's F7/F8 one-liners (cover.rs:1698/:1684 pointing at their documented twins), and the FIRST_INPUT five-vs-ten count line on the fold ticket. A4/A5/A8-A11 are EXCLUDED — they belong to the three filed tickets (reconcile, restate-sequence, renderer-fork).

Every correction is VERIFIED against source before written — the audit is a claim; one prior audit correction in this corpus was itself false. Any item that fails verification is left unchanged and recorded.

## Closes when

The sweep lands as one commit with a per-item verify-then-correct table in the Outcome, full gate green, and no excluded item touched.
