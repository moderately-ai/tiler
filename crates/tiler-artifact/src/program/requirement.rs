//! Backend-neutral requirements the selected route places on a live device.
//!
//! # What belongs here, and the test that decides it
//!
//! A row belongs in this family only when it is **consumed by the selected
//! executable route and not already derivable from its verified program**. That
//! test is not a style preference: a quantity the dispatch record already states
//! has an authority, and a row restating it would be a second one that a
//! producer could contradict. The artifact layer would then hold two answers to
//! one question with nothing able to say which is right.
//!
//! Applying the test to the live-device quantities the platform publishes
//! eliminates every capacity a reader might expect to find here. Enumerating
//! `MTLDevice.h` in the macOS 26.5 SDK, the quantitative live-device properties
//! are `maxThreadsPerThreadgroup`, `maxThreadgroupMemoryLength`,
//! `maxBufferLength`, `recommendedMaxWorkingSetSize`, `currentAllocatedSize`,
//! `maxTransferRate`, `peerCount`, and `maximumConcurrentCompilationTaskCount`.
//! For each of the first four the *requirement* side is already stated: threads
//! per workgroup by the entry's launch geometry and its proven
//! `ResourceRequirements`, threadgroup memory by that same record's
//! `local_memory_bytes`, and both buffer length and resident bytes by each
//! binding's evaluated accessible window. Those are **derived requirements**,
//! checked directly against the device by an adapter — `prototypes/serial-sum-run`
//! already compares its evaluated windows against `max_buffer_length` — and they
//! are deliberately absent from this vocabulary. The remaining four are not
//! correctness predicates on a route at all.
//!
//! So the core quantitative half is narrow by derivation rather than by
//! omission, and [`RouteResourceDimension`] carries exactly the one dimension
//! that survives: a subgroup width, which the neutral kernel IR cannot state
//! because it has no subgroup concept — `ExecutionBinding::GlobalLinearInvocation`
//! is the only execution binding it admits.
//!
//! # The relation belongs to the dimension, not to the producer
//!
//! A required quantity and the comparison that decides it are separate facts,
//! and only the first is on the wire. [`RouteResourceFloor`] carries a dimension
//! and a `u64`; which relation satisfies them is [`RouteResourceDimension`]'s and
//! is written as an exhaustive match, so a dimension added to the vocabulary has
//! to state its own relation rather than inherit whichever one the first
//! dimension happened to need. What does not change is that a *producer* never
//! chooses it: a row states the quantity the route requires, and the vocabulary
//! decides what satisfying it means.
//!
//! # Why a backend-scoped half exists beside it
//!
//! Equal numbers cannot distinguish two devices that differ qualitatively. The
//! two hosts this workspace reaches are the evidence: an Apple M4 Max and an
//! Apple M3 Pro report the *same* highest GPU family and identical threadgroup
//! limits, differing only in buffer and working-set size — quantities that track
//! installed memory rather than capability. A requirement that a device support
//! a named feature is therefore not expressible as a number, and
//! [`BackendFeatureRequirement`] carries it as an owner namespace, a governed
//! key, a version, and a canonical payload the **owning adapter** validates.
//!
//! This layer deliberately does not interpret that payload. It is bytes minted
//! by the backend that emitted the payload, and reading them here would put a
//! backend's vocabulary — an Apple GPU family, say — inside the neutral core.
//! What this layer does own is everything decidable without a device: the
//! owner is a governed [`BackendKey`], the key and version are validated, the
//! payload is bounded and non-empty, and no two rows may name one subject.
//!
//! # Zero rows is a state, not an absence
//!
//! A route that consumes no additional live-device requirement carries none, and
//! that is correct rather than suspicious. What this layer cannot decide is
//! whether a row is *missing*: "missing" is only decidable against a
//! producer-owned exhaustive declaration of what the selected payload actually
//! uses, and no such declaration reaches the artifact. An omitted row is
//! therefore a producer defect this layer cannot detect, which is why the
//! governed feature key exists — a reader that predates this family refuses an
//! artifact that carries rows rather than silently dropping them.

use std::error::Error;
use std::fmt;

use tiler_ir::identity::{push_len, push_slice};

use super::keys::{BackendKey, RouteFeatureKey};

/// Maximum bytes of one backend feature requirement's canonical payload.
///
/// A payload is minted by the owning backend and read only by its adapter, so
/// this is a parser resource ceiling rather than a claim about what a backend
/// may express. It is deliberately far below the governed manifest budget: a
/// payload large enough to approach that bound would be carrying data rather
/// than naming a capability.
pub const MAX_ROUTE_FEATURE_PAYLOAD_BYTES: usize = 1_024;

const ROUTE_REQUIREMENT_DOMAIN: &[u8] = b"tiler.artifact.route-requirement.v1\0";

/// Which live-device quantity one core route requirement constrains.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b). A runtime
/// adapter must observe every dimension a route can require, so a dimension
/// added here has to be a build failure at each adapter rather than a value one
/// of them silently fails to answer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RouteResourceDimension {
    /// Threads the device executes one subgroup at, compared by **equality**.
    ///
    /// A route whose emitted code uses subgroup-cooperative operations states
    /// this nowhere else: the neutral kernel IR admits only whole-grid
    /// invocation binding and so has no subgroup to describe. It is a property
    /// of what the backend emitted, which is exactly the class this family
    /// exists to carry.
    ///
    /// # Why the relation is an equality and not a floor
    ///
    /// A width-`W` combine tree's steps are its content rather than a
    /// consequence of it: a butterfly at width 32 has five steps at masks
    /// `1, 2, 4, 8, 16`, and at width 64 it has six. A device executing more
    /// threads per subgroup than the route was verified at satisfies a floor and
    /// still runs lane arithmetic nothing checked, so a floor admits exactly the
    /// case that is wrong; a narrower device fails under either relation. An
    /// equality is therefore the only comparison under which satisfying this row
    /// implies the route is correct.
    ///
    /// # What this row does not claim
    ///
    /// It is **not** a claim that a subgroup executes in lockstep. Current GPU
    /// families withdraw that guarantee explicitly — from compute capability
    /// 7.0, CUDA's independent thread scheduling "allows full concurrency
    /// between threads, regardless of warp", and the same guide records that the
    /// ability to diverge and reconverge at sub-warp granularity makes
    /// warp-synchronous assumptions invalid. A row over a quantity no device
    /// provides is one no adapter could soundly observe under any relation.
    ///
    /// Nor is it the whole obligation of a subgroup route. A tree at width `W`
    /// additionally requires every lane of `0..W` to be active at each combine
    /// step, and that is a property of the *program*: no target declares it, and
    /// a schedule discharges it intrinsically by deriving a launch geometry that
    /// leaves no partially populated trailing subgroup. Only the width crosses
    /// this boundary, because only the width is a fact about the device.
    ///
    /// # What an adapter is asked to observe
    ///
    /// The device-scoped subgroup width, on the platforms that publish one:
    /// Vulkan reports `subgroupSize` as a physical-device property and CUDA
    /// reports `warpSize` as a device property. Metal publishes no device-scoped
    /// equivalent — `threadExecutionWidth` belongs to a prepared
    /// `MTLComputePipelineState` — so a Metal adapter answers `Unrecognized` and
    /// the route is refused, rather than reporting a documentation constant as a
    /// device observation. A Metal route that genuinely needs the width states it
    /// against the prepared pipeline, which is the authority that has it.
    SubgroupThreads,
}

impl RouteResourceDimension {
    /// Every dimension this vocabulary names.
    ///
    /// Enumerated so a test can state the population it checks rather than
    /// asserting over whatever it happens to have listed.
    pub const ALL: [Self; 1] = [Self::SubgroupThreads];

    /// Returns the governed wire tag of this dimension.
    ///
    /// Written by an exhaustive match rather than read from the discriminant, so
    /// inserting or reordering a variant is a build error here instead of a
    /// silent re-encoding of every artifact ever produced (ADR 0074 §3).
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::SubgroupThreads => 0x01,
        }
    }

    /// Returns the dimension one governed wire tag names.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::SubgroupThreads),
            _ => None,
        }
    }

    /// Returns the stable text this dimension is reported by.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubgroupThreads => "subgroup-threads",
        }
    }
}

impl fmt::Display for RouteResourceDimension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One core quantitative requirement the selected route places on a live device.
///
/// A dimension and a required quantity, and deliberately not a relation: which
/// comparison decides them belongs to [`RouteResourceDimension`] and is fixed by
/// the vocabulary. That keeps what the original floor-only shape bought — a
/// producer cannot state an equality where a minimum was meant, or the reverse,
/// which [`tiler_ir::program::abi::TargetPropertyRequirementRelation`] exists to
/// make impossible for a prepared-entry requirement — without also forcing every
/// later dimension to accept the first one's relation.
///
/// **This type's name is stale, and its rename is filed rather than performed.**
/// The relation correction was made where it decides behaviour; `RouteResourceFloor`
/// and [`RouteRequirement::ResourceFloor`] are additionally named by the runtime
/// loader, two prototype adapters, a spike adapter, and accepted ADR 0092's text,
/// so renaming them crosses four scopes this crate does not hold. Carried by
/// `rename-the-route-resource-floor-vocabulary-for-its-corrected-relation`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RouteResourceFloor {
    dimension: RouteResourceDimension,
    required: u64,
}

impl RouteResourceFloor {
    /// Creates one validated quantitative requirement.
    ///
    /// # Errors
    ///
    /// Returns [`RouteRequirementError::ZeroResourceQuantity`] for a required
    /// quantity of zero.
    pub const fn new(
        dimension: RouteResourceDimension,
        required: u64,
    ) -> Result<Self, RouteRequirementError> {
        if required == 0 {
            return Err(RouteRequirementError::ZeroResourceQuantity { dimension });
        }
        Ok(Self {
            dimension,
            required,
        })
    }

    /// Returns the live-device dimension this row constrains.
    #[must_use]
    pub const fn dimension(self) -> RouteResourceDimension {
        self.dimension
    }

    /// Returns the quantity this row requires of the live device.
    #[must_use]
    pub const fn required(self) -> u64 {
        self.required
    }

    /// Returns whether one observed quantity satisfies this requirement.
    ///
    /// Exhaustive on the dimension rather than applying one relation to all of
    /// them, so a dimension added to the vocabulary has to choose its relation
    /// here and the omission is a build error rather than an inherited default.
    #[must_use]
    pub const fn is_satisfied_by(self, observed: u64) -> bool {
        match self.dimension {
            // A wider device runs lane arithmetic the route never verified, so it
            // fails this row exactly as a narrower device does.
            RouteResourceDimension::SubgroupThreads => self.required == observed,
        }
    }
}

/// One backend-scoped qualitative requirement of the selected route.
///
/// The owner is a governed [`BackendKey`] rather than a free namespace string,
/// which buys a check no device is needed for: a host states the backend it can
/// execute, so a row owned by a different backend is refused before any adapter
/// is consulted.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BackendFeatureRequirement {
    owner: BackendKey,
    key: RouteFeatureKey,
    version: u32,
    payload: Box<[u8]>,
}

impl BackendFeatureRequirement {
    /// Creates one validated backend-scoped requirement.
    ///
    /// # Errors
    ///
    /// Returns [`RouteRequirementError::ZeroFeatureVersion`] for a zero version,
    /// [`RouteRequirementError::EmptyFeaturePayload`] for an empty payload, or
    /// [`RouteRequirementError::FeaturePayloadTooLong`] beyond
    /// [`MAX_ROUTE_FEATURE_PAYLOAD_BYTES`].
    pub fn new(
        owner: BackendKey,
        key: RouteFeatureKey,
        version: u32,
        payload: impl AsRef<[u8]>,
    ) -> Result<Self, RouteRequirementError> {
        let payload = payload.as_ref();
        if version == 0 {
            return Err(RouteRequirementError::ZeroFeatureVersion);
        }
        // Empty is refused rather than admitted as "no argument". At this layer
        // an empty payload and a truncated one are the same bytes, and a
        // capability that takes no argument can spell that explicitly; admitting
        // absence would make the two indistinguishable to the owning adapter.
        if payload.is_empty() {
            return Err(RouteRequirementError::EmptyFeaturePayload);
        }
        if payload.len() > MAX_ROUTE_FEATURE_PAYLOAD_BYTES {
            return Err(RouteRequirementError::FeaturePayloadTooLong {
                bytes: payload.len(),
                limit: MAX_ROUTE_FEATURE_PAYLOAD_BYTES,
            });
        }
        Ok(Self {
            owner,
            key,
            version,
            payload: payload.into(),
        })
    }

    /// Returns the backend that owns and validates this requirement.
    #[must_use]
    pub const fn owner(&self) -> &BackendKey {
        &self.owner
    }

    /// Returns the governed requirement key within that owner's namespace.
    #[must_use]
    pub const fn key(&self) -> &RouteFeatureKey {
        &self.key
    }

    /// Returns the nonzero governed version of this requirement's meaning.
    ///
    /// An adapter matches it exactly. A version it does not know is not a
    /// requirement it may approximate, because the same key at two versions can
    /// mean two different things.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Returns the canonical payload the owning adapter validates.
    ///
    /// Opaque here, deliberately: interpreting it would put a backend's
    /// vocabulary inside the neutral core.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// One additional requirement the selected route places on a live device.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b): a runtime
/// decides every row before committing a route, so a kind added here must stop
/// each consumer's build rather than reach a wildcard arm that skips it.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RouteRequirement {
    /// A neutral quantitative row the runtime compares itself, by the relation
    /// the row's dimension fixes.
    ResourceFloor(RouteResourceFloor),
    /// A backend-scoped qualitative row the owning adapter decides.
    BackendFeature(BackendFeatureRequirement),
}

impl RouteRequirement {
    /// Returns the governed wire tag of this row's kind.
    #[must_use]
    pub const fn tag(&self) -> u8 {
        match self {
            Self::ResourceFloor(_) => 0x01,
            Self::BackendFeature(_) => 0x02,
        }
    }

    /// Returns the subject this row constrains.
    ///
    /// Two rows naming one subject are contradictory rather than redundant, so
    /// this is what both the builder's duplicate check and a runtime refusal
    /// name. It is owned, because a refusal outlives the artifact borrow that
    /// produced it.
    #[must_use]
    pub fn subject(&self) -> RouteRequirementSubject {
        match self {
            Self::ResourceFloor(floor) => RouteRequirementSubject::Resource {
                dimension: floor.dimension(),
            },
            Self::BackendFeature(feature) => RouteRequirementSubject::BackendFeature {
                owner: feature.owner().clone(),
                key: feature.key().clone(),
                version: feature.version(),
            },
        }
    }

    /// Returns the canonical content key of this complete requirement.
    ///
    /// The subject leads, so two rows naming one subject sort adjacent and a
    /// duplicate-subject check over a sorted run is a scan of neighbours.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = ROUTE_REQUIREMENT_DOMAIN.to_vec();
        bytes.push(self.tag());
        match self {
            Self::ResourceFloor(floor) => {
                bytes.push(floor.dimension().tag());
                bytes.extend_from_slice(&floor.required().to_be_bytes());
            }
            Self::BackendFeature(feature) => {
                push_slice(&mut bytes, feature.owner().as_str().as_bytes());
                push_slice(&mut bytes, feature.key().as_str().as_bytes());
                bytes.extend_from_slice(&feature.version().to_be_bytes());
                push_slice(&mut bytes, feature.payload());
            }
        }
        bytes
    }
}

/// The subject one route requirement constrains.
///
/// Distinct from the requirement itself: the subject is what may not repeat,
/// while the requirement adds the quantity or payload that repeating rows would
/// disagree about.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RouteRequirementSubject {
    /// One neutral live-device dimension.
    Resource {
        /// The constrained dimension.
        dimension: RouteResourceDimension,
    },
    /// One governed key in one backend's namespace, at one version.
    BackendFeature {
        /// Backend that owns the key.
        owner: BackendKey,
        /// Governed requirement key.
        key: RouteFeatureKey,
        /// Governed version of the requirement's meaning.
        version: u32,
    },
}

impl fmt::Display for RouteRequirementSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resource { dimension } => write!(formatter, "the live-device {dimension}"),
            Self::BackendFeature {
                owner,
                key,
                version,
            } => write!(formatter, "{owner}'s {key} at version {version}"),
        }
    }
}

/// A rejected route requirement.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a later validation lands as
/// a new cause rather than by widening an existing one's meaning.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RouteRequirementError {
    /// A required quantity of zero names no device.
    ///
    /// No dimension this vocabulary carries is ever observed as zero, so a zero
    /// row is a producer defect under either relation — vacuous under a floor,
    /// which every observation satisfies, and unsatisfiable under an equality,
    /// which none does. Refusing it at construction is what keeps a row whose
    /// verdict was settled before any device was consulted out of the artifact,
    /// the same vacuity that removed the invented barrier count from the entry
    /// resource record.
    ZeroResourceQuantity {
        /// The dimension the zero quantity named.
        dimension: RouteResourceDimension,
    },
    /// A backend feature requirement declared a zero version.
    ZeroFeatureVersion,
    /// A backend feature requirement carried no payload.
    EmptyFeaturePayload,
    /// A backend feature payload exceeded [`MAX_ROUTE_FEATURE_PAYLOAD_BYTES`].
    FeaturePayloadTooLong {
        /// Byte length supplied.
        bytes: usize,
        /// Byte length admitted.
        limit: usize,
    },
}

impl fmt::Display for RouteRequirementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroResourceQuantity { dimension } => {
                write!(formatter, "a required {dimension} of zero names no device")
            }
            Self::ZeroFeatureVersion => {
                formatter.write_str("a backend feature requirement's version must be nonzero")
            }
            Self::EmptyFeaturePayload => {
                formatter.write_str("a backend feature requirement must carry a canonical payload")
            }
            Self::FeaturePayloadTooLong { bytes, limit } => write!(
                formatter,
                "a backend feature payload of {bytes} bytes exceeds {limit}",
            ),
        }
    }
}

impl Error for RouteRequirementError {}

/// Returns the canonical order of one variant's route requirements, as positions.
///
/// Shared by the identity encoder, the codec that stores them, and the validator
/// that re-checks the stored order, so the three cannot drift into three
/// definitions of "canonical".
pub(super) fn canonical_requirement_order(requirements: &[RouteRequirement]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..requirements.len()).collect();
    order.sort_by_cached_key(|index| requirements[*index].canonical_bytes());
    order
}

/// Writes one variant's route requirements, in the order supplied.
pub(super) fn push_requirements(bytes: &mut Vec<u8>, requirements: &[RouteRequirement]) {
    push_len(bytes, requirements.len());
    for requirement in requirements {
        push_slice(bytes, &requirement.canonical_bytes());
    }
}
