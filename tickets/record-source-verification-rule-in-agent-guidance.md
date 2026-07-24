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

## Outcome

**Fact.** Added two bullets to the `AGENTS.md` "Research standards" list,
immediately after the existing "Inspect the exact local dependency revision when
making a source claim and record the commit or version" bullet. The first states
that a failed search is evidence the search was wrong, not that the thing is
absent, until the file has been read, and names the failure modes: multi-line
attributes, wrapped signatures, re-exported names, `git log -S`, and bounded
windows that split the construct. The second requires stating the exact check
when asserting absence, and treats an unreproducible correction as unverified,
including one from a reviewer.

**Fact.** The change is +10/-0 lines in `AGENTS.md` and touches nothing else in
the file. No new section, no restructuring, no rewrapping of existing prose; the
new lines match the file's existing hard wrap (79 columns maximum, against a
file maximum of 80 excluding the one long worktree path).

**Inference.** The placement is correct because the negative source claim ("this
construct does not appear") is the same claim class the adjacent bullet already
governs, and both failures in cases 3 and 4 were source claims about a specific
revision. Grouping them keeps the measurement-oriented bullet ("A failed or
unavailable measurement is useful evidence when the limitation is precise")
undisturbed further down the list, where it addresses a different class.

**Fact.** `AGENTS.md` is outside the documentation validator's file set
(`scripts/docs.py` collects the root `README.md`, `docs/**/*.md`, and
`spikes/**/README.md`), so no catalog render was required and no frontmatter is
involved.

**Measurement.** `uv run --locked python scripts/check_repository.py` exited 0
with "complete repository validation passed"; `ticketsplease lint` reported "ok:
no problems found"; `git diff --check 37f1350 HEAD` produced no output and
exited 0. `ticketsplease guard
tkt/record-source-verification-rule-in-agent-guidance --base 37f1350 --explain`
exited 0 with verdict WARN: two changed files, affected scopes exactly equal to
declared scopes, no under-declaration. The WARN is declared-area overlap on the
shared `project/tickets` scope with other open tickets (plus
`implementation/workspace` with `promote-index-oracle-integration-test` and
`prototype-apple-aot-driver`), which is the expected shared-scope state, not a
proven merge conflict. macOS arm64, toolchain `nightly-2026-07-19` from
`rust-toolchain.toml`.

**Measurement.** The gate is intermittent on this host for a reason unrelated to
this change. Four gate invocations at this tree: runs 1 and 4 exited 0; runs 2
and 3 exited 1 on
`spikes/embedding/test_measure.py::test_run_logged_enforces_output_limit`,
raising `PermissionError: [Errno 1] Operation not permitted` from
`os.killpg(process.pid, signal.SIGKILL)` at `spikes/embedding/measure.py:373`.
The test's child (`print('x' * 1000)`) usually exits before the capture-limit
branch fires, so the target process group holds only an unreaped child, and
signalling it can return EPERM. It did not reproduce under narrower runs: 12/12
for the test alone, 11/11 for `spikes/embedding`, 8/8 for the full gate test set
with `.venv/bin` first on `PATH`, and 6/6 under `uv run`.

**Inference.** That failure cannot originate here: `git diff 37f1350 HEAD
--name-only -- spikes/` is empty, so every file under `spikes/` is byte-identical
to the base commit. It needs its own ticket against the spike; it is outside this
ticket's `implementation/workspace` and `project/tickets` scopes and was not
touched.

**Measurement.** A separate host-local confound, recorded so it is not mistaken
for the same defect: running the gate's pytest set directly from an interactive
shell fails
`spikes/macro-environment/test_probe.py::test_overall_alarm_reaps_child_after_capture_pipes_close`
3/3, because the probe command invokes bare `python3`, which resolves to
`/Users/tsanterre/.pyenv/shims/python3` (~0.76 s startup) and loses the 0.2 s
`setitimer` race. The gate itself is unaffected: `sanitized_environment()` in
`scripts/check_repository.py` puts `.venv/bin` first, and with that `PATH` the
same set passes 8/8. This is an artifact of how the set was invoked, not a
repository defect.

**Measurement.** Wrap conformance, stated as a reproducible check per the rule
this ticket adds: `awk 'length > 79 { print NR, length }' AGENTS.md` lists no
line introduced here. It reports lines 14, 108 and 115 at 80-82 *bytes*, but
those are 76-78 characters — each contains curly quotes that cost three bytes
apiece. The file's true character maximum is 80, set by the ASCII lines 38, 195,
206, 207 and 404, excluding the 117-character worktree path on line 289.
