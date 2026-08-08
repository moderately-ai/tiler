---
id: pin-the-admitted-unsafe-sites-in-the-workspace-gate
title: Pin the admitted unsafe sites in the workspace gate
status: in-progress
priority: p3
dependencies: []
related: [record-the-case-by-case-unsafe-boundary, prototype-metal-runtime-execution]
scopes: [implementation/workspace, contracts/navigation, contracts/decisions, implementation/frontend, implementation/conformance, implementation/runtime, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, workspace, gate, rust-api, deferred]
claimed_from: todo
assignee: sol-unsafe
lease_expires_at: 1786218119
---
ADR 0079 permits unsafe code only at individually admitted sites. The current
compiler lint enforces that an unsafe block needs a local allow, but
`AGENTS.md` correctly states that no check inventories those allows. Review is
the only control over a newly added, moved, removed, or re-justified site.

A former Python workspace gate pinned each admitted
`(package-relative path, item signature, reason)` and had negative mutation
tests. That gate and its tests were deleted when repository verification moved
to the root `Makefile`; no implementation is currently in review.

## Deferred boundary

> **This deferral's premise expired on 2026-08-07 and the boundary below is struck.** It read: keep review-only enforcement "while the complete admitted population is two sites in one non-published prototype", because "a mechanical source scanner would add a second parsing authority to the gate **before the production population exists**". The production population now exists. See the fired trigger-check entry at the end of this ticket.
>
> **One clause of it survives and is now evidence rather than speculation:** "the obvious grep-shaped implementation demonstrably misses both multi-line attributes." That was borne out twice on 2026-08-07 — a single-line `grep` for `allow(unsafe_code` over the current tree returns three hits, **all three of them prose in manifests and doc comments**, and zero of the four real attributes. Any scanner this ticket lands must be multi-line-aware, and the negative test for it is that a single-line matcher fails.

## The admitted population today (2026-07-28)

The ticket asks Tom to choose an enforcement posture without saying what is being enforced over. It is **two sites, both in one non-published prototype**, and that changes how both options read.

Reproduce with `grep -rn --include='*.rs' -B1 '^    unsafe_code,' crates prototypes`, which returns exactly two matches:

| Site | `#[allow(` opens | `unsafe_code,` | `unsafe` block | Item |
| --- | --- | --- | --- | --- |
| `prototypes/serial-sum-run/src/buffer.rs` | `:35` | `:36` | `:52` | `pub fn write_f32` (`:39`) |
| `prototypes/serial-sum-run/src/buffer.rs` | `:67` | `:68` | `:85` | `pub fn read_f32` (`:72`) |

> **Superseded 2026-08-07 — the population is now four, and two of them are under `crates/`.** The table above and the sentence below are the 2026-07-28 state, retained because the options are argued against them and a reader needs to see which population each argument was made for.

**Struck:** "There is no admitted unsafe site anywhere under `crates/`." **`crates/tiler-conformance/src/device_buffer.rs` carries two**, at `write_bytes` and `read_bytes`, both over `std::ptr::copy_nonoverlapping` on `Buffer::contents()`. Tom decided the rule admitting them on 2026-08-07 under [`decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`](decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access.md): `deny` with named per-site allows, **never at the crate**, FFI memory management against Metal as the only admitted justification, and isolation into one module as a design constraint.

**That inverts the strongest argument for review-only enforcement**, which was that the whole population sat in one non-published prototype. It no longer does. It also supplies the shape a mechanical check should take, already built and passing: `crates/tiler-conformance/src/bf16_vertical/tests.rs`'s `the_unsafe_site_population_is_the_two_named_ones` walks every file under that crate's `src/`, requires exactly two blocks and two allows both in `device_buffer.rs`, and **carries a file-count floor so it cannot pass by scanning a shrunken tree** — which is precisely the declare-and-count discipline the paragraph below demands. What is missing is that it is crate-scoped; generalizing it is this ticket's work.

The 2026-07-28 sites remain valid and unchanged: `prototypes/serial-sum-run/Cargo.toml` declares `[lints.rust] unsafe_code = "deny"` with its reason, and both meet ADR 0079's four conditions — `Buffer::contents` the only route to `MTLBuffer` storage, a `reason` on each `#[allow]`, an `assert!` against the buffer's own `length()`, and a `SAFETY` comment naming the invariant. The two new sites meet the same four.

**And here is the fact that bears directly on the mechanical option.** `grep -rn --include='*.rs' 'allow(unsafe_code' crates prototypes | wc -l` returns **0**. Both attributes wrap across lines, so the obvious grep-shaped inventory matches *none of the population* and reports that cleanly — zero hits, exit non-zero, no error. A check written that way would say "no unadmitted sites" and "no sites at all" in exactly the same way it would say "the check did not run".

That is the hazard `AGENTS.md` states as **"a verdict is only as good as the check's ability to say no"**, and it is the same shape as the worktree survey that reported forty-three clean checkouts because `head` was unresolvable inside the loop, and as a `trybuild` glob that stops matching and reports a passing test having compiled nothing. **So it is a requirement on the mechanical option, not a caution:** the check must declare its expected population and count it, so that an empty inventory is a *failure* rather than a pass. A check that only looks for violations cannot distinguish zero violations from zero observations.

## The two options, with the population known

- **Review-only enforcement.** Permitted by ADR 0079, keeps the gate simple, and costs nothing to maintain. *Enables:* a new site is admitted by the same judgement ADR 0079 asks for — a human reading the diff that adds it, which is what "case by case" means. *Prevents:* nothing mechanically. A new allow, a moved one, a removed assertion, or a silently reworded `reason` relies entirely on diff review. **Two sites in one non-published prototype is the strongest available argument for this option, and the ticket currently hides it** by asking the question against an unstated and implicitly larger population.
- **Mechanical inventory.** *Enables:* the admitted population becomes explicit and machine-checked; a moved-plus-added pair cannot net out; the check can be made to prove its own failure path. *Prevents:* nothing about correctness directly — it prevents an *unreviewed* change to the population. *Costs:* a source-scanning authority in the gate, whose parsing boundary must be documented (the zero-hit grep above is the proof that the boundary is not obvious), and whose pin must be updated in the same change as any site edit.

## Decided 2026-08-07 — mechanical inventory. This is no longer a question.

The ticket's title asked Tom to *decide whether*; its id has always said *pin*. It is settled as **mechanical inventory**, by the coordinator, and the ticket is now work rather than a decision. Three things carry it, none of which were available when the question was framed:

- **The argument for review-only has expired with its premise.** The Recommendation below calls this "a close call Tom could reasonably decide either way" and rests that squarely on the population being "**two** sites in a prototype that AGENTS.md already says is rewritten or deleted". The population is four, half of them under `crates/`, and the deferral clause that said so is struck at the top of this ticket.
- **Tom stated the governing policy on 2026-08-07**, deciding `decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`: named per-site allows, **never at the crate**, FFI memory management against Metal as the only admitted justification, and "the goal is to isolate the unsafe code as much as possible". A policy naming *which* sites are admitted is exactly what a path/signature/reason inventory enforces and what review-only cannot.
- **This is a gate mechanism, not a public boundary.** Under AGENTS.md it is the coordinator's to settle. What still returns to Tom is unchanged and stated in the Activation trigger: any resulting workspace-gate or unsafe-policy *boundary*, and any admission of a fifth site.

The Recommendation, its counter-argument, and the two options are kept below **unedited** — the reasoning is what makes the decision reviewable, and a reader needs to see which population each argument was made against.

### What the work is, now that the posture is fixed

Generalize `crates/tiler-conformance/src/bf16_vertical/tests.rs`'s `the_unsafe_site_population_is_the_two_named_ones` from crate-scoped to workspace-scoped, keeping the two properties that already make it sound: it walks files rather than grepping, and it carries a **file-count floor** so it cannot pass by scanning a shrunken tree. The pin is the `(package-relative path, item signature, reason)` triple, so a moved-plus-added pair cannot net out. Requirements that are not optional:

- **Multi-line-aware matching.** The negative test for this is that a single-line matcher *fails* — verified twice on 2026-08-07, where a single-line `grep` for `allow(unsafe_code` returned three hits that were all prose in manifests and doc comments, and zero of the four real attributes.
- **A doc-comment mention must not count.** `crates/tiler-conformance/src/lib.rs` carries one today; it is the live fixture for this.
- **Declare and count the expected population**, so an empty inventory fails rather than passes.
- **Run four mutations and watch each fail** — addition, move, removal, reason change — rather than asserting the check compiles.

## Recommendation

Restore the exact path, item signature, and reason inventory — **but the recommendation is now weaker than it was, and the re-derivation should be visible.** Against an unbounded population the argument is straightforward. Against **two** sites in a prototype that AGENTS.md already says is "rewritten or deleted as the slice they prove moves", the maintenance cost is a larger fraction of the benefit, and the honest summary is that this is a close call Tom could reasonably decide either way.

What still carries it: the permission is case-by-case, so a count alone is insufficient — moving one site while adding another must not pass, and only a path/signature/reason triple catches that. And the population is two *today*; `prototype-metal-runtime-execution` is where a third would arrive, and the moment to install an inventory is before the population grows rather than after.

If mechanical enforcement is selected, a **negative mutation test must prove the check can fail** for each of addition, move, removal, and reason change — run each mutation and watch it fail, rather than asserting the check compiles. And per the zero-hit fact above, the check must name and count its expected population so that finding nothing fails.

## Activation trigger

Reactivate before admitting the first production unsafe site, or when the admitted population grows beyond the two current prototype functions. At activation, derive the inventory mechanism from Rust syntax rather than a zero-observation grep, name and count the expected population, and demonstrate failure for addition, move, removal, and reason change. Tom reviews any resulting workspace-gate or unsafe-policy boundary.

## Trigger check log

- 2026-08-04 — **not fired.** The admitted population is still exactly two, both in `prototypes/serial-sum-run/src/buffer.rs`, and there is still no admitted unsafe site under `crates/` — so neither "before the first production site" nor "the population grows beyond two" has arrived. [`prototype-metal-runtime-execution`](prototype-metal-runtime-execution.md) is `done` and added none. Recheck: `grep -rn --include='*.rs' -B1 '^    unsafe_code,' crates prototypes` returns exactly two matches.
- 2026-08-07 — **FIRED, on both clauses.** Verified independently by the coordinator with a multi-line-aware scan, because a single-line `grep` misses these attributes — the named allow sites are spelled across several lines and an earlier single-line check on this same population returned a misleading count. The real population is **four**: `crates/tiler-conformance/src/device_buffer.rs` (2, over `std::ptr::copy_nonoverlapping` on `Buffer::contents()`) and `prototypes/serial-sum-run/src/buffer.rs` (2). A fifth textual match in `crates/tiler-conformance/src/lib.rs` is **inside a doc comment**, not an attribute, and must not be counted.

  So both clauses hold: the population grew past the two prototype functions, **and** the first non-prototype admission has landed. This ticket's load-bearing Fact — "**There is no admitted unsafe site anywhere under `crates/`**" — is now false and must be rewritten before dispatch. Tom decided the rule that admitted them on 2026-08-07 (`decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`): `deny` with named per-site allows, never at the crate, FFI memory management against Metal as the only admitted justification.

  **Carry this in rather than re-deriving it:** a partial counting check already exists but is crate-scoped — `crates/tiler-conformance/src/bf16_vertical/tests.rs`'s `the_unsafe_site_population_is_the_two_named_ones` walks every file under that crate's `src/` and fails if a third appears, with a file-count floor so it cannot pass by scanning a shrunken tree. That is the shape a workspace-wide pin wants, generalized. Recheck, and **use a multi-line-aware matcher**: `python3 -c "import re,glob; print(sum(len(re.findall(r'#\[allow\(\s*unsafe_code', open(f).read())) for f in glob.glob('crates/**/*.rs',recursive=True)+glob.glob('prototypes/**/*.rs',recursive=True)))"` — four attributes plus one doc-comment mention.


> **The command above was itself wrong and is corrected, 2026-08-07.** It returned **5**, not 4 — it matched the `crates/tiler-conformance/src/lib.rs` doc comment, which is precisely the false positive it was written to exclude. Found by the worker on [`date-adr-0079-s-one-crate-claims-for-the-second-diverging-member`](date-adr-0079-s-one-crate-claims-for-the-second-diverging-member.md) rather than by the coordinator who wrote it. The fix is the `^\s*` line anchor, and printing **per-file locations rather than a bare total**, so a miscount is visible instead of merely wrong:
>
> ```sh
> python3 -c "
> import re, glob
> pat = re.compile(r'^\s*#\[allow\(\s*unsafe_code', re.M)
> for f in sorted(glob.glob('crates/**/*.rs', recursive=True) + glob.glob('prototypes/**/*.rs', recursive=True)):
>     n = len(pat.findall(open(f).read()))
>     if n:
>         print(n, f)
> "
> ```
>
> Correct output is two files at two each: `crates/tiler-conformance/src/device_buffer.rs` and `prototypes/serial-sum-run/src/buffer.rs`. Substituting `r'^\s*unsafe\s*\{'` gives the identical two files at two each, and **that pairing is the evidence that no block escaped its attribute** — a count alone cannot show it. This is the same defect class the ticket exists to prevent, committed in the ticket's own repair text.
