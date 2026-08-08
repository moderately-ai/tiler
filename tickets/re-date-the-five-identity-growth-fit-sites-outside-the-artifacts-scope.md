---
id: re-date-the-five-identity-growth-fit-sites-outside-the-artifacts-scope
title: Re-date the five identity-growth fit sites outside the artifacts scope
status: in-progress
priority: p2
dependencies: []
related: [re-date-the-six-identity-growth-fit-sites-one-displacement-behind]
scopes: [contracts/foundation, research/artifacts, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [identity, documentation, measurement]
claimed_from: todo
assignee: coord
lease_expires_at: 1786187249
---
Five identity-growth fit occurrences outside `contracts/artifacts` still state `3530n + 723` as the **live** fit. It was displaced by exactly `n + 1` on 2026-08-08 and the ladder now measures `3531n + 724`. [`re-date-the-six-identity-growth-fit-sites-one-displacement-behind`](re-date-the-six-identity-growth-fit-sites-one-displacement-behind.md) held only `contracts/artifacts` and repaired the sixth, in `docs/artifact-abi.md`; these five were deliberately left rather than edited across a scope boundary.

## Facts, verified 2026-08-08 at base `c81f9257` by reading each occurrence in place

**Fact.** `grep -rn "3530n + 723" docs/` returns **11 occurrences across 6 files** at that base. **Six are live claims and five are quoted history.** One live claim — `docs/artifact-abi.md` — is repaired by the sibling above. The five below remain.

**Fact — the five live occurrences outside `contracts/artifacts`, each a 2026-08-07 correction whose measurement block states the fit in the present tense with no later supersession beneath it.**

| File | Scope | Count | Anchor |
| --- | --- | --- | --- |
| [`docs/ir.md`](../docs/ir.md) | `contracts/foundation` | 1 | `**Measurement, re-run 2026-08-07 over the whole admitted domain — sixty-one points, 2..=62 operations:** kernel-program identity is exactly` |
| [`docs/research/artifacts/manifest-fixed-content-growth.md`](../docs/research/artifacts/manifest-fixed-content-growth.md) | `research/artifacts` | 3 | `is now the reverse of the truth`; `the ×2 crossing is`; `the sharpest inversion this correction makes` |
| [`docs/research/program-planning/complete-model-ingestion-and-execution.md`](../docs/research/program-planning/complete-model-ingestion-and-execution.md) | `research/program-planning` | 1 | `Superseded a third time, on 2026-08-07 by` |

**Fact — two files need nothing and their occurrences must stay.** [`docs/status.md`](../docs/status.md) (`contracts/navigation`, 1 occurrence) already carries a 2026-08-08 correction that names `3531n + 724` as current and states the displacement chain; its `3530n + 723` is quoted inside that chain. [`docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md`](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) (`contracts/decisions`, 4 occurrences) carries both a `**Superseded — 2026-08-08**` header note and an `**Extended 2026-08-08**` closing note; all four occurrences sit under them. **Deleting either would destroy history, which is the failure mode in the other direction.**

**Fact — the value comes from the retained run and never from arithmetic.** `spikes/program-planning/identity-growth/results/2026-08-08-post-sourced-semantic-shape-apple-m4-max-macos27.0-26A5388g/growth.tsv`, base `cc667626`, Apple M4 Max, macOS 27.0 build `26A5388g`, repository toolchain pin: `program_bytes(n) = 3531n + 724`, residual zero at all sixty-one points, `graph_bytes(n) = 135n + 149`, widest measured point 219,646 bytes at 62 operations. [`results/README.md`](../spikes/program-planning/identity-growth/results/README.md) carries the full displacement chain and the bound on each attribution.

**Fact — the derived figures these five documents carry move with the slope and the conclusions do not.** The spike records the fitted 64 MiB refusal point moving 19,038 → 19,011 → 19,006 operations, and the 1 MiB per-invocation embedding crossing staying between **148 and 149** at multiplicity two through both displacements. Each of the five documents restates its own solved figures — P1/P2/P3 byte counts, the ×2 shares, the whole-model extrapolation — so a re-date that touches only the coefficient leaves those inconsistent with it.

## What closes this

Each of the five live claims stops being a live claim, and the quoted occurrences in `docs/status.md` and ADR 0104 are left exactly as they are.

**Apply the sibling's decision rather than re-deciding it.** `docs/artifact-abi.md` now names the spike and its retained run as the standing authority and states only the displacement-invariant conclusions, dating any unavoidable coefficient to the tree it was measured on. That decision was taken because these figures are pinned by no test, `make citations` resolves links and never checks a number, and this is the third spelling of one curve in four days — so refreshing the digits rebuilds the defect one encoding step later. The same reasoning applies to all five sites here; a document that concludes otherwise should say why in its own correction.

**Watch the derived figures, not only the coefficient.** The previous sweep's failure was partly that a re-date is cheap and re-solving every dependent figure is not. Either re-solve them from the retained run or move them to the spike with the coefficient.

Cite by searchable anchor and **run the anchor's grep before committing to it**. `docs/status.md` spells a crossing as "between 50 and 51 operations", so an anchor written `50/51` from rendered reading fails as absence — the more dangerous reading, because it looks like the text was removed.
