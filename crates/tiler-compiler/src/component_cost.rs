//! Deterministic analytical component costs, reported and never pruned on.
//!
//! This is the framework half of `implement-analytical-component-cost-model`.
//! It computes symbolic per-component costs for a complete plan, attributes them
//! to a governed analytical model key, and reports them. It is **explicitly
//! analytical and uncalibrated**; `calibrate-device-cost-models` owns device
//! measurement and activation.
//!
//! # Why these never reach dominance
//!
//! [`crate::selection::PlanStructuralCost`] is the sole pruning input, and it
//! stays that way. Three places enforce a single governed cost-model key — the
//! frontier rejects a proposal whose estimate is not
//! `tiler.cost.structural.v1`, `aggregate_cost` refuses a plan whose regions
//! mix keys, and `dominates` returns `false` across differing keys. That last
//! one is why a second key cannot simply be admitted alongside the first: plans
//! carrying different keys never dominate each other, so the non-dominated set
//! would silently become the whole set and Pareto pruning would go dark with
//! nothing reporting that it had.
//!
//! So a [`ComponentCost`] is deliberately **not** a
//! [`crate::frontier::PhysicalCostEstimate`] and has no `dominates`. It carries
//! its own model key, it is reported, and the retained plan set is bit-for-bit
//! what it was before this module existed.
//!
//! # Why two of the nine components are `Unknown`
//!
//! The accepted contract keeps `SoundProof`, exhaustive finite evidence,
//! empirical evidence, and `Unknown` as different classes, and this module
//! honours that rather than filling gaps with plausible arithmetic.
//!
//! Seven components are derived from values a plan already carries —
//! allocation, dispatch, the ordering constraints between dispatches,
//! per-element address arithmetic, the work a fused cover repeats, memory
//! traffic, and peak threadgroup memory — and each is computed at its match arm
//! rather than estimated. Memory traffic is the one
//! `Bounded` component, because the plan does not model cache reuse and a point
//! estimate would claim a precision nothing here has.
//!
//! The other two need inputs the compiler does not have: a resource-pressure and
//! occupancy model, and compile time, which is a measurement rather than an
//! analysis and belongs to `calibrate-device-cost-models`.
//!
//! Artifact size was **removed** from the vocabulary rather than left `Unknown`.
//! Costs are reported for every retained plan, and only the *selected* plan is
//! ever encoded, so artifact size could only ever be stated for the winner and
//! never for the alternatives this model exists to compare. A component that is
//! structurally unstateable for every candidate but one cannot inform a choice
//! between them, and calibrating it would mean calibrating against a single
//! self-selected sample. Report artifact size where the artifact is produced. A formula invented to fill
//! one of them would be unfalsifiable at exactly the moment it mattered. An
//! honest `Unknown` is a measurement boundary; a fabricated number is a defect
//! that reads as evidence.
//!
//! Six came off the unreachable list after the source was re-read rather than
//! the note re-trusted, and each time the data was closer to hand than the note
//! claimed. Treat the remaining three the same way.
//!
//! The vocabulary is nine, differing from the accepted nine in two deliberate,
//! recorded moves — artifact size was removed (only the selected plan is ever
//! encoded, so no candidate could state it) and threadgroup memory was
//! split out of `ResourcePressure`, which keeps `Unknown` until registers and an
//! occupancy model exist. Folding bytes into a component whose unit is
//! `Registers` would have been a unit lie, and units here are contract.

use crate::region::SemanticMemberId;
use crate::selection::SelectedPlan;
use core::fmt;
use std::collections::BTreeSet;
use tiler_ir::schedule::element_count;

/// The governed key naming this cost model.
///
/// Distinct from `tiler.cost.structural.v1` by construction: nothing attributed
/// to this key may enter a structural dominance comparison.
pub(crate) const ANALYTICAL_MODEL_KEY: &str = "tiler.cost.analytical.v1";

/// One governed dimension of analytical plan cost.
///
/// The vocabulary is bounded and closed, for the same reason the boundary
/// property axes are: a free-form cost bag cannot be compared, explained, or
/// calibrated. Nine components: the accepted ticket's nine, less artifact size
/// (removed — unstateable for every candidate but the winner), plus threadgroup
/// memory (split out of resource pressure; bytes, not registers).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum CostComponent {
    /// Bytes moved between the device and its memory.
    MemoryTraffic,
    /// Bytes of temporary storage the plan allocates.
    Allocation,
    /// Dispatches the plan issues.
    Dispatch,
    /// Work performed more than once across fused regions.
    RedundantWork,
    /// Address arithmetic performed per element.
    Indexing,
    /// Synchronization the plan requires between dispatches.
    Synchronization,
    /// Register and threadgroup-memory pressure, and the occupancy it implies.
    ResourcePressure,
    /// Peak threadgroup memory any one of the plan's dispatches requires.
    ///
    /// Split out from [`Self::ResourcePressure`] because it is exact today while
    /// registers and occupancy are not, and because it is measured in bytes
    /// rather than registers. Folding it into that component would have meant
    /// reporting bytes under a `Registers` unit, and a number in the wrong unit
    /// is what a calibration pass silently trusts.
    ThreadgroupMemory,
    /// Compiler time the plan costs to produce.
    CompileTime,
}

/// The canonical component order: the single source of truth for evaluation,
/// encoding, and reporting order, matching the derived [`CostComponent`]
/// ordering.
pub(crate) const CANONICAL_COMPONENTS: [CostComponent; 9] = [
    CostComponent::MemoryTraffic,
    CostComponent::Allocation,
    CostComponent::Dispatch,
    CostComponent::RedundantWork,
    CostComponent::Indexing,
    CostComponent::Synchronization,
    CostComponent::ResourcePressure,
    CostComponent::ThreadgroupMemory,
    CostComponent::CompileTime,
];

impl CostComponent {
    /// The governed canonical key naming this component in explain output.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::MemoryTraffic => "cost.memory-traffic",
            Self::Allocation => "cost.allocation",
            Self::Dispatch => "cost.dispatch",
            Self::RedundantWork => "cost.redundant-work",
            Self::Indexing => "cost.indexing",
            Self::Synchronization => "cost.synchronization",
            Self::ResourcePressure => "cost.resource-pressure",
            Self::ThreadgroupMemory => "cost.threadgroup-memory",
            Self::CompileTime => "cost.compile-time",
        }
    }

    /// The unit this component is always expressed in.
    ///
    /// Fixed per component rather than carried per value, so a component and a
    /// unit cannot disagree. Written as an exhaustive match so a tenth component
    /// is a build error here rather than a value with no stated unit.
    pub(crate) const fn unit(self) -> CostUnit {
        match self {
            Self::MemoryTraffic | Self::Allocation | Self::ThreadgroupMemory => CostUnit::Bytes,
            Self::Dispatch | Self::Synchronization => CostUnit::Count,
            Self::RedundantWork | Self::Indexing => CostUnit::Operations,
            Self::ResourcePressure => CostUnit::Registers,
            Self::CompileTime => CostUnit::Nanoseconds,
        }
    }
}

impl fmt::Display for CostComponent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

/// The unit an analytical component cost is expressed in.
///
/// Units are part of the contract, not documentation: an uncalibrated model
/// whose numbers have no stated unit cannot be calibrated later, because nothing
/// says what the device measurement should be compared against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CostUnit {
    /// Bytes.
    Bytes,
    /// A dimensionless count of discrete events.
    Count,
    /// Abstract scalar operations.
    Operations,
    /// Registers per thread.
    Registers,
    /// Nanoseconds.
    Nanoseconds,
}

impl CostUnit {
    /// The governed canonical key naming this unit.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Bytes => "bytes",
            Self::Count => "count",
            Self::Operations => "operations",
            Self::Registers => "registers",
            Self::Nanoseconds => "nanoseconds",
        }
    }
}

impl fmt::Display for CostUnit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

/// What is known about a component's value.
///
/// The three cases are kept apart because they are different evidence classes,
/// not three confidences on one scale. Collapsing them into a number with an
/// error bar would make an unmodelled component indistinguishable from one
/// modelled imprecisely, and only the second is safe to calibrate against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CostValue {
    /// An exact count derived from values the plan already carries.
    Exact(u64),
    /// A modelled estimate, sound within the stated inclusive bounds.
    Bounded {
        /// The lowest value the model admits.
        low: u64,
        /// The highest value the model admits.
        high: u64,
    },
    /// The compiler does not model this component yet.
    ///
    /// This is a measurement boundary, not a zero and not an infinity. A caller
    /// must not substitute either.
    Unknown,
}

impl CostValue {
    /// The stable code naming this evidence class.
    pub(crate) const fn class(self) -> &'static str {
        match self {
            Self::Exact(_) => "exact",
            Self::Bounded { .. } => "bounded",
            Self::Unknown => "unknown",
        }
    }

    /// Whether this value is modelled at all.
    pub(crate) const fn is_known(self) -> bool {
        match self {
            Self::Exact(_) | Self::Bounded { .. } => true,
            Self::Unknown => false,
        }
    }

    /// Whether this value is well formed.
    ///
    /// A bounded value whose low exceeds its high states an empty range, which
    /// no measurement could ever fall in; that is a malformed model rather than
    /// a wide one.
    const fn is_well_formed(self) -> bool {
        match self {
            Self::Exact(_) | Self::Unknown => true,
            Self::Bounded { low, high } => low <= high,
        }
    }
}

impl fmt::Display for CostValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(value) => write!(formatter, "{value}"),
            Self::Bounded { low, high } => write!(formatter, "{low}..={high}"),
            Self::Unknown => formatter.write_str("unknown"),
        }
    }
}

/// One component's analytical cost, with its unit and evidence class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ComponentCost {
    component: CostComponent,
    value: CostValue,
}

impl ComponentCost {
    /// The component this cost is for.
    pub(crate) const fn component(&self) -> CostComponent {
        self.component
    }

    /// The value, and what class of evidence stands behind it.
    pub(crate) const fn value(&self) -> CostValue {
        self.value
    }

    /// The unit, read from the component rather than stored.
    pub(crate) const fn unit(&self) -> CostUnit {
        self.component.unit()
    }
}

impl fmt::Display for ComponentCost {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {} {}",
            self.component,
            self.value,
            self.unit()
        )
    }
}

/// The complete analytical cost of one plan: every governed component, in
/// canonical order, each with its evidence class.
///
/// Every component is always present. A component the compiler does not model
/// appears as [`CostValue::Unknown`] rather than being omitted, because an
/// absent entry and an unmodelled one are indistinguishable to a reader, and a
/// later calibration pass needs to know which of the nine it is still missing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AnalyticalPlanCost {
    components: Vec<ComponentCost>,
}

impl AnalyticalPlanCost {
    /// The components, in canonical order.
    pub(crate) fn components(&self) -> &[ComponentCost] {
        &self.components
    }

    /// The cost recorded for one component.
    ///
    /// Never `None` for a governed component: the constructor emits all nine.
    pub(crate) fn get(&self, component: CostComponent) -> Option<&ComponentCost> {
        self.components
            .iter()
            .find(|cost| cost.component == component)
    }

    /// How many components are modelled rather than `Unknown`.
    ///
    /// This is the number `calibrate-device-cost-models` needs in order to say
    /// what it has left to do, and the number the test below pins so a component
    /// cannot quietly start returning a fabricated value.
    pub(crate) fn known_count(&self) -> usize {
        self.components
            .iter()
            .filter(|cost| cost.value.is_known())
            .count()
    }
}

/// Computes the analytical component costs of one complete plan.
///
/// Deterministic and total: it reads only values the plan already carries, in
/// canonical component order, and allocates one vector.
///
/// # Panics
///
/// Panics only if a component produced a malformed value — a bounded range whose
/// low exceeds its high. No arithmetic here can produce one, so this is an
/// assertion about this function rather than a reachable input condition.
pub(crate) fn analytical_plan_cost(plan: &SelectedPlan) -> AnalyticalPlanCost {
    let components = CANONICAL_COMPONENTS
        .into_iter()
        .map(|component| {
            let value = match component {
                // Two components the plan already carries exactly. Every region's
                // implementation states the temporary bytes it needs and the
                // dispatches it issues, and the plan's totals are their sums.
                //
                // Saturating rather than checked because this is a report, not a
                // feasibility input — a saturated total is visibly wrong at
                // `u64::MAX`, whereas a rejected compile would let a reporting
                // path fail a build.
                CostComponent::Allocation => {
                    CostValue::Exact(plan.selections().iter().fold(0_u64, |total, selection| {
                        total.saturating_add(selection.implementation().cost().temporary_bytes())
                    }))
                }
                // Dispatch duplicates a count the structural model already
                // reports, and that is the point rather than an oversight. A
                // calibration pass compares a device measurement against this
                // model component by component, so a component missing here
                // cannot be correlated with anything measured — and dispatch
                // overhead is one of the first things a device measurement sees.
                // The structural count exists to be *pruned* on; this one exists
                // to be *calibrated* against, and the two uses do not share a
                // consumer.
                CostComponent::Dispatch => {
                    CostValue::Exact(plan.selections().iter().fold(0_u64, |total, selection| {
                        total.saturating_add(u64::from(
                            selection.implementation().cost().dispatch_count(),
                        ))
                    }))
                }
                // Exactly the ordering constraints the plan requires between
                // dispatches: one per (producer, consumer) pair across the
                // satisfied cross-region handoffs. Every consumer of a handoff
                // requires `AvailabilityRequirement::AfterProducingDispatch`,
                // discharged by the producer's `AfterOwnDispatch`, so each pair
                // is one edge that must be ordered.
                //
                // Counted per consumer rather than per handoff because a handoff
                // with three consumers imposes three waits, not one. State that
                // explicitly so the count can be refuted: if a target ever
                // orders a whole handoff with a single barrier, this becomes an
                // upper bound and should be restated as `Bounded` rather than
                // quietly redefined.
                CostComponent::Synchronization => {
                    CostValue::Exact(plan.handoffs().iter().fold(0_u64, |total, handoff| {
                        total.saturating_add(
                            u64::try_from(handoff.consumers().len()).unwrap_or(u64::MAX),
                        )
                    }))
                }
                // One address computation per logical access per iteration
                // point. A region's `IndexRegion` states both: `accesses` is the
                // read/write list the schedule refines, and `iteration_shape` is
                // the parallel domain each is evaluated over.
                //
                // Unlike the three above this can *fail* to have a value: an
                // iteration shape whose element count overflows `u64` has no
                // stateable total, and the whole component then reports
                // `Unknown` rather than a saturated one. A saturated total here
                // would be a number a calibration pass could compare against and
                // silently disagree with; `Unknown` is the honest answer and the
                // one the evidence classes exist to express.
                // An opaque call has no index region — no iteration domain and
                // no access list — so a plan containing one reports `Unknown`.
                // **Not zero**: a plan whose indexing cost silently became zero
                // would be ranked as free.
                CostComponent::Indexing => plan
                    .selections()
                    .iter()
                    .try_fold(0_u64, |total, selection| {
                        let region = &selection.implementation().scheduled()?.region().index;
                        let points = element_count(&region.iteration_shape).ok()?;
                        let accesses = u64::try_from(region.accesses.len()).ok()?;
                        total.checked_add(points.checked_mul(accesses)?)
                    })
                    .map_or(CostValue::Unknown, CostValue::Exact),
                // Not modelled. See this module's header: the inputs do not
                // exist yet, and inventing them would produce numbers that
                // cannot be refuted.
                // Work a fused cover performs more than once: a semantic member
                // appearing in more than one region of the cover is computed in
                // each of them.
                //
                // **The weighting is a definitional choice, not a derivation, so
                // it is stated here to be refuted.** A member's *first* region in
                // canonical order is treated as the original and contributes
                // nothing; every later region containing it contributes that
                // region's own iteration points. The alternative — counting bare
                // occurrences, unweighted — is equally exact and gives a
                // different number, but it would not be comparable with
                // `Indexing`, which is element-weighted. Regions are visited in
                // the plan's canonical selection order, so "first" is
                // deterministic rather than dependent on enumeration.
                //
                // Reports `Unknown` on an overflowing element count, for the same
                // reason `Indexing` does.
                //
                // **`Exact(0)` is forced by the cover authority, not observed on
                // the suite's inputs.** `verify_cover` counts coverage per
                // operation over the re-derived candidates and rejects >1 as
                // `CoverError::IllegalDuplication`, unconditionally, for every
                // cover behind every retained plan — and the enumerator cannot
                // build an overlapping cover in the first place. So the
                // `seen`-set arithmetic below runs on every compile and its
                // non-zero outcome is unreachable until the cover *contract*
                // changes: the reserved `CoverDuplication` seam, i.e. relaxing
                // `IllegalDuplication` under an explicit duplication policy.
                // The trigger is a contract change, not a new input program —
                // whoever relaxes it should check this value moves.
                CostComponent::RedundantWork => {
                    let mut seen: BTreeSet<SemanticMemberId> = BTreeSet::new();
                    plan.selections()
                        .iter()
                        .try_fold(0_u64, |total, selection| {
                            let verified = selection.implementation().scheduled()?;
                            let points =
                                element_count(&verified.region().index.iteration_shape).ok()?;
                            let repeated = verified
                                .semantic_members()
                                .iter()
                                .filter(|member| !seen.insert(**member))
                                .count();
                            total.checked_add(points.checked_mul(u64::try_from(repeated).ok()?)?)
                        })
                        .map_or(CostValue::Unknown, CostValue::Exact)
                }
                // Bytes moved, bounded rather than exact because the plan does
                // not model cache reuse. The low bound counts only owning
                // writes, since no amount of reuse eliminates a store; the high
                // bound counts every access, which is the no-reuse-at-all case.
                // A write is identified by `Access::ownership` being present,
                // which that field documents as holding "only for owning
                // writes" — read from the witness rather than inferred.
                //
                // **The element width is derived fail-closed.** `IndexRegion`
                // carries no dtype; it carries `numerical.profile_key`, naming
                // the governing contract. A recognized f32 contract implies four
                // bytes and anything else declines to answer, so a widened dtype
                // vocabulary arrives with a new key, falls to the wildcard, and
                // reports `Unknown` instead of silently continuing to multiply by
                // four. Deliberately *not* inferred from
                // `canonical_arithmetic_nan_bits` being 32 bits wide, which would
                // read meaning out of a field's type and happen to be right for
                // the wrong reason.
                // `low <= high` by construction: writes are a subset of all
                // accesses and both multiply the same non-negative byte count,
                // so the pair is always a well-formed bound. Recorded so the
                // derivation is not re-litigated.
                CostComponent::MemoryTraffic => {
                    let bounds = plan.selections().iter().try_fold(
                        (0_u64, 0_u64),
                        |(low, high), selection| {
                            let region = &selection.implementation().scheduled()?.region().index;
                            let width = match region.numerical.profile_key {
                                crate::request::NUMERICAL_CONTRACT_KEY
                                | crate::request::FLUSH_CONTRACT_KEY
                                | crate::request::RELAXED_CONTRACT_KEY => 4_u64,
                                _ => return None,
                            };
                            let points = element_count(&region.iteration_shape).ok()?;
                            let bytes = points.checked_mul(width)?;
                            let writes = u64::try_from(
                                region
                                    .accesses
                                    .iter()
                                    .filter(|access| access.ownership.is_some())
                                    .count(),
                            )
                            .ok()?;
                            let all = u64::try_from(region.accesses.len()).ok()?;
                            Some((
                                low.checked_add(bytes.checked_mul(writes)?)?,
                                high.checked_add(bytes.checked_mul(all)?)?,
                            ))
                        },
                    );
                    bounds.map_or(CostValue::Unknown, |(low, high)| CostValue::Bounded {
                        low,
                        high,
                    })
                }
                // The **peak** any single dispatch requires, not a sum.
                // Threadgroup memory is held for the duration of one dispatch
                // and released, so regions dispatched in sequence do not hold
                // theirs simultaneously; summing would report a plan as needing
                // memory no point in its execution ever needs at once. The peak
                // is what a device limit is actually checked against, which is
                // why `target.local-memory-bytes` is assessed per region rather
                // than against a total.
                //
                // **`Exact(0)` everywhere, and by derivation rather than by
                // input**: the bounded profile's only requirements derivation
                // states zero local memory unconditionally, so this is a
                // property of the derivation, not something the suite's inputs
                // happen to avoid. The peak-versus-sum choice is therefore
                // *correct but untested* — with every region at zero, `max` and
                // a sum are indistinguishable — and the first requirements
                // derivation that states local memory is what exercises it.
                CostComponent::ThreadgroupMemory => CostValue::Exact(
                    plan.selections()
                        .iter()
                        .map(|selection| selection.implementation().resources().local_memory_bytes)
                        .max()
                        .unwrap_or(0),
                ),
                CostComponent::ResourcePressure | CostComponent::CompileTime => CostValue::Unknown,
            };
            assert!(
                value.is_well_formed(),
                "{component} produced a malformed value"
            );
            ComponentCost { component, value }
        })
        .collect();
    AnalyticalPlanCost { components }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every governed component is emitted, in canonical order, with a unit.
    ///
    /// The order assertion is what makes explain output stable across runs, and
    /// the completeness assertion is what stops a component being silently
    /// dropped rather than reported `Unknown`.
    #[test]
    fn every_governed_component_is_emitted_in_canonical_order() {
        let emitted: Vec<CostComponent> = CANONICAL_COMPONENTS.into_iter().collect();
        let mut sorted = emitted.clone();
        sorted.sort_unstable();
        assert_eq!(
            emitted, sorted,
            "the canonical list is not in the derived ordering, so encoding and \
             reporting order disagree with it"
        );
        assert_eq!(
            CANONICAL_COMPONENTS.len(),
            9,
            "the accepted ticket's nine, less artifact size which no plan can state, \
             plus threadgroup memory split out of resource pressure"
        );
    }

    /// Keys and units are distinct and total.
    ///
    /// A duplicated key would make two components indistinguishable in explain
    /// output, which is the failure a reader cannot see.
    #[test]
    fn component_keys_are_distinct() {
        let mut keys: Vec<&str> = CANONICAL_COMPONENTS
            .into_iter()
            .map(CostComponent::key)
            .collect();
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();
        assert_eq!(before, keys.len(), "two components share a key");
        for component in CANONICAL_COMPONENTS {
            assert!(
                !component.unit().key().is_empty(),
                "{component} has no stated unit"
            );
        }
    }

    /// `Unknown` is neither zero nor known.
    ///
    /// This is the substitution the module exists to prevent, so it is asserted
    /// rather than left to the reader: a caller that treated `Unknown` as `0`
    /// would report a plan as free.
    #[test]
    fn unknown_is_not_a_zero() {
        assert!(!CostValue::Unknown.is_known());
        assert!(CostValue::Exact(0).is_known());
        assert_ne!(CostValue::Unknown, CostValue::Exact(0));
        assert_eq!(CostValue::Unknown.class(), "unknown");
        assert_eq!(CostValue::Exact(0).class(), "exact");
    }

    /// The well-formedness check can say no.
    ///
    /// An inverted bounded range must be rejected; a test that only ever built
    /// well-formed values would pass against a predicate that returned `true`
    /// unconditionally.
    #[test]
    fn an_inverted_bounded_range_is_malformed() {
        assert!(CostValue::Bounded { low: 2, high: 9 }.is_well_formed());
        assert!(CostValue::Bounded { low: 4, high: 4 }.is_well_formed());
        assert!(
            !CostValue::Bounded { low: 9, high: 2 }.is_well_formed(),
            "an empty range no measurement can fall in must be malformed"
        );
    }
}
