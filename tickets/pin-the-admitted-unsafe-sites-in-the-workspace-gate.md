---
id: pin-the-admitted-unsafe-sites-in-the-workspace-gate
title: Pin the admitted unsafe sites in the workspace gate
status: done
priority: p3
dependencies: []
related: [record-the-case-by-case-unsafe-boundary, prototype-metal-runtime-execution]
scopes: [implementation/workspace, contracts/navigation, contracts/decisions, implementation/frontend, implementation/conformance, implementation/runtime, contracts/foundation, implementation/cargo-lock, implementation/digest, implementation/ir, implementation/reference, implementation/artifact, implementation/compiler, implementation/metal, implementation/metal-aot, implementation/cache, implementation/build, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, workspace, gate, rust-api, deferred]
---
ADR 0079 permits unsafe code only at individually admitted sites. The current
compiler lint enforces that an unsafe block needs a local allow in the two
members that carry `deny`, and the workspace inheritance check keeps every
other member at `forbid`. Before this ticket, one crate-local check counted and
located the two conformance sites, but it could not see the prototype and it
pinned neither item signatures nor reasons. Review was therefore still the
only control over a prototype addition, move, or removal, every reason change,
and a count-preserving move inside `device_buffer.rs`.

A former Python workspace gate pinned each admitted
`(package-relative path, item signature, reason)` and had negative mutation
tests. That gate and its tests were deleted when repository verification moved
to the root `Makefile`; at the audited base, no replacement implementation was
in review.

## Deferred boundary

> **This deferral's premise expired on 2026-08-07 and the boundary below is struck.** It read: keep review-only enforcement "while the complete admitted population is two sites in one non-published prototype", because "a mechanical source scanner would add a second parsing authority to the gate **before the production population exists**". The production population now exists. See the fired trigger-check entry at the end of this ticket.
>
> **One clause of it survives and is now evidence rather than speculation:** "the obvious grep-shaped implementation demonstrably misses both multi-line attributes." That was borne out twice on 2026-08-07. At the audited base on 2026-08-08, `grep -rn --include='*.rs' 'allow(unsafe_code' crates prototypes` returns one doc-comment hit and zero of the four real attributes; including manifests adds three more prose hits. Any scanner this ticket lands must be multi-line-aware, and its positive evidence must show that it reaches the wrapped attributes.

## Historical admitted population (2026-07-28)

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

**And here is the fact that bears directly on the mechanical option.** At the audited base, `grep -rn --include='*.rs' 'allow(unsafe_code' crates prototypes` returns **one prose hit and zero attributes**. All four attributes wrap across lines, so the obvious grep-shaped inventory matches *none of the population* and reports only the doc-comment fixture. A check written that way would observe no admitted sites at all while still producing output, which is more misleading than a clean zero.

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

- **Multi-line-aware matching.** The positive test must show that the scanner reaches the wrapped form. At the audited base, a Rust-source-only single-line `grep` for `allow(unsafe_code` returns one doc-comment hit and zero of the four real attributes; adding member manifests produces four prose hits total and still reaches no attribute.
- **A doc-comment mention must not count.** `crates/tiler-conformance/src/lib.rs` carries one today; it is the live fixture for this.
- **Declare and count the expected population**, so an empty inventory fails rather than passes.
- **Run four mutations and watch each fail** — addition, move, removal, reason change — rather than asserting the check compiles.

## Per-Fact audit, 2026-08-08 at `8259cef4a8962c5f42ae41bf79a4fe53d2a70238`

Every current unsafe site, all sixteen member manifests, the root manifest and
Makefile, ADR 0079, the current lint and unsafe-site checks, and the deleted
gate's scanner and mutation tests were read at this base or from their exact
historical commits. The purpose is unchanged, but three current-tense claims
needed repair above.

- **Opening, governing policy and compiler levels — verified.** ADR 0079's searchable anchors `### 1. Unsafe is permitted case by case, at an individual function or module` and `### 4. What this record does not license` make the permission site-specific and reserve a further admission. Root `Cargo.toml`'s `[workspace.lints.rust]` says `unsafe_code = "forbid"`; `crates/tiler-conformance/Cargo.toml` and `prototypes/serial-sum-run/Cargo.toml` each say `unsafe_code = "deny"`. `crates/tiler/tests/workspace_lint_inheritance.rs`'s `UNINHERITED_LINT_MEMBERS` holds that exact two-member partition, and both members run the shared table reader in `crates/tiler-conformance/src/lints.rs`.
- **Opening, no inventory and review-only enforcement — false as written, repaired above.** `fn the_unsafe_site_population_is_the_two_named_ones()` is a real partial inventory: it roots at `CARGO_MANIFEST_DIR/src`, floors the file walk at twelve, restricts both searched tokens to `device_buffer.rs`, and requires two blocks and two allows. It cannot see `prototypes/serial-sum-run`, never reads an item signature or reason, and a move within `device_buffer.rs` that preserves the count passes. `AGENTS.md` no longer claims that no inventory exists.
- **Former Python gate and its deletion — verified.** Commit `a02c6509` added `ADMITTED_UNSAFE_SITES`, `validate_unsafe_site_pins`, and the addition, move, removal, and reason-change tests in `scripts/tests/test_rust_gate_integrity.py`. Commit `e197176f` deleted the 952-line `scripts/check_workspace.py` and the 470-line test file while replacing the Python gate with the root `Makefile`.
- **Current admitted population — verified as four sites in two files.** The source-safe anchors are `pub(crate) fn write_bytes` and `pub(crate) fn read_bytes` in `crates/tiler-conformance/src/device_buffer.rs`, and `pub fn write_f32` and `pub fn read_f32` in `prototypes/serial-sum-run/src/buffer.rs`. The corrected anchored multi-line scan prints those two files at two allows each; substituting `r'^\s*unsafe\s*\{'` prints the same two files at two blocks each. All four attributes carry their own exact `reason`.
- **Single-line grep count — false as a current count, repaired above.** The Rust-source-only command returns one doc-comment mention, not three, and reaches zero real attributes. Including member manifests returns four prose hits total; including root `Cargo.toml` returns five. The load-bearing claim survives: a single-line matcher observes none of the four wrapped attributes.
- **Historical two-site population — verified as historical and imprecise as a heading.** The section is retained because the option analysis was made against it, but `today` was false at this base and is now `Historical`. The dated supersession already records the later two sites.
- **Current source and file populations — verified.** The workspace member set is sixteen. The tree contains 421 tracked Rust source files under those members before this ticket's new test file, and the admitted-site population is exactly four. The workspace check must derive the member roots, require every member to contribute source, floor the total file population, and compare the four exact `(path, item signature, reason)` triples.

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

## Initial outcome — superseded by the post-review repair below

This was the first landing's result. The dated post-closure Fact repair below
demonstrates why its 422-source and two-package macro boundary was provisional;
the final result is the last paragraph of this ticket.

Implemented the mechanical-inventory decision without changing ADR 0079's
policy or admitting another site. `crates/tiler/tests/workspace_unsafe_sites.rs`
cross-checks the sixteen explicit member roots against Cargo's actual metadata
package set, requires every package to contribute source, enumerates all 63
distinct target roots regardless of extension, and reports 422 source files
against a floor of 400. It follows canonically contained literal `include!` and
source-file-root `#[path]` edges with cycle-safe deduplication. It lexically removes
line comments, nested block comments, strings, raw strings, and character
literals before examining attributes, so the live doc comment and synthetic
prose fixtures do not enter the population. It accepts only a direct
source-file-root-function `#[allow(...)]` with the whole `unsafe_code` lint name and
one ordinary literal `reason`, reads the following complete function signature
through its body brace, and reports unsupported attribute/meta/item/source-load
forms or unclosed lexical constructs rather than skipping them.

The exact pin is four `(workspace-relative path, complete normalized item
signature, exact reason)` triples: `write_bytes` and `read_bytes` in
`crates/tiler-conformance/src/device_buffer.rs`, and `write_f32` and `read_f32`
in `prototypes/serial-sum-run/src/buffer.rs`. The prior conformance-only token
count was removed, leaving one parser and one population. The compiler,
`workspace_lint_inheritance.rs`, and the shared exact lint-table reader remain
the lint-level authority; this inventory neither copies nor relaxes them.

The clean focused run printed:

```text
unsafe-site census: 422 source file(s), 63 Cargo target(s), and 16 package(s); 4 admitted site(s): [("crates/tiler-conformance/src/device_buffer.rs", "pub(crate) fn write_bytes(buffer: &Buffer, bytes: &[u8])"), ("crates/tiler-conformance/src/device_buffer.rs", "pub(crate) fn read_bytes(buffer: &Buffer, len: usize) -> Vec<u8>"), ("prototypes/serial-sum-run/src/buffer.rs", "pub fn write_f32(buffer: &Buffer, values: &[f32])"), ("prototypes/serial-sum-run/src/buffer.rs", "pub fn read_f32(buffer: &Buffer, count: usize) -> Vec<f32>")]
```

That clean run reaches all four live attributes, all of which wrap. The
separate `a_wrapped_attribute_and_signature_are_one_reached_site` fixture also
passed with both the attribute and signature split across lines and a prose
mention ahead of them.

Each required mutation changed the live subject in
`prototypes/serial-sum-run/src/buffer.rs`, ran only
`the_workspace_unsafe_sites_are_exactly_the_four_admitted_ones`, failed with
exit 101, and was restored before the next:

**Addition**, by adding a fifth direct permission:

```text
unsafe-sites.prototypes/serial-sum-run/src/buffer.rs: `fn added_unsafe_site()` admits unsafe_code and is not pinned; ADR 0079 makes a new site a new decision
```

**Move**, by renaming the admitted `write_f32` item:

```text
unsafe-sites.prototypes/serial-sum-run/src/buffer.rs: `pub fn write_f32_moved(buffer: &Buffer, values: &[f32])` admits unsafe_code and is not pinned; ADR 0079 makes a new site a new decision
unsafe-sites.prototypes/serial-sum-run/src/buffer.rs: pinned site `pub fn write_f32(buffer: &Buffer, values: &[f32])` is gone; remove its pin in the same reviewed change that removes the permission
```

**Removal**, by deleting `read_f32`'s direct permission:

```text
unsafe-sites.prototypes/serial-sum-run/src/buffer.rs: pinned site `pub fn read_f32(buffer: &Buffer, count: usize) -> Vec<f32>` is gone; remove its pin in the same reviewed change that removes the permission
```

**Reason change**, by changing the first clause of `read_f32`'s reason:

```text
unsafe-sites.prototypes/serial-sum-run/src/buffer.rs: `pub fn read_f32(buffer: &Buffer, count: usize) -> Vec<f32>` states reason "the changed read half of the same constraint: MTLBuffer storage is reachable only through `Buffer::contents`. Bounded by an asserted length check, reads a plain-old-data type, and copies out rather than retaining a borrow of device memory.", pinned as "the read half of the same constraint: MTLBuffer storage is reachable only through `Buffer::contents`. Bounded by an asserted length check, reads a plain-old-data type, and copies out rather than retaining a borrow of device memory."
```

The added mapped scopes reflect the paths the audited implementation actually
needed: `implementation/frontend` owns the facade integration test,
`implementation/conformance` owns removal of the superseded local test and its
references, `implementation/runtime` owns the prototype manifest correction,
`contracts/foundation` owns the architecture correction, and
`implementation/cargo-lock` owns the direct test-dependency edge recorded in
the lockfile. The original `implementation/workspace` and
`contracts/decisions` scopes own the root lint comment and ADR closure.
`contracts/navigation` remains declared from the claimed ticket, but the full
navigation read found no catalog row whose current meaning changed, so no
navigation file was edited.

### Review amendment, 2026-08-08

Independent review of `1baf7cdf` found two population escapes and two identity
gaps. The original literal-member/`.rs` walk was correct for the current
422/16/4 census but not authoritative over what Cargo can compile: an implicit
in-tree path member, non-`.rs` target, literal or generated include, and
`#[path]` source could all carry an unseen permission. A `macro_rules!`
template invoked twice was one lexical pin but two compiled permissions, and a
same-signature/reason function moved beneath an inline module retained the old
path/signature/reason key. The three conformance-manifest sentences saying
every other member inherited and no workspace check existed were stale too.

The amendment uses `cargo metadata --locked --no-deps`, already used by
`workspace_population.rs`, because it is the only current source-truthful
authority for Cargo's actual workspace membership and target roots. It resolves
manifests but builds or runs no target, so invoking it from this integration
test is not recursive. `Cargo.lock` cannot substitute: it records resolved
packages, not which are workspace members or which source path each target
compiles. The explicit roots and metadata roots must agree exactly; every
target must be unique, readable, and canonically inside its owning package.

Literal local `include!` and source-file-root `#[path]` files are followed inside the
governed package roots. The visited set terminates cycles and canonicalizes
aliases; any permission found through those forms is refused because the same
lexical file may be loaded more than once. Computed/`OUT_DIR` includes, nested
`#[path]`, target roots outside their owner, and visible permissions within
`macro_rules!` or another token-generating invocation fail closed. Current
permission pins admit source-file-root functions only; their file paths carry
the current out-of-line module identity. Deeper inline module/impl/function
sites are rejected until a reviewed semantic-path identity is designed. This
narrows the scanner's supported identity, not ADR 0079's policy, and admits no
new site.

Each escape was applied to the live prototype, compiled successfully, refused
by the inventory with exit 101, and restored:

```text
unsafe-sites.prototypes/serial-sum-run/src/hidden.inc: `pub(super) fn hidden_included_site() -> u8` carries a permission in a source reached through include!/#[path] (["prototypes/serial-sum-run/src/buffer.rs:25 via include!", "prototypes/serial-sum-run/src/buffer.rs:26 via include!"]); nonstandard loads can duplicate semantic sites and are outside the file-root pin boundary
unsafe-sites.prototypes/serial-sum-run/src/hidden.inc: `pub(super) fn hidden_path_site() -> u8` carries a permission in a source reached through include!/#[path] (["prototypes/serial-sum-run/src/buffer.rs:25 via #[path]"]); nonstandard loads can duplicate semantic sites and are outside the file-root pin boundary
unsafe-site census: explicit root members and cargo metadata workspace packages differ; implicit/metadata-only: ["prototypes/serial-sum-run/hidden-member"]; explicit-only: []
unsafe-sites.prototypes/serial-sum-run/tests/hidden.inc: `fn hidden_non_rs_target_site() -> u8` admits unsafe_code and is not pinned; ADR 0079 makes a new site a new decision
unsafe-sites.prototypes/serial-sum-run/src/buffer.rs:25: computed include! is unsupported; generated or OUT_DIR sources cannot be inventoried
unsafe-site census: Cargo target root /Users/tsanterre/workspace/github.com/moderately-ai/.worktrees/tiler/pin-the-admitted-unsafe-sites-in-the-workspace-gate/edit/hidden-outside.inc escapes owning package /Users/tsanterre/workspace/github.com/moderately-ai/.worktrees/tiler/pin-the-admitted-unsafe-sites-in-the-workspace-gate/edit/prototypes/serial-sum-run
unsafe-sites.prototypes/serial-sum-run/src/buffer.rs:29: unsafe-code permission appears inside a token-generating macro context; expansion multiplicity has no admitted pin identity
unsafe-sites.prototypes/serial-sum-run/src/buffer.rs:38: nested permission is outside the file-root pin boundary; module, impl, and function semantic paths are unsupported
```

The include probe loaded the same file from two modules, the macro probe
invoked one template in two modules, the implicit member appeared as the
seventeenth `workspace_members` entry, the non-`.rs` target appeared as the
64th target, the generated probe used a real temporary build script and
`OUT_DIR`, and the semantic-move probe re-exported the nested function under
its original public name.

The restored focused scanner run passed all eleven tests and printed the clean
422-source/63-target/16-package/four-pin census. The final `make full` passed:
workspace check and Clippy with warnings denied, 3,240 nextest tests with eight
skipped, workspace doc-tests, rustdoc with warnings denied, 1,116 release
numerical tests with three skipped, ticket lint, and shellcheck. The separate
citation gate resolved 932 pinned citations and 6,219 local links.

### Final review Fact repair, 2026-08-08

Independent review of `bce4862b` found that the amended source census was
complete over lexical files but still incomplete over code rustc compiles. The
ticket's purpose is unchanged: each probe compiled an unsafe operation without
changing one of the four admitted path/signature/reason pins, which is exactly
the unreviewed population change this workspace-wide ADR 0079 inventory exists
to refuse. No probe supplied evidence for admitting another site or changing
the policy.

- **Doc comments are prose — imprecise.** Ordinary mentions remain excluded,
  but rustdoc extracts Rust code blocks as separate crates. A reasoned allow
  plus unsafe block in `crates/tiler-conformance/src/lib.rs` passed
  `cargo test --locked -p tiler-conformance --doc`, while the lexical inventory
  passed at 422/63/16/4. Removing the allow still compiled: Cargo does not carry
  the member's `unsafe_code = "deny"` into an extracted doctest. Neither
  `RUSTFLAGS=--forbid unsafe_code` nor the equivalent `RUSTDOCFLAGS` closed that
  boundary. The rustdoc-supported crate attribute
  `#![doc(test(attr(forbid(unsafe_code))))]` did: the reasoned probe failed with
  `allow(unsafe_code) incompatible with previous forbid` and the unsafe block
  failed separately. Cargo metadata derives thirteen current `doctest: true`
  library/proc-macro roots; all thirteen need that sentinel and a counted check.
- **Visible macro tokens bound expansion — false.** A local workspace proc macro
  emitted a complete reasoned allow and unsafe function from a token-free call
  site. The prototype compiled and the inventory passed at 422/63/16/4. Source
  inspection cannot derive an external procedural macro's output. A bounded
  experiment corrected the first proposed repair: rustc suppresses unsafe-code
  diagnostics from that expansion even under `--force-warn=unsafe-code`, so the
  JSON census remains useful for visible expanded operations and target
  multiplicity but cannot close macros. `-Zunpretty=expanded` preserved the
  unsafe block, reason, and item but lost stable source attribution, while the
  HIR debug form retaining spans was 666,114 unstable lines for one target. The
  maintainable fail-closed repair is therefore a closed macro/attribute language
  in the two reopenable packages, not an expanded-output parser.
- **Literal `include!` coverage — false for macro name resolution.** Rust accepts
  `use std::include as imported_include; imported_include!("hidden.inc");`.
  The hidden reasoned permission compiled and the inventory passed unchanged,
  because the lexer recognizes only the literal invocation name. The compiler
  census sees the resulting operation; the lexical check must also reject an
  imported or aliased `include` spelling rather than claim to follow it.
- **Literal source-load resolution — imprecise for a non-`mod.rs` module root.**
  Rustc's `#[path]` module-directory rules depend on whether the containing
  source is a target/`mod.rs` root and on semantic module context. The live
  top-level probe in `outer.rs` resolved beside that file and compiled, but the
  lexical scanner has no semantic module identity with which to state the full
  rule. The supported boundary therefore rejects `#[path]` in a non-`mod.rs`
  non-target module source rather than claiming a universal base.
- **Unique metadata target roots imply unique compiled identity — false.** The
  admitted `buffer.rs` was compiled both as `mod buffer` under `main.rs` and as
  a second Cargo binary target. `cargo check --bins` succeeded; the inventory
  merely reported 64 unique targets and still passed four pins, although both
  prototype permissions now had two compiled identities. The compiler census
  must pin multiplicity per package, target, and source identity rather than
  deduplicating the lexical file.
- **The prototype and ADR 0106 current lint/inventory prose — false or
  imprecise.** `prototypes/serial-sum-run/Cargo.toml` still says every other
  member keeps `forbid`, although conformance is the second `deny` exception.
  ADR 0106's dated/current transition retains the same stale partition and its
  2026-08-08 update overstates the lexical scanner's expansion boundary. Both
  require repair alongside the mechanism they describe.

The added scopes are the exact metadata target-root population that must carry
the rustdoc sentinel: the existing frontend, conformance, and runtime scopes,
plus `implementation/digest`, `implementation/ir`,
`implementation/reference`, `implementation/artifact`,
`implementation/compiler`, `implementation/metal`,
`implementation/metal-aot`, `implementation/cache`, and
`implementation/build`. The existing workspace, Cargo-lock, decision,
foundation, navigation, and shared ticket declarations remain; navigation has
no expected edit, but was part of the claimed ticket and remains declared.

### Final closure amendment, 2026-08-08

The purpose remains unchanged and no fifth site is admitted. Cargo metadata now
includes the resolved dependency graph so a direct proc-macro package edge from
either `unsafe_code = "deny"` package fails before source scanning. In those two
packages the lexical pass rejects every `macro_rules!` definition, custom or
path-qualified macro invocation, glob import, import/alias shadowing an invoked
built-in macro name, custom/path-qualified attribute, and custom/path-qualified
derive. Nested `cfg_attr` entries are parsed recursively. Only the exact sixteen
compiler/std macros, ten built-in attributes, and five built-in derives in
the current source population are classified. This preserves ADR 0079 across
macro-generated code by refusing source forms whose expansion identity cannot
be pinned; it does not declare those operations out of scope.

The compiler JSON census remains independently load-bearing for active ordinary
and test targets. Its clean population is six diagnostics: two at
`tiler-conformance`/`tiler_conformance`/`src/lib.rs` from `device_buffer.rs`, and
four at `tiler-prototype-run`/`tiler-prototype-run`/`src/main.rs` from
`buffer.rs`, reflecting Cargo's normal and test compilation forms. The lexical
census remains 422 source files, 63 targets, 13 doctest roots, 16 packages, and
four exact pins. Every metadata `doctest: true` library/proc-macro root carries
the exact `#![doc(test(attr(forbid(unsafe_code))))]` sentinel, against a floor of
twelve roots.

The final independent subjects compiled first, then failed the focused gate,
and were restored:

```text
raw doctest: error: usage of an `unsafe` block; note: the lint level is defined here: #![forbid(unsafe_code)]
sentinel removal: doctest-enabled Cargo target must contain exactly one `#![doc(test(attr(forbid(unsafe_code))))]` crate-root sentinel; found 0
nested sentinel move: doctest unsafe sentinel must be in the Cargo target's crate-level attribute population; a nested module attribute does not govern every extracted doctest crate; found 0
include alias: the include macro name appears outside direct `include!(literal)` syntax; imported or aliased include forms have no lexical source identity
include alias compiler census: 8 compiler diagnostic(s) ... hidden.inc: 2 (pinned population is 6)
non-mod.rs path: #[path] in a non-mod.rs module source is unsupported; rustc's module-directory rules depend on semantic context that this lexical inventory refuses to guess
duplicate target/module: 10 compiler diagnostic(s) ... (`buffer-duplicate-probe`, `src/buffer.rs`, `src/buffer.rs`): 4 (pinned population is 6)
token-free and imported proc macro, and proc attribute: reopenable package `tiler-prototype-run` directly depends on proc-macro package `tiler-macros`; external macro expansion suppresses unsafe-code diagnostics and is outside the admitted source-language boundary
path-qualified macro: path-qualified macro invocation `format!` is unsupported in an unsafe-code-reopenable package
macro import shadow: use declaration binds `panic`, which is invoked as an admitted unqualified built-in macro name; macro imports and shadowing are unsupported
macro_rules shadow: macro_rules! definitions are unsupported in an unsafe-code-reopenable package; generated permissions have no lexical site identity
glob import: glob use is unsupported in an unsafe-code-reopenable package because it can import an untracked macro name
```

The restored final tree passed fresh `make full`: citation resolution at
932/6,219, formatting, workspace check, the repository's Clippy population with
warnings denied, 3,243 nextest tests with eight skipped, all workspace doctests,
rustdoc with warnings denied, 1,116 release numerical tests with three skipped,
ticket lint, and shellcheck. The earlier raw `cargo clippy --workspace
--all-targets -D warnings` additionally surfaced four pre-existing prototype
style lints; the repository Clippy gate deliberately excludes all three
prototype packages, and the targeted command over every touched `crates/`
package passed with warnings denied.

### Post-closure coordinator Fact repair, 2026-08-08

The compiler boundary stated above is necessary but still insufficient for a
workspace-wide claim. A workspace-owned procedural macro invoked from the
ordinary `tiler` library emitted raw unsafe code from a token-free call site.
Inherited `unsafe_code = "forbid"` rejected the same expansion when it also
emitted `#[allow(unsafe_code)]`, but accepted the raw unsafe operation when that
attribute was removed. The focused inventory then passed unchanged at
422/63/13/16/four lexical pins and six deny-package compiler diagnostics. Rustc
therefore suppresses the raw operation's unsafe-code diagnostic for an external
procedural-macro expansion even in a member whose command-line level is
`forbid`; closing only the two locally reopenable `deny` packages was false.

The purpose is unchanged and no new site is admitted. The repair must cover
all sixteen member source trees while preserving the four lexical pins, the two
deny-package compiler census, Cargo/source-load closure, and the thirteen
doctest sentinels. Direct dependency edges are only an early diagnostic: a
normal dependency can re-export a transitive procedural macro, so an edge census
cannot establish the source invocation boundary.

The re-derived current source population contains no third-party custom derive,
attribute, or function-like procedural-macro invocation. Its only procedural
macro source form is the workspace-owned `tensor!`: 73 syntactic invocations
across the nineteen pass/fail fixture files plus one facade crate doctest,
using the exact `tiler::tensor!` qualified spelling or the facade's directly
imported `tensor!` re-export. Separately, the member trees contain fifteen private,
non-exported `macro_rules!` definitions. Compiler/std macro invocations,
built-in attributes, and built-in derives form finite exact populations that
the all-member scan must derive and pin; a future custom or re-exported macro,
attribute, or derive invocation fails closed.

That all-member population includes rustdoc-extracted Rust fences, including
their hidden `#` lines, from every source reachable beneath the thirteen
doctest-enabled target roots. The ordinary lexer intentionally discards doc
comments, but the extracted crates are compiler input and an external macro can
suppress unsafe diagnostics there despite the injected `forbid` sentinel just
as it can in an ordinary target. The inventory must therefore parse their
function-like macro, attribute, and derive vocabulary separately. Dynamic
`#[doc = include_str!(...)]` and any other documentation source the scanner
cannot recursively enumerate fail closed rather than becoming invisible code.
The nineteen trybuild fixtures are ordinary `.rs` sources and remain in the
normal member walk; the all-source census pins their 73 `tensor!` forms plus
the facade crate doctest separately, and every invocation's emitted tokens
cross the producer guard.

The sole workspace procedural-macro exporter is exactly
`#[proc_macro] pub fn tensor`, and `crates/tiler/src/lib.rs` carries its sole
facade re-export, `pub use tiler_macros::tensor`. Both success and refusal token
streams must pass one recursive producer guard before return. The guard refuses
emitted `unsafe` or `unsafe_code`, any emitted macro invocation other than exact
absolute `::core::compile_error!`, and any emitted attribute other
than exact `#[cfg(...)]`. A second function-like exporter, attribute exporter,
derive exporter, public/exported local declarative macro, dynamic macro-name
emission, or unguarded return is an inventory failure. This closes the confirmed
escape without adopting unstable expanded/HIR parsing and without treating
macro-generated unsafe as outside ADR 0079.

### Completed all-member source-authority closure, 2026-08-08

The purpose remains ADR 0079's workspace-wide inventory and no fifth unsafe
site is admitted. The finished split integration target reports 426 source
files, 63 Cargo targets, 13 doctest roots, 16 packages, 73 fixture `tensor!`
invocations, one rustdoc invocation, and the same four exact lexical pins. Its
closed current vocabularies are 23 compiler/std function-like macros, 19
built-in/tool attributes, nine built-in derives, fifteen private non-exported
local `macro_rules!` definitions, one guarded procedural exporter, and one
facade re-export. The compiler JSON census remains the separate two-`deny`
package authority at six diagnostics over three metadata targets and two
target/source identities.

The fixture count is source-classified rather than a grep count. The command
`rg -o --glob '*.rs' 'tensor!' crates/tiler/tests/facade/pass crates/tiler/tests/facade/fail | wc -l`
prints 78 textual spellings. Five are not syntax: two unqualified and two
qualified spellings are in `//!`/`///` prose (the shortest anchors are
`A consumer reaches \`tensor!\`` and `let d = tiler::tensor!`), and a third
qualified spelling is inside the literal beginning
`::core::compile_error!("\`tiler::tensor!\` could not compile`. The lexer drops
those five and reports 72 qualified syntactic calls plus the imported call at
`let imported = tensor!`, for 73. The separate rustdoc extractor reports the
facade doctest as one rather than letting either population mask the other.

The producer now routes both `Ok(expanded)` and the refusal diagnostic through
one recursive guard. Its own structural census requires exactly one final
`guarded_emission` call and rejects an explicit early return. The guard rejects
`unsafe`, `unsafe_code`, `use`, `extern`, `mod`, `macro`, and `macro_rules`;
attributes other than exact `#[cfg(...)]`; and macro invocations other than
exact absolute `::core::compile_error!`. Its refusal is itself emitted through
that absolute diagnostic and revalidated before return. A second
`#[proc_macro]`, any proc attribute/derive exporter, a second facade re-export,
or an unguarded alternate tensor return is a population failure.

The all-member lexical authority rejects custom/path-qualified macro calls,
custom attributes and derives, `extern crate`, external globs, admitted-name
shadowing (including built-in attribute/derive names), workspace-local macro
exports/re-exports even under aliases, macro-2.0 definitions, additional
`macro_rules!` definitions, and dynamic `$name!` or `#[$name]` emission. Direct
proc-macro edges from the two `deny` packages are only an early diagnostic;
transitive and re-exported authorities are closed at their workspace source
invocations. A dependency-authored macro merely re-exported but never invoked
by workspace source is outside this compiled-workspace inventory and remains a
public-boundary review concern. Compiler-builtin generated implementation
details and dependency internals are likewise outside the workspace-authored
ADR 0079 site population.

Rustdoc scanning covers line and nested block documentation while respecting
the Rust lexer state, literal `#[doc]` strings, hidden `#` lines, indented and
tab-indented code, and the current literal macro-generated documentation.
Unsupported blockquote/list containers, dynamic documentation sources, and
doctest-local `include!`, `#[path]`, or out-of-line `mod` loads fail closed.
The masking subject `const S: &str = "/*"; /** ... */` reaches the later real
block doc rather than turning the cooked string into a false comment opener.

Independent subject changes compiled where the compiler accepts the source and
then made the focused gate refuse with these diagnostics (each was restored):

```text
successful producer raw unsafe: identifier `unsafe` is outside the unsafe-free emitted vocabulary
successful producer lint attribute: emitted attribute `#[allow(unsafe_code)]` is outside the exact `#[cfg(...)]` vocabulary
refusal producer raw unsafe: identifier `unsafe` is outside the unsafe-free emitted vocabulary
unapproved producer macro: emitted macro invocation `format!` is outside the exact absolute `::core::compile_error!` diagnostic
unapproved producer attribute: emitted attribute `#[doc = "probe"]` is outside the exact `#[cfg(...)]` vocabulary
bare diagnostic macro: emitted macro invocation `compile_error!` is outside the exact absolute `::core::compile_error!` diagnostic
prefixed diagnostic path: emitted macro invocation `compile_error!` is outside the exact absolute `::core::compile_error!` diagnostic
producer namespace/load forms: identifier `use` / `extern` / `mod` would emit a source-loading or namespace authority
producer macro-2.0: identifier `macro` would emit a source-loading or namespace authority
second proc function / proc attribute / proc derive: unguarded procedural-macro exporter; exact population is `#[proc_macro] pub fn tensor`
path and imported proc invocation: path-qualified/custom macro invocation `unsafe_probe!` is unsupported
exported external macro_rules: custom attribute `macro_export`; unpinned macro_rules definition; path-qualified macro invocation
dynamic pinned-template names: dynamic macro invocation name `$macro_name!`; dynamic attribute name emission is unsupported
local macro alias: use declaration imports or re-exports pinned local macro name `emit`
extern alias: extern crate declarations and aliases are unsupported
block-doc custom macro: `<rustdoc:2>` path-qualified macro invocation `emit!` is unsupported
list-container doctest: rustdoc fence marker in an unsupported container or position
doctest include of raw unsafe: include! is unsupported in an extracted doctest
fixture addition / removal: guarded fixture tensor invocation census changed; found 74 / 72, expected 73
raw unsafe doctest without sentinel compiled; inventory: doctest-enabled Cargo target ... sentinel; found 0
```

The earlier direct-site addition, move, removal, reason-change, wrapped-attribute,
macro-template-multiplicity, implicit member, include alias, non-`mod.rs`
`#[path]`, duplicate target/module, and compiler-census perturbations remain
independent and green after the support-module split. The implementation is
partitioned under `workspace_unsafe_sites_support/` into metadata/target graph,
compiler census, macro/rustdoc boundary, and Rust syntax/source-load modules;
`workspace_unsafe_sites.rs` remains the single integration-test entry point.

The completed tree passed the focused inventory's eighteen tests with the
censuses above, followed by a fresh `make full`: 932 pinned citations and 6,219
local links resolved; formatting, workspace check, Clippy, and rustdoc with
warnings denied passed; 3,247 nextest tests passed with eight skipped; every
workspace doctest passed; 1,116 release tests passed with three skipped; and
`ticketsplease lint` plus shellcheck passed. No subject perturbation remains in
the finished diff.

**Authoritative final result — 2026-08-08.** The census holds 426 workspace
source files, 63 Cargo targets, 13 doctest roots, 16 packages, the 73 fixture
plus one rustdoc `tensor!` invocation counts, and the unchanged four exact
unsafe-site pins. Its all-member lexical and rustdoc boundary closes
workspace-authored and source-generating macro authorities and invocations;
both the successful and refusal output streams from the sole procedural
producer separately cross the recursive token guard, whose only admitted macro
output is exact absolute `::core::compile_error!`. The compiler JSON census is
deliberately the complementary two-`deny`-package authority. Compiler-builtin
generated implementation details, dependency-internal expansions, and
dependency-authored macros merely re-exported but not invoked by workspace
source are excluded rather than falsely claimed as enumerated ADR 0079 sites.

### Post-commit namespace-ownership Fact repair, 2026-08-08

- **False — an admitted qualified spelling did not prove its namespace owner.**
  At `7b679f6d`, the source mutation `use evil as tiler; tiler::tensor! {}`
  produced no macro-language error. `is_exact_tensor_path` classified only the
  token spelling, while the guarded import bindings contained `tensor` but not
  its owning namespace `tiler`. This is the same identity-spoof class as the
  already rejected `extern crate evil as tiler` form, and it makes the claimed
  all-member source-authority closure false.
- **Purpose unchanged.** No site or macro authority is being admitted. The
  repair must make every namespace used by an admitted qualified macro spelling
  unshadowable through `use` or `extern crate`, inspect the corresponding
  `core`/`std` spellings, perturb each live subject independently, and retain
  the existing four-site, 73-plus-one invocation, and producer-output pins.

- **False — the 73-plus-one count was not yet an identity pin.** The 73
  ordinary calls occur in thirteen of the nineteen facade fixture files, not
  in all nineteen as the earlier wording could be read to say. Only their
  aggregate count was asserted, so moving one call between source files kept
  the gate green. The one rustdoc call likewise needs its extracted source/block
  identity, not only a separate total.
- **False — forwarded documentation was called enumerable without being
  reconstructed.** In a pinned `draft_handle` template, fragmenting a Rust
  fence over three literal metavariables inside `#[doc = concat!(...)]` made
  `doc_attribute_markdown` return an empty document even though rustdoc would
  assemble the code block. The existing direct `$docs`/`$limit_doc` templates
  need no general macro expander: the sound narrow boundary is a directly
  forwarded literal as the entire doc expression, literal/stringify-only
  composition, and failure for forwarded values inside composition.

### Completed namespace and rustdoc identity repair, 2026-08-08

The source-language check now rejects `use` bindings of `tiler`,
`tiler_macros`, `std`, and `core`; `extern crate` remains rejected wholesale.
Cargo metadata separately requires the facade binding named `tiler_macros` to
resolve to the one workspace `tiler-macros` package. The invocation census is
an exact thirteen-entry fixture path/multiplicity map totaling 73 and the exact
`crates/tiler/src/lib.rs<rustdoc:1>` identity at one. A count-preserving move
failed with:

```text
unsafe-site guarded fixture tensor invocation identities changed: crates/tiler/tests/facade/fail/generated_operand_reference_spans.rs: found absent, expected 1; crates/tiler/tests/facade/fail/region_syntax_diagnostics.rs: found 10, expected 9
```

Compiled source-alias probes then failed the inventory with `use declaration
binds guarded macro namespace` for `tiler`, `tiler_macros`, `core`, and `std`.
A compiled facade dependency rename failed earlier in metadata with:

```text
unsafe-site tensor facade dependency identity changed: Cargo binding `tiler_macros` resolves to [], expected workspace producer [.../crates/tiler-macros#0.0.0]
```

Finally, pinned macro invocations feed every literal argument into the rustdoc
Markdown classifier. A literal metavariable is admitted only when it is the
whole `#[doc = $name]` expression; the permanent three-fragment
`concat!($a, $b, $c)` fence subject fails as `dynamic documentation source is
unsupported`. All probes were restored, and the focused eighteen-test suite
returned green at 426/63/13/16, the exact 73-plus-one identities, and the same
four unsafe pins. This later completion supersedes the preceding authoritative
result only on namespace, invocation-location, and generated-document identity;
its policy and exclusions are unchanged.

**Authoritative amended result — 2026-08-08.** The exact 426-source,
63-target, 13-doctest-root, 16-package, four-site population is unchanged. The
thirteen-entry fixture identity map holds 73 calls and the exact rustdoc block
holds one; source aliases and Cargo dependency renames cannot change their
producer silently, and forwarded documentation cannot compose hidden compiler
input. The fresh post-repair `make full` passed 932 citations, 6,219 local
links, formatting, workspace check, Clippy, 3,247 nextest tests with eight
skipped, every workspace doctest, rustdoc with warnings denied, 1,116 release
tests with three skipped, ticket lint, and shellcheck. Compiler-builtin and
dependency-internal generated details remain explicitly excluded.

### Post-amendment emitted-diagnostic identity Fact repair, 2026-08-08

- **False — leading `::` did not make the emitted diagnostic macro
  unshadowable.** At `b825b181`, the producer guard admitted
  `::core::compile_error!`, but Rust's leading path segment is an extern-prelude
  crate binding. A downstream Cargo dependency renamed to `core` can therefore
  supply a different exported `compile_error!` macro without any source-level
  `use` or `extern crate` for the all-member lexer to reject. The producer's
  exact punctuation check cannot establish the package identity behind `core`.
- **Purpose unchanged.** The repair must remove this downstream namespace
  authority from both unconditional refusals and successful streams' gated
  diagnostics. It must retain a deterministic compile failure and useful span,
  admit only the exact facade-owned compiler-builtin diagnostic invocation,
  and admit no new unsafe site or procedural/local declarative macro producer.
- **False — forwarded-literal proof was macro-global rather than arm-local.**
  At `b825b181`, one `($docs:literal) => {}` arm could make a separate
  `($docs:expr) => { #[doc = $docs] ... }` arm appear literal because
  `enumerable_macro_doc_expression` searched every token in the file for a
  same-named literal binder. Invoking the expression arm with
  `include_str!("hidden.md")` returned `Ok(["", "hidden.md"])` instead of
  refusing the dynamic documentation source. Literal forwarding must be
  established within the exact macro arm that emits the doc attribute.
- **False — cooked doc strings were scanned before Rust escape decoding.** A
  direct `#[doc = "\\x60\\x60\\x60rust\\nevil::emit!();..." ]` retained the
  backslash spellings in `TokenKind::StringLiteral`, so the Markdown extractor
  saw neither the fence nor the custom macro rustdoc would compile. Direct and
  forwarded doc literals must be cooked by the admitted Rust string-escape
  grammar before Markdown classification, with unknown escapes refused.
- **False — raw identifiers did not share their semantic name.** The lexer kept
  `r#tiler` distinct from `tiler`, so `use evil as r#tiler; tiler::tensor! {}`
  compiled and crossed the admitted spelling check without a namespace error.
  Every guarded macro, attribute, derive, import, and namespace comparison must
  normalize raw identifiers while still treating a raw keyword as an
  identifier rather than control syntax.
- **False — the final diagnostic spelling was checked as a suffix, not an exact
  absolute path.** Both producer and source classifiers admitted
  `evil::tiler::__private::__tiler_compile_error!` because they matched the
  immediate `::tiler::__private::__tiler_compile_error!` suffix without proving
  its first `::` began at a token-tree or classified statement/attribute
  boundary. The identity check must reject a namespace prefix while retaining
  an exact path after a separate statement or admitted `#[cfg]`.

### Completed emitted-diagnostic and documentation-identity repair, 2026-08-08

The accepted ADR 0053/frontend contract ruled out the provisional
`const _: () = "diagnostic"` type-mismatch direction: it requires both
unconditional and family-gated refusals to preserve the actionable
`compile_error!` message, and the frontend toolchain contract does not authorize
the unstable procedural-macro diagnostic API. Tom's 2026-08-08 coordination
instruction delegated acceptance of the correctness-dominant repair to the
coordinator. After those alternatives and a caller-resolved declarative wrapper
were disproved, the coordinator selected one narrow addition to ADR 0088's
existing doc-hidden generated-code namespace.
`tiler::__private` now re-exports compiler `core::compile_error` as
`__tiler_compile_error`; both producer streams emit only the exact
`::tiler::__private::__tiler_compile_error!(<one literal>)` invocation. The
producer guard checks the absolute path, joined `::` punctuation, parenthesized
single-literal argument, and a closed token-tree/statement/`cfg` start boundary;
it still rejects every other macro, unsupported
attribute, unsafe spelling, or namespace/source-loading authority. No second
procedural exporter or local declarative macro was added.

The facade's own Cargo namespace is now part of the identity proof. Metadata
requires its `tiler_macros` binding to resolve to the workspace producer and
rejects a facade dependency binding named `core` or `std`, while consumer
bindings of those names are irrelevant because the builtin was resolved during
facade compilation. With a temporary no-op proc macro exported from a consumer
dependency renamed to `core`, the actual unconditional tensor refusal remained:

```text
error: `tiler::tensor!` was given no region; a region declares its operands and its result ...
 --> src/main.rs:2:13
```

The same consumer's matching family-gated subject failed with the exact retained
text, while changing only the `#[cfg]` to a nonmatching family compiled:

```text
error: retained diagnostic survives consumer core rebinding
 --> src/main.rs:3:5
```

A temporary facade dependency alias produced the inventory's own early refusal:

```text
unsafe-site compiler diagnostic identity changed: facade Cargo dependency binding(s) ["core"] shadow compiler namespaces used by the exact `core::compile_error` re-export
```

The documentation repair is likewise fail-closed rather than an approximate
macro expander. Raw identifiers normalize before guarded comparisons. Cooked
doc literals decode the admitted Rust escapes before Markdown extraction, so a
`\x60\x60\x60rust` fence reaches and rejects its custom macro. A forwarded
literal is proven against the exact matcher whose expansion contains the doc
attribute; the permanent semicolon- and comma-separated cross-arm subjects both
refuse the `expr` arm's `include_str!` as `dynamic documentation source is
unsupported`, regardless of nesting. Unknown escapes and forwarded composition
remain refusals.

The exact population remains 426 source files, 63 Cargo targets, thirteen
doctest roots, sixteen packages, four admitted unsafe sites, the thirteen-file
73-call fixture map, and one exact rustdoc call. The macro vocabulary still has
23 compiler/std forms because the compiler diagnostic call remains one current
source form; its identity is now held by the separately pinned facade builtin
re-export. Compiler-builtin generated implementation details, dependency
internals, and dependency-authored macros merely re-exported but not invoked by
workspace source remain excluded.

Final verification ran after the exact-path repair and after every temporary
subject was restored. The focused macro suite passed 181 tests with one ignored
cross-target matrix; the workspace inventory passed all eighteen tests and
printed the 426-source, 63-target, thirteen-doctest-root, sixteen-package,
73-fixture-plus-one-rustdoc, four-site census above; the facade suite passed
both its complete trybuild contract and downstream `core` rebinding subject.
The prefixed producer mutation failed as
`emitted macro invocation '__tiler_compile_error!' is outside the exact
facade-owned '::tiler::__private::__tiler_compile_error!' diagnostic`, and the
permanent source subject rejects that same longer path as path-qualified. Fresh
`make full` then resolved 932 pinned citations and 6,219 local links, ran
workspace check and Clippy, passed 3,248 nextest tests with eight skipped, every
workspace doctest, workspace rustdoc with warnings denied, 1,116 release tests
with three skipped, ticket lint, and shellcheck.

### Post-review Unicode and generated-document Fact repair, 2026-08-08

- **False — the lexer did not recognize every Rust identifier.**
  `is_ident_start`/`is_ident_continue` used Unicode alphabetic/alphanumeric
  categories rather than Rust's XID start/continue grammar. The valid facade
  subject `use tiler::tensor as ℘; ℘! {}` compiled, but the pre-repair scan
  reported neither an invocation nor an error. Identifier identity must follow
  Rust outside comments and string literals; ASCII-only scanning or global
  Unicode refusal would reject ordinary documentation rather than close the
  source language.
- **False — `stringify!` documentation was called enumerable without
  reconstructing it.** The doc-expression classifier admitted a metavariable
  merely because `$name` appeared immediately inside `stringify!(...)`; it did
  not prove that the exact arm bound `$name:ident`. Even with an `ident`
  matcher, the Markdown reconstruction concatenated only static string
  literals, so `concat!("```rust\\nstd::", stringify!($name),
  "!();\\n```")` hid the macro name from the scan while rustdoc compiled the
  generated `std::println!()` example. The exact probe passed one doctest.
- **False — a forwarded raw doc literal was treated as no documentation.** The
  lexer deliberately represented a raw string only as `<raw-string>`, while a
  pinned `($docs:literal) => { #[doc = $docs] ... }` invocation admitted that
  opaque token and reconstructed an empty document. Raw strings must either be
  decoded exactly or refused anywhere they can supply generated rustdoc.
- **False — nested documentation metadata was invisible.**
  `#[cfg_attr(doc, doc = "```rust ... ```")]` is compiler input when rustdoc
  sets `cfg(doc)`, but the Markdown reader considered only attributes whose
  first meta name was directly `doc`. The macro-language classifier's admitted
  `cfg_attr` did not extract the nested documentation and the focused scan
  returned no error. Documentation metadata nested through one or more
  `cfg_attr` layers must be enumerated or refused recursively.
- **False — the private macro producer set discarded multiplicity.** Two
  same-named `macro_rules!` definitions in one pinned file are valid Rust
  shadowing, but insertion into `BTreeSet<(path, name)>` deduplicated them. A
  second `draft_handle` in its admitted file therefore inherited the first
  definition's producer authority. Duplicate insertion must be an immediate
  refusal even though the final identity set still equals the pin.
- **Purpose unchanged.** The first four findings let workspace-authored compiler
  input disappear from the source-language census. Closing them changes no
  unsafe admission, producer, public API, or ADR 0079 policy; the fifth prevents
  an additional producer from inheriting an existing identity. Together they
  make the existing workspace-wide claim truthful.

### Completed Unicode, generated-document, and producer-multiplicity repair, 2026-08-08

The lexer now uses the already locked `unicode-ident` crate's Rust XID
start/continue predicates outside comments and string literals. The compiled
facade alias subject failed the restored inventory with
``custom macro invocation `℘!` is unsupported in the workspace``; the exact
fixture identity simultaneously fell from three invocations to two, proving
both the lexical refusal and the per-source population pin reached it.

Generated documentation remains a deliberately bounded reader, not a macro or
Markdown expander. `stringify!($name)` requires the exact emitting arm to bind
`$name:ident`; if its static literal composition contains a rustdoc code block,
the gate refuses it because the invocation value is not reconstructed. A real
temporary crate compiled and ran the generated `std::println!()` doctest before
the live template mutation failed with:

```text
unsafe-sites.crates/tiler-ir/src/index/handles.rs:26: stringify-composed rustdoc code is unsupported; stringified invocation values are not reconstructed
```

The existing `draft_handle` use stays admitted because its stringify is prose
only. Raw literals passed to a pinned local macro fail as
`raw-string macro argument is unsupported for a pinned documentation-generating
macro`, rather than becoming empty Markdown. Documentation nested through
`cfg_attr`, including another nested `cfg_attr`, fails recursively as
`documentation nested in cfg_attr is unsupported`; an exact temporary
`cfg_attr(doc, doc = "...")` subject produced and ran its rustdoc test before
the live source mutation was refused. Finally, a second same-path/same-name
`draft_handle` definition compiled as valid Rust shadowing and then failed the
inventory as ``duplicate pinned macro_rules! definition `draft_handle` ``. Every
temporary subject was restored.

The final focused checks passed formatting, all-target `tiler` check and Clippy
with warnings denied, rustdoc with warnings denied, the complete facade suite,
and all eighteen workspace unsafe-site tests. The exact population remains 426
source files, 63 Cargo targets, thirteen doctest roots, sixteen packages,
thirteen fixture identities holding 73 tensor invocations, one exact rustdoc
invocation, six expanded diagnostics, fifteen private local macro definitions,
and four admitted unsafe sites. Fresh `make full` then resolved 932 citations
and 6,219 local links, passed workspace check and Clippy, 3,248 nextest tests
with eight skipped, every workspace doctest, rustdoc with warnings denied,
1,116 release tests with three skipped, ticket lint, and shellcheck. The
workspace-authored/source-generating boundary and its compiler/dependency-
internal exclusions are unchanged.
