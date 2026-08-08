---
id: date-the-artifact-abis-metal-golden-enumeration-to-its-step
title: Date the artifact ABI's Metal golden enumeration to the step it records
status: done
priority: p3
dependencies: []
related: [cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, identity, goldens]
---
## A historical Fact reads as a standing one, and the obvious repair would be false

Verified 2026-08-07 at base `7c371155` by `verify-and-file-the-remaining-maturity-audit-leads`.

**Fact — the passage.** The `tiler.schedule.v5` step's chronology carries `docs/artifact-abi.md "including the five that carry no cooperative tile"`, naming `pointwise_scale_bias`, `reduction_single_axis`, `reduction_multi_axis`, `reduction_fused_multiply_add`, and `contraction_strict_tensor`, and then `docs/artifact-abi.md "their entry symbol, kernel identity digest, and scheduled-region identity digest move for the eighteen separator bytes alone"`.

**Fact — the directory holds ten today, and exactly one stages.** `ls crates/tiler-metal/goldens` returns ten `.metal` files, and `grep -c threadgroup_barrier crates/tiler-metal/goldens/*.metal` returns `1` for `cooperative_workgroup_reduction` and `0` for the other nine; `cooperative_workgroup_reduction.metal` is also the only file containing the token `threadgroup` at all. So five of the nine non-staging goldens are named.

## The audit lead this came from drew the wrong conclusion, and writing that conclusion down would be a fresh false claim

The lead proposed that "the claim survives and is stronger — 1 of 10 — and the enumeration no longer names its population". **Do not repair it that way.**

**Measurement — the enumeration was exactly complete when written.** The paragraph landed at `a395852a` on 2026-08-02, the ADR 0097 two-dimensional-staging step. Read the directory on that tree rather than inferring it:

```sh
git log -S'including the five that carry no cooperative tile' --oneline -- docs/artifact-abi.md
# a395852a Let a cooperative tile state a staged access over a participant space
git ls-tree --name-only a395852a crates/tiler-metal/goldens/ | wc -l
# 6
```

The six are exactly `cooperative_workgroup_reduction` plus the five the paragraph names. The other four were added after the step — `pointwise_scale_bias_bf16` at `7a24ed20` on 2026-08-05, and `elementary_silu_activation`, `structural_mirrored_reindex`, and `structural_widening_broadcast` on 2026-08-06.

**So "every Metal golden's identity moved" is a claim about what moved at that step, and it is true.** Restating it as nine of ten would assert that four goldens' identities moved at a step that predates their existence. That is the failure mode this repository has hit repeatedly this week: a repair block that fixes a stale sentence by introducing a false one.

## What actually survives

A narrow reader hazard, not a false claim. The paragraph is written in a tense that reads as a standing statement over the current population, and it carries no marker that the population was six. A reader today counts ten goldens, finds five named, and cannot tell from the passage whether the enumeration decayed or was scoped to its step.

**The document already has the idiom for this**, used in the same chronology: a neighbouring paragraph writes "`tiler.schedule.v4` when that step landed, and `v5` since" and marks a superseded premise "**That premise has since expired**". The repair is to bring this paragraph to that form, not to recount it.

**Checked and clean, so it is not swept in.** The paragraph two below it, which reads "a `v6` kernel identity built over a `v4` region can never be confused with one built over a `v3` region", was last written at `e4d2aa7d` — the `v4` step — so its version pair is correct for the step it belongs to and must not be renumbered.

## Requirements

- Mark the enumeration as the population at the `tiler.schedule.v5` step, in the chronology's existing dated idiom. Naming the six-golden population explicitly is the cheapest way to make it self-evident.
- **Do not renumber the claim to nine or to ten**, and do not add the four later goldens to the list. State the reason inline so the next audit does not re-raise it.
- Prefer a searchable anchor to a line number; `make citations` covers `docs/**`.

## Scheduling note — one file, two live claims

`cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check` also holds `contracts/artifacts` and also edits `docs/artifact-abi.md`, to reconcile that document's digest-domain counts. The two subjects do not overlap — digest domain separators against the Metal golden corpus — but the scope and the file do, so these must not run concurrently. Sequence them rather than merging them.

## Closes when

The enumeration states the population it is about and the step it belongs to; no golden that postdates the step is claimed to have moved at it; the reason a recount would be wrong is recorded inline; and `make citations` is green.
