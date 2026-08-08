---
id: correct-the-status-record-s-manifest-schema-version
title: Correct the status record's manifest schema version
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coord
lease_expires_at: 1786182999
---
## The version drifted by two major steps, and it did not drift alone

`docs/status.md` states "neutral manifest schema **14.0**" and that "the manifest took a major step to **`14.0`**".

**Fact — corrected 2026-08-08 at base `acc26984`.** The ticket asserted that `crates/tiler-artifact/src/program/codec/encode.rs` declares `MANIFEST_SCHEMA: (u16, u16) = (15, 0)`, marked coordinator-verified. That is **false at this base and names a value the constant has already left**: `crates/tiler-artifact/src/program/codec/encode.rs "pub(super) const MANIFEST_SCHEMA"` reads `(16, 0)`. The drift is two major steps, not one — the `15.0` step replaced the manifest's trailing identity preimage with its digest, and the `16.0` step gave every entry row its derived index-arithmetic requirement.

**Fact — corrected 2026-08-08.** The ticket asserted that "every other constant in that sentence checks out". That is **false**: the roll call carried seven ledger rows and **three** had gone stale, all in the same enumeration. Beside the manifest schema, `crates/tiler-artifact/src/program/model.rs "pub(crate) const ARTIFACT_DOMAIN"` reads `tiler.artifact-program.v16` against a stated artifact program v15, and `crates/tiler-ir/src/kernel/model.rs "const KERNEL_DOMAIN"` reads `tiler.kernel.v7` against a stated structured kernel v6. The remaining four rows — resolved value type v3, scheduled region v5, verified kernel program v11, artifact stage key v3 — are verified correct, as are all nine figures in the bullet's second enumeration.

**The three stale rows did not move by one common step**, so no arithmetic repair was available: the manifest crossed both intervening steps while the artifact and kernel domains crossed only the second. `docs/artifact-abi.md "carrying the derived index-arithmetic requirement moved the structured kernel"` records that joint step.

**Fact — verified.** The "moved again, alone" narrative and the `14.0` step account are dated history and survive unchanged; `git show 42f0051f:<path>` confirms all three retired figures were **true when written** at the commit that wrote them, so the treatment is dated beside with the retired wording quoted rather than substituted.

**Fact — verified.** `make citations` covers `docs/**`: `check-citations.sh "find docs -type f -name '*.md'"` appends that population.

## Closes when

`docs/status.md` no longer restates the ledger by hand. The roll call is replaced by a reference to its owner — `docs/artifact-abi.md "the current identity ledger is source-derived and each step has one owner"`, which is current on all three moved rows where the restatement was stale — the correction records what the retired figures read and when they were true, and `make citations` passes.
