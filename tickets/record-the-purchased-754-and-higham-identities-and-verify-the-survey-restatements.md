---
id: record-the-purchased-754-and-higham-identities-and-verify-the-survey-restatements
title: Record the purchased IEEE 754 and Higham identities and verify the survey restatements
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [sources, numerics, acquisition]
claimed_from: todo
assignee: agent-754-higham
lease_expires_at: 1786042934
---

## What Tom pulled (2026-08-06)

- `/Users/tsanterre/Downloads/IEEE Standard 754-2019.pdf` — 3,805,037 bytes, sha256 `2fe5f245fa6fd027a64067e2d91d9000f51e9c61ad23fe1914d8cae41f2b0fb4`; pdfinfo title "IEEE Std 754™-2019 (Revision of IEEE Std 754-2008) IEEE Standard for Floating-Point Arithmetic", author "Microprocessor Standards Committee of the IEEE Computer Society".
- `/Users/tsanterre/Downloads/Higham_2002_Accuracy and Stability of Numerical Algorithms (1).pdf` — 9,461,221 bytes, sha256 `7f7b3e32f946563830e2999e614fd3cba75d3694817da5fb36bbbdb80c7a4a75`.

Neither may be vendored (IEEE and SIAM for-sale copyright; the manifest rows already state this). The realistic best outcome both rows named is metadata-only with a digest over a legitimately acquired copy, which these are.

## The work

1. `ieee-754-2019` manifest row: record the digest and the acquisition note (who pulled, date, the file's pdfinfo identity) per the metadata-only row format; the bytes stay outside the repository.
2. `higham-asna-2002`: `pending-acquisition` moves to `metadata-only` with the digest and note; update the verifier populations (pending 1 to 0) and watch the verifier fail on a perturbation before trusting the pass.
3. **The reading that is the acquisition's stated purpose:** read §3.4 (the `gamma_h` notation and composition rules) and §4.2 (summation error analysis: tree-height result, recursive/pairwise/blocked cases) in Tom's copy, and verify the certified-bounds record's three foundations against the proofs rather than the survey's restatements — specifically whether `acta-numerica-fp-2023`'s restatement dropped any side condition the record's worked online-softmax bound depends on. Record held/moved per claim in the array-API re-check shape, at the record's own citation sites. The row's own warning governs: this must NOT be closed by summarizing from the table of contents or a secondary description — the sections are read or the claim is not made.
4. If §4.2's treatment of the blocked and compensated cases adds anything the record's boundary statements should carry, record it as a dated note; if nothing moves, say the re-check happened and held.

## Closes when

Both rows carry their digests and notes, the verifier passes on the stepped population after being watched failing, and the certified-bounds record states the proof-level re-check's verdict per foundation.
