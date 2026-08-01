---
id: correct-the-sdk-apple-family-range-in-the-runtime-answer-record
title: Correct the SDK Apple family range in the runtime answer record
status: done
priority: p2
dependencies: []
related: [close-the-metal-gpu-family-out-of-crate-total-map, widen-the-metal-gpu-family-vocabulary-to-apple10]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, metal, adr-0074]
---
## User-visible outcome

`docs/research/runtime/backend-scoped-route-requirement-answers.md` states the SDK's Apple family range correctly, and its account of `MetalGpuFamily`'s attribute and consumers describes the tree as it is rather than as it was at `6f7caf3`.

## What is stale, found by reading

**Fact — the SDK range is wrong.** The record's b1 evidence paragraph and its measurement-boundary bullet both say `MTLDevice.h` "declares `MTLGPUFamilyApple1 = 1001` through `MTLGPUFamilyApple9 = 1009` (`...MTLDevice.h:233-241`)". The same header in the same installed macOS 26.5 SDK declares `MTLGPUFamilyApple10 = 1010` on line 242. Reproduce: `grep -n MTLGPUFamilyApple "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h"`. The record's conclusions do not depend on the omission — the `"Apple10" < "Apple9"` elimination is arithmetic on ASCII and is unaffected — but the record explicitly says "whether Apple ships an `MTLGPUFamilyApple10` is unknown here", and it is no longer unknown. `widen-the-metal-gpu-family-vocabulary-to-apple10` owns whether Tiler's vocabulary should follow; this ticket owns only the recorded fact.

**Fact — the attribute claim is closed.** The record states that "both its attribute and its stated reason are wrong at `6f7caf3`". `close-the-metal-gpu-family-out-of-crate-total-map` corrected the stated reason and removed `#[non_exhaustive]` under ADR 0074 convention 5b. The paragraph should record the closure and keep its reasoning rather than continue to assert a live defect.

**Fact — one of the two total maps it cites is gone and the other is not.** The record quotes `prototypes/candle-metal-adapter/src/adapter.rs:584-590`'s pair table; that site now calls `tiler_metal::applicability::observe_highest_gpu_family` and names no family. The record does not mention `prototypes/serial-sum-run/src/proof.rs:703-716`, which carries the identical table and still does; `close-the-serial-sum-run-gpu-family-probe-table` owns it. The "Fact — a working implementation of the surviving design already exists in-workspace" paragraph cites `adapter.rs:582-596` and `713-742` by line and both moved.

**Inference — the design's own conclusion is strengthened, not weakened.** The record derives that the observation should cross as a raw Apple constant supplied by `tiler-metal` rather than by publishing `MetalGpuFamily`, and item b1's proposed sketch (`AppleGpuFamilyConstant`, `observe_highest_gpu_family`) is what landed, half of it, for exactly the reason the record gives. What is now measured rather than proposed is worth marking as such: the constant crosses as `isize`, because `MTLDevice.h` declares the enumeration as `NS_ENUM(NSInteger, MTLGPUFamily)` and `objc2-metal` models `NSInteger` as `isize`; the record's sketch says `i64`, which compiles against `metal` 0.33.0 and forces a fallible conversion at the `objc2-metal` call site.

## Implementation keys

- Correct the SDK range and the measurement-boundary bullet, keeping the reproduction command so the next reader re-runs rather than trusts.
- Move the `MetalGpuFamily` attribute paragraph from a live defect to a recorded closure, citing the ticket that closed it, and preserve the 5b/5c reasoning that made it a defect.
- Re-point the line citations into `prototypes/candle-metal-adapter/src/adapter.rs`, and add `prototypes/serial-sum-run` as the remaining out-of-crate total map with its owning ticket.
- Mark the b1 sketch's `i64` as superseded by the landed `isize` and say why.

## Explicit non-goals

- **Do not accept or implement the answer surface.** The record stays a proposal; this is a factual correction to it.
- **Do not widen `MetalGpuFamily`.**

## Closes when

Every fact above is either corrected or shown to be already right by a reproduction a reader can run in one line, and no sentence in the record asserts a defect that has been closed.

## Outcome

Every claim in "What is stale" was reproduced before it was written, and one implementation key was answered differently from how it was posed.

### The SDK fact, reproduced rather than inherited

`grep -n MTLGPUFamilyApple "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h"` on `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk` (SDK version 26.5, build `25F70`) prints ten lines, `233` through `242`, ending `242: MTLGPUFamilyApple10 = 1010`. The record's window `233-241` stops one line short. Both sites now read `Apple1 = 1001` through `Apple10 = 1010` at `233-242`, with the build recorded and the command kept beside them.

**A related fact found while checking the paragraph's second clause, and recorded because it is an input to the widening question.** `metal` 0.33.0 declares `MTLGPUFamily::Apple1 … Apple9` at `src/device.rs:74-82` and names no `Apple10` — the binding is one family behind the header it transcribes. The record said it "transcribes the same values", which was true of the nine it has and became misleading once the header's range was corrected; it now says `Apple1` through `Apple9` and states the gap.

### The defect assertion keeps its truth and gains its closure

The "A defect found while deriving this" section was not rewritten into a closure and was not deleted. Its `6f7caf3` derivation stands verbatim, relabelled **Fact at `6f7caf3`** with the tense of the site's description moved to past, and two `Fact` paragraphs follow it: the closure at `662d9be` naming what changed, what the check that can say no now is, and the public-boundary consequence that went with removing `#[non_exhaustive]`; and the second, uncited site. The `Inference` paragraph that follows converted a prediction into a measurement — "every consumer that answers the row writes that table" became "both prototypes that answered the row wrote that table independently", which is what the two-site finding actually shows.

**The second site is why the observation stays live rather than historical.** `prototypes/serial-sum-run/src/proof.rs:703-716` carries the identical five-element `(MTLGPUFamily, MetalGpuFamily)` pairing and is unchanged at `8252312`; `git diff 6f7caf3..HEAD -- prototypes/serial-sum-run/src/proof.rs` is empty, so every line number this record already cited into that file remains exact. Owned by `close-the-serial-sum-run-gpu-family-probe-table`.

### The span was not edited, and one span sentence was flagged instead

**Verdict: no correction fell inside the drafted-ADR span, and one span sentence is affected and was left alone.** The span between the rules is `335-393` at `8252312` and `347-405` after this change; `diff` of the two windows is empty, and the same comparison against a one-line-shifted window reports a difference, so the check was watched saying no before it was believed saying yes.

The affected sentence is in the alternatives-considered entry *Publish the family vocabulary and let each consumer observe the device itself*: "written as a table rather than a match — which is what the existing prototype does". Its singular referent was `prototypes/candle-metal-adapter`, whose table `662d9be` removed the same evening the span landed. It is not false — `serial-sum-run` still carries the table — but it now names a different prototype than its author had in view. `docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md:62` carries the sentence byte-identically, so the discrepancy is in both documents. Recorded in a new paragraph immediately above the opening rule, in the same form as the existing link-consequence note, and **flagged for the ADR 0092 acceptance sweep**: the ADR is the authority over that subject, so the wording should change there and this span should be re-transferred from it, not corrected here first.

### The `i64` supersession

The b1 sketch keeps `i64` with an inline comment marking it superseded and pointing at property 2, and property 2 carries the reasoning: `MTLDevice.h` declares `NS_ENUM(NSInteger, MTLGPUFamily)`, `NSInteger` is pointer-sized, and `objc2-metal`'s `MTLGPUFamily(pub NSInteger)` takes an `isize` directly while `i64` forces a fallible conversion at the one binding naming Apple's type exactly. Stated as a supersession rather than a defect, because `i64` is correct on every target Tiler supports.

### A key answered differently from how it was posed, and why

The third implementation key says "re-point the line citations into `prototypes/candle-metal-adapter/src/adapter.rs`". Done — five citations moved to `8252312` and each says so. But `662d9be` also moved `crates/tiler-metal/src/applicability.rs` by about 180 lines, and roughly a dozen citations into it drifted with no change to the claims they support. **Those were deliberately not re-pointed**, and the preamble now says so and names the check: `MetalGpuFamilySupport`'s exhaustiveness is at `354`, the exact-equality comparison at `1025`, `every_outcome_is_a_refusal` at `1082`, each re-checked by symbol. The record's own convention is that symbols are durable and numbers are a convenience; re-pointing a dozen numbers would go stale again at `widen-the-metal-gpu-family-vocabulary-to-apple10`, which is already filed against that same file. Converting invisible drift into stated drift is what the preamble is for. The adapter citations are the exception because the *code* at those positions changed, not merely its offset — a reader following `adapter.rs:584-590` today finds a doc comment about the table's removal, not the table.

### The rest of the sweep

- **Nothing was compiled or measured** (measurement boundary): corrected. Two of the four proposed shapes stopped being type-system reservations at `662d9be` — `observe_highest_gpu_family` and `AppleGpuFamilyConstant` are implemented, doc-tested, and covered by a probe-recording test — while remaining unreachable out of workspace and unaccepted. The bullet now holds reservation, implementation, and accepted guarantee apart rather than collapsing them, and names the three shapes that still compile nowhere.
- **Public-boundary items 3, 4, and 6**: item 3 records that two of its four named shapes landed as concrete drafts and that a tested implementation is not implicit approval; item 4 records that the raw-constant crossing is now checkable against a running site; item 6 records that `MetalGpuFamily`'s exhaustiveness became an out-of-crate promise at `662d9be`, ahead of this design and for a different reason, and is Tom's to accept.
- **The question-1 elimination** "it is what the prototypes do today" now reads "what both prototypes did at `6f7caf3` and what `prototypes/serial-sum-run` still does".
- **The deferral bullet** for the 5b/5c defect records the closure, the surviving site, why it needs a different fix, and both owning tickets, instead of asserting a live defect.
- **Checked and left alone:** the b1b elimination (`"Apple10" < "Apple9"` is ASCII arithmetic and unaffected, and now rests on a shipped member rather than a hypothetical one — stated in the measurement boundary); "the payload codec is written twice" (still two hand-written codecs, `tiler-metal` owns neither); every `serial-sum-run` line citation (file unchanged since `6f7caf3`); the `applicability.rs:107` reviewed-draft, `:118-131` ordering, and `:172-177` exhaustiveness claims (verified by symbol, numbers left per the preamble).

### Not done, deliberately

The record's `disposition: pending` and `research_status: complete` frontmatter is unchanged: this is a factual correction and the record stays a proposal, per the ticket's non-goals. `MetalGpuFamily` was not widened. Nothing outside `docs/research/runtime/` and this ticket was edited.
