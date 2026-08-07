---
id: date-adr-0079-s-one-crate-claims-for-the-second-diverging-member
title: Date ADR 0079's one-crate claims for the second diverging member
status: todo
priority: p3
dependencies: []
related: [record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [docs, doc-drift]
---
## User-visible outcome

[ADR 0079](../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md)'s Consequences state the extent of the unsafe exception as of *today* rather than as of a superseded 2026-07-25, so a reader auditing the workspace's unsafe posture finds two diverging members where the record names one.

## Why this exists

Found on 2026-08-07 by [`refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew`](refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew.md), which was repairing the same defect class in ADR 0106 and read ADR 0079 in full to check whether it carried it. It does, and it is out of that ticket's scope: ADR 0079 is a different record and its own decision is untouched.

**Fact — there are now two members that drop `[lints] workspace = true`, verified at `3e0074d5`.** `for f in crates/*/Cargo.toml prototypes/*/Cargo.toml; do grep -q '^\[lints\]' "$f" || echo "$f"; done` returns `crates/tiler-conformance/Cargo.toml` and `prototypes/serial-sum-run/Cargo.toml`. Both declare `[lints.rust] unsafe_code = "deny"`; every other member inherits the workspace `forbid`. Tom authorized the second on 2026-08-07 on [`decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`](decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access.md), which is exactly the acceptance ADR 0079 item 4 reserves to him, so **the decision is not violated — only the record's arithmetic is stale.**

The stale statements, each in unpinned present tense:

- **Consequences bullet 2** — "spent only in the one layer that must speak to an Objective-C API". Now two.
- **Consequences bullet 4 (the per-site gap)** — "Nothing counts, locates, or constrains `#[allow(unsafe_code)]` attributes inside **the one crate permitted to have them**". Two things changed. The population is two crates, and the gap is now asymmetric: `tiler-conformance` carries `bf16_vertical::tests::the_unsafe_site_population_is_the_two_named_ones`, which walks `src/`, counts the `unsafe` blocks and the reasoned allows, and fails on a third site or a new file carrying one — precisely the check this bullet says nothing performs, implemented in-crate rather than in the deleted Python gate. It does **not** cover `prototypes/serial-sum-run`, where the gap is unchanged.
- **Consequences bullet 5** — "A production runtime crate will face this boundary again. Nothing here pre-approves it: `tiler-prototype-run` is a non-published proof executable". A `crates/` member now has the divergence. It is still not a *reusable library* — `tiler-conformance`'s reverse-dependent set is empty and stays empty under ADR 0106 item 2 — so the bullet's actual reservation survives and its stated ground does not.

`docs/decisions/0079…md:31`'s complete-extent grep is pinned to `43f685f` and is correctly left alone.

## How to repair it

**Date rather than overwrite**, for the same reason [ADR 0106's refresh](refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew.md) did: every statement above was true when accepted, which is the ADR 0077/0088 shape. Append a dated `**Superseded — 2026-08-07**` note to each affected bullet naming what changed, with the verifying command and the commit it was run at. Do not substitute — that shape is reserved for a clause that was never true at any commit, as `correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence` used it.

Also worth stating in the same pass, because it is the most consequential half: the in-crate site-population test is real enforcement of item 3 that ADR 0079's Implementation boundary lists as review-only, and it is a *pattern* the prototype could adopt without re-implementing the deleted Python gate. Whether it should is [`pin-the-admitted-unsafe-sites-in-the-workspace-gate`](pin-the-admitted-unsafe-sites-in-the-workspace-gate.md)'s question, not this ticket's.

## Explicit non-goals

Do not change what ADR 0079 decides — items 1 through 4 stand, including that a second member dropping inheritance needs Tom, which is exactly what happened. Do not re-open the unsafe rule, the site count, or the workspace `forbid`. Do not edit `crates/` or `prototypes/`; this is a documentation repair.

## Closes when

Each stale Consequences bullet carries a dated note with its verifying command and commit, the asymmetry between the two diverging members is stated rather than averaged, and no repaired site carries a bare count without its command and commit.
