---
id: scope-the-ieee-decimal-vertical
title: Scope the IEEE decimal vertical
status: deferred
priority: p3
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, decimal]
---
## User-visible outcome

`tiler::decimal32@1`, `tiler::decimal64@1`, and `tiler::decimal128@1` have an owner for the step past identity, and their two storage encodings are treated as the distinct things they are.

## Why this exists

**Fact.** [ADR 0035](../docs/decisions/0035-recognize-ieee-decimal-floating-formats.md) recognizes the three formats and keeps densely packed decimal and binary-integer-decimal as physical encodings of the same logical dtype. [The dtype support ledger](../docs/dtype-support.md) records them registered with interchange width and coefficient precision, an architectural seam at the physical carrier, and absent everywhere else.

**Fact — the obligation that separates this from every binary float is the storage one.** [The mature dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md) states that "DPD and BID are separate storage encodings, and a bit-preserving operation or ABI must distinguish them even though they encode the same logical decimal format". A binary float has one encoding per format; decimal has two, so the ABI and any bit-preserving operation owe a distinction no binary track owes.

**Fact.** decimal32 is exhaustively enumerable and decimal128 is not, so this track also consumes [`state-the-non-enumerable-float-conformance-profile`](state-the-non-enumerable-float-conformance-profile.md)'s answer rather than deriving its own.

## Activation trigger

A named frontend or accelerator consumer requires a decimal tensor element. The taxonomy is explicit that current GPU tensor arithmetic does not imply execution support, so backend availability does not fire this.

## Closes when

The trigger has fired and the vertical is stated including which storage encoding a program's bytes carry — or decimal is explicitly excluded from the intended product surface by a recorded decision.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-7 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
