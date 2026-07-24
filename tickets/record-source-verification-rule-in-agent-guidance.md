---
id: record-source-verification-rule-in-agent-guidance
title: Record the read-the-file source-verification rule in agent guidance
status: todo
priority: p2
dependencies: []
related: [draft-public-api-conventions-adr, draft-public-boundary-approval-policy-adr]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, process, research-standards]
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

The three occurrences, in increasing severity:

1. The agent drafting ADR 0074 hit it, noticed the result was implausible, opened
   the file, and self-corrected — the outcome we want.
2. The coordinator hit it while reviewing and nearly recorded that no precedent
   existed for the private-draft pattern.
3. The agent drafting ADR 0075 hit it, did **not** re-check by reading, and
   reported that ADR 0074 — already accepted — contained a factual error, with a
   `git log -S` citation as evidence. That correction was itself false and was
   caught only by manual verification before merge.

The third case is the argument for this ticket: a confident false negative,
carrying a citation that *looks* rigorous, aimed at an accepted contract. Review
caught it once; review is not a reliable filter for a mistake that arrives
pre-justified.

Add a short rule to the research/verification standards in `AGENTS.md`: **a
failed search is evidence that the pattern is wrong, not that the thing is
absent, until the file has been read.** Never assert absence from a search alone;
multi-line attributes, wrapped signatures, and re-exported names all defeat
substring matching, and `git log -S` inherits the same weakness. When a search
result contradicts a documented claim, open the file before concluding the
document is wrong.

Keep it brief and place it with the existing source-claim guidance (which already
requires inspecting the exact revision when making a source claim) rather than
creating a new section. This is a working rule, not a new contract area.
