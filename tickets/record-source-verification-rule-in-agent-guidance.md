---
id: record-source-verification-rule-in-agent-guidance
title: Record the read-the-file source-verification rule in agent guidance
status: in-progress
priority: p2
dependencies: []
related: [draft-public-api-conventions-adr, draft-public-boundary-approval-policy-adr]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, process, research-standards]
claimed_from: todo
assignee: agent-record-source-verification-rule-in-agent-guidance
lease_expires_at: 1784912789
---
Three separate agents in a single session drew **false conclusions from failed
substring searches**, each time against the same construct: a multi-line Rust
attribute.

The construct is

```
#![allow(
    dead_code,
    reason = "…"
)]
```

so the substring `allow(dead_code` never occurs, and both `grep "allow(dead_code"`
and `git log -S'allow(dead_code'` return nothing while the attribute is present
in five compiler modules.

Four occurrences, in increasing severity:

1. The agent drafting ADR 0074 hit it, noticed the result was implausible, opened
   the file, and self-corrected — the outcome we want.
2. The coordinator hit it while reviewing and nearly recorded that no precedent
   existed for the private-draft pattern.
3. The agent drafting ADR 0075 hit it, did **not** re-check by reading, and
   reported that ADR 0074 — already accepted — contained a factual error, with a
   `git log -S` citation as evidence. That correction was itself false and was
   caught only by manual verification before merge.
4. The coordinator, while *correcting* case 3, introduced a fresh false negative
   from a **different mechanism**: the search pattern was right, but the window
   was wrong. A `head -60` fallback truncated `selection.rs` exactly between
   `#![allow(` on line 60 and `dead_code,` on line 61, producing a confident
   claim that one module did not conform. The ADR-0075 agent refused the
   resulting instruction to write up a conformance gap, supplied
   `sed -n '58,64p' crates/tiler-compiler/src/selection.rs` as a reproducible
   check, and noted that recording a gap that does not exist would be the same
   error with its sign flipped. It was right; all six modules conform uniformly.

Cases 3 and 4 together are the argument for this ticket. Case 3 is a confident
false negative carrying a citation that *looks* rigorous, aimed at an accepted
contract. Case 4 shows the rule cannot be narrowed to "beware multi-line
patterns" — the second failure had a correct pattern and a truncated window, so
the unsound step is concluding **absence** from any bounded search, whatever
bounds it. Case 4 also shows the mitigation that actually worked: a downstream
reader who refuses an instruction it can disprove, and hands back a one-line
reproduction rather than complying.

Review is not a reliable filter for a mistake that arrives pre-justified — in
case 4 the reviewer *was* the source.

Add a short rule to the research/verification standards in `AGENTS.md`: **a
failed search is evidence that the search was wrong, not that the thing is
absent, until the file has been read.** Never assert absence from a search alone.
Multi-line attributes, wrapped signatures, and re-exported names defeat substring
matching; `git log -S` inherits the same weakness; and a bounded window
(`head -N`, a `sed` range, a truncated diff) can split the very construct being
searched for. When a search result contradicts a documented claim, open the file
before concluding the document is wrong.

Pair it with the practice that caught case 4: when asserting absence, **state the
exact check** so a reader can reproduce or refute it in one line, and treat a
correction that cannot be reproduced that way as unverified — including one
arriving from a reviewer.

Keep it brief and place it with the existing source-claim guidance (which already
requires inspecting the exact revision when making a source claim) rather than
creating a new section. This is a working rule, not a new contract area.
