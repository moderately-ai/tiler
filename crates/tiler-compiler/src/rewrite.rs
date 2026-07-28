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

use core::fmt;

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
}
