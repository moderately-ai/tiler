Ticket: refresh-the-forkless-physical-provider-spike-against-the-landed-seam
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/refresh-the-forkless-physical-provider-spike-against-the-landed-seam/0bf790874af3_c99ac54950f2.md
Pre-edit content hash (from ledger): 0bf790874af3c0f907ca4ce0f4cfd78cbbc3ecb27aa2046b60373c8413182d68
Post-edit content hash: 199b68d165c33b954f026966a9388790665b60681711964b076454998877d380

Changes applied:
  - In `## Current completion correction — 2026-08-09`, replaced the imprecise claim that both disclosure tickets “have also since landed” with board-accurate wording: `disclose-offered-and-selected-physical-provider-sets-separately` closed after `788b0c03` delivered `Compilation::offered_physical_providers`; `disclose-the-physical-provider-environment-a-compilation-was-offered` remains `awaiting-decision` for the artifact `CompilationEnvironment` subject (compiler-side documentation/accessor half already discharged). Kept that acceptance remains at `accept-the-installed-physical-provider-public-surface` and that no remaining provider-environment implementation gap sits on this spike ticket. Direct live-sentence repair (no stacked dated layer).

Optional items skipped (with reason):
  - Optional dated correction note: skipped because the imprecise live sentence was repaired in place, so stacking another greppable correction layer was unnecessary.

Residuals not applied (docs/crates/new tickets/authority):
  - Spike nextest not re-run; residual uncertainty in the audit only.
  - Spike README/results framing of `offered_providers` lowering-only as a current-tree gap description after `offered_physical_providers` landed — outside this closed ticket’s scope.
  - `docs/operation-extensions.md` counterfactual “would upgrade” prose after the re-run — docs drift not owned by this ticket.
  - No new remainder tickets; no depends-on forced onto the awaiting-decision disclosure ticket (metadata left as-is per report).

Verification:
  - files read: assigned audit report; full ticket; frontmatter of `disclose-the-physical-provider-environment-a-compilation-was-offered` (status: awaiting-decision) and `disclose-offered-and-selected-physical-provider-sets-separately` (status: done)
  - checks: shasum -a 256 of ticket after edit; related statuses re-read from ticket frontmatter

Recommended next ledger state:
  integrated
