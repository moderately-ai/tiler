---
id: search-a-tie-demonstrating-prompt-for-the-model-level-corpus
title: Search a tie-demonstrating prompt for the model-level corpus
status: deferred
priority: p3
dependencies: [reclassify-language-model-work-as-a-conformance-track]
related: [define-the-model-level-conformance-corpus, build-the-model-level-measurement-harness, prove-the-c1-complete-model-execution, define-first-metal-lm-workload]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, conformance, testing, language-model, qwen, deferred, class-conformance-fixture]
---
## Why this is deferred and not todo

The oracle's tie policy is declared, must be implemented, and is demonstrated by no row. The model-level corpus records that as `Unknown` — declared-and-untested — rather than as absent, and this ticket is the work that would close it. It is filed `deferred` because nothing today can consume the row: no Tiler execution of this workload exists, and a demonstrating prompt with nothing to run it against would be a fixture waiting for a consumer rather than evidence.

## Activation triggers

Either fires it, and both are checkable in one line.

1. **A tie-resolving implementation exists.** [`drive-the-complete-forward-pass-over-three-artifacts`](drive-the-complete-forward-pass-over-three-artifacts.md) lands, so a greedy selection with the declared tie policy — the lowest vocabulary index attaining the maximum, with a bit-identical top-two pair recorded rather than resolved — is code that a demonstrating row could fail.
2. **A workload with a narrower margin is named.** C1's smallest runner-up gap is 0.266; a row whose gap approaches the comparison band changes the tie question from a structural curiosity into a live risk, and the search should be re-run against it rather than inherited from this checkpoint.

## What the deferred search already knows

From [the corpus reachability probe](../spikes/program-planning/qwen3-corpus-reachability/README.md), whose retained record is the starting point rather than something to redo.

- **The structural route exists.** The checkpoint ties its embedding and vocabulary projection, so two bit-identical embedding rows are two bit-identical logit columns at every position of every prompt. There are **28 duplicate groups covering 2,226 of 151,936 rows**, the largest with 505 members. A prompt whose greedy token is any of those 2,226 produces a tie by construction.
- **The searched drivers did not reach them.** 19 prompts and 330 positions, prompts varied only by repetition of a duplicate-group member at lengths 8, 16, and 32 across group sizes 505 down to 2: 0 positions with a bit-identical top-two pair, and the best-placed duplicate-group member ranked 86,718th at its best and sat 17.45 logits below the maximum.
- **So the search that remains is over prompt content, not over repetition length.** The duplicated rows are untrained vocabulary slots; what is unknown is whether any prompt drives one to the top. A gradient-guided or beam search over prompt tokens against the objective "maximize the best duplicate-group member's logit minus the maximum" is the obvious next instrument, and the probe already emits that quantity per position.

## Required work when it activates

- Extend the retained probe rather than writing a second one, so the search runs against the same verified checkpoint bytes and the same pinned reference under the same stop-on-mismatch discipline.
- Report the search as a bounded experiment: the prompt space enumerated or the optimizer and its budget, the best gap and rank reached, and whether a tie was produced.
- If a tie is produced, the corpus row `A-tie` gains exact inputs — the prompt token IDs, `T`, `C`, `S`, capacity, the position, and both attaining indices — and its required outcome is that the greedy token is the lower index and the position is recorded as a tie rather than resolved silently.
- If none is produced, the negative replaces the current one with its own boundary and this ticket is re-deferred rather than closed, because a wider negative is still not a proof.

## Explicit non-goals

No change to the tie policy, which is L1's and is declared. No second checkpoint. No relaxation of the corpus row to something a reachable prompt happens to satisfy — a row that tested a near-tie instead of a tie would be testing the comparison band, which is already tested.

## Closes when

Either a demonstrating prompt exists and `A-tie` carries exact inputs and a required outcome, or a search with a stated instrument and budget reports a negative wider than the retained one and this ticket is re-deferred with that boundary recorded.
