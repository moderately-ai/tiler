---
id: explain-allocs
title: Remove the explain writer's per-record allocations
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: [crates/tiler-compiler/src/explain.rs]
tags: [performance]
---

Compile is ~900 µs for the governed five-operation program and retains 85 explain records. A samply profile said the remaining cost was diffuse allocation and copying rather than any single function. This ticket attacks the share of that owned by `crates/tiler-compiler/src/explain.rs`.

## Measurement

**Measurement.** Host: `cc52bc9`, release profile with `CARGO_PROFILE_RELEASE_DEBUG=true`, samply at 4000 Hz over `hot_path::hot_path_profile_loop` for 20 s. Shares below are of *active* (non-idle) samples; the harness's second thread parks and accounts for the 50 % idle the analyzer splits out. Subtree attribution was produced by reusing the parser in `quiltdb-bench/scripts/analyze_profile.py` to sum samples whose stack contains a given frame anywhere, which self-time alone cannot show.

**Fact.** The compilation subject is 20 194 bytes for this five-operation program (probed by printing `CompilationSubject::canonical.len()`). That single number explains the profile: `ExplainWriter::subject` cloned the blob per subject reference and `push` compared it per subject, ~85 times each per compile.

| subtree | before | after |
| --- | --- | --- |
| `explain::` (whole file) | 18.64 % | 7.40 % |
| `ExplainWriter::push` | 7.39 % (4.27 % in `memcmp`) | 1.50 % |
| `ExplainWriter::subject` | 2.79 % (2.19 % in `memmove`) | 0.28 % |
| `ExplainWriter::new` | 2.71 % | 2.74 % |
| `seal` | 2.59 % | 0.31 % |
| `encode_record` / `push_record` | 4.39 % | ~0.3 % |

**Measurement.** `hot_path_compile_time_by_shape`, min of 200 per run, best of five runs, both states measured with the same harness on the same host:

| shape | before | after | change |
| --- | --- | --- | --- |
| 4x3 | 899.6 µs | 760.8 µs | −15.4 % |
| 1024x3 | 871.6 µs | 760.9 µs | −12.7 % |
| 4x1024 | 907.5 µs | 751.6 µs | −17.2 % |

The profile loop's own throughput rose from 12 928 to 17 600 compiles per 20 s (−26.6 % mean time). The min-of-200 figure is the conservative one and is the headline; the wider mean gain is consistent with removing allocator pressure from the tail.

## What changed

- `CompilationSubject.canonical` is `Arc<[u8]>` rather than `Box<[u8]>`, with a hand-written `PartialEq` that takes a pointer-identity fast path and keeps byte equality as the definition. The cross-compilation guard rejects exactly the subjects it rejected before; only its cost changed.
- The writer keeps the trace preamble it encoded at `new` and the concatenated canonical encodings of the records it has admitted. A record is encoded once, into that buffer, and the byte budget reads its length from the buffer's growth. `seal` appends the record count and the records to the retained preamble. A record the bounds refuse is truncated away again.
- `encode_record` became `push_record(&mut Vec<u8>, …)`, and the trace preamble became `push_trace_preamble`, so `encode_trace` and the writer's incremental buffer share one encoder and one preamble rather than agreeing by inspection.
- `push` asks `ProviderRef::is_builtin()` instead of constructing a `ProviderRef::builtin()` — and allocating its key — once per retained record.
- `validate_key` scans bytes for an all-ASCII key. Below `0x80`, `char::is_control` is the C0 range plus `DEL` and `char::is_whitespace` adds only the space, so the byte scan reaches the same verdict without decoding or the Unicode property tables. The character scan stays as the definition for a non-ASCII key.

**Fact.** Explain output is unchanged: the byte-pinned render golden (`request=bb089e78b94e892c`) and every canonical-identity assertion pass untouched.

The one added assertion is a regression test. `terminal_ledger_rejects_duplicates_unknowns_and_max_detail_pressure` already refused a record at the detail ceiling and then sealed, but never called `verify()`, so nothing compared the incrementally built identity against the reference encoder on that path. **Measurement.** Removing the truncate-on-rejection makes that assertion fail and nothing else; the failure path is reachable.

## Findings outside this file, not acted on

1. **`ExplainWriter::new` is now the largest remaining item in the file, at 2.74 % of active work, and it is `stable_qualifier`.** **Measurement.** Short-circuiting `stable_qualifier` for inputs over 64 bytes moved min-of-200 from 760.8/760.9/751.6 µs to 748.3/741.0/730.4 µs — 13–21 µs per compile, matching its 2.24 % self-time. It is FNV-1a byte-at-a-time over the 20 194-byte subject, a serial multiply chain at roughly five cycles per byte. **It cannot be changed here:** the value is rendered as `request={:016x}` and pinned by a golden, so any other hash function is an output change. The real fix is upstream — see the next item.

2. **`VerifiedRequestSubject::canonical_explain_subject_bytes` (`crates/tiler-compiler/src/request.rs:837`) returns 20 194 bytes for a five-operation program.** Everything above is downstream of that: the qualifier hashes it, `seal` frames it, `VerifiedEvidenceRef::from_fusion_numerical` rebuilds it per proof, and it is what made the per-reference clone expensive. Nothing in this ticket examined *why* it is that large; that is the question worth a ticket against `request.rs`.

3. **`VerifiedEvidenceRef.compilation` is still an independently built `Box<[u8]>`** and is compared against the writer's subject with a full 20 KB `memcmp` per sound-proof record. Measured at 0.40 % of active before the change, so secondary. Sharing the writer's `Arc` would remove it, but `from_fusion_numerical` takes `&VerifiedTargetRequest` and its callers are in `fusion.rs`, outside this ticket's file.

4. **The `key_type!` macro allocates a `String` per key**, and nearly every key minted on the compile path is a `&'static str` literal. A `Cow<'static, str>` representation with a static constructor would remove several hundred small allocations per compile, but every construction site is in other files.

No dependency was added. Nothing measured here justified one: the wins were removals of copying and allocation, not faster data structures.
