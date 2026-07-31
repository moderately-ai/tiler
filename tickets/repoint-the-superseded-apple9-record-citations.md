---
id: repoint-the-superseded-apple9-record-citations
title: Repoint the two out-of-scope citations of the superseded Apple9 record
status: todo
priority: p3
dependencies: []
related: [close-or-retype-the-operand-permutation-inference, construct-and-bind-the-first-authoritative-metal-compile-profile, define-first-metal-lm-workload]
scopes: [implementation/metal, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, metal, target-profile, provenance, docs]
---
## User-visible outcome

Every live citation of the named-profile Apple9/F32 measurement names the record that is currently authoritative, so a reader following one arrives at the row the compile profile is built from rather than at its predecessor.

## Why this is a separate ticket

**Fact.** `close-or-retype-the-operand-permutation-inference` widened the shared kernel table by one pair to isolate the operand-permutation row, which moved `probe.harness_sha256` and required a new run of all four retained records. The named-profile pair is now `results/2026-07-31-numerics-{covering,exhaustive}-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/`; the 2026-07-30 pair it replaces is retained beside it as the previous row.

**Fact.** That ticket held `research/target-profiles`, `research/apple-targets`, and `implementation/build`, and repointed every citation inside them — the [authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md), the [numerical-behaviour record](../docs/research/apple-targets/numerical-behaviour.md), the spike README, and `test_numerical_probe.py`. Two citations fall outside those scopes:

- `crates/tiler-metal/src/applicability.rs`, whose module documentation lists the 2026-07-30 covering record as the source of the measured host row (`implementation/metal`).
- `docs/research/program-planning/first-metal-lm-workload.md`, which names both 2026-07-30 records as "the exact retained records" for the workload's target row (`research/program-planning`).

**Inference — this is staleness, not breakage, which is why it is p3.** Both links resolve, the 2026-07-30 records are retained, and every value either citation depends on reproduced byte for byte in the newer records: the diff between the two generations is 84 added `permutation_chain*` rows plus four provenance rows, with nothing removed and no measured value changed. Neither citation asserts anything false today. What they do is send a reader to the previous row, and the environment and toolchain identities they quote are identical in both.

## Work

1. Repoint `crates/tiler-metal/src/applicability.rs`'s record reference at the 2026-07-31 covering record, keeping the surrounding claim — that the deployment minimum in the directory name is the offline request and not the host OS version — exactly as it stands.
2. Repoint both record links in `docs/research/program-planning/first-metal-lm-workload.md`, and state there, as the other records now do, that the 2026-07-30 pair is retained as the previous row rather than deleted.
3. Re-verify rather than transcribe: read the identifiers the two sites quote — offline `metalfe-32023.883`, source-JIT `metalfe-32023.921`, schema `tiler.apple-numerical-behaviour/v7`, profile identity `apple9-f32-unified-msl4-macos26` — out of the 2026-07-31 records before repeating them.
4. Run `cargo nextest run -p tiler-metal` and `cargo test -p tiler-metal --doc`; a documentation-only change must still compile its doc-tests.

## Explicitly not in scope

Deleting the 2026-07-30 records. They are retained evidence for their own producer revision, exactly as the 2026-07-25 and 2026-07-27 records are, and the corpus keeps superseded rows rather than rewriting history.

## Closes when

No file outside `spikes/apple-targets/results/` and the historical ticket outcomes cites the 2026-07-30 named-profile records as current, and the two sites above name the 2026-07-31 pair with their quoted identifiers re-verified against it.
