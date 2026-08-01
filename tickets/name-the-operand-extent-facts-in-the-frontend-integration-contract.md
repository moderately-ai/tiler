---
id: name-the-operand-extent-facts-in-the-frontend-integration-contract
title: Name the operand-extent facts in the frontend integration contract
status: todo
priority: p3
dependencies: []
related: [check-a-literal-operand-extent-against-the-supplied-value]
scopes: [contracts/integrations]
shared_scopes: []
paths: []
tags: [docs, inline-dx]
---
## Why this exists

**Fact.** `docs/integration/frontends.md` enumerates the region-facts vocabulary generated tokens name: "`RegionFacts`, `OperandFacts`, `SymbolFacts`, `AxisRef`, `ResultFacts`, `ResultAxis`, `bind_region`, and `build_result`".

**Fact.** `check-a-literal-operand-extent-against-the-supplied-value` added `::tiler::__private::OperandExtent` to that vocabulary — `OperandFacts` now carries `extents: &[OperandExtent]` in place of `rank`, and an expansion emits `OperandExtent::Literal(_)` and `OperandExtent::Symbolic`. The list is therefore incomplete, and the `rank` spelling it implies is gone.

**Inference.** The fixing branch could not correct it: `docs/integration/**` is `contracts/integrations`, which that ticket does not hold.

## Closes when

The vocabulary sentence names `OperandExtent`, and any sentence in that document implying an operand fact carries a rank rather than its declared extents is corrected. Verify by reading the emitted text in `crates/tiler-macros/src/binding.rs` `facts_source`, not by pattern match.
