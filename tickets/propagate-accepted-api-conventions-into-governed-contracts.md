---
id: propagate-accepted-api-conventions-into-governed-contracts
title: Propagate accepted ADR 0074 conventions into the contracts it governs
status: in-progress
priority: p2
dependencies: []
related: [draft-public-api-conventions-adr, harden-public-enums-non-exhaustive, extend-canonical-identity-encodings-for-reserved-variants, disambiguate-presentation-label-from-semantic-key-accessors]
scopes: [contracts/foundation]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, public-api, contracts]
claimed_from: todo
assignee: agent-propagate-accepted-api-conventions-into-governed-contracts
lease_expires_at: 1784912789
---
ADR 0074 is now **accepted**, and declares
`applies_to: ["tiler.contract.ir", "tiler.contract.architecture"]`. The typed
governance edge exists and the documentation gate validates it structurally, but
a passing gate does not check that the destination contracts' **prose** actually
states the rules the ADR now governs. An accepted decision whose destination
contract is silent is exactly the "conflicting terminology / stale status" drift
the documentation contract warns about.

Check each destination and propagate what is genuinely normative *there*:

- `docs/ir.md` — its "Shared IR construction lifecycle" section is where the
  construction, identity, encoding, and error conventions become normative for
  the shared IR.
- `docs/architecture.md` — owns component boundaries, which is where the rule
  about when a module may be `pub` versus a crate-private draft authority
  belongs.

Do **not** duplicate the ADR into the contracts. The ADR is the decision and its
reasoning; a contract states the resulting rule and cites the ADR. Restate only
the conventions that are actually normative for that document — if only some of
the seven apply to a given contract, say only those, and record why the others do
not belong there rather than copying all seven for symmetry.

Leave ADR 0074's open questions alone: the descriptor-accessor style is owned by
`unify-schedule-index-region-with-verified-index-region`, the `key()` naming
hazard by `disambiguate-presentation-label-from-semantic-key-accessors`, and
whether ADR 0074 should also name `tiler.contract.optimizer` is deliberately
deferred until a reviewed compiler facade exists. If propagation reveals that a
convention is wrong or unstatable in its destination, amend the ADR explicitly
rather than quietly weakening the contract text.

Run `uv run --locked python scripts/docs.py render` and the full documentation
gate before completion.

## Outcome

Both destinations now state what ADR 0074 makes normative *there* and cite the
record rather than reproducing it.

`docs/ir.md` gains `### Accepted public API conventions` inside "Shared IR
construction lifecycle", stating conventions 1, 2, 3, 4, and 6 as rules for the
shared IR surfaces this contract owns: typed non-erasing errors with
`Error::source()` preserving the wrapped type and a two-layer convenience
generic over both concrete error types; opaque identities whose only reader is
`as_bytes()`, with no public constructor on a derived identity, a documented
opaque wrapping constructor only for an identity received at a boundary, and a
presentation-only label that is never an equality input; canonical encodings
that are domain-separated, length-prefixed, ordinal-free, and exhaustively
matched; a consuming `build` terminal whose verified product cannot be forged,
mutated, thawed, or reached through a closure convenience; and verified products
without public fields alongside the leaf value-data descriptor exception. The
descriptor-style and accessor-naming open questions are referenced as open, not
settled.

`docs/architecture.md` gains convention 7 in "Component ownership": an
implemented authority that is not yet reachable from its crate entry point stays
a private module with crate-visible items and a module-level
`#![allow(dead_code, reason = "…")]` naming what it reserves, becoming public
only when Tom accepts the exact facade, with an already-public draft boundary
saying so in its module documentation. A second paragraph records that the
shape conventions bind any workspace crate exposing such a surface, are stated
normatively in the IR contract, and are deliberately not restated here — this
states scope without touching ADR 0074's deferred `tiler.contract.optimizer`
question, which is about which document is normative, not about which crates the
conventions bind.

`docs/glossary.md` gains the four terms the new normative text depends on and
that the corpus did not define: canonical identity, presentation label, verified
product, and leaf value-data descriptor.

**Convention 5 was deliberately not propagated.** It is under amendment by
`resolve-non-exhaustive-recognizer-hole` because `#[non_exhaustive]` on an enum
a consumer exhaustively recognizes makes a later variant compile at every
cross-crate consumer while silently routing it into reject-unknown. `docs/ir.md`
records the omission and its owner rather than leaving a silent gap between the
ADR's seven conventions and the contract's five; no growth-marking rule is
stated until that record distinguishes recognized enums from produced or read
ones.

**One convention did not survive propagation intact, and the ADR — not this
contract — has to resolve it.** Convention 4 states that "`build`, not
`freeze`, is the terminal vocabulary" for every public workspace API. The tree
at `37f1350` has five landed consuming `freeze` terminals:
`SemanticRegistryBuilder::freeze` (`crates/tiler-ir/src/semantic/registry.rs`),
`ScalarRegistryBuilder::freeze` (`crates/tiler-ir/src/index/scalar.rs`),
`LoweringCapabilityRegistryBuilder::freeze`
(`crates/tiler-compiler/src/capability.rs`), `ReferenceRegistryBuilder::freeze`
(`crates/tiler-reference/src/lib.rs`), and
`ScalarReferenceRegistryBuilder::freeze`
(`crates/tiler-reference/src/oracle.rs`). All five consume `self` and yield an
immutable unforgeable snapshot, so they satisfy the substance of convention 4;
two are infallible and the three fallible ones return only a typed error,
without recoverable builder ownership. `docs/ir.md` already describes that
lifecycle normatively as "consumes the builder at freeze". ADR 0074's own consequences
list names the detectable violation as "a `freeze`-style *non-consuming*
terminal", which suggests the intended rule is consuming-versus-non-consuming
rather than a spelling rule; the decision text does not say so. The contract
therefore states the terminal rule for the shared IR layer builders, where it is
exactly true, and records the registry-freeze family as a question for ADR 0074
rather than weakening or over-claiming either side. Amending that record needs
`contracts/decisions`, which this ticket does not hold.

`uv run --locked python scripts/docs.py render` reports 177 records with no
generated-catalog change. `uv run --locked python scripts/check_repository.py`
passes end to end, including the Rust sub-gate. It must be run outside the agent
sandbox: `spikes/extensions/run.py --self-test` calls `os.killpg`, which a
sandboxed run rejects with `PermissionError: [Errno 1] Operation not permitted`
before the gate reaches its Rust stage. That failure is environmental and
unrelated to this change.
