---
id: pin-the-tiler-compiler-identity-domain-spellings-the-ir-census-does-not-reach
title: Pin the tiler-compiler identity domain spellings the ir census does not reach
status: in-progress
priority: p1
dependencies: []
related: [pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [identity, tests, versioning]
claimed_from: todo
assignee: w-sol-identity
lease_expires_at: 1786201849
---

`tiler-ir` now carries a source census pinning its identity domain spellings. The scan is `CARGO_MANIFEST_DIR`-rooted, so it reaches that crate only. **`tiler-compiler` is the largest uncovered population.**

## Facts

**Correct the framing before you start — an earlier version of this finding was overstated by the coordinator and repaired by measurement.** It is **not** true that no test asserts an identity domain. Several do: `crates/tiler-ir/src/kernel/tests.rs` binds `b"tiler.kernel.v7\0"` and asserts two identities open with it; `crates/tiler-ir/src/semantic/catalog/tests.rs` asserts `starts_with(b"tiler.value-type-descriptor.v1\0")`; and `STRICT_F32_REGION_IDENTITY_HEX` opens with the hex of `tiler.schedule.v5\0`. Both of those first two were coordinator-verified.

**The true finding is that coverage is incidental.** It exists wherever someone pinned a digest over a subject that happens to fold a domain, and it reports only that *a digest moved*. Measured on `tiler-ir` before the census landed: reverting `GRAPH_DOMAIN` v3→v2 failed **2** tests, `INDEX_REGION_DOMAIN` v11→v10 failed **4**, and `OBLIGATION_DOMAIN` v2→v1 failed **0** of 3,184. Assume `tiler-compiler` has the same uneven shape; **measure it, do not assume either extreme**.

**Reported, not coordinator-verified.** `tiler-compiler` carries **25** distinct `tiler.`-spelled literals, including versioned subjects that step: `tiler.compiler.request-subject.v6`, `tiler.target-profile.declaration.v11`, `tiler.target-profile.descriptor.v10`, `tiler.compiler.boundary-property-set.v3`.

## What closes this

A census for this crate, modelled on `crates/tiler-ir/src/domains.rs` — read it first, and read its module documentation for why it is shaped as it is.

**Do not reach for `variant_count` without checking whether it fits.** It is right for `tiler-artifact`, where every domain is a named constant a variant can mirror. It was **wrong** for `tiler-ir`, where 15 of 60 pinned spellings are inline literals no constant names — an enum could have named 45 of 60 while reporting a complete population, which is the exact failure the enumeration exists to prevent. Determine which case `tiler-compiler` is, by counting, and say so.

**Measure the baseline first.** Revert each versioned domain in turn and record what fails, as the sibling did. That tells you which domains already have incidental coverage and which have none, and it is the evidence that this work is needed rather than the assumption.

**Perturb each guard separately and quote the failure text.** The sibling ran nine perturbations, four of which reddened exactly one assertion — including a self-exclusion guard proving the scan cannot satisfy itself from its own pin table, and a shadowing guard catching an admitted prefix that would swallow a pinned domain. Both are failure modes a census invites; carry them across or argue why they do not apply.

**A byte pin costs two edits on a deliberate step**, not one — the constant plus its table row. That is the floor, because a pin compares a value against a second copy. Make both assertions name the file, the spelling, and the table so the second edit is located rather than hunted. A per-domain version *floor* would cost zero edits and was rejected: it cannot see a revert to any version at or above the floor.

**Report the crates still uncovered with their counts.** `tiler-artifact` has a `domains` module that checks completeness and no-prefix but **never a value**, so an artifact domain can still be reverted silently there — that is a separate ticket, not this one. Do not widen.
