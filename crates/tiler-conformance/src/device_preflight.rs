//! What a host owes itself **before** a routing commit, and how each refusal is
//! classified.
//!
//! # Why every comparison here is device-free
//!
//! A device contributes numbers — a pipeline's threadgroup capacity, a device's
//! threadgroup memory, a buffer-length bound, the length an allocation came back
//! with — and this module contributes the comparison and the classification. The
//! split is what lets every case, including the ones no machine in this
//! workspace can produce, run in the ordinary gate on any host. A comparison
//! written inside the device call could only be exercised where the device
//! exists, which is exactly the coverage that was missing.
//!
//! # Why the phase and class are typed rather than rendered
//!
//! A host that cannot tell these apart either retries work that can never
//! succeed or abandons an artifact that had a working route. They are a
//! contract, not a diagnostic convenience: ADR 0051 permits a fallback only
//! before the commit, so a refusal's class is what decides whether the fallback
//! that is still held should be taken.

use std::fmt;

/// Which stage of a device preflight reached a decision.
///
/// Ordered as they run, and the order is the useful one: a refusal names the
/// earliest obligation that failed, so a library that will not load is never
/// reported as a launch-geometry problem.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreflightPhase {
    /// Building an executable library from a payload's object bytes.
    Library,
    /// Resolving the entry symbol a payload names.
    Function,
    /// Creating compute pipeline state for a resolved function.
    Pipeline,
    /// Comparing a declared launch against what the pipeline admits.
    LaunchGeometry,
    /// Allocating and sizing every bound buffer and every internal scratch slot.
    Resources,
}

impl PreflightPhase {
    /// A stable lowercase identifier for this stage.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Function => "function",
            Self::Pipeline => "pipeline",
            Self::LaunchGeometry => "launch-geometry",
            Self::Resources => "resources",
        }
    }
}

/// What a caller should do about a refusal, which is why phases are typed at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreflightClass {
    /// This route does not fit *this device*, and another variant might.
    ///
    /// A fallback is permitted and is the indicated response. Every refusal in
    /// this class compares something the artifact declared against something the
    /// device reported, so a differently-declared variant is exactly the remedy.
    RouteMiss,
    /// These bytes passed decode and integrity validation and still do not yield
    /// a runnable library.
    ///
    /// Distinct from an integrity failure, which the codec already refused
    /// before any of this ran: the digest matched, so the object *is* what the
    /// producer published, and it is content that will not execute. A caller
    /// re-fetches or rebuilds; retrying another variant of the same bytes is not
    /// indicated.
    CorruptArtifact,
    /// The host cannot serve any route, whatever it declares.
    Systemic,
}

impl PreflightClass {
    /// A stable lowercase identifier for this class.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::RouteMiss => "route-miss",
            Self::CorruptArtifact => "corrupt-artifact",
            Self::Systemic => "systemic",
        }
    }
}

/// One refusal a device preflight reached, before any commit.
///
/// Carries the numbers the decision was made from rather than a rendered
/// sentence, so [`Self::phase`] and [`Self::class`] are total functions over the
/// variant and a caller acts on the class without parsing anything.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PreflightRefusal {
    /// A payload's object bytes did not produce a library.
    LibraryRejected {
        /// Position of the entry whose object was rejected.
        entry: usize,
        /// What the binding said.
        detail: String,
    },
    /// The library loaded and publishes no function by the entry symbol.
    FunctionAbsent {
        /// Position of the entry whose symbol was absent.
        entry: usize,
        /// The symbol that was looked up.
        symbol: String,
        /// What the binding said.
        detail: String,
    },
    /// The device refused pipeline state for a function it did publish.
    PipelineRejected {
        /// Position of the entry whose pipeline was refused.
        entry: usize,
        /// The symbol the pipeline was asked for.
        symbol: String,
        /// What the binding said.
        detail: String,
    },
    /// The declared workgroup is larger than this pipeline admits.
    WorkgroupTooLarge {
        /// Position of the entry whose launch was refused.
        entry: usize,
        /// The symbol whose pipeline was asked.
        symbol: String,
        /// Threads per workgroup the declaration states.
        declared: u64,
        /// The maximum this prepared pipeline admits.
        capacity: u64,
    },
    /// An entry reserves more threadgroup memory than this device admits.
    ///
    /// **The one derived requirement that had no reader.**
    /// `crates/tiler-artifact/src/program/requirement.rs` states that
    /// threadgroup memory is deliberately absent from the neutral
    /// `RouteResourceDimension` vocabulary because the requirement side is
    /// already stated — by `ResourceRequirements::local_memory_bytes` — and is
    /// "checked directly against the device by an adapter". A cooperative
    /// reduction is the first strategy that reserves any, so the gap became
    /// reachable at the same moment a plan that uses it did.
    ThreadgroupMemoryExceeded {
        /// Position of the entry whose reservation was refused.
        entry: usize,
        /// The symbol whose entry reserves it.
        symbol: String,
        /// Bytes of threadgroup memory the entry reserves.
        declared: u64,
        /// Bytes this device admits per threadgroup.
        capacity: u64,
    },
    /// A binding must reach more bytes than one buffer can hold here.
    BindingExceedsBufferLimit {
        /// Position of the entry whose binding was refused.
        entry: usize,
        /// The ABI slot that binding occupies.
        slot: usize,
        /// Bytes the binding must reach.
        needed: u64,
        /// Bytes this device holds in one buffer.
        limit: u64,
    },
    /// An allocation came back shorter than the route requires.
    UndersizedAllocation {
        /// Position of the entry whose allocation was short.
        entry: usize,
        /// The ABI slot that allocation serves.
        slot: usize,
        /// Bytes the route requires.
        needed: u64,
        /// Bytes the allocation actually holds.
        held: u64,
    },
    /// No entry of the route binds the program output a run compares.
    ///
    /// Systemic rather than a route miss: placement already refused every
    /// binding target the run does not place, so a route that reaches here
    /// declares an interface the run cannot observe at all.
    NoOutputBinding,
    /// A routed slot takes a program input the caller supplied no operands for.
    ///
    /// Systemic, and an assertion against the run's own composition rather than
    /// against the artifact: the ordinal was resolved from the same declared
    /// interface the operand set is built from, so reaching this means a caller
    /// passed an operand set of the wrong arity. It is a typed refusal rather
    /// than an index panic because a wrong-arity operand set would otherwise
    /// either abort or, worse, silently reuse operand zero for every input.
    UnsuppliedOperand {
        /// Position of the entry whose slot was unsupplied.
        entry: usize,
        /// The ABI slot that takes the input.
        slot: usize,
        /// The declared-interface ordinal that slot resolves to.
        ordinal: usize,
        /// How many operand sets the caller supplied.
        supplied: usize,
    },
}

impl PreflightRefusal {
    /// The stage this refusal came from.
    ///
    /// Exhaustive rather than a wildcard, so a refusal added later is placed in
    /// a stage deliberately instead of inheriting whichever one a catch-all
    /// named.
    pub(crate) const fn phase(&self) -> PreflightPhase {
        match self {
            Self::LibraryRejected { .. } => PreflightPhase::Library,
            Self::FunctionAbsent { .. } => PreflightPhase::Function,
            Self::PipelineRejected { .. } => PreflightPhase::Pipeline,
            Self::WorkgroupTooLarge { .. } => PreflightPhase::LaunchGeometry,
            // A resource stage rather than a launch-geometry one: the quantity
            // is storage the entry reserves, and it is compared against a device
            // capacity rather than against a pipeline's thread capacity.
            Self::ThreadgroupMemoryExceeded { .. }
            | Self::BindingExceedsBufferLimit { .. }
            | Self::UndersizedAllocation { .. }
            | Self::NoOutputBinding
            | Self::UnsuppliedOperand { .. } => PreflightPhase::Resources,
        }
    }

    /// What a caller should do about this refusal.
    ///
    /// **`PipelineRejected` is a route miss, and the direction is derived rather
    /// than guessed.** Metal reports pipeline-creation failure as a message
    /// string that does not reliably separate "this function exceeds a device
    /// limit" from "the device is out of resources". Of the two ways to be
    /// wrong, calling a systemic failure a route miss costs a retry that then
    /// fails; calling a route miss systemic abandons an artifact that had a
    /// working variant. Only the second forfeits the fallback ADR 0051 grants
    /// while it is still held, so the classification takes the recoverable
    /// direction.
    ///
    /// `UndersizedAllocation` is systemic rather than a route miss because it is
    /// an assertion against the device's own report — every buffer is requested
    /// at the length the route states — so reaching it means the allocator did
    /// not honour a request it accepted, which no other variant improves.
    pub(crate) const fn class(&self) -> PreflightClass {
        match self {
            Self::LibraryRejected { .. } | Self::FunctionAbsent { .. } => {
                PreflightClass::CorruptArtifact
            }
            Self::PipelineRejected { .. }
            | Self::WorkgroupTooLarge { .. }
            | Self::ThreadgroupMemoryExceeded { .. }
            | Self::BindingExceedsBufferLimit { .. } => PreflightClass::RouteMiss,
            Self::UndersizedAllocation { .. }
            | Self::NoOutputBinding
            | Self::UnsuppliedOperand { .. } => PreflightClass::Systemic,
        }
    }
}

impl fmt::Display for PreflightRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/{}: ",
            self.phase().as_str(),
            self.class().as_str(),
        )?;
        match self {
            Self::LibraryRejected { entry, detail } => write!(
                formatter,
                "entry {entry}'s carried object did not load: {detail}",
            ),
            Self::FunctionAbsent {
                entry,
                symbol,
                detail,
            } => write!(
                formatter,
                "entry {entry}'s library publishes no {symbol:?}: {detail}",
            ),
            Self::PipelineRejected {
                entry,
                symbol,
                detail,
            } => write!(
                formatter,
                "no pipeline state for entry {entry}'s {symbol:?}: {detail}",
            ),
            Self::WorkgroupTooLarge {
                entry,
                symbol,
                declared,
                capacity,
            } => write!(
                formatter,
                "entry {entry}'s {symbol:?} admits {capacity} thread(s) per threadgroup and the \
                 declaration states {declared}",
            ),
            Self::ThreadgroupMemoryExceeded {
                entry,
                symbol,
                declared,
                capacity,
            } => write!(
                formatter,
                "entry {entry}'s {symbol:?} reserves {declared} byte(s) of threadgroup memory and \
                 this device admits {capacity}",
            ),
            Self::BindingExceedsBufferLimit {
                entry,
                slot,
                needed,
                limit,
            } => write!(
                formatter,
                "entry {entry} slot {slot} must reach {needed} byte(s) and one buffer holds at \
                 most {limit}",
            ),
            Self::UndersizedAllocation {
                entry,
                slot,
                needed,
                held,
            } => write!(
                formatter,
                "entry {entry} slot {slot} needs {needed} byte(s) and the allocation returned \
                 {held}",
            ),
            Self::NoOutputBinding => {
                formatter.write_str("no entry of this route binds the program output")
            }
            Self::UnsuppliedOperand {
                entry,
                slot,
                ordinal,
                supplied,
            } => write!(
                formatter,
                "entry {entry} slot {slot} takes declared input {ordinal} and this run supplied \
                 {supplied} operand set(s)",
            ),
        }
    }
}

impl std::error::Error for PreflightRefusal {}

/// Whether a declared workgroup fits what a pipeline admits.
///
/// # Errors
///
/// Returns [`PreflightRefusal::WorkgroupTooLarge`] when it does not.
pub(crate) fn workgroup_fits(
    entry: usize,
    symbol: &str,
    declared: u64,
    capacity: u64,
) -> Result<(), PreflightRefusal> {
    if declared > capacity {
        return Err(PreflightRefusal::WorkgroupTooLarge {
            entry,
            symbol: symbol.to_owned(),
            declared,
            capacity,
        });
    }
    Ok(())
}

/// Whether one entry's reserved threadgroup memory fits what a device admits.
///
/// The relation is `declared > capacity`, not `>=`: a function reserving exactly
/// the device maximum fits, and refusing it would reject a legal route.
///
/// # Errors
///
/// Returns [`PreflightRefusal::ThreadgroupMemoryExceeded`] when it does not.
pub(crate) fn local_memory_fits(
    entry: usize,
    symbol: &str,
    declared: u64,
    capacity: u64,
) -> Result<(), PreflightRefusal> {
    if declared > capacity {
        return Err(PreflightRefusal::ThreadgroupMemoryExceeded {
            entry,
            symbol: symbol.to_owned(),
            declared,
            capacity,
        });
    }
    Ok(())
}

/// Whether one binding's accessible range fits in a single buffer here.
///
/// # Errors
///
/// Returns [`PreflightRefusal::BindingExceedsBufferLimit`] when it does not.
pub(crate) fn binding_fits(
    entry: usize,
    slot: usize,
    needed: u64,
    limit: u64,
) -> Result<(), PreflightRefusal> {
    if needed > limit {
        return Err(PreflightRefusal::BindingExceedsBufferLimit {
            entry,
            slot,
            needed,
            limit,
        });
    }
    Ok(())
}

/// Whether an allocation a device returned reaches the length it was asked for.
///
/// # Errors
///
/// Returns [`PreflightRefusal::UndersizedAllocation`] when it does not.
pub(crate) fn allocation_fits(
    entry: usize,
    slot: usize,
    needed: u64,
    held: u64,
) -> Result<(), PreflightRefusal> {
    if held < needed {
        return Err(PreflightRefusal::UndersizedAllocation {
            entry,
            slot,
            needed,
            held,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        PreflightClass, PreflightPhase, PreflightRefusal, allocation_fits, binding_fits,
        local_memory_fits, workgroup_fits,
    };

    /// Every device-preflight refusal lands in the phase and class it claims.
    ///
    /// The classification is what a caller acts on — re-route, re-fetch, or stop
    /// — so a refusal filed under the wrong class is a wrong instruction rather
    /// than a wrong label. Each variant is listed explicitly rather than derived
    /// from the functions under test, so a variant that silently changed class
    /// fails here instead of agreeing with itself.
    #[test]
    fn each_device_preflight_refusal_carries_its_phase_and_class() {
        let cases = [
            (
                PreflightRefusal::LibraryRejected {
                    entry: 0,
                    detail: "not a metallib".to_owned(),
                },
                PreflightPhase::Library,
                PreflightClass::CorruptArtifact,
            ),
            (
                PreflightRefusal::FunctionAbsent {
                    entry: 0,
                    symbol: "absent".to_owned(),
                    detail: "no such function".to_owned(),
                },
                PreflightPhase::Function,
                PreflightClass::CorruptArtifact,
            ),
            (
                PreflightRefusal::PipelineRejected {
                    entry: 1,
                    symbol: "k".to_owned(),
                    detail: "too many registers".to_owned(),
                },
                PreflightPhase::Pipeline,
                PreflightClass::RouteMiss,
            ),
            (
                PreflightRefusal::WorkgroupTooLarge {
                    entry: 1,
                    symbol: "k".to_owned(),
                    declared: 2,
                    capacity: 1,
                },
                PreflightPhase::LaunchGeometry,
                PreflightClass::RouteMiss,
            ),
            (
                PreflightRefusal::ThreadgroupMemoryExceeded {
                    entry: 1,
                    symbol: "k".to_owned(),
                    declared: 2,
                    capacity: 1,
                },
                PreflightPhase::Resources,
                PreflightClass::RouteMiss,
            ),
            (
                PreflightRefusal::BindingExceedsBufferLimit {
                    entry: 1,
                    slot: 0,
                    needed: 2,
                    limit: 1,
                },
                PreflightPhase::Resources,
                PreflightClass::RouteMiss,
            ),
            (
                PreflightRefusal::UndersizedAllocation {
                    entry: 0,
                    slot: 0,
                    needed: 2,
                    held: 1,
                },
                PreflightPhase::Resources,
                PreflightClass::Systemic,
            ),
            (
                PreflightRefusal::NoOutputBinding,
                PreflightPhase::Resources,
                PreflightClass::Systemic,
            ),
            (
                PreflightRefusal::UnsuppliedOperand {
                    entry: 0,
                    slot: 1,
                    ordinal: 1,
                    supplied: 1,
                },
                PreflightPhase::Resources,
                PreflightClass::Systemic,
            ),
        ];
        assert_eq!(cases.len(), 9, "a refusal was added without a case here");
        for (refusal, phase, class) in cases {
            assert_eq!(refusal.phase(), phase, "wrong phase for {refusal}");
            assert_eq!(refusal.class(), class, "wrong class for {refusal}");
            // The rendered form leads with both, because a log line that does
            // not carry the class makes the reader infer what the type states.
            let rendered = refusal.to_string();
            assert!(
                rendered.starts_with(&format!("{}/{}: ", phase.as_str(), class.as_str())),
                "the rendering drops the phase or the class: {rendered}",
            );
        }
    }

    /// The four comparisons refuse exactly at their boundary, not near it.
    ///
    /// Each is tested at the largest accepted value and the smallest refused
    /// one, because an off-by-one here either rejects a route the device would
    /// have run or admits one it cannot — and the second is the failure the
    /// whole stage exists to move before the commit.
    #[test]
    fn the_device_comparisons_refuse_exactly_at_their_boundary() {
        workgroup_fits(1, "k", 1024, 1024).expect("a workgroup at capacity fits");
        assert!(matches!(
            workgroup_fits(1, "k", 1025, 1024),
            Err(PreflightRefusal::WorkgroupTooLarge {
                entry: 1,
                declared: 1025,
                capacity: 1024,
                ..
            })
        ));

        // Zero is the ordinary case rather than an edge one: every
        // non-cooperative entry reserves no threadgroup memory, so a comparison
        // that refused it would refuse the serial fold on every device.
        local_memory_fits(1, "k", 0, 0).expect("an entry reserving nothing fits");
        local_memory_fits(1, "k", 32_768, 32_768).expect("an entry at the device maximum fits");
        assert!(matches!(
            local_memory_fits(1, "k", 32_769, 32_768),
            Err(PreflightRefusal::ThreadgroupMemoryExceeded {
                entry: 1,
                declared: 32_769,
                capacity: 32_768,
                ..
            })
        ));

        binding_fits(1, 0, 4096, 4096).expect("a binding at the limit fits");
        assert!(matches!(
            binding_fits(1, 0, 4097, 4096),
            Err(PreflightRefusal::BindingExceedsBufferLimit {
                entry: 1,
                slot: 0,
                needed: 4097,
                limit: 4096,
            })
        ));

        allocation_fits(1, 0, 48, 48).expect("an allocation of exactly the needed length fits");
        allocation_fits(1, 0, 48, 64).expect("a longer allocation fits");
        assert!(matches!(
            allocation_fits(1, 0, 48, 47),
            Err(PreflightRefusal::UndersizedAllocation {
                entry: 1,
                slot: 0,
                needed: 48,
                held: 47,
            })
        ));
    }
}
