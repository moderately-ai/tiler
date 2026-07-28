//! Governed identity for external rewrite rule providers.
//!
//! This is the first slice of `implement-transactional-rewrite-engine`. It
//! defines *who* may propose a rewrite and how that authority is named; it does
//! not yet run one.
//!
//! # Why identity comes before the engine
//!
//! Six of the properties that ticket names are already implemented in
//! [`crate::normalize`] — termination, budgets, rollback, semantic
//! revalidation, deterministic traversal, and typed explain — for exactly one
//! hard-coded rule. What is absent is that rules may come from *outside* and
//! that the result may be a set of alternatives rather than one canonical graph.
//!
//! Identity is the half that has to exist first, and not for tidiness. The
//! ticket's governing constraint is that unknown provider behaviour is never
//! optimizable merely because it is registered. A rewrite that cannot be
//! attributed to a named, versioned rule cannot be explained, cannot be
//! reproduced, and cannot be excluded when a provider turns out to be wrong. So
//! the engine may not accept a proposal before it can name what proposed it.
//!
//! # What is deliberately not here
//!
//! No trait for proposing rewrites, no registry, and no engine. Adding them
//! before the transaction is generalized would create a seam with nothing on
//! the other side of it, and the shape of the proposal type depends on whether
//! alternatives are produced per rule or per traversal — a question the
//! normalize stage does not answer because it never produces alternatives.

use crate::explain::{ExplainError, ExplainRecordId, ExplainWriter};
use core::fmt;
use std::sync::Arc;

/// The governed identity of one rewrite rule.
///
/// A rule is named by its provider and its own key, and carries a revision that
/// changes whenever its *output* can change. Two rules from different providers
/// may share a key without being the same rule, which is why the provider is
/// part of the identity rather than a label beside it.
///
/// The revision is output-affecting rather than a version number: a rule whose
/// implementation is refactored without changing what it produces keeps its
/// revision, and a rule whose output changes must not. This mirrors
/// [`tiler_ir::semantic::ProviderIdentity`], which separates provider
/// provenance from semantic meaning for the same reason (ADR 0072).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(
    dead_code,
    reason = "the first slice of implement-transactional-rewrite-engine: identity exists before the engine that consumes it, and is exercised by this module's own tests"
)]
pub(crate) struct RewriteRuleIdentity {
    provider: &'static str,
    rule: &'static str,
    revision: u32,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: accessors for a reviewed draft identity whose consumer is the not-yet-written engine"
)]
impl RewriteRuleIdentity {
    /// Builds a rule identity.
    ///
    /// Both names must be non-empty: an empty provider or rule key would make
    /// two distinct rules render identically in explain output, which is the
    /// failure a reader cannot see.
    pub(crate) const fn new(
        provider: &'static str,
        rule: &'static str,
        revision: u32,
    ) -> Option<Self> {
        if provider.is_empty() || rule.is_empty() {
            return None;
        }
        Some(Self {
            provider,
            rule,
            revision,
        })
    }

    /// The provider that owns this rule.
    pub(crate) const fn provider(&self) -> &'static str {
        self.provider
    }

    /// The rule's own key, unique within its provider.
    pub(crate) const fn rule(&self) -> &'static str {
        self.rule
    }

    /// The output-affecting revision.
    pub(crate) const fn revision(&self) -> u32 {
        self.revision
    }

    /// Appends this identity's canonical bytes.
    ///
    /// Length-prefixed rather than delimiter-separated, so a provider named
    /// `"a.b"` with rule `"c"` cannot encode identically to provider `"a"` with
    /// rule `"b.c"`. A delimiter would make those two collide, and a collision
    /// here means two rules sharing one identity.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        for part in [self.provider, self.rule] {
            let part = part.as_bytes();
            bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
            bytes.extend_from_slice(part);
        }
        bytes.extend_from_slice(&self.revision.to_be_bytes());
    }
}

impl fmt::Display for RewriteRuleIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/{}@{}",
            self.provider, self.rule, self.revision
        )
    }
}

/// The identity of the one rule the normalize stage already proves.
///
/// Registering this rule and nothing else is the correctness pin for the engine:
/// it must then produce exactly what [`crate::normalize`] produces today,
/// compared on `SemanticIdentity`. The keys match that stage's existing governed
/// constants so the two cannot drift apart silently.
#[allow(
    dead_code,
    reason = "the pin the engine will be built against; consumed once the engine exists"
)]
pub(crate) const COMMON_SUBEXPRESSION_RULE: Option<RewriteRuleIdentity> =
    RewriteRuleIdentity::new("tiler.normalize", "common-subexpression.v1", 1);

/// A rule's own explain records, emitted only if its proposal is adopted.
///
/// # Why the rule supplies a payload instead of recording as it works
///
/// A proposal may be abandoned by the budget or rejected by structural
/// revalidation *after* `propose` returns. A rule that recorded while proposing
/// would claim a rewrite that never happened, and the abandoned-run case is
/// where explain output matters most. `normalize_semantics` records after
/// adopting, and that ordering is load-bearing.
///
/// # Why the engine cannot reconstruct these
///
/// The facts are rule-specific and describe the *original* program — the
/// common-subexpression rule reports which graph-local operation was canonical
/// and which was merged. The engine holds only the candidate, so recovering
/// them would mean re-running the rule's own detection, a second authority over
/// what the rule already decided.
///
/// The engine threads this opaquely and never interprets it. That is what keeps
/// rule-specific facts out of a generic type while still letting them reach
/// explain.
pub(crate) trait RuleExplain: fmt::Debug + Send + Sync {
    /// Emits this rule's records, returning the new causal head.
    ///
    /// Called **only** for a proposal that survived revalidation and the budget.
    /// An implementation may assume its rewrite was adopted.
    fn record(
        &self,
        explain: &mut ExplainWriter,
        cause: ExplainRecordId,
    ) -> Result<ExplainRecordId, ExplainError>;
}

/// One rewrite a provider proposes: the rule that proposed it, and the whole
/// program it would produce.
///
/// # Why a whole candidate program rather than an edit script
///
/// A structured edit vocabulary — delete this operation, replace that operand —
/// is more expressive to reason about, and it is refused here for two reasons
/// that compound.
///
/// It would need a closed vocabulary covering every kind of edit any rule might
/// ever make, and this engine's whole purpose is to admit rules from *outside*.
/// A closed edit vocabulary either constrains external rules to what was
/// imagined when it was written, or it grows an escape hatch that puts unchecked
/// structure back into the graph — which is the thing the ticket forbids when it
/// says unknown provider behaviour is never optimizable merely because it is
/// registered.
///
/// And it would need its own validator. A candidate program does not: the
/// normalize stage already revalidates by rebuilding through the checked
/// [`SemanticProgramBuilder`], so the frozen semantic authority re-infers and
/// re-validates every operation, and a malformed candidate is rejected by the
/// authority that already owns that judgement rather than by a second one
/// written for edits. `AGENTS.md` calls a second authority over an encoding a
/// defect rather than a design, and this is the same shape.
///
/// The cost is real and worth stating: the engine cannot see *what* a rule
/// changed, only that the result is valid and what it costs. That is enough for
/// what the engine does — revalidate, compare identity, and choose — and a rule
/// that wants to explain itself does so through its own typed explain records,
/// not by handing the engine a diff to interpret.
///
/// [`SemanticProgramBuilder`]: tiler_ir::semantic::SemanticProgramBuilder
#[derive(Clone, Debug)]
#[allow(
    dead_code,
    reason = "the proposal shape the engine will consume; landed with its derivation before the engine that reads it"
)]
pub(crate) struct RewriteProposal<Program> {
    rule: RewriteRuleIdentity,
    candidate: Program,
    rewrites: u64,
    explain: Option<Arc<dyn RuleExplain>>,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: accessors for a reviewed draft proposal whose consumer is the not-yet-written engine"
)]
impl<Program> RewriteProposal<Program> {
    /// Pairs a candidate program with the rule that proposed it.
    ///
    /// Generic over the program type so this module does not depend on the
    /// semantic IR before the engine does. The engine instantiates it at
    /// `SemanticProgram`; the tests below at a stand-in, which is what lets the
    /// pairing be tested without building a program.
    /// Pairs a candidate with the rule that proposed it and the number of
    /// rewrites it represents.
    ///
    /// **`rewrites` is a required argument and deliberately has no default.**
    /// One proposal is not one rewrite: the common-subexpression rule returns a
    /// single candidate however many redundant operations it merged. The budget
    /// counts rewrites, so a default would have to guess — and every guess is
    /// wrong in the dangerous direction, because under-counting lets a rewrite
    /// commit past a budget meant to forbid it. Only the rule knows.
    pub(crate) const fn new(rule: RewriteRuleIdentity, candidate: Program, rewrites: u64) -> Self {
        Self {
            rule,
            candidate,
            rewrites,
            explain: None,
        }
    }

    /// How many rewrites this proposal represents, for the budget.
    pub(crate) const fn rewrites(&self) -> u64 {
        self.rewrites
    }

    /// Attaches the rule's own explain records.
    ///
    /// Optional because a rule with nothing rule-specific to report is
    /// legitimate — the engine emits stage-level records regardless. `None`
    /// means "this rule adds no records of its own", never "this rewrite was
    /// not recorded".
    #[must_use]
    pub(crate) fn with_explain(mut self, explain: Arc<dyn RuleExplain>) -> Self {
        self.explain = Some(explain);
        self
    }

    /// The rule's own explain records, if it supplied any.
    ///
    /// The engine must call this **only** for a proposal it adopted; see
    /// [`RuleExplain`].
    pub(crate) fn explain(&self) -> Option<&Arc<dyn RuleExplain>> {
        self.explain.as_ref()
    }

    /// The rule that proposed this rewrite.
    ///
    /// Always present. A proposal that could not name its rule would be
    /// unattributable, and an unattributable rewrite cannot be explained,
    /// reproduced, or excluded — which is why this is a field rather than an
    /// `Option` the engine would have to handle.
    pub(crate) const fn rule(&self) -> RewriteRuleIdentity {
        self.rule
    }

    /// The program this rewrite would produce, before revalidation.
    ///
    /// Named `candidate` rather than `program` because it is not yet trusted:
    /// nothing may adopt it until it has been rebuilt through the checked
    /// builder and has passed every postcondition.
    pub(crate) const fn candidate(&self) -> &Program {
        &self.candidate
    }
}

/// An external authority that proposes rewrites of a whole program.
///
/// Generic over the program type for the same reason [`RewriteProposal`] is:
/// the engine instantiates it at the semantic IR, and this module stays
/// independent of that IR until the engine exists.
///
/// A provider owns exactly one rule. Bundling several under one provider would
/// make [`Self::identity`] ambiguous, and the whole point of the identity is
/// that a rewrite can be attributed, reproduced, and *excluded* — excluding a
/// bundle would take out rules that were never implicated.
#[allow(
    dead_code,
    reason = "the seam the engine will call; landed with its attribution invariant before the engine that drives it"
)]
pub(crate) trait RewriteRuleProvider<Program> {
    /// The governed identity of the rule this provider implements.
    ///
    /// Must be stable across calls. The engine relies on it to attribute
    /// proposals, so a provider that returned a different identity per call
    /// would make its own rewrites unreproducible.
    fn identity(&self) -> RewriteRuleIdentity;

    /// Proposes zero or more rewrites of `program`.
    ///
    /// Whole-program in, proposals out, because detection in this codebase is
    /// whole-program: `normalize::detect_shared_values` takes the entire
    /// program and returns its complete result rather than walking sites. An
    /// empty result means "nothing to do here", never a failure.
    ///
    /// Every returned proposal must carry [`Self::identity`]. That is not
    /// enforceable by the type, so the engine must check it with
    /// [`misattributed`] before adopting anything.
    ///
    /// Fallible, and deliberately: `Ok(vec![])` means "nothing to do here",
    /// while `Err` means the rule could not run. Collapsing those into one
    /// empty vector would hide a compiler fault behind the most common ordinary
    /// outcome — see [`ProviderDefect::Failed`].
    fn propose(&self, program: &Program) -> Result<Vec<RewriteProposal<Program>>, ProviderDefect>;
}

/// Returns the proposals that do not carry `expected`, in the order given.
///
/// # Why this check exists
///
/// A provider constructs its own [`RewriteProposal`]s, so nothing stops one
/// from stamping another rule's identity on its work — by mistake when a
/// provider is copied from another, or deliberately. Either way the consequence
/// is the same and it is not a cosmetic one: attribution is what lets a rule be
/// *excluded* when it turns out to be wrong, so a misattributed proposal
/// survives the exclusion of the rule that actually produced it, and the
/// exclusion of an innocent one takes its place.
///
/// The engine must treat a non-empty result as a typed provider defect and
/// reject the whole batch, not filter it. A provider that misattributes one
/// proposal has demonstrated it does not know what it is, and its other
/// proposals are not thereby trustworthy — which is the same reasoning that
/// makes a cache-key mismatch a protocol defect rather than a miss.
#[allow(
    dead_code,
    reason = "the invariant the engine enforces; landed with the trait that makes it necessary"
)]
pub(crate) fn misattributed<Program>(
    expected: RewriteRuleIdentity,
    proposals: &[RewriteProposal<Program>],
) -> Vec<&RewriteProposal<Program>> {
    proposals
        .iter()
        .filter(|proposal| proposal.rule() != expected)
        .collect()
}

/// Why a provider could not be registered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "registration outcome for the seam the engine will build on"
)]
pub(crate) enum RegistrationError {
    /// Another provider already claims this rule identity.
    DuplicateRule(RewriteRuleIdentity),
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRule(rule) => {
                write!(
                    formatter,
                    "rewrite.duplicate-rule: {rule} is already registered"
                )
            }
        }
    }
}

/// The set of rule providers an engine run may draw on.
///
/// # Two invariants, both load-bearing
///
/// **No two providers may claim one rule identity.** Registration refuses a
/// duplicate rather than replacing or shadowing. If two providers shared an
/// identity, a proposal could not be traced to the code that made it, and
/// excluding the rule would exclude both — which is the attribution failure
/// [`misattributed`] exists to prevent, arriving through the registry instead.
///
/// **Iteration is in canonical identity order, not registration order.** The
/// engine's alternative set must be reproducible across runs, and registration
/// order is exactly the kind of incidental ordering that varies between a host
/// that registers providers from a static list and one that discovers them.
/// Sorting by identity makes the order a property of *which* rules are present
/// rather than of how they arrived. This is the same reason
/// `enumerate_frontier`'s identity is independent of provider order.
#[allow(
    dead_code,
    reason = "the registry the engine will drive; landed with its invariants before the engine exists"
)]
pub(crate) struct RuleRegistry<Program> {
    providers: Vec<Box<dyn RewriteRuleProvider<Program>>>,
}

#[allow(
    dead_code,
    reason = "see the type's own allow: reviewed draft registry whose consumer is the not-yet-written engine"
)]
impl<Program> RuleRegistry<Program> {
    /// An empty registry.
    ///
    /// Empty is a legitimate state, not a misconfiguration: an engine run with
    /// no registered rules proposes nothing and adopts nothing, which is the
    /// correct behaviour rather than an error to report.
    pub(crate) const fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Registers a provider, refusing a rule another provider already claims.
    pub(crate) fn register(
        &mut self,
        provider: Box<dyn RewriteRuleProvider<Program>>,
    ) -> Result<(), RegistrationError> {
        let identity = provider.identity();
        if self
            .providers
            .iter()
            .any(|existing| existing.identity() == identity)
        {
            return Err(RegistrationError::DuplicateRule(identity));
        }
        let position = self
            .providers
            .partition_point(|existing| existing.identity() < identity);
        self.providers.insert(position, provider);
        Ok(())
    }

    /// The registered providers, in canonical identity order.
    pub(crate) fn providers(&self) -> &[Box<dyn RewriteRuleProvider<Program>>] {
        &self.providers
    }

    /// The registered rules, in canonical order.
    pub(crate) fn rules(&self) -> Vec<RewriteRuleIdentity> {
        self.providers
            .iter()
            .map(|provider| provider.identity())
            .collect()
    }
}

/// A provider misbehaved in a way that makes its whole batch untrustworthy.
///
/// Separate from a rewrite being *rejected*: a rejected candidate is an ordinary
/// outcome of revalidation, while this says the provider itself violated the
/// registration contract. The two must not share a type, or a caller counting
/// rejections would count defects among them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the provider-contract failure the engine reports; landed with the collection step that detects it"
)]
pub(crate) enum ProviderDefect {
    /// The rule could not run at all.
    ///
    /// Distinct from proposing nothing, and the distinction is the reason
    /// `propose` is fallible. An empty proposal set is the *normal* result for
    /// most rules on most programs, so a rule that failed and a rule with
    /// nothing to say would be indistinguishable if both returned an empty
    /// vector — and the failure would be invisible by construction rather than
    /// merely unreported.
    Failed {
        /// The rule that could not run.
        rule: RewriteRuleIdentity,
        /// A stable reason code from the rule's own error vocabulary.
        reason: &'static str,
    },
    /// The provider returned a proposal attributed to another rule.
    Misattributed {
        /// The identity the provider declares.
        declared: RewriteRuleIdentity,
        /// The identity it stamped on a proposal.
        found: RewriteRuleIdentity,
    },
}

impl fmt::Display for ProviderDefect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed { rule, reason } => {
                write!(
                    formatter,
                    "rewrite.rule-failed: {rule} could not run ({reason})"
                )
            }
            Self::Misattributed { declared, found } => write!(
                formatter,
                "rewrite.misattributed-proposal: {declared} proposed work attributed to {found}"
            ),
        }
    }
}

/// Collects every registered provider's proposals for `program`.
///
/// This is the engine's front half: it drives the registry, enforces the
/// attribution contract, and hands back the candidates that revalidation will
/// then judge. It deliberately does **not** revalidate, adopt, or budget —
/// those belong to the transaction, which is [`crate::normalize`]'s to
/// generalize.
///
/// # Order
///
/// Providers are visited in the registry's canonical identity order and each
/// provider's proposals are kept in the order it returned them, so the result
/// is reproducible across runs. A provider that returns its proposals in a
/// varying order makes only its own block vary, which is its defect to fix and
/// not something this function can detect.
///
/// # Failing the whole batch
///
/// One misattributed proposal fails the entire call, including proposals from
/// providers already collected. That is deliberate and it is the stricter of
/// the two available readings. A provider that misattributes has demonstrated
/// it does not know what it is; keeping its correctly-attributed proposals
/// would mean trusting the same code that just proved untrustworthy, and
/// keeping *other* providers' proposals would silently produce a smaller
/// alternative set than the registry describes — a partial result that reads
/// like a complete one. The same reasoning makes a cache key/subject mismatch a
/// protocol defect rather than an ordinary miss.
#[allow(
    dead_code,
    reason = "the engine's collection step; landed ahead of the transaction that consumes it"
)]
pub(crate) fn collect_proposals<Program>(
    registry: &RuleRegistry<Program>,
    program: &Program,
) -> Result<Vec<RewriteProposal<Program>>, ProviderDefect>
where
    Program: Clone,
{
    let mut collected = Vec::new();
    for provider in registry.providers() {
        let declared = provider.identity();
        let proposals = provider.propose(program)?;
        if let Some(offender) = misattributed(declared, &proposals).first() {
            return Err(ProviderDefect::Misattributed {
                declared,
                found: offender.rule(),
            });
        }
        collected.extend(proposals);
    }
    Ok(collected)
}

/// Emits the rule-supplied explain records of alternatives that were adopted.
///
/// Separate from [`collect_proposals`] and from the engine for the same reason
/// readmission is: this is a *policy* about when records may be written, and it
/// only holds if the caller passes alternatives it actually adopted.
///
/// **The obligation this cannot enforce, stated where a caller will read it:**
/// pass only survivors. A proposal abandoned by the budget or rejected by
/// revalidation must never reach here, because its payload describes a rewrite
/// that did not happen. Nothing in the type system distinguishes an adopted
/// proposal from a discarded one, so this is a contract on the call site rather
/// than a check — which is exactly why the alternatives are threaded through
/// revalidation before they arrive.
///
/// Records form one linear causal chain rooted at `cause`, matching how
/// `NormalizationOutcome::record` chains its own, so a later stage depends on a
/// single receipt rather than an unbounded cause set.
#[allow(
    dead_code,
    reason = "the explain step, landed with its tests ahead of the routing that will call it"
)]
pub(crate) fn record_adopted_alternatives<Program>(
    adopted: &[RewriteProposal<Program>],
    explain: &mut ExplainWriter,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, ExplainError> {
    for alternative in adopted {
        if let Some(records) = alternative.explain() {
            cause = records.record(explain, cause)?;
        }
    }
    Ok(cause)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An empty provider or rule key is refused.
    ///
    /// Driven against both the accepting and the rejecting case, so a
    /// constructor that returned `Some` unconditionally would fail here rather
    /// than pass silently.
    #[test]
    fn an_unnamed_rule_or_provider_is_refused() {
        assert!(RewriteRuleIdentity::new("p", "r", 0).is_some());
        assert!(
            RewriteRuleIdentity::new("", "r", 0).is_none(),
            "an unnamed provider would render identically to another"
        );
        assert!(
            RewriteRuleIdentity::new("p", "", 0).is_none(),
            "an unnamed rule would render identically to another"
        );
    }

    /// The canonical encoding separates the provider from the rule.
    ///
    /// This is the collision a delimiter would allow: `"a.b"/"c"` and
    /// `"a"/"b.c"` render differently and must encode differently, or two rules
    /// share one identity.
    #[test]
    fn a_dotted_name_cannot_collide_across_the_provider_boundary() {
        let mut left = Vec::new();
        let mut right = Vec::new();
        RewriteRuleIdentity::new("a.b", "c", 1)
            .expect("named")
            .encode(&mut left);
        RewriteRuleIdentity::new("a", "b.c", 1)
            .expect("named")
            .encode(&mut right);
        assert_ne!(
            left, right,
            "two distinct rules encoded identically, so they share one identity"
        );
    }

    /// The revision is part of the identity.
    ///
    /// A rule whose output changes must not compare equal to its earlier self,
    /// or a cached decision taken under the old behaviour would still be
    /// attributed to the new one.
    #[test]
    fn a_revision_change_is_a_different_rule() {
        let old = RewriteRuleIdentity::new("p", "r", 1).expect("named");
        let new = RewriteRuleIdentity::new("p", "r", 2).expect("named");
        assert_ne!(old, new);
        let (mut left, mut right) = (Vec::new(), Vec::new());
        old.encode(&mut left);
        new.encode(&mut right);
        assert_ne!(left, right);
    }

    /// The normalize stage's rule is nameable, and renders as its parts.
    #[test]
    fn the_normalize_rule_has_a_governed_identity() {
        let rule = COMMON_SUBEXPRESSION_RULE.expect("the normalize rule is named");
        assert_eq!(rule.provider(), "tiler.normalize");
        assert_eq!(rule.rule(), "common-subexpression.v1");
        assert_eq!(rule.revision(), 1);
        assert_eq!(
            rule.to_string(),
            "tiler.normalize/common-subexpression.v1@1"
        );
    }

    /// A proposal carries its rule, and the rule is not optional.
    ///
    /// The stand-in program type is what keeps this test independent of the
    /// semantic IR: the property under test is the pairing, and using a real
    /// program would test the IR's constructors instead.
    #[test]
    fn a_proposal_names_the_rule_that_made_it() {
        let rule = COMMON_SUBEXPRESSION_RULE.expect("the normalize rule is named");
        let proposal = RewriteProposal::new(rule, "candidate-program-stand-in", 1);
        assert_eq!(proposal.rule(), rule);
        assert_eq!(*proposal.candidate(), "candidate-program-stand-in");
    }

    /// Two rules proposing the same candidate remain distinguishable.
    ///
    /// If the rule were dropped or defaulted, two providers proposing an
    /// identical program would become one proposal, and excluding a provider
    /// that turned out to be wrong would silently exclude the other's work too.
    #[test]
    fn identical_candidates_from_different_rules_stay_distinct() {
        let left = RewriteProposal::new(
            RewriteRuleIdentity::new("p", "a", 1).expect("named"),
            "same",
            1,
        );
        let right = RewriteProposal::new(
            RewriteRuleIdentity::new("p", "b", 1).expect("named"),
            "same",
            1,
        );
        assert_eq!(*left.candidate(), *right.candidate());
        assert_ne!(
            left.rule(),
            right.rule(),
            "two providers' proposals collapsed into one"
        );
    }

    /// A provider that stamps another rule's identity on its work is caught.
    ///
    /// Driven against a clean batch *and* a tainted one, so a checker that
    /// returned an empty list unconditionally fails here rather than passing.
    /// This is the check that keeps rule exclusion meaningful: without it, a
    /// misattributed proposal survives the exclusion of the rule that made it.
    #[test]
    fn a_misattributed_proposal_is_reported() {
        let mine = RewriteRuleIdentity::new("p", "mine", 1).expect("named");
        let theirs = RewriteRuleIdentity::new("p", "theirs", 1).expect("named");

        let honest = [
            RewriteProposal::new(mine, "a", 1),
            RewriteProposal::new(mine, "b", 1),
        ];
        assert!(
            misattributed(mine, &honest).is_empty(),
            "a provider's own proposals were reported as misattributed"
        );

        let tainted = [
            RewriteProposal::new(mine, "a", 1),
            RewriteProposal::new(theirs, "b", 1),
        ];
        let caught = misattributed(mine, &tainted);
        assert_eq!(caught.len(), 1, "the foreign proposal was not caught");
        assert_eq!(caught[0].rule(), theirs);
    }

    /// A trait implementation's identity is what its proposals are checked
    /// against, not a value passed alongside.
    #[test]
    fn a_provider_is_checked_against_its_own_declared_identity() {
        struct Cse;
        impl RewriteRuleProvider<&'static str> for Cse {
            fn identity(&self) -> RewriteRuleIdentity {
                COMMON_SUBEXPRESSION_RULE.expect("the normalize rule is named")
            }
            fn propose(
                &self,
                _program: &&'static str,
            ) -> Result<Vec<RewriteProposal<&'static str>>, ProviderDefect> {
                Ok(vec![RewriteProposal::new(self.identity(), "candidate", 1)])
            }
        }

        let provider = Cse;
        let proposals = provider.propose(&"program").expect("no defect");
        assert_eq!(proposals.len(), 1);
        assert!(
            misattributed(provider.identity(), &proposals).is_empty(),
            "a provider proposing under its own identity was rejected"
        );
    }

    struct Named(RewriteRuleIdentity);
    impl RewriteRuleProvider<&'static str> for Named {
        fn identity(&self) -> RewriteRuleIdentity {
            self.0
        }
        fn propose(
            &self,
            _program: &&'static str,
        ) -> Result<Vec<RewriteProposal<&'static str>>, ProviderDefect> {
            Ok(Vec::new())
        }
    }

    fn named(rule: &'static str) -> Box<dyn RewriteRuleProvider<&'static str>> {
        Box::new(Named(
            RewriteRuleIdentity::new("p", rule, 1).expect("named"),
        ))
    }

    /// Two providers cannot claim one rule identity.
    ///
    /// Driven against a distinct second registration as well, so a `register`
    /// that rejected everything would fail here rather than pass.
    #[test]
    fn a_duplicate_rule_is_refused() {
        let mut registry = RuleRegistry::new();
        assert!(registry.register(named("a")).is_ok());
        assert!(
            registry.register(named("b")).is_ok(),
            "a distinct rule was refused"
        );
        let duplicate = registry.register(named("a"));
        assert!(
            matches!(duplicate, Err(RegistrationError::DuplicateRule(_))),
            "a second provider claimed a registered rule: {duplicate:?}"
        );
        assert_eq!(registry.rules().len(), 2);
    }

    /// Iteration order is canonical, not the order providers arrived in.
    ///
    /// Registering the same rules in opposite orders must yield the same
    /// sequence, or the engine's alternative set depends on how a host happened
    /// to register its providers.
    #[test]
    fn registration_order_does_not_reach_iteration_order() {
        let mut forward = RuleRegistry::new();
        for rule in ["c", "a", "b"] {
            forward.register(named(rule)).expect("distinct");
        }
        let mut reverse = RuleRegistry::new();
        for rule in ["b", "a", "c"] {
            reverse.register(named(rule)).expect("distinct");
        }
        assert_eq!(
            forward.rules(),
            reverse.rules(),
            "two registration orders produced different iteration orders"
        );
        let mut sorted = forward.rules();
        sorted.sort_unstable();
        assert_eq!(forward.rules(), sorted, "iteration order is not canonical");
    }

    /// An empty registry is a legitimate state.
    #[test]
    fn an_empty_registry_has_no_rules() {
        let registry: RuleRegistry<&'static str> = RuleRegistry::new();
        assert!(registry.rules().is_empty());
        assert!(registry.providers().is_empty());
    }

    struct Proposing(RewriteRuleIdentity, RewriteRuleIdentity);
    impl RewriteRuleProvider<&'static str> for Proposing {
        fn identity(&self) -> RewriteRuleIdentity {
            self.0
        }
        fn propose(
            &self,
            _program: &&'static str,
        ) -> Result<Vec<RewriteProposal<&'static str>>, ProviderDefect> {
            Ok(vec![RewriteProposal::new(self.1, "candidate", 1)])
        }
    }

    /// Well-behaved providers' proposals are collected in canonical order.
    #[test]
    fn proposals_are_collected_in_canonical_rule_order() {
        let mut registry = RuleRegistry::new();
        for rule in ["c", "a", "b"] {
            let identity = RewriteRuleIdentity::new("p", rule, 1).expect("named");
            registry
                .register(Box::new(Proposing(identity, identity)))
                .expect("distinct");
        }
        let collected = collect_proposals(&registry, &"program").expect("no defect");
        let rules: Vec<&'static str> = collected
            .iter()
            .map(|proposal| proposal.rule().rule())
            .collect();
        assert_eq!(
            rules,
            ["a", "b", "c"],
            "collection order followed registration rather than identity"
        );
    }

    /// One misattributed proposal fails the whole batch, not just its provider.
    ///
    /// The honest provider is registered *first* in canonical order, so this
    /// also confirms already-collected proposals are discarded rather than
    /// returned as a partial result that would read like a complete one.
    #[test]
    fn one_misattributed_proposal_fails_the_whole_batch() {
        let honest = RewriteRuleIdentity::new("p", "a", 1).expect("named");
        let liar = RewriteRuleIdentity::new("p", "b", 1).expect("named");

        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(Proposing(honest, honest)))
            .expect("distinct");
        registry
            .register(Box::new(Proposing(liar, honest)))
            .expect("distinct");

        let outcome = collect_proposals(&registry, &"program");
        assert_eq!(
            outcome.err(),
            Some(ProviderDefect::Misattributed {
                declared: liar,
                found: honest,
            }),
            "a provider stamping another rule's identity was accepted"
        );
    }

    /// An empty registry collects nothing and is not a defect.
    #[test]
    fn an_empty_registry_collects_no_proposals() {
        let registry: RuleRegistry<&'static str> = RuleRegistry::new();
        let collected = collect_proposals(&registry, &"program").expect("empty is not a defect");
        assert!(collected.is_empty());
    }

    /// A rule that cannot run is reported, not read as having nothing to say.
    ///
    /// This is the distinction the fallible signature exists for. The failing
    /// provider is registered *second* in canonical order, so the test also
    /// confirms the first provider's proposals are discarded rather than
    /// returned as a partial result — the same all-or-nothing rule
    /// misattribution follows.
    #[test]
    fn a_rule_that_cannot_run_is_not_read_as_proposing_nothing() {
        struct Broken(RewriteRuleIdentity);
        impl RewriteRuleProvider<&'static str> for Broken {
            fn identity(&self) -> RewriteRuleIdentity {
                self.0
            }
            fn propose(
                &self,
                _program: &&'static str,
            ) -> Result<Vec<RewriteProposal<&'static str>>, ProviderDefect> {
                Err(ProviderDefect::Failed {
                    rule: self.0,
                    reason: "builder-create",
                })
            }
        }

        let working = RewriteRuleIdentity::new("p", "a", 1).expect("named");
        let broken = RewriteRuleIdentity::new("p", "b", 1).expect("named");

        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(Proposing(working, working)))
            .expect("distinct");
        registry
            .register(Box::new(Broken(broken)))
            .expect("distinct");

        assert_eq!(
            collect_proposals(&registry, &"program").err(),
            Some(ProviderDefect::Failed {
                rule: broken,
                reason: "builder-create",
            }),
            "a rule that could not run was read as having nothing to propose"
        );

        // The same registry without the broken provider must succeed, or the
        // assertion above would pass for the wrong reason.
        let mut healthy = RuleRegistry::new();
        healthy
            .register(Box::new(Proposing(working, working)))
            .expect("distinct");
        assert_eq!(
            collect_proposals(&healthy, &"program")
                .expect("no defect")
                .len(),
            1
        );
    }

    #[derive(Debug)]
    struct Silent;

    impl RuleExplain for Silent {
        fn record(
            &self,
            _explain: &mut ExplainWriter,
            cause: ExplainRecordId,
        ) -> Result<ExplainRecordId, ExplainError> {
            Ok(cause)
        }
    }

    /// A payload is attached to the proposal that carries it and to no other.
    ///
    /// **What this does not cover, said plainly rather than implied:**
    /// `record_adopted_alternatives` itself is untested. Constructing an
    /// `ExplainWriter` requires a `VerifiedTargetRequest`, which this module has
    /// no fixture for and should not grow one for — the emission loop belongs to
    /// the routing change, and it is covered there against a real writer. What is
    /// pinned here is the part routing depends on: that a rule's payload reaches
    /// its own proposal and does not leak to a sibling.
    #[test]
    fn a_payload_attaches_to_its_own_proposal_only() {
        let rule = RewriteRuleIdentity::new("p", "r", 1).expect("named");
        let silent = RewriteProposal::new(rule, "no-payload", 1);
        let loud = RewriteProposal::new(rule, "payload", 1).with_explain(Arc::new(Silent));

        assert!(
            silent.explain().is_none(),
            "a proposal was given a payload it never attached"
        );
        assert!(
            loud.explain().is_some(),
            "an attached payload did not reach its proposal"
        );
    }
}
