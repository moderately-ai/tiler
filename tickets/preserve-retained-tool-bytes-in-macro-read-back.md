---
id: preserve-retained-tool-bytes-in-macro-read-back
title: Preserve retained tool bytes in macro read-back
status: in-progress
priority: p1
dependencies: [emit-from-a-populated-retention-in-the-inline-expansion]
related: [accept-the-retention-read-back-s-caller-visible-boundary]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [frontend, diagnostics, correctness, public-boundary]
claimed_from: todo
assignee: worker-retained-tool-bytes
lease_expires_at: 1786585710
---
## User-visible outcome

The inline macro writes each retained tool byte without trimming or lossy substitution, while keeping provenance, invalid-UTF-8 status, and truncation metadata distinguishable from tool output.

## Facts to re-verify

**Fact — storage is already exact.** `RetainedText` stores `Vec<u8>` and exposes `as_bytes`; the cache frames and digests those bytes without interpreting them.

**False accepted-ticket premise — the landed read-back is not verbatim.** `SpokenRetention::fmt` delegates each run to `RetainedText::Display`, which renders `String::from_utf8_lossy(&self.retained).trim()`. Leading/trailing whitespace is removed and invalid byte sequences are substituted. The invalid-UTF-8 and truncation markers remain truthful, but the tool byte run is not exact.

**False current message — it claims a later phase already succeeded.** `aot::deliver` calls `report_retained_output` immediately after cache/artifact acceptance but before payload-cardinality validation, route-fact construction, `DeliveryPlan::new`, token emission, and `guarded_emission`'s final token validation. The note currently says “The expansion succeeded” and the artifact is “embedded”; a later typed refusal can still prevent both. The retained output is real, but its phase attribution overclaims.

## Fact audit at `612468048d541a1017640fc5dcbe5ff9160716cf`

Re-read at this base: `RetainedText`, `SpokenRetention`, `report_retained_output`, and `aot::deliver`. Each ticket claim is judged against the file, not the last summary.

**Fact — storage is already exact.** Verified. `RetainedText` holds `retained: Vec<u8>` and `as_bytes` returns `&self.retained`. `DebugRetention::encode` frames those bytes with `push_run(..., &run.retained)` and does not decode them. `bundle::encode` then places the encoded section in the frame and digests it with `section_digest(content)` — `DigestAlgorithm::GOVERNED.digest(SECTION_DIGEST_DOMAIN, content)` over the section's exact bytes. Decode copies the framed span back into `RetainedText` without a UTF-8 check on the run body. The retention module's own statement is that the crate "frames, counts, bounds, and digests the runs; it never interprets one".

**False accepted-ticket premise — the landed read-back is not verbatim.** Verified as false of the landed renderer. `SpokenRetention::fmt` does `write!(formatter, "\n{run}")`, which is `RetainedText::Display`. That impl writes `String::from_utf8_lossy(&self.retained).trim()`, then the typed markers ` [output was not valid UTF-8]` and ` [truncated: {} of {} bytes retained]`. The markers are truthful. The tool run is not exact.

**False current message — it claims a later phase already succeeded.** Verified as false of the landed preamble, with one non-load-bearing imprecision. `aot::deliver` calls `report_retained_output` after `accept_or_publish_metal_plan` returns `Ok` and after the optional `Published` eviction sweep, then payload-cardinality (`let [payload] = artifact.payloads()` → `AotRefusal::MalformedArtifact`), `RouteFacts` construction, `DeliveryPlan::new` (`AotRefusal::MalformedPlan`), return into `expand`'s `emit` (token lex / `Refusal::MalformedEmission`), and `tensor`'s `guarded_emission` (`validate_emitted_tokens`). The preamble still says "The expansion succeeded" and "the artifact is compiled, validated, and embedded". The imprecision is "immediately after cache/artifact acceptance": the eviction sweep can run between acceptance and the report. That does not change the later-refusal claim. No caller inspects `report_retained_output`'s unit return or otherwise relies on final-success timing, so the note stays at AOT resolution and the preamble must name only that phase.

**Consumer census of `RetainedText::Display`.** Production formatting of a `RetainedText` is only `SpokenRetention::fmt`. Cache tests also render it: `a_retained_run_is_bounded_and_records_its_truncation` asserts `run.to_string().contains("truncated")` and `contains("not valid UTF-8")` while separately pinning `as_bytes() == &[0xff, 0xfe]`. `ToolOutput::Display` in `tiler-metal-aot` is a parallel lossy renderer on a different type. `fmt::Write` cannot carry invalid UTF-8, so `Display` cannot be the byte-faithful stderr path. Changing `RetainedText::Display` would rewrite the accepted public cache surface. One shared renderer is not the only coherent authority; the frontend write stays a private `io::Write` renderer.

## Perturbation evidence

Assertions unchanged. Each perturbation was applied only to `SpokenRetention::write_to`'s `write_all(run.as_bytes())` call, then restored. A first attempt that searched for the run anywhere in the note stayed green when the leading space was dropped, because the provenance separator is itself `": "`. The assertion is now the bytes immediately after `{label}: `.

**Trim a leading byte** (`write_all(&run.as_bytes()[1..])`). `a_leading_whitespace_byte_is_written_exactly` failed:

```text
assertion `left == right` failed: the tool's own bytes must follow the run provenance unaltered
  left: [32, 112, 114, ...]
 right: [32, 32, 112, 114, ...]
```

Left is one space then `program_source`; right is the retained two-space run.

**Trim a trailing byte** (`write_all(&run.as_bytes()[..len - 1])`). `a_trailing_whitespace_byte_is_written_exactly` failed:

```text
assertion `left == right` failed: the tool's own bytes must follow the run provenance unaltered
  left: [..., 39, 120, 39, 32, 10]
 right: [..., 39, 120, 39, 32, 32]
```

Left ends `x` quote, one space, then the note's newline; right ends with the retained two trailing spaces.

**Replace an invalid byte** (`write_all(String::from_utf8_lossy(run.as_bytes()).as_bytes())`). `a_run_that_is_not_utf8_is_written_exactly_and_labelled` failed:

```text
assertion `left == right` failed: the tool's own bytes must follow the run provenance unaltered
  left: [239, 191, 189]
 right: [255, 254, 253]
```

Left is U+FFFD; right is the retained `0xff 0xfe 0xfd`. The rendered line was `tiler.metal.0.metal: ��� [output was not valid UTF-8]`.

## Required outcome

- Write the exact retained bytes through the existing `io::Write` seam, after the macro/run provenance and before separately distinguishable metadata.
- Preserve leading and trailing whitespace, embedded newlines, and invalid byte sequences exactly.
- Keep invalid-UTF-8 and truncation state explicit without inserting marker bytes into what is claimed as the tool's own run.
- Preserve silence for absent and all-empty retentions, all speaking runs in producer order, every resolution path, nonfatal behavior, and the `` `tiler::tensor!` `` attribution.
- Prefer a private frontend renderer over changing the accepted public `RetainedText::Display` surface unless the complete consumer census proves one shared renderer is the only coherent authority.
- Make the phase claim exact. Either carry the report to the final successful emission boundary, or keep it at AOT resolution and say only that offline compilation plus cache/artifact acceptance succeeded and that later frontend emission can still refuse. Prefer the latter narrow wording unless a complete control-flow audit finds callers rely on final-success timing.

## Subject perturbations

With assertions unchanged, independently trim a leading byte, trim a trailing byte, and replace an invalid byte; quote the exact failing diagnostics. Retain the existing multi-line, quiet, no-run, truncation, elision, and all-speaking-run census.

## Stop conditions

Stop if exact stderr bytes cannot coexist with an unambiguous metadata boundary, if the coherent repair changes the accepted public cache surface, or if truthful phase attribution requires a new caller-visible result shape. File the required public-boundary decision rather than silently redefining `RetainedText::Display` or expansion success.

## Required checks

Run the macro package tests and doctests, Clippy/rustdoc with warnings denied, exact consumer fixtures, citations, lint, exact-base guard, and the full exact-tip repository gate required by the touched crate path.

## Worker notes

Private frontend renderer only. `RetainedText::Display` is unchanged. The note stays at AOT resolution; the preamble now says offline compilation plus cache/artifact acceptance succeeded and later frontend emission can still refuse. Existing quiet, no-run, multi-line, truncation, elision, and all-speaking-run tests remain.

Checks at this worktree: `cargo test -p tiler-macros --lib retention` 14/14; `cargo test -p tiler-macros` lib 182/184 with one unrelated `preflight::tests::a_root_that_answers_for_everything_reports_nothing` flake under parallel load (advisory lock refuted; isolated re-run passed); `cargo test -p tiler-macros --doc` 0 doctests; `cargo clippy -p tiler-macros --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-macros --no-deps`; `cargo test -p tiler --doc`; `cargo test -p tiler --test facade`; `tkt lint`; `git diff --check`; `make citations`; `tkt guard` against `61246804`. Not merged, not closed.
