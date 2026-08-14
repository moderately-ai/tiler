//! Instrumented physical providers used as census and sweep subjects.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use tiler_compiler::physical_provider::{
    DeclinedStrategy, ImplementationContext, ImplementationProposal,
    PhysicalImplementationProvider, PhysicalProviderProvenance, PhysicalProviderProvenanceError,
    ProviderOffer, StrategyDeclineCause, TargetApplicability,
};
use tiler_ir::semantic::ProviderIdentity;

/// How an instrumented provider answers one region subject.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Answer {
    /// Propose nothing and decline nothing.
    Empty,
    /// Decline a named strategy on every subject.
    Decline,
    /// Specialize the host baseline workgroup when one exists; decline otherwise.
    Specialize { threads: u32 },
    /// Propose a workgroup the declared-capacity profile cannot satisfy.
    Infeasible { threads: u32 },
}

/// Per-compile emission and subject census.
#[derive(Clone, Debug, Default)]
pub struct Tally {
    /// `propose` invocations observed.
    pub invocations: u64,
    /// Proposal bodies emitted.
    pub proposals: u64,
    /// Named strategy declines emitted.
    pub declines: u64,
    /// Distinct subject keys observed.
    pub subjects: BTreeSet<String>,
    /// Invocation ordinals used to keep coverless subjects distinct.
    next_coverless: u64,
    /// Distinct presentation roles observed.
    pub roles: BTreeSet<String>,
    /// Subjects that had a single-dispatch baseline.
    pub baseline_subjects: u64,
    /// Subjects that had no single-dispatch baseline.
    pub coverless_or_unspellable: u64,
}

impl Tally {
    /// Raw proposal-plus-decline outcomes this tally observed.
    #[must_use]
    pub const fn raw_outcomes(&self) -> u64 {
        self.proposals.saturating_add(self.declines)
    }

    /// Distinct region subjects observed.
    #[must_use]
    pub fn distinct_subjects(&self) -> u64 {
        u64::try_from(self.subjects.len()).unwrap_or(u64::MAX)
    }
}

/// Shared interior tally so several providers of one environment report as one
/// installed population.
pub type SharedTally = Rc<RefCell<Tally>>;

/// Returns an empty shared tally.
#[must_use]
pub fn shared_tally() -> SharedTally {
    Rc::new(RefCell::new(Tally::default()))
}

/// An installable provider that records what the host asked and what it emitted.
pub struct CountingProvider {
    identity: ProviderIdentity,
    answer: Answer,
    tally: SharedTally,
}

impl CountingProvider {
    /// Builds one named provider of `answer` that reports into `tally`.
    #[must_use]
    pub fn new(name: &str, answer: Answer, tally: SharedTally) -> Self {
        Self {
            identity: ProviderIdentity::new("acme", name, 1)
                .expect("the acme provider identity is valid"),
            answer,
            tally,
        }
    }
}

impl PhysicalImplementationProvider for CountingProvider {
    fn provenance(&self) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
        PhysicalProviderProvenance::new(self.identity.clone())
    }

    fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
        let subject = context.subject();
        let baseline = context.baseline();
        let key = {
            let mut tally = self.tally.borrow_mut();
            subject_key(
                context.target_profile_key(),
                subject.role(),
                subject.covered_occurrences(),
                baseline,
                &mut tally.next_coverless,
            )
        };
        {
            let mut tally = self.tally.borrow_mut();
            tally.invocations = tally.invocations.saturating_add(1);
            tally.subjects.insert(key);
            tally.roles.insert(subject.role().to_owned());
            if baseline.is_some() {
                tally.baseline_subjects = tally.baseline_subjects.saturating_add(1);
            } else {
                tally.coverless_or_unspellable = tally.coverless_or_unspellable.saturating_add(1);
            }
        }

        match self.answer {
            Answer::Empty => ProviderOffer::default(),
            Answer::Decline => {
                {
                    let mut tally = self.tally.borrow_mut();
                    tally.declines = tally.declines.saturating_add(1);
                }
                ProviderOffer::default().decline(DeclinedStrategy::new(
                    "acme.calibrate-decline",
                    StrategyDeclineCause::NoAdmissibleShape {
                        rule: "acme.calibrate.forced-decline",
                        extent: u64::try_from(subject.covered_occurrences()).unwrap_or(u64::MAX),
                    },
                ))
            }
            Answer::Specialize { threads } | Answer::Infeasible { threads } => {
                if let Some(baseline) = baseline {
                    {
                        let mut tally = self.tally.borrow_mut();
                        tally.proposals = tally.proposals.saturating_add(1);
                    }
                    let mut region = baseline.region().clone();
                    region.schedule.threads_per_workgroup = threads;
                    region.schedule.launch.threads_per_workgroup = threads;
                    ProviderOffer::proposing(vec![ImplementationProposal::scheduled_kernel(
                        region,
                        TargetApplicability::for_targets([context
                            .target_profile()
                            .profile_key()
                            .clone()]),
                        baseline.cost(),
                    )])
                } else {
                    {
                        let mut tally = self.tally.borrow_mut();
                        tally.declines = tally.declines.saturating_add(1);
                    }
                    ProviderOffer::default().decline(DeclinedStrategy::new(
                        "acme.wide-workgroup",
                        StrategyDeclineCause::NoAdmissibleShape {
                            rule: "acme.no-single-dispatch-baseline",
                            extent: u64::try_from(subject.covered_occurrences())
                                .unwrap_or(u64::MAX),
                        },
                    ))
                }
            }
        }
    }
}

fn subject_key(
    target: &str,
    role: &str,
    covered: usize,
    baseline: Option<&tiler_compiler::physical_provider::BaselineImplementation>,
    next_coverless: &mut u64,
) -> String {
    if let Some(baseline) = baseline {
        let region = baseline.region();
        format!(
            "{target}:{role}:{covered}:wg={}:grid={}:iters={:?}",
            region.schedule.threads_per_workgroup,
            region.schedule.launch.threads_per_workgroup,
            region.index.iteration_shape
        )
    } else {
        let ordinal = *next_coverless;
        *next_coverless = next_coverless.saturating_add(1);
        format!("{target}:{role}:{covered}:none:{ordinal}")
    }
}

/// Builds `count` providers of one answer sharing one tally.
#[must_use]
pub fn flock(
    prefix: &str,
    count: usize,
    answer: Answer,
    tally: &SharedTally,
) -> Vec<CountingProvider> {
    (0..count)
        .map(|index| CountingProvider::new(&format!("{prefix}-{index}"), answer, Rc::clone(tally)))
        .collect()
}

/// Trait-object view of a flock that lives as long as the flock.
#[must_use]
pub fn as_dyn(flock: &[CountingProvider]) -> Vec<&dyn PhysicalImplementationProvider> {
    flock
        .iter()
        .map(|provider| provider as &dyn PhysicalImplementationProvider)
        .collect()
}
