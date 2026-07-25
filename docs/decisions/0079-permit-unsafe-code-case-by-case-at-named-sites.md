---
schema: "tiler-doc/v1"
id: "ADR-0079"
kind: "decision"
title: "Permit unsafe code case by case at named sites"
topics: ["rust", "workspace", "lints", "runtime", "metal"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.architecture"]
evidence: ["tiler.research.workspace.prototype-crate-layout-and-msrv", "tiler.research.runtime.execution-contract"]
ticket: "record-the-case-by-case-unsafe-boundary"
---

# 0079: Permit unsafe code case by case at named sites

**Status:** accepted. Tom made this decision on 2026-07-25 and realized it himself in commit `a56bff8` before any record existed. This record catches the corpus up; it proposes nothing and widens nothing. The change that lands it also amends `AGENTS.md`, because until then the working contract states a rule the tree has already departed from — which is the drift that sentence exists to prevent.

## Context

**Fact — what the prohibition was.** The root [`Cargo.toml`](../../Cargo.toml) declares `[workspace.lints.rust]` with `missing_docs = "warn"` and `unsafe_code = "forbid"`, and `[workspace.lints.clippy]` with `all` and `pedantic` at `warn` with `priority = -1`. Every member inherited that table through `[lints] workspace = true`, and [`scripts/check_workspace.py`](../../scripts/check_workspace.py) pinned both workspace tables exactly.

**Fact — `forbid` is the level that cannot be reopened.** The rustc lint-level rule is that `forbid` "is the same as `deny`, but also forbids changing the lint level afterwards" ([rustc book, lint levels](https://doc.rust-lang.org/rustc/lints/levels.html)). A `forbid`ed lint therefore cannot be relaxed by an inner `#[allow]` at any scope: not on a module, not on a function, not on a block. The workspace-level choice of `forbid` over `deny` was not a stylistic intensifier — it made the prohibition unopenable from inside a crate, so the only place it can be opened at all is the crate's own `[lints]` table.

**Fact — the runtime proof reached a wall that is structural, not stylistic.** [`prototype-metal-runtime-execution`](../../tickets/prototype-metal-runtime-execution.md) recorded on 2026-07-25 that first GPU dispatch was blocked by the prohibition rather than by the missing dependency: `metal v0.33.0` resolves cleanly and adding it to a prototype is not an ADR 0075 always-ask category, but no route from a Rust `&[f32]` into `MTLBuffer` storage avoids `unsafe`. **Exhaustive-finite check.** `metal-0.33.0/src/buffer.rs` declares ten public methods on the buffer type — `length`, `contents`, `did_modify_range`, `new_texture_with_descriptor`, `remote_storage_buffer`, `new_remote_buffer_view_for_device`, `add_debug_marker`, `remove_all_debug_markers`, `gpu_address`, and `gpu_resource_id`. Exactly one of them reaches storage, and `contents(&self) -> *mut std::ffi::c_void` hands back a raw pointer. None returns a slice or any borrowed view. Every other Metal call the proof makes — device creation, library loading from `metallib` bytes, pipeline construction, encoder setup, dispatch, commit, and completion wait — is a safe call in that binding, so the entire unavoidable surface is moving `f32` bytes across the pointer `contents` returns.

**Correction to the record that stated the blocker.** That ticket attributes the necessity partly to `Device::new_buffer_with_data` "taking a `*const c_void`". Read at the pinned version, that function is a *safe* `fn`, so calling it is not what requires `unsafe`; the binding encapsulates its own `msg_send!` there. The accurate statement is the one above: the binding exposes buffer storage only as a raw pointer, and dereferencing that pointer is Tiler's own `unsafe`. The proof does not call `new_buffer_with_data` at all — [`prototypes/serial-sum-run/src/main.rs`](../../prototypes/serial-sum-run/src/main.rs) allocates with the safe `Device::new_buffer` and copies through the two functions this record governs. The conclusion the ticket drew is unchanged; its stated reason is narrowed to what the source supports.

**Fact — Tom chose a third, tighter option than either form put to him.** The ticket offered a narrow form (`unsafe_code = "allow"` on the runtime prototype crate, `forbid` everywhere else) and a broad form (relax the workspace default). Neither was taken. What landed sets `deny` on the one crate and opts in per function, which is strictly narrower than the narrow form: under a crate-level `allow`, a later unsafe block anywhere in that crate compiles silently, and under `deny` it is a hard error unless someone writes an attribute naming it.

**Fact — the complete extent of unsafe in the workspace at `43f685f`.** The exact check is `grep -rn 'unsafe' --include='*.rs' crates prototypes`, which returns twelve lines. Two are `unsafe` blocks, both in [`prototypes/serial-sum-run/src/buffer.rs`](../../prototypes/serial-sum-run/src/buffer.rs) at lines 52 and 85. Two more are the `unsafe_code` lint name inside the attributes admitting those blocks. The remaining eight are the word in prose — module and item documentation in the same file, `numerically unsafe` in `crates/tiler-metal-aot/src/input.rs`'s optimization-level documentation, and an `unsafe_division` test binding in `crates/tiler-artifact/src/program/tests.rs`.

**Fact — how the exception is spelled.** [`prototypes/serial-sum-run/Cargo.toml`](../../prototypes/serial-sum-run/Cargo.toml) omits `[lints] workspace = true` and declares `[lints.rust]` with `missing_docs = "warn"` and `unsafe_code = "deny"`, plus a `[lints.clippy]` table byte-identical to the workspace one. The difference between the inherited table and the declared one is exactly one lint level.

**Fact — the exception is pinned, not merely permitted.** `scripts/check_workspace.py` carries `UNINHERITED_LINT_MEMBERS`, a single-entry map from `tiler-prototype-run` to the exact table it may declare. It is consulted twice: `expected_member_manifest` substitutes it for `{workspace = true}` when building the manifest every member is compared against in full, and a second explicit comparison reports the lint table on its own so the failure names the right thing. A second member dropping inheritance fails the gate, and so does widening this member's `deny` to `allow`, adding a lint to its table, or removing one.

## Decision

### 1. Unsafe is permitted case by case, at an individual function or module

**Fact — this is Tom's decision as made, restated rather than derived.** Unsafe code is admitted one site at a time, isolated to a specific function or module. It is explicitly not admitted for a whole crate and explicitly not by any relaxation of the workspace default. The unit of permission is the site, and each site is its own decision.

### 2. The three properties that make the exception safe

**Fact — narrowest-scope opt-in.** The permission is attached to the item that needs it. Both admitted sites carry `#[allow(unsafe_code, reason = "…")]` on the function, which is the narrowest position at which the attribute reliably applies, and neither the crate root nor any module carries one.

**Fact — `deny` at crate level, never `allow`.** The diverging crate replaces `forbid` with `deny` and nothing else. Unsafe therefore remains a hard compile error throughout that crate except where an attribute names it, so adding an unsafe block is an act that has to be written down rather than one that merely compiles.

**Fact — a pinned gate check.** `UNINHERITED_LINT_MEMBERS` names the one member permitted to diverge and the exact table it may carry. Both halves are load-bearing: without the member pin a second crate could quietly drop inheritance, and without the table pin the named member could widen `deny` to `allow` without any check noticing.

### 3. What a future site must satisfy

**Proposal.** An unsafe block is admitted only when all four hold. These conditions state what the two landed sites already do; they are not a new discipline invented here.

- **Structurally unavoidable.** A foreign API leaves no safe route to the same result. Convenience, performance, and avoiding a copy are not qualifying reasons, and the argument must name the API surface that is missing rather than assert that none exists.
- **`reason` on the attribute.** The `#[allow(unsafe_code, reason = "…")]` states why the site is unavoidable, in terms a reader who does not know the foreign API can check.
- **An assertion bounding what it touches, before the block.** The bound must be checked against the foreign object's own report of itself rather than against the caller's belief. Both landed sites assert the byte length they are about to touch against `buffer.length()`, so a caller that mismatched allocation and element count gets an attributable panic instead of a silent out-of-bounds copy.
- **A `SAFETY` comment naming the invariant relied on.** Not a restatement of what the block does — the specific property that makes it sound, and where that property comes from.

### 4. What this record does not license

**Fact — it is a case-by-case permission and it is stated so that citing it is not sufficient.** A third site is a new decision. This record is not precedent that carries to it, and a reader who reaches for it as authority has read it wrong: what generalizes is the four conditions in item 3, not the conclusion that they were met once.

**Proposal — three specific extensions are outside it.** A crate-level `unsafe_code = "allow"`, a second member dropping lint inheritance, and any change to the workspace `forbid` are each outside this decision and each needs Tom. The second of those is enforced mechanically; the other two are not, because no check reads the value a crate would put in a table it is not permitted to have.

## Consequences

- The working contract stops stating a rule the tree has departed from. `AGENTS.md`'s "unsafe code remains forbidden unless an accepted decision changes that boundary" is amended by the change that lands this record, and its neighbouring "keep workspace Rust and Clippy lints inherited by every crate" gains the pinned exception, because that sentence was falsified by the same commit.
- The compiler, IR, artifact, reference, Metal-emission, and driver crates keep `forbid` unchanged. The property the prohibition buys is preserved exactly where it is load-bearing, and spent only in the one layer that must speak to an Objective-C API.
- Widening the exception is now visible as a decision rather than reachable as an edit. `scripts/check_workspace.py` fails on a second diverging member or a widened table, so that half of item 4 is checked rather than trusted.
- The per-site half is **not** checked, and this is the gap a reader must not assume away. Nothing counts, locates, or constrains `#[allow(unsafe_code)]` attributes inside the one crate permitted to have them, so a third site added there compiles and passes the complete gate. Item 3 is a rule enforced by review, and item 2's third property is a claim about crates, not about sites. [`pin-the-admitted-unsafe-sites-in-the-workspace-gate`](../../tickets/pin-the-admitted-unsafe-sites-in-the-workspace-gate.md) owns closing it.
- A production runtime crate will face this boundary again. Nothing here pre-approves it: `tiler-prototype-run` is a non-published proof executable, and admitting the same divergence for a reusable library is a different question because a library's unsafe becomes its consumers' unsafe.
- The `metal` dependency itself is unaffected either way. It is a normal `crates.io` dependency of one prototype, pinned at `0.33.0` in `[workspace.dependencies]` and in the gate's expected tables, and this record decides nothing about which Metal binding Tiler settles on.

## Implementation boundary

**Fact — nothing in items 1, 2, or 4's mechanical half is unimplemented.** `implementation_status` is `implemented`. The crate's `[lints]` table, both `#[allow]` sites with their reasons, assertions, and `SAFETY` comments, and the `UNINHERITED_LINT_MEMBERS` pin all exist at `43f685f` and are exercised by the complete gate.

**What is stated but not mechanized.** Item 3's four conditions and item 4's two unchecked extensions are review obligations. They are written here so a reviewer can cite them, which is a weaker guarantee than a check and is recorded as such rather than described as enforcement.

## Alternatives considered

**Keep `forbid` workspace-wide and write no unsafe.** The honest baseline, and it was not available: the binding exposes buffer storage only as a raw pointer, exhaustively over its ten public buffer methods, so the alternative is not "write it safely" but "do not execute on the GPU". A hand-written Objective-C shim behind a `cc`-driven `build.rs` moves the unsafe out of Rust rather than removing it, and buys that with a build-script dependency, a C toolchain requirement, and an FFI boundary of its own — a worse trade for two straight-line copies.

**The narrow form as first put to Tom: `unsafe_code = "allow"` on the one crate.** Rejected in favour of something tighter. Under `allow`, every later unsafe block in that crate compiles with no attribute, no reason, and no diff that a reviewer would look at twice; the crate's posture would then be "unsafe is fine here", which is exactly the whole-crate permission item 1 excludes. Under `deny` the same block does not compile until someone writes the attribute and its reason, which is what makes the site the unit of permission rather than the crate.

**The broad form: relax the workspace default.** Rejected. It spends the prohibition across the compiler, IR, artifact, reference, emission, and driver crates — every layer where it is genuinely load-bearing and none of which has ever needed a foreign pointer — to serve two functions in one proof executable.

**Enumerate the admitted sites in `scripts/check_workspace.py` as part of this change.** Not rejected on merit; it is the right check and it is deliberately not landed here. Writing it is engineering in `implementation/workspace` rather than decision custody, and the ticket that produced this record holds neither the mandate nor the evidence to design the predicate — whether it pins file-and-function pairs, a count, or the attribute text. It is split out rather than left implicit, and the gap is stated in Consequences so nobody reads this record as claiming the check exists.

**Record the decision as `proposed` and wait for acceptance.** Rejected because it would misdescribe what happened. ADR 0077 is `proposed` because the workspace ran ahead of a decision Tom had not made; here Tom made the decision and wrote the code himself, and the only thing missing was the record. Marking it `proposed` would tell a reader that the divergence in the tree is unauthorized, which is false.

## Traceability

The [prototype crate layout research](../research/workspace/prototype-crate-layout-and-msrv.md) is the evidence behind the workspace-and-lint discipline this record makes one exception to — that the member set and its pinned configuration are the mechanical enforcement of Tiler's layer separation rather than a packaging convenience, which is what makes an exception to it a decision rather than a settings change. It does not itself discuss lint levels or unsafe, and is cited for the contract this record constrains rather than for its subject. The [consumer-neutral runtime execution contract](../research/runtime/runtime-execution-contract.md) is the evidence behind the ordering the read site depends on: it requires exact terminal success of the specific execution unit before a host reads back what it produced, which is the property that makes `read_f32`'s `SAFETY` claim about GPU-then-host ordering true rather than assumed. The primary evidence for everything this record decides is inspected source at `43f685f` and the pinned gate, both cited inline above; neither research record establishes it.

The [architecture contract](../architecture.md) owns the prototype packaging and toolchain profile this record adds a clause to. `AGENTS.md` is the operative statement of the rule and is not a governed record, so no `applies_to` edge can name it — the same schema gap ADR 0075 records under its open questions, and the reason the amendment is made directly in the working contract by the change that lands this record instead of being propagated by a later one. The work records are [`prototype-metal-runtime-execution`](../../tickets/prototype-metal-runtime-execution.md) for the blocker and the options put to Tom, and [`record-the-case-by-case-unsafe-boundary`](../../tickets/record-the-case-by-case-unsafe-boundary.md) for this record.
