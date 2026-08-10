Ticket: adopt-candle-command-stream-once-a-terminal-check-is-reachable
Exact audit base: c99ac54950f242d88d8dfe8335332bef0cf75f2d
Ticket content hash: daae4a9ecfc483d9d88ed6e21ba166ebeab59ed8af4311bfa9157808038dfcf4
Assigned checkout: /Users/tsanterre/workspace/github.com/moderately-ai/.worktrees/tiler/ticket-audit-2026-08-10-ro
Initial repository status: deferred
Worker: wave6def-08

Identity checks:
  - Worktree HEAD file (`tiler/.git/worktrees/ticket-audit-2026-08-10-ro/HEAD`) = `c99ac54950f242d88d8dfe8335332bef0cf75f2d`.
  - Ticket content hash matches assignment, `ledger.json` / `ledger.jsonl` claimed row, and `inventory/scope.json` entry for this ticket id (`daae4a9ecfc483d9d88ed6e21ba166ebeab59ed8af4311bfa9157808038dfcf4`).

Files read in full:
  - tickets/adopt-candle-command-stream-once-a-terminal-check-is-reachable.md
  - tickets/prototype-candle-metal-adapter.md (dependency; frontmatter status done; Outcome criterion 3 + deferred stream note; Candle pin section)
  - docs/integration/candle.md (Command-stream behavior; synchronous-readback exception; ownership boundary)
  - docs/research/runtime/candle-metal-post-wait-error-checking.md (Result; Required check; ensure_completed structural claim)
  - prototypes/candle-metal-adapter/Cargo.toml
  - prototypes/candle-metal-adapter/src/adapter.rs (module docs on checked boundary; plan_dispatch flush; dispatch own queue/buffer/commit/wait/status; submission_outcome)
  - Cargo.lock (candle-core / candle-metal-kernels 0.11.0 registry pins)
  - ticketsplease.toml (`implementation/candle` glob)
  - crates.io resolved sources under `~/.cargo/registry/src/index.crates.io-*/candle-metal-kernels-0.11.0/src/metal/commands.rs` and `candle-core-0.11.0/src/metal_backend/device.rs`
  - spikes/runtime/check_candle_post_wait_source.py (header + EXPECTED_REVISION; not re-executed)
  - tickets/repin-candle-numerical-scope-citation-at-adapter-admission.md (frontmatter + Outcome pin agreement only)

Per-Fact verdicts:
  1. [VERIFIED] Frontmatter: `status: deferred`; `priority: p2`; `dependencies: [prototype-candle-metal-adapter]`; `related: []`; `scopes: [implementation/candle]`; `shared_scopes: [project/tickets]`; `paths: []`; tags candle/runtime/integration.
     Evidence: ticket frontmatter; dependency ticket `status: done` at this base; scope glob maps to `prototypes/candle-*/**`.
     Raw source anchor: `status: deferred`
     Construction path: deferred after prototype landed private checked stream.
     Consumption path: trigger recheck; future implementation claim under `implementation/candle`.
     Reproduction: open dependency frontmatter `status: done`.

  2. [VERIFIED] User-visible outcome describes encoding into Candle's active stream without private buffer/commit/wait, while still refusing host reads of device memory before terminal success on the exact command buffer.
     Evidence: matches `docs/integration/candle.md` Command-stream section ordinary path plus post-wait terminal requirement on the exception path.
     Raw source anchor: `The adapter encodes into Candle's active command stream`
     Construction path: accepted candle integration contract.
     Consumption path: this ticket's activation and close conditions.
     Reproduction: open `docs/integration/candle.md` section `## Command-stream behavior`.

  3. [VERIFIED] Why deferred — contract requires stream overlap; synchronous readback exception is allowed only until the verified post-wait gap is fixed **or** the adapter supplies an equivalent checked boundary; the prototype supplies the latter and pays with lost overlap.
     Evidence: candle.md "is not sufficient until the verified gap is fixed or the adapter supplies an equivalent checked boundary"; adapter module docs and dispatch own queue/buffer/commit/wait/status; plan_dispatch flushes Candle pending work because streams are separate.
     Raw source anchor: `or the adapter supplies an equivalent checked boundary`
     Construction path: ADR/research post-wait finding + prototype criterion 3.
     Consumption path: this deferred ticket.
     Reproduction: `rg -n 'equivalent checked boundary|own command queue|wait_until_completed' docs/integration/candle.md prototypes/candle-metal-adapter/src/adapter.rs`

  4. [VERIFIED] Fact — Candle 0.11.0 `Commands::ensure_completed` performs no post-wait terminal check: pre-wait status match only; `NotEnqueued`/`Enqueued` commit then wait; `Committed`/`Scheduled` wait; both arms fall through to `Ok(())` without re-reading status; only pre-wait `Error` is reported.
     Evidence: exact crates.io `candle-metal-kernels-0.11.0` source `fn ensure_completed` at the cited file; same shape as research record and adapter module comment.
     Raw source anchor: `fn ensure_completed(cb: &CommandBuffer)`
     Construction path: upstream 0.11.0 release resolved by workspace.
     Consumption path: deferred activation condition 1; prototype private stream.
     Reproduction: open registry `candle-metal-kernels-0.11.0/src/metal/commands.rs` and inspect `ensure_completed` after both wait arms.

  5. [VERIFIED] Fact — the terminal check is unreachable from outside Candle at 0.11.0: `MetalDevice.commands` is `pub(crate)`; `EntryState::{current,in_flight}` and `ensure_completed` are private; `Commands` public API exposes encoders and wait/flush helpers but no accessor for the in-flight `CommandBuffer`; `MetalDevice::wait_until_completed` returns `Result<()>` with no status payload.
     Evidence: crates.io `candle-core-0.11.0` `pub(crate) commands: Arc<Commands>`; `wait_until_completed` maps kernel result and returns `Ok(())`; `CommandsGuard` only exposes `ComputeCommandEncoder`.
     Raw source anchor: `pub(crate) commands: Arc<Commands>`
     Construction path: Candle 0.11.0 crate boundaries.
     Consumption path: activation condition 2; prototype cannot encode into Candle stream and still read terminal status of that buffer.
     Reproduction: `rg -n 'pub\\(crate\\) commands|fn wait_until_completed|fn ensure_completed|struct EntryState' ~/.cargo/registry/src/*/candle-core-0.11.0/src/metal_backend/device.rs ~/.cargo/registry/src/*/candle-metal-kernels-0.11.0/src/metal/commands.rs`

  6. [VERIFIED] Activation trigger — any of: post-wait status re-read in `ensure_completed` (or successor); public in-flight `CommandBuffer` or wait returning terminal status; Tiler pins a fork under `git = … rev = …`.
     Evidence: three conditions are MECE ways to make a checked overlapping stream possible; each is re-checkable against the revision the workspace resolves.
     Raw source anchor: `Candle's \`ensure_completed\` (or its successor) re-reads the status after the wait`
     Construction path: post-wait research Required check + crate API facts.
     Consumption path: Trigger check log; status remains deferred until one fires.
     Reproduction: re-evaluate the three bullets against current Cargo pin and public API.

  7. [VERIFIED] Closes when — four unfinished conditions: encode into `MetalDevice::command_encoder` guard with no private queue/buffer; no device-memory read before terminal success on the exact buffer; remove or re-justify plan_dispatch flush of Candle pending work; measure overlap rather than assert.
     Evidence: adapter still owns `queue`, mints its own `CommandBuffer`, commits/waits, reads status after wait; `plan_dispatch` still calls `self.device.wait_until_completed()` as `PendingCandleWork` refusal path; no overlap measurement harness on this ticket.
     Raw source anchor: `brings Candle's own pending work to a terminal state`
     Construction path: prototype closed with private stream.
     Consumption path: future implementation after trigger fire.
     Reproduction: `rg -n 'fn plan_dispatch|new_command_queue|command_buffer.commit|submission_outcome' prototypes/candle-metal-adapter/src/adapter.rs`

  8. [VERIFIED] Trigger check log 2026-08-04 and 2026-08-09 — **not fired.** Workspace still resolves `candle-core` / `candle-metal-kernels` `"0.11.0"` from crates.io with no `git`/`rev` fork; `ensure_completed` unchanged; no public terminal-status command-buffer channel.
     Evidence: `prototypes/candle-metal-adapter/Cargo.toml` version pins; `Cargo.lock` `source = "registry+https://github.com/rust-lang/crates.io-index"` and no `git =` for candle; crates.io 0.11.0 source still lacks post-wait re-read and public CB access.
     Raw source anchor: `candle-core = { version = "0.11.0", features = ["metal"] }`
     Construction path: prototype admission pin choice.
     Consumption path: deferred remains correct at this base.
     Reproduction: `rg -n 'candle-core|candle-metal-kernels' prototypes/candle-metal-adapter/Cargo.toml Cargo.lock` plus registry source re-read of `ensure_completed`.

  9. [VERIFIED] Independent recheck at audit base (2026-08-10 tree) — trigger still **not fired**. Same three conditions false: post-wait gap present; public CB/status channel absent; no git/rev fork pin.
     Evidence: same sources as verdicts 4–5 and 8 at frozen audit base commit and resolved registry crates.
     Raw source anchor: `Ok(())` immediately after the pre-wait match in `ensure_completed` with no intervening `status()` call
     Construction path: this audit.
     Consumption path: confirms log entries remain accurate; no status flip required.
     Reproduction: re-open crates.io 0.11.0 `ensure_completed` and adapter Cargo.toml.

  10. [VERIFIED] Dependency `prototype-candle-metal-adapter` is `done` and explicitly deferred this work under criterion 3 / "Asynchronous encoding into Candle's stream".
     Evidence: dependency Outcome criterion 3 names this ticket id as carrying the activation trigger; unsupported-cases bullet defers async Candle stream.
     Raw source anchor: `adopt-candle-command-stream-once-a-terminal-check-is-reachable`
     Construction path: prototype landing.
     Consumption path: this ticket's dependency edge and Why section.
     Reproduction: `rg -n 'adopt-candle-command-stream' tickets/prototype-candle-metal-adapter.md`

Current repository behavior:
  - Prototype `prototypes/candle-metal-adapter` is the only Candle consumer; it implements a private command queue/buffer, flushes Candle pending work in `plan_dispatch`, commits and waits its own buffer in `dispatch`, and classifies post-wait `MTLCommandBufferStatus` via `submission_outcome` before returning `CandleCompletion` storage.
  - Ordinary contract path (encode into Candle's active encoder without commit/wait) is not implemented; overlap with surrounding Candle GPU work is forfeited by design.
  - Workspace resolves crates.io Candle 0.11.0 only; post-wait terminal-check gap remains in `ensure_completed`.

Board and graph verdict:
  status: `deferred` correct — close conditions unmet; trigger not fired at this base; dated Trigger check log present with two **not fired** entries that still match the tree.
  dependencies: `prototype-candle-metal-adapter` is `done` and is the correct sole dependency (lands the equivalent checked boundary that makes deferral coherent).
  related work: empty list acceptable; research ticket `verify-candle-metal-post-wait-error-checking` and contract `docs/integration/candle.md` are cited in prose rather than related edges — not a graph defect for this deferred node.
  scopes: `implementation/candle` + shared `project/tickets` match future edit surface under `ticketsplease.toml` candle glob; empty `paths` is correct while deferred.
  trigger state: **not fired** at c99ac54950f2. All three activation conditions remain false against crates.io 0.11.0 and the adapter Cargo pin. Log currency (latest 2026-08-09) still correctly describes 2026-08-10 audit-base reality; no status promotion.
  closure state: open/deferred; none of the four Closes-when bullets are satisfied by current adapter code.

Repair required:
  - exact metadata changes OR none: none
  - exact prose correction OR none: none
  - exact dated correction OR none: none (optional process hygiene only: append a 2026-08-10 **not fired** log line; not required for truth of the ticket body)
  - exact new or connected remainder OR none: none

Public/API/identity/architecture consequences:
  none while deferred. When the trigger fires, implementation will touch the prototype/real candle adapter public runtime path and must preserve post-wait terminal observation before any device-memory host read; that is the ticket's own close condition, not an audit escalation.

Tests and checks:
  - Registry source audit of `ensure_completed` (structural; no cargo run).
  - Adapter source audit of private stream + plan_dispatch flush + post-wait `submission_outcome`.
  - Cargo.toml / Cargo.lock pin audit for absence of git/rev fork.
  - No `make full` / workspace cargo: Facts do not require it.

Exact files expected to change:
  - none for repair
  - (future activation only) primarily under `prototypes/candle-*/**` or eventual `crates/tiler-candle/**` per scope map — out of this audit's write set

Residual uncertainty:
  - Local huggingface/candle git checkout is not at the research pin `31f35b14` (feature branch); authority for this audit is crates.io `candle-*-0.11.0` registry sources matching `Cargo.lock`, which still exhibit the gap. No evidence a newer Candle release is already resolved by this workspace.

Recommended audit_state:
  audited-clean
