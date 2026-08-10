---
id: close-the-fmt-blind-spot-over-the-tiler-ir-trybuild-fixtures
title: Close the fmt blind spot over the tiler-ir trybuild fixtures
status: in-progress
priority: p3
dependencies: []
related: [close-the-fmt-blind-spot-over-the-trybuild-facade-fixtures]
scopes: [implementation/workspace, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: terra-ir-trybuild-fmt
lease_expires_at: 1786416406
---
## Why this exists

**Fact, measured on 2026-08-05 by `close-the-fmt-blind-spot-over-the-trybuild-facade-fixtures` at `d5960e81`.** The facade ticket closed the blind spot over `crates/tiler/tests/facade/**` only. `tiler-ir` carries three more `trybuild` fixture populations with the identical blind spot — they are not cargo targets, so `cargo fmt --all --check` never reaches them:

- `crates/tiler-ir/tests/index-region/{pass,fail}/*.rs` — 5 files, all currently clean
- `crates/tiler-ir/tests/shape-evidence/{pass,fail}/*.rs` — 9 files, 2 dirty (`pass/checked_refinement.rs`, `pass/inspect_additive_relation_variant.rs`)
- `crates/tiler-ir/tests/typed-handles/{pass,fail}/*.rs` — 7 files, 4 dirty (`fail/forge_semantic_identity.rs`, `fail/mixed_types.rs`, `fail/private_authority_limits.rs`, `pass/checked_authoring.rs`)

Reproduce: `for f in crates/tiler-ir/tests/*/*/*.rs; do rustfmt --check "$f" >/dev/null 2>&1 || echo "DIRTY $f"; done`

## What this ticket owes

The same treatment the facade ticket applied, derived independently rather than copied:

- Extend the `fmt` target in the `Makefile` to reach each `pass/` population, asserting each population count so an unmatched glob cannot read as clean.
- The `fail/` populations are excluded by default for the same reason as the facade `fail/` set — each is paired with a `.stderr` golden `trybuild` compares byte for byte, and a reflow moves a line number or a caret column. **Verify this per population by reading the `.stderr` files** rather than assuming it; three `typed-handles/fail/` fixtures being dirty is only a blind spot if their goldens really are span-sensitive.
- Fix the dirty `pass/` fixtures so the new check starts green, and confirm `cargo nextest run -p tiler-ir` still passes — these fixtures are compiled by `trybuild` at test run time, so reformatting changes what is compiled.
- Check whether any span inside a `pass/` fixture is verbatim generated text pinned by another test (the facade set had four such spans); if so, use `#[rustfmt::skip]` at the narrowest node rather than excluding the file.
- Watch the new check fail once against a deliberately misformatted fixture.

## Closes when

Each `tiler-ir` fixture population the goldens permit is reached by the gate, every exclusion is recorded with a reason verified from the `.stderr` files, and the check was watched failing.
