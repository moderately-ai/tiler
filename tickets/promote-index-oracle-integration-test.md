---
id: promote-index-oracle-integration-test
title: Promote the index-region oracle test to an admitted target
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/workspace, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, workspace]
---
The IndexRegion oracle's end-to-end cases live in `crates/tiler-reference/src/oracle.rs` rather than a dedicated integration target. That was a scope-respecting choice, not a shortcut: `scripts/check_workspace.py` holds a closed `EXPECTED_TESTS` allowlist that admits only `serial_sum_slice` for `tiler-reference`, so adding `crates/tiler-reference/tests/index_region_oracle.rs` fails the workspace contract, and amending the allowlist needs `implementation/workspace`, which the oracle ticket did not hold.

Move the oracle's integration-style cases into an admitted `tests/` target and register it in `EXPECTED_TESTS`. Keep genuinely unit-level cases (bounded arithmetic, registry identity) in their modules. Verify the moved target exercises the crate through its public API only, since that is the boundary the oracle is meant to prove. Run the complete repository gate before completion.
