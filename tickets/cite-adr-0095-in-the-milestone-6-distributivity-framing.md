---
id: cite-adr-0095-in-the-milestone-6-distributivity-framing
title: Cite ADR 0095 at the reserved-framing sites the landing branch could not reach
status: in-progress
priority: p3
dependencies: []
related: [decide-whether-to-admit-a-distributivity-permission, realize-the-strict-contraction-on-metal]
scopes: [contracts/navigation, research/shapes, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, contracts]
claimed_from: todo
assignee: agent-adr0095
lease_expires_at: 1785940859
---
## User-visible outcome

A reader arriving at any surviving "whether to admit a distributivity permission" framing — in the Milestone 6 sections of `docs/roadmap.md` or in either of the two research records that route to it — learns that the choice was **decided and declined**, rather than that it is a product choice still reserved for Tom.

**Four sites the landing branch *did* reach, listed so this ticket is not re-done over them:** three passages in `docs/numerical-semantics.md`, Q-SEM-015 in `docs/open-questions.md`, ADR 0080's traceability (item 4's reservation discharged), and ADR 0087's traceability (which said the distributivity choice "remains reserved"). Those are done.

## Why this is a separate ticket

**Fact — the file was under concurrent edit and the collision was file-level, not scope-level.** [`decide-whether-to-admit-a-distributivity-permission`](decide-whether-to-admit-a-distributivity-permission.md) landed [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) on 2026-08-01 and swept every reservation site it could reach: three passages in [Numerical semantics](../docs/numerical-semantics.md) and Q-SEM-015 in [open questions](../docs/open-questions.md). `docs/roadmap.md` maps to `contracts/navigation`, which that branch declared and held, but [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) was `in-progress` with uncommitted changes to that exact file — the two branches were disjoint at file level only because roadmap was left alone. Editing it would have been the collision the file-level partition exists to prevent, so the remainder is filed rather than absorbed or silently skipped.

**That worker has since landed, so this is dispatchable now and the file is free.** Checked at the end of the ADR 0095 branch's work: `git worktree list` no longer carries that ticket's worktree, its branch is gone, and `docs/roadmap.md` is among the files `main` changed between `1bf1c2d` and `457739c`. **Which is exactly why the correction still belongs here rather than being folded back in:** the ADR branch is based at `1bf1c2d`, so editing roadmap from it would now conflict against content that branch cannot see. Take this ticket from current `main` and read the two passages there rather than trusting the line numbers below.

## What to do

Two passages, both in the Milestone 6 framing, both currently describing the choice as open. Read them in full before editing — the line numbers below were read at `1bf1c2d` and the concurrent branch may have moved them.

1. **`docs/roadmap.md:282`, the contraction-order paragraph.** Its closing sentence reads "Whether to admit the dimension at all is a product choice reserved under [Q-SEM-015] and owned by [`decide-whether-to-admit-a-distributivity-permission`]." That is now false: the choice was made. Replace the reservation with the decline, cite ADR 0095, and keep the paragraph's surviving conclusion intact — contraction-order exploration is *illegal* as a settled legality position, which the decline strengthens rather than changes, and the rejection must still name the missing distributivity dimension.
2. **`docs/roadmap.md:304`.** It reads "A third choice belongs in that same record but is framed elsewhere: whether to admit a distributivity permission at all … [Q-SEM-015] indexes all three." The sentence above it says "Two questions in this framing are genuine product and architecture choices … and this section deliberately does not settle them." With ADR 0087 settling the keyed-family question and ADR 0095 settling this one, only the multi-operand question survives as reserved; Q-SEM-015 already says so and the roadmap should agree. Check the numbered list beneath it (item 2 is the multi-operand question) for the same drift.

Sweep the rest of the file for the same claim while in it: `grep -n -i distribut docs/roadmap.md` returned **five** hits at `1bf1c2d` — `:282`, `:304`, `:307`, `:372`, and `:421`. Three need no change on this account and were checked rather than assumed: `:307` states the derivation holds under either answer to the multi-operand question, which the decline does not disturb; `:372` is the "Distributed execution" deferral row and matches the substring only, not the concept; and `:421` says the distributivity permission "remains separately gated and neither is admitted by this rung", which a decline makes more true rather than less. Re-check all three rather than trusting this note — the point of counting them here is that a sweep which finds only two hits has not run.

## Two research records route a reader to the same reserved framing, and they are this ticket's too

Both are outside `contracts/navigation` and were left for the same reason the roadmap was — the landing branch did not hold their scopes — so this ticket carries `research/shapes` and `research/program-planning` for them. Neither statement is *false*, which is why they are a second-order fix rather than a correction: what each says about legality survives the decline unchanged, and only the pointer is stale.

- **`docs/research/shapes/transformer-operation-and-shape-surface.md`, the open-questions list.** "**Whether to admit a distributivity permission.** Owned by [`decide-whether-to-admit-a-distributivity-permission`]. Contraction-order exploration over the SwiGLU or attention chains is illegal today as a settled legality position, not as an unexplored one." The second sentence is exactly right and gets *more* right under a decline. The first should name [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) and say the question is answered, so a reader does not open a `done` ticket looking for an open choice. This record is one of ADR 0095's three cited evidence records, which is the other reason it should point forward.
- **`docs/research/program-planning/first-attention-program-vertical.md`, the open-questions list.** "**Whether a distributivity permission should exist.** … This record supplies the first workload evidence that something valuable depends on it, and takes no position on the answer." Still true and worth keeping; add that the answer arrived on 2026-08-01 and is a decline. **Worth reading closely while there:** ADR 0095's grounds are read off this record's own operation table, and the sharpest case is that operations 19 through 21 are a regroupable pair separated only by a structural `Reindex` — not by a nonlinearity. If this record's open-question entry is updated, it should point at that, because it is the workload evidence the decision was actually weighed against.

## Non-goals

Any edit to `docs/numerical-semantics.md`, `docs/open-questions.md`, or either catalog — all swept by the landing ticket. Any change to the decision itself. Reopening the dependent question [`decide-whether-distributivity-directions-share-one-permission`](decide-whether-distributivity-directions-share-one-permission.md), which stays parked.

## Closes when

Both roadmap passages and both research-record entries state the decline and cite ADR 0095, the three remaining roadmap `distribut` hits are re-checked and confirmed or corrected, `grep -rn "decide-whether-to-admit-a-distributivity-permission" docs/` returns no site that presents the question as open, and `tkt lint` passes.
