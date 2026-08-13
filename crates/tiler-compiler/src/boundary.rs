#![allow(
    dead_code,
    reason = "the satisfaction relation, the derived contracts, the set encodings, and boundary dominance are on the compile path through `crate::frontier` and `crate::selection`; what stays unconstructed is the goal-directed surface no bottom-up enumeration reaches — `derive_child_requirements`, the standalone `encode_property_identity` the accepted memo contract's optimization key would use, and the reserved property values that remain unconstructed (opaque materialization, bit-packed encodings, the non-device memory-domain classes, host-readback availability, and the explicit-coherence visibility guarantee) — which only a top-down property search or a second execution profile can produce. Alias materialization is no longer on that list: `call_declaration::guaranteed_properties_for` produces `AliasView` for an opaque call declaring `MayAliasInputs`, which is why the enforcers deferral trigger had to be restated (see that ticket)"
)]

//! The typed physical boundary-property model: what a region implementation
//! requires of an incoming value and what it guarantees of an outgoing one.
//!
//! This module owns the property *vocabulary* and the four relations over it
//! that `docs/compiler/optimizer.md` requires a boundary-contract system to
//! define — canonical keys, satisfaction and subsumption, child-requirement
//! derivation, and dominance — plus the canonical identity encoding and the
//! typed dissatisfaction records an explanation reports. It owns no executable
//! step: materialization, layout conversion, encoding repacking, transfers,
//! synchronization, and lifetime verification belong to
//! `implement-boundary-property-enforcers` and, per ADR 0047, are represented at
//! `KernelProgram` scope rather than here.
//!
//! # Reconciling the two accepted sources
//!
//! Two accepted documents name boundary contents and they do not coincide, so
//! the dimension list below is a stated reconciliation rather than a copy of
//! either.
//!
//! `docs/compiler/optimizer.md` names five properties: storage layout class and
//! contiguous axes; storage encoding; alignment and vectorizable width;
//! materialized buffer, alias/view, or opaque runtime value; and device and
//! address space. It introduces them with "Initial boundary contracts *include*",
//! so the list is a floor and not a closed set.
//!
//! ADR 0047 names the two sides separately and asymmetrically. A requirement
//! names "a symbolic execution affinity, admitted memory domain, access mode,
//! value version, storage encoding, byte range, alignment, and availability
//! dependency"; a delivered placement names "a materialized value,
//! allocation/domain, authoritative version, visibility state, ownership, and
//! the dependency after which it is usable".
//!
//! The reconciliation, dimension by dimension:
//!
//! - the optimizer contract's "device and address space" is ADR 0047's *affinity*
//!   and *memory domain* as two dimensions, not one. ADR 0047 makes them separate
//!   node and edge kinds of its capability multigraph — an affinity is where work
//!   runs, a domain is where storage lives, and the same domain may be reachable
//!   from several affinities with different accessibility. Collapsing them would
//!   reintroduce the `Disk < RAM < VRAM < Scratch` ordering the ADR rejects;
//! - ADR 0047's "availability dependency" and "the dependency after which it is
//!   usable" are the two sides of [`BoundaryProperty::Availability`];
//! - ADR 0047's "visibility state" is [`BoundaryProperty::Visibility`];
//! - ADR 0047's "access mode" and "ownership" are not dimensions. They are the
//!   two sides' own qualifiers and are carried beside the property sets by
//!   [`crate::frontier::BoundaryRequirement`] and
//!   [`crate::frontier::BoundaryGuarantee`], because an access mode is a fact
//!   about the *access* — `tiler_ir::schedule::Access::mode` already carries it —
//!   and never something a producer supplies at a boundary;
//! - ADR 0047's "byte range" is deliberately absent. A boundary byte range is an
//!   `AbiExpr` over an endpoint, which ADR 0068 places in `tiler-ir` and binds at
//!   the artifact layer; representing it here would put a second authority on the
//!   same subject;
//! - ADR 0047's "value version" and "authoritative version" are deliberately
//!   absent. ADR 0047 states that "new materialized versions" are represented at
//!   `KernelProgram` scope, and a version is produced by an enforcer rather than
//!   chosen at a boundary. Broadening to a mutating profile requires adding the
//!   dimension here *and* the version-producing step there, together.
//!
//! Resolved value dtype is absent by construction, not by omission:
//! `docs/compiler/optimizer.md` establishes that satisfaction on this list is
//! subsumption and that the dtype analogue of "16-byte alignment satisfies a
//! 4-byte requirement" is the erased narrowing ADRs 0009 and 0010 forbid.
//! Storage encoding, by contrast, is on the list because a producer can realize
//! one semantic value packed or unpacked and the choice is unobservable in the
//! value — and its relation is stated per family rather than assumed to be an
//! ordering, for exactly the reason dtype is excluded.
//!
//! # What is implemented, and what is only reserved
//!
//! Every relation below is implemented and tested over the whole vocabulary.
//! What is *bounded* is which values the P0 profile can produce: ADR 0047's
//! initial execution profile is one symbolic affinity, one live device, and one
//! ordered command stream, with every input already accessible. So the derived
//! contracts in [`crate::frontier`] name exactly [`ExecutionAffinity::PRIMARY`],
//! [`MemoryDomainClass::Device`], [`MaterializationForm::MaterializedBuffer`],
//! [`StorageEncoding::Unpacked`], and [`LayoutGuarantee::DenseRowMajor`]. The
//! remaining values are reachable by construction and reject explicitly rather
//! than being silently approximated: a requirement no guarantee discharges
//! becomes an [`UnsatisfiedProperty`], never a satisfied one at a higher cost.
//!
//! Nothing here is a compiler API: `boundary` is a private module carrying no
//! `pub` item and no re-export, so this is a crate-internal draft vocabulary.
//! The draft discipline still holds — every shape below is provisional and
//! carries no compatibility story — and the acceptance ADR 0074 convention 7
//! requires is owed at the point any of it is first exported, not here.

use std::fmt;

use tiler_ir::identity::{push_len, push_slice};
pub(crate) use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, ByteAlignment, StorageEncoding, StorageScalar,
};
use tiler_ir::program::{BitPackedEncoding, PackedBitOrder, PackedTailRule};
use tiler_ir::shape::Axis;

/// Canonical domain-separation tag for one boundary property set.
///
/// NUL-terminated and versioned per ADR 0074 convention 3, so property bytes can
/// never be read as another subject's and a widened vocabulary mints a new tag
/// rather than silently changing what `v1` meant.
const PROPERTY_SET_IDENTITY_TAG: &[u8] = b"tiler.compiler.boundary-property-set.v3\0";

/// A governed dimension of a physical boundary contract.
///
/// The vocabulary is bounded and closed, for the reason ADR 0043 gives for the
/// capability axes and ADR 0076 for the numerical dimensions: a free-form
/// property bag cannot prove that a producer met what a consumer asked for. The
/// derived ordering is the canonical evaluation, encoding, and reporting order.
///
/// Each dimension carries *two* value spaces rather than one, because ADR 0047
/// states the requirement and delivered field lists asymmetrically and two of
/// the asymmetries are load-bearing: a memory-domain requirement names an
/// admitted *set* while a guarantee names the one domain an allocation is in,
/// and a visibility guarantee can name a state — an explicit coherence action
/// still owed — that no requirement can ask for.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum BoundaryProperty {
    /// Which logical coordinate maps to which storage position.
    StorageLayout,
    /// How one element is represented at the position layout assigns it.
    StorageEncoding,
    /// Byte alignment of the boundary value's first element.
    Alignment,
    /// Whether the value is a materialized buffer, an alias/view, or opaque.
    Materialization,
    /// The symbolic execution affinity the value is placed for.
    ExecutionAffinity,
    /// The memory domain class the value's allocation lives in.
    MemoryDomain,
    /// The dependency after which the value may be consumed.
    Availability,
    /// Whether a consumer's reads see the produced value without further action.
    Visibility,
}

/// The canonical dimension order: the single source of truth for evaluation,
/// encoding, and reporting order, matching the derived [`BoundaryProperty`]
/// ordering.
pub(crate) const CANONICAL_PROPERTIES: [BoundaryProperty; 8] = [
    BoundaryProperty::StorageLayout,
    BoundaryProperty::StorageEncoding,
    BoundaryProperty::Alignment,
    BoundaryProperty::Materialization,
    BoundaryProperty::ExecutionAffinity,
    BoundaryProperty::MemoryDomain,
    BoundaryProperty::Availability,
    BoundaryProperty::Visibility,
];

impl BoundaryProperty {
    /// The governed canonical key naming this dimension in explain output.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::StorageLayout => "boundary.storage-layout",
            Self::StorageEncoding => "boundary.storage-encoding",
            Self::Alignment => "boundary.alignment",
            Self::Materialization => "boundary.materialization",
            Self::ExecutionAffinity => "boundary.execution-affinity",
            Self::MemoryDomain => "boundary.memory-domain",
            Self::Availability => "boundary.availability",
            Self::Visibility => "boundary.visibility",
        }
    }

    /// The governed tag naming this dimension in a canonical encoding.
    ///
    /// Written by an exhaustive match rather than read from the discriminant, so
    /// adding or reordering a dimension is a build error here instead of a
    /// silent change to every boundary contract ever encoded (ADR 0074
    /// convention 3).
    const fn tag(self) -> u8 {
        match self {
            Self::StorageLayout => 0x01,
            Self::StorageEncoding => 0x02,
            Self::Alignment => 0x03,
            Self::Materialization => 0x04,
            Self::ExecutionAffinity => 0x05,
            Self::MemoryDomain => 0x06,
            Self::Availability => 0x07,
            Self::Visibility => 0x08,
        }
    }

    /// Whether a requirement on this dimension propagates through a region to
    /// the requirements it places on its own inputs.
    ///
    /// Only the execution affinity does, and only because ADR 0047's initial
    /// execution profile says so in as many words: "Inputs must already be
    /// accessible, and all stages, temporaries, and outputs use that affinity."
    /// A goal affinity therefore fixes the region's affinity, which fixes its
    /// inputs'.
    ///
    /// Nothing else propagates, and the reason is the same for each: a goal
    /// constrains the value *leaving* the region, and a region's layout,
    /// encoding, alignment, materialization, memory domain, availability, and
    /// visibility needs on its *inputs* are properties of the access it performs,
    /// not of where its result goes. A reduction that needs a unit-stride input
    /// axis needs it whether its output is written to device memory or handed to
    /// a later pass. Propagating them would manufacture requirements no
    /// implementation asked for, and satisfaction would then reject producers
    /// that are in fact adequate.
    ///
    /// A profile with more than one affinity makes this dimension stop
    /// propagating too: the region would choose an affinity, a transfer enforcer
    /// would reconcile it with the goal, and the goal would constrain the
    /// enforcer rather than the inputs.
    const fn propagates_to_children(self) -> bool {
        match self {
            Self::ExecutionAffinity => true,
            Self::StorageLayout
            | Self::StorageEncoding
            | Self::Alignment
            | Self::Materialization
            | Self::MemoryDomain
            | Self::Availability
            | Self::Visibility => false,
        }
    }
}

impl fmt::Display for BoundaryProperty {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

/// A storage layout a consumer requires of an incoming value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LayoutRequirement {
    /// Dense row-major addressing over the value's logical shape, with no
    /// padding between elements.
    DenseRowMajor,
    /// Unit stride along one axis of a value of the stated rank.
    ///
    /// This is `docs/compiler/optimizer.md`'s own example of a requirement a
    /// vectorized reduction places on its input. It carries the rank because
    /// satisfaction depends on it: a dense row-major value has unit stride on
    /// its last axis and on no other.
    UnitStrideOnAxis {
        /// The axis that must be unit-stride.
        axis: Axis,
        /// Rank of the value the requirement is over.
        rank: u32,
    },
}

impl LayoutRequirement {
    /// The governed canonical key naming this requirement.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::DenseRowMajor => "dense-row-major",
            Self::UnitStrideOnAxis { .. } => "unit-stride-on-axis",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::DenseRowMajor => 0x01,
            Self::UnitStrideOnAxis { .. } => 0x02,
        }
    }

    /// Whether this requirement is well formed.
    ///
    /// A rank of zero has no axes, and an axis at or beyond the rank names one
    /// that does not exist. Both are malformed inputs rather than unsatisfiable
    /// requirements: nothing could ever discharge them, and reporting them as an
    /// ordinary dissatisfaction would hide a compiler fault behind a plan
    /// rejection.
    const fn is_well_formed(self) -> bool {
        match self {
            Self::DenseRowMajor => true,
            Self::UnitStrideOnAxis { axis, rank } => rank > 0 && axis.get() < rank,
        }
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        match self {
            Self::DenseRowMajor => {}
            Self::UnitStrideOnAxis { axis, rank } => {
                bytes.extend_from_slice(&axis.get().to_be_bytes());
                bytes.extend_from_slice(&rank.to_be_bytes());
            }
        }
    }
}

/// A storage layout a producer guarantees of an outgoing value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LayoutGuarantee {
    /// Dense row-major addressing over the value's logical shape.
    ///
    /// The one layout the bounded profile produces. `tiler_ir::kernel` linearizes
    /// a contributor address as "the row-major linearization" of the logical
    /// coordinates, and the reference evaluator holds "dense exact elements in
    /// logical row-major order", so this is the layout a scheduled region's
    /// `LinearIdentity` and `ReductionContributor` access maps already assume.
    DenseRowMajor,
}

impl LayoutGuarantee {
    /// The governed canonical key naming this guarantee.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::DenseRowMajor => "dense-row-major",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::DenseRowMajor => 0x01,
        }
    }

    /// Whether this layout discharges `required`.
    ///
    /// Dense row-major addressing has unit stride on the last axis and, for a
    /// rank above one, a stride of the trailing extents' product on every other
    /// axis. So it satisfies a unit-stride requirement on the last axis and on
    /// no other. A rank-one value's only axis is its last, which is why the
    /// comparison is against `rank - 1` rather than a special case.
    const fn satisfies(self, required: LayoutRequirement) -> bool {
        match (self, required) {
            (Self::DenseRowMajor, LayoutRequirement::DenseRowMajor) => true,
            (Self::DenseRowMajor, LayoutRequirement::UnitStrideOnAxis { axis, rank }) => {
                rank > 0 && axis.get() == rank - 1
            }
        }
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
    }
}

/// Whether an encoded byte window will be read or written.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorageAccessKind {
    /// Reading may inspect a shared boundary byte and discard unrelated bits.
    Read,
    /// Writing must not overwrite a byte containing an element outside the range.
    Write,
}

/// Exact byte window and edge-bit treatment for one logical element range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StorageAccess {
    byte_offset: u64,
    byte_length: u64,
    leading_shared_bits: u32,
    trailing_shared_bits: u32,
    trailing_padding_bits: u32,
}

impl StorageAccess {
    /// First byte the access reaches.
    pub(crate) const fn byte_offset(self) -> u64 {
        self.byte_offset
    }

    /// Number of bytes the access reaches.
    pub(crate) const fn byte_length(self) -> u64 {
        self.byte_length
    }

    /// Low bits in the first byte belonging to a preceding logical element.
    pub(crate) const fn leading_shared_bits(self) -> u32 {
        self.leading_shared_bits
    }

    /// High bits in the last byte belonging to a following logical element.
    pub(crate) const fn trailing_shared_bits(self) -> u32 {
        self.trailing_shared_bits
    }

    /// Canonical padding bits after the complete value's final logical element.
    pub(crate) const fn trailing_padding_bits(self) -> u32 {
        self.trailing_padding_bits
    }
}

/// Why an encoded logical range has no safe byte access.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorageAccessError {
    /// The requested logical range exceeds the complete logical value.
    OutOfBounds,
    /// Computing an endpoint or byte count overflowed.
    ArithmeticOverflow,
    /// A packed write would overwrite a byte shared with an element outside the range.
    UnalignedPartialWrite,
    /// The encoding has no executable accessor in this bounded proof.
    UnsupportedEncoding,
}

/// Why packing or unpacking a bounded code carrier failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorageCodecError {
    /// A code does not fit the encoding's physical element width.
    CodeOutOfRange {
        /// Index of the rejected code.
        index: usize,
        /// Rejected code value.
        code: u8,
    },
    /// The byte payload length disagrees with the logical element count.
    ByteLength {
        /// Required byte count.
        expected: usize,
        /// Supplied byte count.
        actual: usize,
    },
    /// Unused bits in the final byte violate the encoding's tail rule.
    NonCanonicalTail {
        /// Final byte containing the invalid padding bits.
        byte: u8,
    },
    /// The encoding has no executable codec in this bounded proof.
    UnsupportedEncoding,
    /// A logical element count cannot be represented on this host.
    ElementCountOverflow,
}

const fn storage_key(encoding: StorageEncoding) -> &'static str {
    match encoding {
        StorageEncoding::Unpacked => "unpacked",
        StorageEncoding::BitPacked(_) => "bit-packed",
    }
}

const fn storage_satisfies(guaranteed: StorageEncoding, required: StorageEncoding) -> bool {
    match (guaranteed, required) {
        (StorageEncoding::Unpacked, StorageEncoding::Unpacked) => true,
        (StorageEncoding::BitPacked(left), StorageEncoding::BitPacked(right)) => {
            left.element_bits() == right.element_bits()
                && matches!(
                    (left.bit_order(), right.bit_order()),
                    (
                        PackedBitOrder::LeastSignificantElementFirst,
                        PackedBitOrder::LeastSignificantElementFirst
                    ) | (
                        PackedBitOrder::MostSignificantElementFirst,
                        PackedBitOrder::MostSignificantElementFirst
                    )
                )
                && matches!(
                    (left.tail(), right.tail()),
                    (PackedTailRule::Zero, PackedTailRule::Zero)
                )
        }
        (StorageEncoding::Unpacked, StorageEncoding::BitPacked(_))
        | (StorageEncoding::BitPacked(_), StorageEncoding::Unpacked) => false,
    }
}

fn encode_storage(encoding: StorageEncoding, bytes: &mut Vec<u8>) {
    match encoding {
        StorageEncoding::Unpacked => bytes.push(0x01),
        StorageEncoding::BitPacked(packed) => {
            bytes.push(0x02);
            bytes.push(packed.element_bits());
            bytes.push(match packed.bit_order() {
                PackedBitOrder::LeastSignificantElementFirst => 0x01,
                PackedBitOrder::MostSignificantElementFirst => 0x02,
            });
            bytes.push(match packed.tail() {
                PackedTailRule::Zero => 0x01,
            });
        }
    }
}

/// Derives the exact byte window for ordinary one-byte carriers or packed-u4.
///
/// `logical_elements` names the complete value so an odd packed tail can be
/// distinguished from a partial write ending halfway through an interior
/// byte. Reads may include such a shared byte and report which edge bits to
/// discard. Writes fail unless both shared edges are outside the write, with
/// one exception: the zero-filled high nibble after the complete value's odd
/// final element belongs to the encoding and may be written canonically.
fn code_access(
    encoding: StorageEncoding,
    logical_elements: u64,
    first: u64,
    count: u64,
    kind: StorageAccessKind,
) -> Result<StorageAccess, StorageAccessError> {
    let end = first
        .checked_add(count)
        .ok_or(StorageAccessError::ArithmeticOverflow)?;
    if end > logical_elements {
        return Err(StorageAccessError::OutOfBounds);
    }
    if count == 0 {
        let byte_offset = match encoding {
            StorageEncoding::Unpacked => first,
            StorageEncoding::BitPacked(packed) if packed == packed_u4() => first / 2,
            StorageEncoding::BitPacked(_) => {
                return Err(StorageAccessError::UnsupportedEncoding);
            }
        };
        return Ok(StorageAccess {
            byte_offset,
            byte_length: 0,
            leading_shared_bits: 0,
            trailing_shared_bits: 0,
            trailing_padding_bits: 0,
        });
    }
    match encoding {
        StorageEncoding::Unpacked => Ok(StorageAccess {
            byte_offset: first,
            byte_length: count,
            leading_shared_bits: 0,
            trailing_shared_bits: 0,
            trailing_padding_bits: 0,
        }),
        StorageEncoding::BitPacked(packed) if packed == packed_u4() => {
            let leading_shared_bits = if first.is_multiple_of(2) { 0 } else { 4 };
            let trailing_shared_bits = if !end.is_multiple_of(2) && end != logical_elements {
                4
            } else {
                0
            };
            let trailing_padding_bits = if !end.is_multiple_of(2) && end == logical_elements {
                4
            } else {
                0
            };
            if kind == StorageAccessKind::Write
                && (leading_shared_bits != 0 || trailing_shared_bits != 0)
            {
                return Err(StorageAccessError::UnalignedPartialWrite);
            }
            let byte_offset = first / 2;
            let byte_end = end / 2 + end % 2;
            Ok(StorageAccess {
                byte_offset,
                byte_length: byte_end - byte_offset,
                leading_shared_bits,
                trailing_shared_bits,
                trailing_padding_bits,
            })
        }
        StorageEncoding::BitPacked(_) => Err(StorageAccessError::UnsupportedEncoding),
    }
}

/// Packs ordinary one-byte carriers or four-bit carriers into canonical bytes.
fn pack_codes(encoding: StorageEncoding, codes: &[u8]) -> Result<Vec<u8>, StorageCodecError> {
    match encoding {
        StorageEncoding::Unpacked => Ok(codes.to_vec()),
        StorageEncoding::BitPacked(packed) if packed == packed_u4() => {
            let mut bytes = Vec::with_capacity(codes.len().div_ceil(2));
            for (pair_index, pair) in codes.chunks(2).enumerate() {
                let first_index = pair_index * 2;
                let low = pair[0];
                if low > 0x0f {
                    return Err(StorageCodecError::CodeOutOfRange {
                        index: first_index,
                        code: low,
                    });
                }
                let high = pair.get(1).copied().unwrap_or(0);
                if high > 0x0f {
                    return Err(StorageCodecError::CodeOutOfRange {
                        index: first_index + 1,
                        code: high,
                    });
                }
                bytes.push(low | (high << 4));
            }
            Ok(bytes)
        }
        StorageEncoding::BitPacked(_) => Err(StorageCodecError::UnsupportedEncoding),
    }
}

/// Unpacks and validates ordinary one-byte or canonical packed-u4 carriers.
fn unpack_codes(
    encoding: StorageEncoding,
    bytes: &[u8],
    logical_elements: u64,
) -> Result<Vec<u8>, StorageCodecError> {
    let logical_elements =
        usize::try_from(logical_elements).map_err(|_| StorageCodecError::ElementCountOverflow)?;
    match encoding {
        StorageEncoding::Unpacked => {
            if bytes.len() != logical_elements {
                return Err(StorageCodecError::ByteLength {
                    expected: logical_elements,
                    actual: bytes.len(),
                });
            }
            Ok(bytes.to_vec())
        }
        StorageEncoding::BitPacked(packed) if packed == packed_u4() => {
            let expected = logical_elements.div_ceil(2);
            if bytes.len() != expected {
                return Err(StorageCodecError::ByteLength {
                    expected,
                    actual: bytes.len(),
                });
            }
            if !logical_elements.is_multiple_of(2)
                && bytes.last().is_some_and(|byte| byte & 0xf0 != 0)
            {
                return Err(StorageCodecError::NonCanonicalTail {
                    byte: *bytes
                        .last()
                        .expect("an odd element count requires one byte"),
                });
            }
            let mut codes = Vec::with_capacity(logical_elements);
            for byte in bytes {
                codes.push(byte & 0x0f);
                if codes.len() < logical_elements {
                    codes.push(byte >> 4);
                }
            }
            Ok(codes)
        }
        StorageEncoding::BitPacked(_) => Err(StorageCodecError::UnsupportedEncoding),
    }
}

const fn packed_u4() -> BitPackedEncoding {
    match StorageEncoding::PACKED_U4_LSB_ZERO_TAIL {
        StorageEncoding::BitPacked(packed) => packed,
        StorageEncoding::Unpacked => unreachable!(),
    }
}

/// The form a boundary value takes.
///
/// These are the three the accepted optimizer contract names: "materialized
/// buffer, alias/view, or opaque runtime value". The relation is equality: a
/// materialized buffer is not an alias view, and a consumer that requires a view
/// — because it must not pay for a copy, or because the value it needs is a
/// window onto an allocation it does not own — is not served by one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MaterializationForm {
    /// A materialized buffer the region reads or writes in full.
    MaterializedBuffer,
    /// A metadata-only alias or view onto another allocation.
    ///
    /// Reserved. `crate::frontier::ProposalBody::View` is the proposal body that
    /// would guarantee one, and the bounded frontier rejects it explicitly.
    AliasView,
    /// A runtime value whose representation the compiler does not model.
    ///
    /// Reserved. `crate::frontier::ProposalBody::OpaqueCall` is the proposal body
    /// that would produce one; `implement-opaque-physical-call-providers` owns
    /// its typed ABI, effect, aliasing, and placement contracts.
    OpaqueRuntimeValue,
}

impl MaterializationForm {
    /// The governed canonical key naming this form.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::MaterializedBuffer => "materialized-buffer",
            Self::AliasView => "alias-view",
            Self::OpaqueRuntimeValue => "opaque-runtime-value",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::MaterializedBuffer => 0x01,
            Self::AliasView => 0x02,
            Self::OpaqueRuntimeValue => 0x03,
        }
    }

    /// Whether this form discharges a requirement for `required`.
    ///
    /// Written as a full pairwise match rather than an equality test so that a
    /// fourth form is a build error here — the arms state which pairs compose,
    /// and a new form has no defensible default.
    const fn satisfies(self, required: Self) -> bool {
        match (self, required) {
            (Self::MaterializedBuffer, Self::MaterializedBuffer)
            | (Self::AliasView, Self::AliasView)
            | (Self::OpaqueRuntimeValue, Self::OpaqueRuntimeValue) => true,
            (Self::MaterializedBuffer, Self::AliasView | Self::OpaqueRuntimeValue)
            | (Self::AliasView, Self::MaterializedBuffer | Self::OpaqueRuntimeValue)
            | (Self::OpaqueRuntimeValue, Self::MaterializedBuffer | Self::AliasView) => false,
        }
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
    }
}

/// A symbolic execution affinity.
///
/// ADR 0047 requires portable plans to use symbolic affinities rather than
/// runtime ordinals: "Bare runtime ordinals are not portable identities."
/// Runtime preflight binds one to a live device; nothing here does.
///
/// Unlike [`MemoryDomainClass`], this is an open vocabulary of names rather than
/// a closed classification, and the asymmetry is ADR 0047's. A memory domain
/// class is a kind of node in its capability multigraph, so the set of kinds is
/// the model; an affinity is a *symbol* a target profile declares, so a profile
/// with two accelerators names two of them without changing the model. What the
/// ADR forbids is reading meaning out of the name, which is why satisfaction
/// below is equality and nothing else.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExecutionAffinity(&'static str);

impl ExecutionAffinity {
    /// The bounded profile's single symbolic affinity.
    ///
    /// ADR 0047's initial execution profile is "one symbolic affinity, one live
    /// device, and one ordered command stream", with every stage, temporary, and
    /// output using that affinity. A second affinity is what makes cross-device
    /// transfer enforcers reachable, and the ADR rejects those as executable
    /// until a separate transfer/synchronization contract exists.
    pub(crate) const PRIMARY: Self = Self("tiler.affinity.primary");

    /// Names a symbolic affinity a target profile declares.
    ///
    /// The key is `&'static str` so a profile's declared affinities are
    /// compile-time data rather than assembled strings, matching how
    /// `crate::frontier::TargetApplicability` holds governed target keys.
    pub(crate) const fn new(key: &'static str) -> Self {
        Self(key)
    }

    /// The governed canonical key naming this affinity.
    pub(crate) const fn key(self) -> &'static str {
        self.0
    }

    /// Whether this affinity discharges a requirement for `required`.
    ///
    /// Equality, and deliberately nothing more. ADR 0047 forbids inferring any
    /// implicit meaning from a name, so no affinity is substitutable for another
    /// on the strength of what it is called.
    fn satisfies(self, required: Self) -> bool {
        self.0 == required.0
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        push_slice(bytes, self.0.as_bytes());
    }
}

impl fmt::Display for ExecutionAffinity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// A class of memory domain a boundary value's allocation may live in.
///
/// The classes are a closed vocabulary and carry no order. ADR 0047 is explicit:
/// "No portable total order or implicit meaning is inferred from domain names",
/// and it rejects a `Disk < RAM < VRAM < Scratch` enum precisely because such an
/// order cannot represent unified physical memory, pairwise peer access, or
/// execution-scoped scratch. That is why a requirement names an admitted *set*
/// and satisfaction is membership rather than comparison.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum MemoryDomainClass {
    /// Storage explicitly addressable by the device executing the region.
    Device,
    /// Storage addressable by both a host and a device without an explicit copy.
    ///
    /// Reserved: the bounded profile allocates nothing in it.
    Shared,
    /// Storage a host addresses directly.
    ///
    /// Reserved: the bounded profile requires its inputs to be device-accessible
    /// already.
    HostVisible,
    /// Storage whose lifetime is scoped to one execution.
    ///
    /// Reserved. ADR 0047 keeps execution-local scratch a scoped-lifetime domain
    /// rather than a level of a hierarchy, and no boundary value in the bounded
    /// profile lives in one.
    ExecutionScratch,
    /// Storage whose backing is owned outside the compiler.
    ///
    /// Reserved. ADR 0047 makes external storage an explicit I/O resource reached
    /// by import/read stages, which the initial profile rejects.
    Imported,
}

impl MemoryDomainClass {
    /// The governed canonical key naming this class.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Device => "device",
            Self::Shared => "shared",
            Self::HostVisible => "host-visible",
            Self::ExecutionScratch => "execution-scratch",
            Self::Imported => "imported",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Device => 0x01,
            Self::Shared => 0x02,
            Self::HostVisible => 0x03,
            Self::ExecutionScratch => 0x04,
            Self::Imported => 0x05,
        }
    }
}

impl fmt::Display for MemoryDomainClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

/// The memory-domain classes a consumer admits for an incoming value.
///
/// ADR 0047's requirement field is an "admitted memory domain", and the plural
/// form is what keeps satisfaction from becoming an order: a consumer that can
/// read from either shared or device storage says so, and a producer in either
/// discharges it, without any claim that one domain is above the other.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdmittedMemoryDomains(Vec<MemoryDomainClass>);

impl AdmittedMemoryDomains {
    /// Builds an admitted set, normalized to canonical ascending order without
    /// duplicates.
    ///
    /// Returns `None` for an empty set. An empty admission is not a strict
    /// requirement but an unsatisfiable one, and a requirement that nothing can
    /// ever discharge is a malformed input rather than a plan rejection.
    pub(crate) fn new(classes: impl IntoIterator<Item = MemoryDomainClass>) -> Option<Self> {
        let mut admitted: Vec<MemoryDomainClass> = classes.into_iter().collect();
        admitted.sort_unstable();
        admitted.dedup();
        if admitted.is_empty() {
            None
        } else {
            Some(Self(admitted))
        }
    }

    /// The admitted classes in canonical order.
    pub(crate) fn classes(&self) -> &[MemoryDomainClass] {
        &self.0
    }

    /// Whether `class` is admitted.
    fn admits(&self, class: MemoryDomainClass) -> bool {
        self.0.contains(&class)
    }

    /// Whether every class this set admits is also admitted by `other`.
    fn is_subset_of(&self, other: &Self) -> bool {
        self.0.iter().all(|class| other.admits(*class))
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        push_len(bytes, self.0.len());
        for class in &self.0 {
            bytes.push(class.tag());
        }
    }
}

impl fmt::Display for AdmittedMemoryDomains {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("{")?;
        for (index, class) in self.0.iter().enumerate() {
            if index != 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "{class}")?;
        }
        formatter.write_str("}")
    }
}

/// The dependency after which a consumer needs a boundary value to be usable.
///
/// This states the *kind* of dependency, not that any schedule discharges it.
/// ADR 0047 places enforcers, new materialized versions, and dependencies at
/// `KernelProgram` scope, so which command precedes which is decided there and
/// witnessed there. What this dimension contributes is that a consumer whose
/// dependency no producer can name is rejected here rather than discovered at
/// encoding time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AvailabilityRequirement {
    /// Usable once the dispatch that produces it has completed on the same
    /// ordered command stream.
    AfterProducingDispatch,
    /// Usable by a host read after terminal completion has been observed.
    ///
    /// Reserved. ADR 0033 makes host observation a separate boundary — terminal
    /// completion, a post-completion status check, error-record visibility, and
    /// only then semantic interpretation — and no boundary guarantee in this
    /// vocabulary discharges it. A requirement naming it therefore rejects
    /// explicitly rather than being served by an ordinary dispatch dependency.
    AfterObservedHostCompletion,
}

impl AvailabilityRequirement {
    /// The governed canonical key naming this requirement.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::AfterProducingDispatch => "after-producing-dispatch",
            Self::AfterObservedHostCompletion => "after-observed-host-completion",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::AfterProducingDispatch => 0x01,
            Self::AfterObservedHostCompletion => 0x02,
        }
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
    }
}

/// The dependency after which a producer's boundary value becomes usable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AvailabilityGuarantee {
    /// Usable once this region's own dispatch has completed.
    AfterOwnDispatch,
}

impl AvailabilityGuarantee {
    /// The governed canonical key naming this guarantee.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::AfterOwnDispatch => "after-own-dispatch",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::AfterOwnDispatch => 0x01,
        }
    }

    /// Whether this guarantee discharges `required`.
    ///
    /// A value usable after its producer's dispatch discharges a consumer that
    /// waits for the producing dispatch, and discharges nothing else: a host
    /// readback needs observed terminal completion, which a dispatch dependency
    /// does not establish (ADR 0033).
    const fn satisfies(self, required: AvailabilityRequirement) -> bool {
        match (self, required) {
            (Self::AfterOwnDispatch, AvailabilityRequirement::AfterProducingDispatch) => true,
            (Self::AfterOwnDispatch, AvailabilityRequirement::AfterObservedHostCompletion) => false,
        }
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
    }
}

/// The visibility a consumer needs of an incoming value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VisibilityRequirement {
    /// Reads on the requiring affinity see the produced value with no further
    /// coherence action.
    ReadableOnRequiringAffinity,
}

impl VisibilityRequirement {
    /// The governed canonical key naming this requirement.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::ReadableOnRequiringAffinity => "readable-on-requiring-affinity",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::ReadableOnRequiringAffinity => 0x01,
        }
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
    }
}

/// ADR 0047's "visibility state": what a consumer sees once the value's
/// availability dependency is met.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VisibilityGuarantee {
    /// Coherent to reads on the producing affinity once the availability
    /// dependency is met, with no further action.
    CoherentOnProducingAffinity,
    /// Coherent only after an explicit coherence action a later stage must
    /// perform.
    ///
    /// Reserved, and the reason it exists is that it must not be satisfiable.
    /// ADR 0047 makes an affinity-to-domain edge declare its own visibility and
    /// coherence requirements, so a domain that owes a flush or invalidate can
    /// be guaranteed by a producer and is not readable by a consumer until an
    /// enforcer supplies the action. Modelling that as a satisfied guarantee at
    /// a higher cost is exactly the substitution ADR 0043 forbids.
    RequiresExplicitCoherenceAction,
}

impl VisibilityGuarantee {
    /// The governed canonical key naming this guarantee.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::CoherentOnProducingAffinity => "coherent-on-producing-affinity",
            Self::RequiresExplicitCoherenceAction => "requires-explicit-coherence-action",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::CoherentOnProducingAffinity => 0x01,
            Self::RequiresExplicitCoherenceAction => 0x02,
        }
    }

    /// Whether this guarantee discharges `required`.
    const fn satisfies(self, required: VisibilityRequirement) -> bool {
        match (self, required) {
            (
                Self::CoherentOnProducingAffinity,
                VisibilityRequirement::ReadableOnRequiringAffinity,
            ) => true,
            (
                Self::RequiresExplicitCoherenceAction,
                VisibilityRequirement::ReadableOnRequiringAffinity,
            ) => false,
        }
    }

    fn encode(self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
    }
}

/// One typed property a region implementation requires of an incoming value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RequiredProperty {
    /// The storage layout the incoming value must have.
    StorageLayout(LayoutRequirement),
    /// The storage encoding the incoming value must have.
    StorageEncoding(StorageEncoding),
    /// The minimum byte alignment the incoming value must have.
    Alignment(AlignmentRequirement),
    /// The form the incoming value must take.
    Materialization(MaterializationForm),
    /// The symbolic affinity the incoming value must be placed for.
    ExecutionAffinity(ExecutionAffinity),
    /// The memory-domain classes the incoming value may live in.
    MemoryDomain(AdmittedMemoryDomains),
    /// The dependency after which the incoming value must be usable.
    Availability(AvailabilityRequirement),
    /// The visibility the incoming value must have.
    Visibility(VisibilityRequirement),
}

impl RequiredProperty {
    /// The dimension this requirement ranges over.
    pub(crate) const fn property(&self) -> BoundaryProperty {
        match self {
            Self::StorageLayout(_) => BoundaryProperty::StorageLayout,
            Self::StorageEncoding(_) => BoundaryProperty::StorageEncoding,
            Self::Alignment(_) => BoundaryProperty::Alignment,
            Self::Materialization(_) => BoundaryProperty::Materialization,
            Self::ExecutionAffinity(_) => BoundaryProperty::ExecutionAffinity,
            Self::MemoryDomain(_) => BoundaryProperty::MemoryDomain,
            Self::Availability(_) => BoundaryProperty::Availability,
            Self::Visibility(_) => BoundaryProperty::Visibility,
        }
    }

    /// The governed canonical key naming this requirement's value.
    ///
    /// The two set- and quantity-valued dimensions name their *shape* rather
    /// than their contents, because a key is a stable vocabulary term and an
    /// alignment or an admitted set has no bounded set of terms. Their values are
    /// rendered by [`fmt::Display`] instead.
    pub(crate) const fn value_key(&self) -> &'static str {
        match self {
            Self::StorageLayout(value) => value.key(),
            Self::StorageEncoding(value) => storage_key(*value),
            Self::Alignment(_) => "byte-alignment",
            Self::Materialization(value) => value.key(),
            Self::ExecutionAffinity(value) => value.key(),
            Self::MemoryDomain(_) => "admitted-memory-domains",
            Self::Availability(value) => value.key(),
            Self::Visibility(value) => value.key(),
        }
    }

    /// Whether this requirement is well formed.
    ///
    /// A malformed requirement is a compiler fault, not a plan rejection: the
    /// distinction matters because nothing could ever discharge one, so reporting
    /// it as an unsatisfied property would present a bug as a considered verdict.
    /// Only the dimensions whose value spaces have an invalid inhabitant can fail;
    /// the rest are well formed by their type.
    const fn is_well_formed(&self) -> bool {
        match self {
            Self::StorageLayout(value) => value.is_well_formed(),
            Self::StorageEncoding(_)
            | Self::Alignment(_)
            | Self::Materialization(_)
            | Self::ExecutionAffinity(_)
            | Self::MemoryDomain(_)
            | Self::Availability(_)
            | Self::Visibility(_) => true,
        }
    }

    /// Whether every guarantee that discharges `other` also discharges this
    /// requirement.
    ///
    /// This is the "boundary requirements are no stronger" half of the accepted
    /// dominance relation. It is a partial order: two requirements on the same
    /// dimension may be incomparable, and requirements on different dimensions
    /// always are, so a caller compares dimension by dimension rather than
    /// collapsing to one verdict.
    ///
    /// The closing wildcard is the mismatched-dimension arm and is deliberately
    /// fail-closed rather than exhaustive. Convention 3's ban on wildcards is
    /// about encoders, where an unmatched pair produces bytes that mean something
    /// wrong; here an unmatched pair produces `false`, which withholds dominance
    /// and therefore prunes nothing. A ninth dimension whose arm is forgotten
    /// makes this relation conservative, never permissive.
    pub(crate) fn is_no_stronger_than(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StorageLayout(mine), Self::StorageLayout(theirs)) => {
                layout_requirement_is_no_stronger(*mine, *theirs)
            }
            // Equality within a family and no relation across families: an
            // encoding requirement admits exactly one encoding, so one is weaker
            // than another only by being the same.
            (Self::StorageEncoding(mine), Self::StorageEncoding(theirs)) => mine == theirs,
            // A smaller alignment is weaker exactly when it divides the larger:
            // everything 16-byte aligned is 4-byte aligned, so requiring 4 is no
            // stronger than requiring 16.
            (Self::Alignment(mine), Self::Alignment(theirs)) => {
                AlignmentGuarantee::from_alignment(theirs.alignment()).satisfies(*mine)
            }
            (Self::Materialization(mine), Self::Materialization(theirs)) => mine == theirs,
            (Self::ExecutionAffinity(mine), Self::ExecutionAffinity(theirs)) => mine == theirs,
            // A superset admits more producers, so it is the weaker requirement.
            (Self::MemoryDomain(mine), Self::MemoryDomain(theirs)) => theirs.is_subset_of(mine),
            (Self::Availability(mine), Self::Availability(theirs)) => {
                availability_requirement_is_no_stronger(*mine, *theirs)
            }
            (Self::Visibility(mine), Self::Visibility(theirs)) => mine == theirs,
            _ => false,
        }
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.property().tag());
        match self {
            Self::StorageLayout(value) => value.encode(bytes),
            Self::StorageEncoding(value) => encode_storage(*value, bytes),
            Self::Alignment(value) => bytes.extend_from_slice(&value.bytes().to_be_bytes()),
            Self::Materialization(value) => value.encode(bytes),
            Self::ExecutionAffinity(value) => value.encode(bytes),
            Self::MemoryDomain(value) => value.encode(bytes),
            Self::Availability(value) => value.encode(bytes),
            Self::Visibility(value) => value.encode(bytes),
        }
    }
}

impl fmt::Display for RequiredProperty {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alignment(value) => write!(formatter, "{}={value}", self.property()),
            Self::MemoryDomain(value) => write!(formatter, "{}={value}", self.property()),
            Self::StorageLayout(_)
            | Self::StorageEncoding(_)
            | Self::Materialization(_)
            | Self::ExecutionAffinity(_)
            | Self::Availability(_)
            | Self::Visibility(_) => {
                write!(formatter, "{}={}", self.property(), self.value_key())
            }
        }
    }
}

/// Whether a layout requirement is no stronger than another.
///
/// A separate function rather than an inline arm because it is a genuine
/// two-dimensional comparison: a dense requirement admits only dense producers,
/// while a unit-stride requirement on the last axis also admits any future
/// layout with that stride, so the dense one is the stronger of the pair.
const fn layout_requirement_is_no_stronger(
    mine: LayoutRequirement,
    theirs: LayoutRequirement,
) -> bool {
    match (mine, theirs) {
        (LayoutRequirement::DenseRowMajor, LayoutRequirement::DenseRowMajor) => true,
        // A dense requirement rejects producers a trailing-axis unit-stride
        // requirement would accept, so it is never the weaker of the two.
        (LayoutRequirement::DenseRowMajor, LayoutRequirement::UnitStrideOnAxis { .. }) => false,
        (LayoutRequirement::UnitStrideOnAxis { axis, rank }, LayoutRequirement::DenseRowMajor) => {
            rank > 0 && axis.get() == rank - 1
        }
        (
            LayoutRequirement::UnitStrideOnAxis { axis, rank },
            LayoutRequirement::UnitStrideOnAxis {
                axis: other_axis,
                rank: other_rank,
            },
        ) => axis.get() == other_axis.get() && rank == other_rank,
    }
}

/// Whether an availability requirement is no stronger than another.
/// Observed host completion implies the producing dispatch completed, so a
/// dispatch dependency is the weaker of the two and a host-readback dependency
/// is never weaker than one.
const fn availability_requirement_is_no_stronger(
    mine: AvailabilityRequirement,
    theirs: AvailabilityRequirement,
) -> bool {
    match (mine, theirs) {
        (
            AvailabilityRequirement::AfterProducingDispatch,
            AvailabilityRequirement::AfterProducingDispatch
            | AvailabilityRequirement::AfterObservedHostCompletion,
        )
        | (
            AvailabilityRequirement::AfterObservedHostCompletion,
            AvailabilityRequirement::AfterObservedHostCompletion,
        ) => true,
        (
            AvailabilityRequirement::AfterObservedHostCompletion,
            AvailabilityRequirement::AfterProducingDispatch,
        ) => false,
    }
}

/// One typed property a region implementation guarantees of an outgoing value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GuaranteedProperty {
    /// The storage layout the outgoing value has.
    StorageLayout(LayoutGuarantee),
    /// The storage encoding the outgoing value has.
    StorageEncoding(StorageEncoding),
    /// The byte alignment the outgoing value has.
    Alignment(AlignmentGuarantee),
    /// The form the outgoing value takes.
    Materialization(MaterializationForm),
    /// The symbolic affinity the outgoing value is placed for.
    ExecutionAffinity(ExecutionAffinity),
    /// The memory-domain class the outgoing value's allocation lives in.
    MemoryDomain(MemoryDomainClass),
    /// The dependency after which the outgoing value is usable.
    Availability(AvailabilityGuarantee),
    /// The visibility state the outgoing value is delivered in.
    Visibility(VisibilityGuarantee),
}

impl GuaranteedProperty {
    /// The dimension this guarantee ranges over.
    pub(crate) const fn property(&self) -> BoundaryProperty {
        match self {
            Self::StorageLayout(_) => BoundaryProperty::StorageLayout,
            Self::StorageEncoding(_) => BoundaryProperty::StorageEncoding,
            Self::Alignment(_) => BoundaryProperty::Alignment,
            Self::Materialization(_) => BoundaryProperty::Materialization,
            Self::ExecutionAffinity(_) => BoundaryProperty::ExecutionAffinity,
            Self::MemoryDomain(_) => BoundaryProperty::MemoryDomain,
            Self::Availability(_) => BoundaryProperty::Availability,
            Self::Visibility(_) => BoundaryProperty::Visibility,
        }
    }

    /// The governed canonical key naming this guarantee's value.
    pub(crate) const fn value_key(&self) -> &'static str {
        match self {
            Self::StorageLayout(value) => value.key(),
            Self::StorageEncoding(value) => storage_key(*value),
            Self::Alignment(_) => "byte-alignment",
            Self::Materialization(value) => value.key(),
            Self::ExecutionAffinity(value) => value.key(),
            Self::MemoryDomain(value) => value.key(),
            Self::Availability(value) => value.key(),
            Self::Visibility(value) => value.key(),
        }
    }

    /// Whether this guarantee is well formed.
    const fn is_well_formed(&self) -> bool {
        match self {
            Self::StorageEncoding(_)
            | Self::StorageLayout(_)
            | Self::Alignment(_)
            | Self::Materialization(_)
            | Self::ExecutionAffinity(_)
            | Self::MemoryDomain(_)
            | Self::Availability(_)
            | Self::Visibility(_) => true,
        }
    }

    /// Whether this guarantee discharges `required`.
    ///
    /// A guarantee on one dimension never discharges a requirement on another.
    /// That is the fail-closed property this relation exists for: a boundary
    /// whose consumer names a dimension the producer is silent about must not
    /// pass, because silence is not a guarantee. The closing wildcard is that
    /// mismatched-dimension arm, and an unmatched pair yields `false` — a
    /// rejected boundary, never an admitted one.
    pub(crate) fn satisfies(&self, required: &RequiredProperty) -> bool {
        match (self, required) {
            (Self::StorageLayout(mine), RequiredProperty::StorageLayout(needed)) => {
                mine.satisfies(*needed)
            }
            (Self::StorageEncoding(mine), RequiredProperty::StorageEncoding(needed)) => {
                storage_satisfies(*mine, *needed)
            }
            (Self::Alignment(mine), RequiredProperty::Alignment(needed)) => mine.satisfies(*needed),
            (Self::Materialization(mine), RequiredProperty::Materialization(needed)) => {
                mine.satisfies(*needed)
            }
            (Self::ExecutionAffinity(mine), RequiredProperty::ExecutionAffinity(needed)) => {
                mine.satisfies(*needed)
            }
            (Self::MemoryDomain(mine), RequiredProperty::MemoryDomain(needed)) => {
                needed.admits(*mine)
            }
            (Self::Availability(mine), RequiredProperty::Availability(needed)) => {
                mine.satisfies(*needed)
            }
            (Self::Visibility(mine), RequiredProperty::Visibility(needed)) => {
                mine.satisfies(*needed)
            }
            _ => false,
        }
    }

    /// Whether every requirement `other` discharges is also discharged by this
    /// guarantee.
    ///
    /// This is the "guarantees are at least as strong" half of the accepted
    /// dominance relation, and like its requirement counterpart it is a partial
    /// order rather than a total one. Its closing wildcard is fail-closed for the
    /// same reason: an unmatched pair withholds dominance and prunes nothing.
    pub(crate) fn is_at_least_as_strong_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StorageLayout(mine), Self::StorageLayout(theirs)) => mine == theirs,
            (Self::StorageEncoding(mine), Self::StorageEncoding(theirs)) => mine == theirs,
            // A 16-byte guarantee discharges every requirement a 4-byte guarantee
            // does, and some it does not.
            (Self::Alignment(mine), Self::Alignment(theirs)) => {
                mine.satisfies(AlignmentRequirement::from_alignment(theirs.alignment()))
            }
            (Self::Materialization(mine), Self::Materialization(theirs)) => mine == theirs,
            (Self::ExecutionAffinity(mine), Self::ExecutionAffinity(theirs)) => mine == theirs,
            (Self::MemoryDomain(mine), Self::MemoryDomain(theirs)) => mine == theirs,
            (Self::Availability(mine), Self::Availability(theirs)) => mine == theirs,
            (Self::Visibility(mine), Self::Visibility(theirs)) => {
                visibility_guarantee_is_at_least_as_strong(*mine, *theirs)
            }
            _ => false,
        }
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.property().tag());
        match self {
            Self::StorageLayout(value) => value.encode(bytes),
            Self::StorageEncoding(value) => encode_storage(*value, bytes),
            Self::Alignment(value) => bytes.extend_from_slice(&value.bytes().to_be_bytes()),
            Self::Materialization(value) => value.encode(bytes),
            Self::ExecutionAffinity(value) => value.encode(bytes),
            Self::MemoryDomain(value) => bytes.push(value.tag()),
            Self::Availability(value) => value.encode(bytes),
            Self::Visibility(value) => value.encode(bytes),
        }
    }
}

impl fmt::Display for GuaranteedProperty {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alignment(value) => write!(formatter, "{}={value}", self.property()),
            Self::StorageLayout(_)
            | Self::StorageEncoding(_)
            | Self::Materialization(_)
            | Self::ExecutionAffinity(_)
            | Self::MemoryDomain(_)
            | Self::Availability(_)
            | Self::Visibility(_) => {
                write!(formatter, "{}={}", self.property(), self.value_key())
            }
        }
    }
}

/// Whether a visibility guarantee is at least as strong as another.
const fn visibility_guarantee_is_at_least_as_strong(
    mine: VisibilityGuarantee,
    theirs: VisibilityGuarantee,
) -> bool {
    match (mine, theirs) {
        (VisibilityGuarantee::CoherentOnProducingAffinity, _)
        | (
            VisibilityGuarantee::RequiresExplicitCoherenceAction,
            VisibilityGuarantee::RequiresExplicitCoherenceAction,
        ) => true,
        (
            VisibilityGuarantee::RequiresExplicitCoherenceAction,
            VisibilityGuarantee::CoherentOnProducingAffinity,
        ) => false,
    }
}

/// A malformed boundary property set: compiler output, never a plan verdict.
///
/// ADR 0074 convention 1: distinct failure kinds are distinct variants carrying
/// the structured data a caller reacts to, not a preformatted message. Each of
/// these says the set could not describe a boundary at all, which is different
/// from a set that describes one no producer serves — that outcome is an
/// [`UnsatisfiedProperty`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BoundaryPropertyError {
    /// One dimension was named twice in one set.
    ///
    /// A set with two values on one dimension has no defined satisfaction: the
    /// two could disagree, and picking either would be an invented answer.
    DuplicateProperty {
        /// The dimension named more than once.
        property: BoundaryProperty,
    },
    /// A value is not an inhabitant its dimension admits.
    MalformedValue {
        /// The dimension whose value was malformed.
        property: BoundaryProperty,
        /// The governed key naming the malformed value.
        value: &'static str,
    },
}

impl BoundaryPropertyError {
    /// The stable reason code of the fault.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::DuplicateProperty { .. } => "duplicate-property",
            Self::MalformedValue { .. } => "malformed-property-value",
        }
    }

    /// The dimension the fault is about.
    pub(crate) const fn property(&self) -> BoundaryProperty {
        match self {
            Self::DuplicateProperty { property } | Self::MalformedValue { property, .. } => {
                *property
            }
        }
    }
}

impl fmt::Display for BoundaryPropertyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateProperty { property } => {
                write!(formatter, "boundary.duplicate-property: {property}")
            }
            Self::MalformedValue { property, value } => {
                write!(
                    formatter,
                    "boundary.malformed-property-value: {property}={value}"
                )
            }
        }
    }
}

impl std::error::Error for BoundaryPropertyError {}

/// The typed properties one boundary requires of an incoming value.
///
/// The set holds at most one value per dimension, in canonical dimension order,
/// so two sets over the same requirements share one identity encoding regardless
/// of the order they were assembled in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RequiredProperties(Vec<RequiredProperty>);

impl RequiredProperties {
    /// Builds a requirement set, normalized to canonical dimension order.
    ///
    /// # Errors
    ///
    /// Returns [`BoundaryPropertyError`] when a dimension is named twice or a
    /// value is not one its dimension admits.
    pub(crate) fn new(
        properties: impl IntoIterator<Item = RequiredProperty>,
    ) -> Result<Self, BoundaryPropertyError> {
        let mut collected: Vec<RequiredProperty> = properties.into_iter().collect();
        for property in &collected {
            if !property.is_well_formed() {
                return Err(BoundaryPropertyError::MalformedValue {
                    property: property.property(),
                    value: property.value_key(),
                });
            }
        }
        collected.sort_by_key(RequiredProperty::property);
        if let Some(duplicate) = first_duplicate(collected.iter().map(RequiredProperty::property)) {
            return Err(BoundaryPropertyError::DuplicateProperty {
                property: duplicate,
            });
        }
        Ok(Self(collected))
    }

    /// The requirements in canonical dimension order.
    pub(crate) fn properties(&self) -> &[RequiredProperty] {
        &self.0
    }

    /// The requirement on `property`, when the set names one.
    pub(crate) fn get(&self, property: BoundaryProperty) -> Option<&RequiredProperty> {
        self.0.iter().find(|entry| entry.property() == property)
    }

    /// Whether this set is no stronger than `other` on every dimension.
    ///
    /// A dimension `other` names and this set does not is *weaker*, not stronger:
    /// requiring nothing on a dimension admits every producer. A dimension this
    /// set names and `other` does not is therefore stronger and fails the test.
    pub(crate) fn is_no_stronger_than(&self, other: &Self) -> bool {
        self.0.iter().all(|mine| {
            other
                .get(mine.property())
                .is_some_and(|theirs| mine.is_no_stronger_than(theirs))
        })
    }

    /// Appends this set's canonical bytes to a larger encoding.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        push_len(bytes, self.0.len());
        for property in &self.0 {
            property.encode(bytes);
        }
    }
}

/// The typed properties one boundary guarantees of an outgoing value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GuaranteedProperties(Vec<GuaranteedProperty>);

impl GuaranteedProperties {
    /// Builds a guarantee set, normalized to canonical dimension order.
    ///
    /// # Errors
    ///
    /// Returns [`BoundaryPropertyError`] when a dimension is named twice or a
    /// value is not one its dimension admits.
    pub(crate) fn new(
        properties: impl IntoIterator<Item = GuaranteedProperty>,
    ) -> Result<Self, BoundaryPropertyError> {
        let mut collected: Vec<GuaranteedProperty> = properties.into_iter().collect();
        for property in &collected {
            if !property.is_well_formed() {
                return Err(BoundaryPropertyError::MalformedValue {
                    property: property.property(),
                    value: property.value_key(),
                });
            }
        }
        collected.sort_by_key(GuaranteedProperty::property);
        if let Some(duplicate) = first_duplicate(collected.iter().map(GuaranteedProperty::property))
        {
            return Err(BoundaryPropertyError::DuplicateProperty {
                property: duplicate,
            });
        }
        Ok(Self(collected))
    }

    /// The guarantees in canonical dimension order.
    pub(crate) fn properties(&self) -> &[GuaranteedProperty] {
        &self.0
    }

    /// The guarantee on `property`, when the set names one.
    pub(crate) fn get(&self, property: BoundaryProperty) -> Option<&GuaranteedProperty> {
        self.0.iter().find(|entry| entry.property() == property)
    }

    /// Whether this set is at least as strong as `other` on every dimension.
    ///
    /// A dimension `other` guarantees and this set does not is weaker, so the
    /// test fails; a dimension this set guarantees and `other` does not is extra
    /// strength and does not.
    pub(crate) fn is_at_least_as_strong_as(&self, other: &Self) -> bool {
        other.0.iter().all(|theirs| {
            self.get(theirs.property())
                .is_some_and(|mine| mine.is_at_least_as_strong_as(theirs))
        })
    }

    /// Appends this set's canonical bytes to a larger encoding.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        push_len(bytes, self.0.len());
        for property in &self.0 {
            property.encode(bytes);
        }
    }
}

/// Why one required property was not discharged.
///
/// The two reasons are kept apart because they say different things to a caller
/// and to an enforcer: a producer that guarantees the wrong value on a dimension
/// may be reconciled by an enforcer that supplies the right one, while a producer
/// silent on the dimension has made no claim an enforcer can start from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnsatisfiedReason {
    /// The producer guarantees nothing on the required dimension.
    NotGuaranteed,
    /// The producer guarantees a value on the dimension that does not discharge
    /// the requirement.
    NotSatisfied,
}

impl UnsatisfiedReason {
    /// The stable reason code.
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::NotGuaranteed => "property-not-guaranteed",
            Self::NotSatisfied => "property-not-satisfied",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::NotGuaranteed => 0x01,
            Self::NotSatisfied => 0x02,
        }
    }
}

/// One required property a guarantee set did not discharge, with what was asked
/// and what was offered.
///
/// This is the explain shape a boundary rejection reports: the dimension, its
/// governed key, the required value, the guaranteed value when there was one, and
/// a stable reason code. It is never a cost.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnsatisfiedProperty {
    property: BoundaryProperty,
    required: RequiredProperty,
    guaranteed: Option<GuaranteedProperty>,
    reason: UnsatisfiedReason,
}

impl UnsatisfiedProperty {
    /// The dimension that was not discharged.
    pub(crate) const fn property(&self) -> BoundaryProperty {
        self.property
    }

    /// The value the consumer required.
    pub(crate) const fn required(&self) -> &RequiredProperty {
        &self.required
    }

    /// The value the producer guaranteed, when it guaranteed one.
    pub(crate) const fn guaranteed(&self) -> Option<&GuaranteedProperty> {
        self.guaranteed.as_ref()
    }

    /// Why the requirement was not discharged.
    pub(crate) const fn reason(&self) -> UnsatisfiedReason {
        self.reason
    }

    /// Appends this record's canonical bytes.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.property.tag());
        bytes.push(self.reason.tag());
        self.required.encode(bytes);
        match &self.guaranteed {
            Some(guaranteed) => {
                bytes.push(1);
                guaranteed.encode(bytes);
            }
            None => bytes.push(0),
        }
    }
}

impl fmt::Display for UnsatisfiedProperty {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.guaranteed {
            Some(guaranteed) => write!(
                formatter,
                "boundary.{}: required {}, guaranteed {guaranteed}",
                self.reason.code(),
                self.required,
            ),
            None => write!(
                formatter,
                "boundary.{}: required {}",
                self.reason.code(),
                self.required,
            ),
        }
    }
}

/// Decides whether `guaranteed` discharges every requirement in `required`.
///
/// Returns every unsatisfied requirement in canonical dimension order rather
/// than the first, because an explanation that names one missing property at a
/// time makes a caller rediscover the rest one recompilation apart. An empty
/// result means the boundary composes.
///
/// A dimension the guarantee set is silent on is [`UnsatisfiedReason::NotGuaranteed`],
/// never a pass. This is the whole point of the relation: nothing may be assumed
/// of a producer that did not claim it.
pub(crate) fn unsatisfied_properties(
    required: &RequiredProperties,
    guaranteed: &GuaranteedProperties,
) -> Vec<UnsatisfiedProperty> {
    let mut unsatisfied = Vec::new();
    for requirement in required.properties() {
        let property = requirement.property();
        match guaranteed.get(property) {
            Some(offer) if offer.satisfies(requirement) => {}
            Some(offer) => unsatisfied.push(UnsatisfiedProperty {
                property,
                required: requirement.clone(),
                guaranteed: Some(offer.clone()),
                reason: UnsatisfiedReason::NotSatisfied,
            }),
            None => unsatisfied.push(UnsatisfiedProperty {
                property,
                required: requirement.clone(),
                guaranteed: None,
                reason: UnsatisfiedReason::NotGuaranteed,
            }),
        }
    }
    unsatisfied
}

/// A goal requirement that conflicts with what an implementation itself needs.
///
/// Only a dimension that propagates through a region can conflict, because only
/// then do a goal and an implementation both speak about the same input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChildRequirementConflict {
    property: BoundaryProperty,
    goal: RequiredProperty,
    implementation: RequiredProperty,
}

impl ChildRequirementConflict {
    /// The stable reason code.
    ///
    /// A `const` rather than a method, because a struct with one failure shape
    /// has one reason: an accessor that ignores its receiver would imply the
    /// value varies with the instance.
    pub(crate) const REASON: &'static str = "child-requirement-conflict";

    /// The dimension the goal and the implementation disagree on.
    pub(crate) const fn property(&self) -> BoundaryProperty {
        self.property
    }

    /// The value the goal demanded of the region's output.
    pub(crate) const fn goal(&self) -> &RequiredProperty {
        &self.goal
    }

    /// The value the implementation needs of its own input.
    pub(crate) const fn implementation(&self) -> &RequiredProperty {
        &self.implementation
    }
}

impl fmt::Display for ChildRequirementConflict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "boundary.{}: goal {} against implementation {}",
            Self::REASON,
            self.goal,
            self.implementation
        )
    }
}

/// Derives what a region must require of one input, given the goal placed on its
/// output and what the implementation needs of that input by itself.
///
/// This is the accepted rule interface's `child_requirements`. The derivation is
/// per dimension and driven by [`BoundaryProperty::propagates_to_children`]:
///
/// - a propagating dimension takes the goal's value when the goal names one, and
///   fails closed when the implementation names a different value on it, because
///   the region cannot both serve the goal and satisfy itself;
/// - every other dimension takes the implementation's own value, because a goal
///   constrains the value leaving the region and says nothing about the value
///   entering it.
///
/// In the bounded profile only the execution affinity propagates, so the derived
/// set differs from the implementation's own requirements exactly when a goal
/// names an affinity the implementation does not read from — which the single-
/// affinity profile cannot produce, and which the relation rejects rather than
/// approximates when a second affinity exists.
///
/// # Errors
///
/// Returns [`ChildRequirementConflict`] when a propagating dimension is named
/// with different values by the goal and the implementation.
pub(crate) fn derive_child_requirements(
    goal: &RequiredProperties,
    implementation: &RequiredProperties,
) -> Result<RequiredProperties, ChildRequirementConflict> {
    let mut derived: Vec<RequiredProperty> = Vec::new();
    for property in CANONICAL_PROPERTIES {
        let own = implementation.get(property);
        if !property.propagates_to_children() {
            if let Some(own) = own {
                derived.push(own.clone());
            }
            continue;
        }
        match (goal.get(property), own) {
            (Some(inherited), Some(own)) if inherited != own => {
                return Err(ChildRequirementConflict {
                    property,
                    goal: inherited.clone(),
                    implementation: own.clone(),
                });
            }
            (Some(inherited), _) => derived.push(inherited.clone()),
            (None, Some(own)) => derived.push(own.clone()),
            (None, None) => {}
        }
    }
    // Constructed directly rather than through the checked constructor because
    // the invariant already holds by construction: the loop visits each dimension
    // once in canonical order and pushes at most one value per visit, and every
    // value came from a set the constructor already validated. Routing through
    // `new` would add a failure arm nothing can reach and that no caller could
    // act on.
    Ok(RequiredProperties(derived))
}

/// The canonical identity of one boundary's property sets.
///
/// Opaque per ADR 0074 convention 2: the bytes are private, `as_bytes` is the
/// only reader, and there is no public constructor, because the encoder below is
/// what establishes what the bytes name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct BoundaryPropertyIdentity(Vec<u8>);

impl BoundaryPropertyIdentity {
    /// The canonical identity bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Encodes one boundary's required and guaranteed property sets as a standalone
/// identity.
///
/// Domain-separated, length-prefixed, and free of enumeration order: the sets
/// are already in canonical dimension order, so two boundaries assembled in
/// different orders share one identity and two boundaries that differ on any
/// dimension do not.
///
/// This is the encoder the accepted "Possible memo contract" would key an
/// optimization entry on — `(group, boundary requirements, target profile,
/// numerical policy, constraint region)` — and no memo exists, so nothing outside
/// this module's own tests calls it yet. The property bytes that *are* on the
/// compile path today reach identity through
/// [`crate::frontier::BoundaryContract`], which folds each set into an
/// implementation proposal's identity rather than minting a second one.
pub(crate) fn encode_property_identity(
    required: &RequiredProperties,
    guaranteed: &GuaranteedProperties,
) -> BoundaryPropertyIdentity {
    let mut bytes = PROPERTY_SET_IDENTITY_TAG.to_vec();
    required.encode(&mut bytes);
    guaranteed.encode(&mut bytes);
    BoundaryPropertyIdentity(bytes)
}

/// Returns the first dimension that appears twice in a sorted sequence.
fn first_duplicate(
    properties: impl IntoIterator<Item = BoundaryProperty>,
) -> Option<BoundaryProperty> {
    let mut previous: Option<BoundaryProperty> = None;
    for property in properties {
        if previous == Some(property) {
            return Some(property);
        }
        previous = Some(property);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        AdmittedMemoryDomains, AlignmentGuarantee, AlignmentRequirement, AvailabilityGuarantee,
        AvailabilityRequirement, BitPackedEncoding, BoundaryProperty, BoundaryPropertyError,
        ByteAlignment, CANONICAL_PROPERTIES, ChildRequirementConflict, ExecutionAffinity,
        GuaranteedProperties, GuaranteedProperty, LayoutGuarantee, LayoutRequirement,
        MaterializationForm, MemoryDomainClass, PackedBitOrder, PackedTailRule, RequiredProperties,
        RequiredProperty, StorageAccessError, StorageAccessKind, StorageCodecError,
        StorageEncoding, StorageScalar, UnsatisfiedProperty, UnsatisfiedReason,
        VisibilityGuarantee, VisibilityRequirement, code_access, derive_child_requirements,
        encode_property_identity, pack_codes, unpack_codes, unsatisfied_properties,
    };
    use tiler_ir::shape::Axis;

    /// The properties the bounded profile's producers actually guarantee.
    fn bounded_guarantees() -> GuaranteedProperties {
        GuaranteedProperties::new([
            GuaranteedProperty::StorageLayout(LayoutGuarantee::DenseRowMajor),
            GuaranteedProperty::StorageEncoding(StorageEncoding::Unpacked),
            GuaranteedProperty::Alignment(AlignmentGuarantee::natural_for(StorageScalar::F32)),
            GuaranteedProperty::Materialization(MaterializationForm::MaterializedBuffer),
            GuaranteedProperty::ExecutionAffinity(ExecutionAffinity::PRIMARY),
            GuaranteedProperty::MemoryDomain(MemoryDomainClass::Device),
            GuaranteedProperty::Availability(AvailabilityGuarantee::AfterOwnDispatch),
            GuaranteedProperty::Visibility(VisibilityGuarantee::CoherentOnProducingAffinity),
        ])
        .unwrap()
    }

    /// The properties the bounded profile's consumers actually require.
    fn bounded_requirements() -> RequiredProperties {
        RequiredProperties::new([
            RequiredProperty::StorageLayout(LayoutRequirement::DenseRowMajor),
            RequiredProperty::StorageEncoding(StorageEncoding::Unpacked),
            RequiredProperty::Alignment(AlignmentRequirement::natural_for(StorageScalar::F32)),
            RequiredProperty::Materialization(MaterializationForm::MaterializedBuffer),
            RequiredProperty::ExecutionAffinity(ExecutionAffinity::PRIMARY),
            RequiredProperty::MemoryDomain(
                AdmittedMemoryDomains::new([MemoryDomainClass::Device]).unwrap(),
            ),
            RequiredProperty::Availability(AvailabilityRequirement::AfterProducingDispatch),
            RequiredProperty::Visibility(VisibilityRequirement::ReadableOnRequiringAffinity),
        ])
        .unwrap()
    }

    #[test]
    fn the_bounded_profile_contract_composes_on_every_dimension() {
        let unsatisfied = unsatisfied_properties(&bounded_requirements(), &bounded_guarantees());
        assert!(
            unsatisfied.is_empty(),
            "the profile's own producer must serve its own consumer: {unsatisfied:?}"
        );
    }

    #[test]
    fn a_dimension_the_producer_is_silent_on_fails_closed() {
        // Silence is not a guarantee. A producer that says nothing about
        // visibility does not thereby deliver a coherent value.
        let guaranteed = GuaranteedProperties::new(
            bounded_guarantees()
                .properties()
                .iter()
                .filter(|property| property.property() != BoundaryProperty::Visibility)
                .cloned(),
        )
        .unwrap();
        let unsatisfied = unsatisfied_properties(&bounded_requirements(), &guaranteed);
        assert_eq!(unsatisfied.len(), 1);
        assert_eq!(unsatisfied[0].property(), BoundaryProperty::Visibility);
        assert_eq!(unsatisfied[0].reason(), UnsatisfiedReason::NotGuaranteed);
        assert!(unsatisfied[0].guaranteed().is_none());
        assert_eq!(
            unsatisfied[0].reason().code(),
            "property-not-guaranteed",
            "the reason code is what an explanation reports"
        );
    }

    #[test]
    fn alignment_subsumption_is_divisibility_in_one_direction_only() {
        // The accepted contract's own example: 16-byte alignment satisfies a
        // 4-byte requirement, and the converse does not hold.
        let sixteen = AlignmentGuarantee::new(16).unwrap();
        let four = AlignmentRequirement::natural_for(StorageScalar::F32);
        let coarse = GuaranteedProperties::new([GuaranteedProperty::Alignment(sixteen)]).unwrap();
        let fine = GuaranteedProperties::new([GuaranteedProperty::Alignment(
            AlignmentGuarantee::natural_for(StorageScalar::F32),
        )])
        .unwrap();
        let needs_four = RequiredProperties::new([RequiredProperty::Alignment(four)]).unwrap();
        let needs_sixteen = RequiredProperties::new([RequiredProperty::Alignment(
            AlignmentRequirement::new(16).unwrap(),
        )])
        .unwrap();

        assert!(unsatisfied_properties(&needs_four, &coarse).is_empty());
        let refused = unsatisfied_properties(&needs_sixteen, &fine);
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].reason(), UnsatisfiedReason::NotSatisfied);
        assert!(refused[0].guaranteed().is_some());
    }

    /// Alignment is the element type's, and a narrower element gets less of it.
    ///
    /// This is exactly what the constant this replaced could not say. It answered
    /// four bytes for every boundary, which was the right answer for `f32` only
    /// because `f32` was the only element there was; a one-byte carrier told the
    /// same four is being handed a requirement that is not its own. If the `U8`
    /// case ever reports four again, a derivation has been replaced by a constant.
    #[test]
    fn alignment_derives_from_the_element_width_rather_than_the_profile() {
        assert_eq!(
            ByteAlignment::natural_for(StorageScalar::F32).bytes(),
            4,
            "the four-byte carrier stopped deriving four bytes"
        );
        assert_eq!(
            ByteAlignment::natural_for(StorageScalar::U8).bytes(),
            1,
            "a one-byte carrier was told the four-byte carrier's alignment"
        );
    }

    /// No carrier in the vocabulary has a width that cannot be an alignment.
    ///
    /// What this proves is the [`ByteAlignment::natural_for`] panic's absence,
    /// not the widths themselves: the equality below is close to a restatement of
    /// the derivation, but reaching it at all establishes that every carrier's
    /// width is a positive power of two within `u32`. A carrier whose width were
    /// six — legal as a byte count, impossible as an alignment — would otherwise
    /// first reach that panic from a construction site on the compile path.
    ///
    /// The `match` is what keeps the list honest: it has no wildcard, so a widened
    /// [`StorageScalar`] is an `E0004` here and the author has to extend the array
    /// beside it. A bare array would keep passing while silently covering one
    /// fewer variant than the vocabulary has.
    #[test]
    fn every_storage_carrier_has_a_representable_alignment() {
        for scalar in [StorageScalar::U8, StorageScalar::F32, StorageScalar::Bf16] {
            match scalar {
                StorageScalar::U8 | StorageScalar::F32 | StorageScalar::Bf16 => {}
            }
            let width = scalar.byte_width();
            assert!(
                width != 0 && width.is_power_of_two() && u32::try_from(width).is_ok(),
                "{scalar:?} has byte width {width}, which cannot be a byte alignment"
            );
            assert_eq!(
                u64::from(ByteAlignment::natural_for(scalar).bytes()),
                width,
                "{scalar:?} derived an alignment that is not its own byte width"
            );
        }
    }

    #[test]
    fn a_non_power_of_two_alignment_is_rejected_rather_than_compared() {
        // Divisibility is a partial order over powers of two and not over
        // arbitrary integers: 12 and 8 divide neither way, yet a 12-byte-aligned
        // value is not 8-byte aligned.
        assert!(ByteAlignment::new(12).is_err());
        assert!(ByteAlignment::new(0).is_err());
        assert_eq!(ByteAlignment::new(16).unwrap().bytes(), 16);
    }

    #[test]
    fn encoding_subsumption_is_stated_per_family_and_is_not_an_ordering() {
        // Neither direction holds across families: an unpacked producer does not
        // serve a packed requirement by being cheaper to read, and a packed one
        // does not serve an unpacked requirement by being denser.
        let unpacked = GuaranteedProperties::new([GuaranteedProperty::StorageEncoding(
            StorageEncoding::Unpacked,
        )])
        .unwrap();
        let packed = GuaranteedProperties::new([GuaranteedProperty::StorageEncoding(
            StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
        )])
        .unwrap();
        let needs_unpacked =
            RequiredProperties::new([RequiredProperty::StorageEncoding(StorageEncoding::Unpacked)])
                .unwrap();
        let needs_packed = RequiredProperties::new([RequiredProperty::StorageEncoding(
            StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
        )])
        .unwrap();

        assert!(unsatisfied_properties(&needs_unpacked, &unpacked).is_empty());
        assert!(unsatisfied_properties(&needs_packed, &packed).is_empty());
        assert_eq!(unsatisfied_properties(&needs_packed, &unpacked).len(), 1);
        assert_eq!(unsatisfied_properties(&needs_unpacked, &packed).len(), 1);

        // A different packed width is a different family member, not a coarser
        // one that subsumes the other.
        let needs_two_bit = RequiredProperties::new([RequiredProperty::StorageEncoding(
            StorageEncoding::BitPacked(
                BitPackedEncoding::new(
                    2,
                    PackedBitOrder::LeastSignificantElementFirst,
                    PackedTailRule::Zero,
                )
                .unwrap(),
            ),
        )])
        .unwrap();
        assert_eq!(unsatisfied_properties(&needs_two_bit, &packed).len(), 1);
    }

    #[test]
    fn ordinary_u8_codes_have_identity_byte_addresses_and_round_trip_exactly() {
        let encoding = StorageEncoding::Unpacked;
        let access = code_access(encoding, 5, 1, 3, StorageAccessKind::Write).unwrap();
        assert_eq!(access.byte_offset(), 1);
        assert_eq!(access.byte_length(), 3);
        assert_eq!(access.leading_shared_bits(), 0);
        assert_eq!(access.trailing_shared_bits(), 0);
        assert_eq!(access.trailing_padding_bits(), 0);

        let codes = [0x00, 0x7f, 0xff, 0x31, 0x80];
        let bytes = pack_codes(encoding, &codes).unwrap();
        assert_eq!(bytes, codes);
        assert_eq!(unpack_codes(encoding, &bytes, 5).unwrap(), codes);
        assert_eq!(
            unpack_codes(encoding, &bytes, 4),
            Err(StorageCodecError::ByteLength {
                expected: 4,
                actual: 5,
            })
        );
    }

    #[test]
    fn packed_u4_is_lsb_first_and_zero_fills_an_odd_tail() {
        let encoding = StorageEncoding::PACKED_U4_LSB_ZERO_TAIL;
        let codes = [0x0, 0x1, 0xf, 0x7, 0x3];
        let bytes = pack_codes(encoding, &codes).unwrap();
        assert_eq!(bytes, [0x10, 0x7f, 0x03]);
        assert_eq!(unpack_codes(encoding, &bytes, 5).unwrap(), codes);

        assert_eq!(
            pack_codes(encoding, &[0, 0x10]),
            Err(StorageCodecError::CodeOutOfRange {
                index: 1,
                code: 0x10,
            })
        );
        assert_eq!(
            unpack_codes(encoding, &[0x10, 0x7f, 0xf3], 5),
            Err(StorageCodecError::NonCanonicalTail { byte: 0xf3 })
        );
    }

    #[test]
    fn packed_u4_reads_name_shared_edge_bits_and_partial_writes_refuse_them() {
        let encoding = StorageEncoding::PACKED_U4_LSB_ZERO_TAIL;

        let unaligned_read = code_access(encoding, 6, 1, 2, StorageAccessKind::Read).unwrap();
        assert_eq!(unaligned_read.byte_offset(), 0);
        assert_eq!(unaligned_read.byte_length(), 2);
        assert_eq!(unaligned_read.leading_shared_bits(), 4);
        assert_eq!(unaligned_read.trailing_shared_bits(), 4);
        assert_eq!(unaligned_read.trailing_padding_bits(), 0);
        assert_eq!(
            code_access(encoding, 6, 1, 2, StorageAccessKind::Write),
            Err(StorageAccessError::UnalignedPartialWrite)
        );

        let aligned_write = code_access(encoding, 6, 2, 2, StorageAccessKind::Write).unwrap();
        assert_eq!(aligned_write.byte_offset(), 1);
        assert_eq!(aligned_write.byte_length(), 1);
        assert_eq!(aligned_write.leading_shared_bits(), 0);
        assert_eq!(aligned_write.trailing_shared_bits(), 0);
        assert_eq!(aligned_write.trailing_padding_bits(), 0);

        assert_eq!(
            code_access(encoding, 6, 0, 3, StorageAccessKind::Write),
            Err(StorageAccessError::UnalignedPartialWrite),
            "an interior high nibble belongs to the next logical element"
        );
    }

    #[test]
    fn packed_u4_whole_value_writes_own_the_zero_filled_odd_tail_only() {
        let encoding = StorageEncoding::PACKED_U4_LSB_ZERO_TAIL;
        let whole = code_access(encoding, 5, 0, 5, StorageAccessKind::Write).unwrap();
        assert_eq!(whole.byte_offset(), 0);
        assert_eq!(whole.byte_length(), 3);
        assert_eq!(whole.trailing_shared_bits(), 0);
        assert_eq!(whole.trailing_padding_bits(), 4);

        let last = code_access(encoding, 5, 4, 1, StorageAccessKind::Write).unwrap();
        assert_eq!(last.byte_offset(), 2);
        assert_eq!(last.byte_length(), 1);
        assert_eq!(last.trailing_shared_bits(), 0);
        assert_eq!(last.trailing_padding_bits(), 4);

        assert_eq!(
            code_access(encoding, 5, 1, 4, StorageAccessKind::Write),
            Err(StorageAccessError::UnalignedPartialWrite),
            "owning the tail does not authorize overwriting a preceding nibble"
        );
        assert_eq!(
            code_access(encoding, 5, 5, 1, StorageAccessKind::Read),
            Err(StorageAccessError::OutOfBounds)
        );

        let empty = code_access(encoding, 5, 3, 0, StorageAccessKind::Write).unwrap();
        assert_eq!(empty.byte_length(), 0);
        assert_eq!(empty.leading_shared_bits(), 0);
        assert_eq!(empty.trailing_shared_bits(), 0);
        assert_eq!(empty.trailing_padding_bits(), 0);
    }

    #[test]
    fn a_dense_layout_has_unit_stride_on_its_last_axis_and_no_other() {
        let dense = GuaranteedProperties::new([GuaranteedProperty::StorageLayout(
            LayoutGuarantee::DenseRowMajor,
        )])
        .unwrap();
        let trailing = RequiredProperties::new([RequiredProperty::StorageLayout(
            LayoutRequirement::UnitStrideOnAxis {
                axis: Axis::new(1),
                rank: 2,
            },
        )])
        .unwrap();
        let leading = RequiredProperties::new([RequiredProperty::StorageLayout(
            LayoutRequirement::UnitStrideOnAxis {
                axis: Axis::new(0),
                rank: 2,
            },
        )])
        .unwrap();

        assert!(unsatisfied_properties(&trailing, &dense).is_empty());
        let refused = unsatisfied_properties(&leading, &dense);
        assert_eq!(
            refused.len(),
            1,
            "a vectorized reduction over a leading axis needs a layout enforcer, \
             which this authority reports rather than supplies"
        );
        assert_eq!(refused[0].reason(), UnsatisfiedReason::NotSatisfied);
    }

    #[test]
    fn an_axis_outside_the_rank_is_malformed_output_not_an_unsatisfiable_plan() {
        let error = RequiredProperties::new([RequiredProperty::StorageLayout(
            LayoutRequirement::UnitStrideOnAxis {
                axis: Axis::new(2),
                rank: 2,
            },
        )])
        .unwrap_err();
        assert_eq!(error.reason(), "malformed-property-value");
        assert_eq!(error.property(), BoundaryProperty::StorageLayout);

        let zero_rank = RequiredProperties::new([RequiredProperty::StorageLayout(
            LayoutRequirement::UnitStrideOnAxis {
                axis: Axis::new(0),
                rank: 0,
            },
        )])
        .unwrap_err();
        assert_eq!(zero_rank.reason(), "malformed-property-value");
    }

    #[test]
    fn memory_domain_satisfaction_is_admitted_set_membership_and_never_an_order() {
        // A consumer that reads from either shared or device storage is served by
        // a producer in either; nothing infers that one domain outranks another.
        let admits_both = RequiredProperties::new([RequiredProperty::MemoryDomain(
            AdmittedMemoryDomains::new([MemoryDomainClass::Shared, MemoryDomainClass::Device])
                .unwrap(),
        )])
        .unwrap();
        let admits_device = RequiredProperties::new([RequiredProperty::MemoryDomain(
            AdmittedMemoryDomains::new([MemoryDomainClass::Device]).unwrap(),
        )])
        .unwrap();
        let in_device = GuaranteedProperties::new([GuaranteedProperty::MemoryDomain(
            MemoryDomainClass::Device,
        )])
        .unwrap();
        let in_shared = GuaranteedProperties::new([GuaranteedProperty::MemoryDomain(
            MemoryDomainClass::Shared,
        )])
        .unwrap();

        assert!(unsatisfied_properties(&admits_both, &in_device).is_empty());
        assert!(unsatisfied_properties(&admits_both, &in_shared).is_empty());
        assert!(unsatisfied_properties(&admits_device, &in_device).is_empty());
        assert_eq!(unsatisfied_properties(&admits_device, &in_shared).len(), 1);
    }

    #[test]
    fn an_empty_admitted_domain_set_is_rejected_at_construction() {
        assert!(AdmittedMemoryDomains::new([]).is_none());
        // Normalization is canonical and duplicate-free, so two spellings of one
        // admitted set are one value.
        let forward =
            AdmittedMemoryDomains::new([MemoryDomainClass::Shared, MemoryDomainClass::Device])
                .unwrap();
        let reverse = AdmittedMemoryDomains::new([
            MemoryDomainClass::Device,
            MemoryDomainClass::Shared,
            MemoryDomainClass::Device,
        ])
        .unwrap();
        assert_eq!(forward, reverse);
        assert_eq!(forward.classes().len(), 2);
    }

    #[test]
    fn a_producer_owing_a_coherence_action_does_not_satisfy_a_readable_requirement() {
        // ADR 0043's separation applied to visibility: a value that still owes a
        // flush is not delivered at a higher cost, it is not delivered.
        let owes_action = GuaranteedProperties::new([GuaranteedProperty::Visibility(
            VisibilityGuarantee::RequiresExplicitCoherenceAction,
        )])
        .unwrap();
        let needs_readable = RequiredProperties::new([RequiredProperty::Visibility(
            VisibilityRequirement::ReadableOnRequiringAffinity,
        )])
        .unwrap();
        let refused = unsatisfied_properties(&needs_readable, &owes_action);
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].reason(), UnsatisfiedReason::NotSatisfied);
    }

    #[test]
    fn a_host_readback_dependency_is_not_discharged_by_a_dispatch_dependency() {
        // ADR 0033 makes observed terminal completion a separate boundary from a
        // dispatch dependency, so no bounded-profile guarantee reaches it.
        let after_dispatch = GuaranteedProperties::new([GuaranteedProperty::Availability(
            AvailabilityGuarantee::AfterOwnDispatch,
        )])
        .unwrap();
        let needs_host = RequiredProperties::new([RequiredProperty::Availability(
            AvailabilityRequirement::AfterObservedHostCompletion,
        )])
        .unwrap();
        assert_eq!(
            unsatisfied_properties(&needs_host, &after_dispatch).len(),
            1
        );
    }

    #[test]
    fn a_materialized_buffer_does_not_serve_a_view_or_opaque_requirement() {
        let materialized = GuaranteedProperties::new([GuaranteedProperty::Materialization(
            MaterializationForm::MaterializedBuffer,
        )])
        .unwrap();
        for form in [
            MaterializationForm::AliasView,
            MaterializationForm::OpaqueRuntimeValue,
        ] {
            let required =
                RequiredProperties::new([RequiredProperty::Materialization(form)]).unwrap();
            assert_eq!(
                unsatisfied_properties(&required, &materialized).len(),
                1,
                "{} must reject explicitly rather than be approximated",
                form.key()
            );
        }
    }

    /// A second symbolic affinity, used only to exercise relations the bounded
    /// single-affinity profile cannot reach end to end.
    const SECOND_AFFINITY: ExecutionAffinity = ExecutionAffinity::new("tiler.affinity.secondary");

    #[test]
    fn a_foreign_affinity_is_never_substituted_by_name() {
        // ADR 0047 forbids inferring meaning from a domain or affinity name, so a
        // producer placed for another affinity is refused rather than treated as
        // an equivalent placement at a transfer's cost.
        let elsewhere =
            GuaranteedProperties::new([GuaranteedProperty::ExecutionAffinity(SECOND_AFFINITY)])
                .unwrap();
        let here = RequiredProperties::new([RequiredProperty::ExecutionAffinity(
            ExecutionAffinity::PRIMARY,
        )])
        .unwrap();
        let refused = unsatisfied_properties(&here, &elsewhere);
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].reason(), UnsatisfiedReason::NotSatisfied);

        let same = GuaranteedProperties::new([GuaranteedProperty::ExecutionAffinity(
            ExecutionAffinity::PRIMARY,
        )])
        .unwrap();
        assert!(unsatisfied_properties(&here, &same).is_empty());
    }

    #[test]
    fn every_unsatisfied_dimension_is_reported_not_only_the_first() {
        let required = bounded_requirements();
        let guaranteed = GuaranteedProperties::new([
            GuaranteedProperty::StorageLayout(LayoutGuarantee::DenseRowMajor),
            GuaranteedProperty::Materialization(MaterializationForm::MaterializedBuffer),
        ])
        .unwrap();
        let unsatisfied = unsatisfied_properties(&required, &guaranteed);
        assert_eq!(unsatisfied.len(), 6);
        // Reported in canonical dimension order so an explanation is stable.
        let reported: Vec<BoundaryProperty> = unsatisfied
            .iter()
            .map(UnsatisfiedProperty::property)
            .collect();
        let mut sorted = reported.clone();
        sorted.sort_unstable();
        assert_eq!(reported, sorted);
    }

    #[test]
    fn duplicate_dimensions_are_malformed_output() {
        let error = RequiredProperties::new([
            RequiredProperty::Alignment(AlignmentRequirement::natural_for(StorageScalar::F32)),
            RequiredProperty::Alignment(AlignmentRequirement::new(16).unwrap()),
        ])
        .unwrap_err();
        assert_eq!(
            error,
            BoundaryPropertyError::DuplicateProperty {
                property: BoundaryProperty::Alignment
            }
        );
        assert_eq!(error.reason(), "duplicate-property");

        let guarantee_error = GuaranteedProperties::new([
            GuaranteedProperty::MemoryDomain(MemoryDomainClass::Device),
            GuaranteedProperty::MemoryDomain(MemoryDomainClass::Shared),
        ])
        .unwrap_err();
        assert_eq!(guarantee_error.reason(), "duplicate-property");
    }

    #[test]
    fn shared_packed_encoding_rejects_whole_byte_and_cross_byte_widths() {
        for bits in [0, 3, 5, 6, 7, 8] {
            assert!(
                BitPackedEncoding::new(
                    bits,
                    PackedBitOrder::LeastSignificantElementFirst,
                    PackedTailRule::Zero,
                )
                .is_none()
            );
        }
    }

    #[test]
    fn requirement_dominance_is_a_partial_order_over_the_dimensions() {
        let four = RequiredProperties::new([RequiredProperty::Alignment(
            AlignmentRequirement::natural_for(StorageScalar::F32),
        )])
        .unwrap();
        let sixteen = RequiredProperties::new([RequiredProperty::Alignment(
            AlignmentRequirement::new(16).unwrap(),
        )])
        .unwrap();
        assert!(four.is_no_stronger_than(&sixteen));
        assert!(!sixteen.is_no_stronger_than(&four));

        // A requirement on a dimension the other set does not name is stronger,
        // because the other admits every producer on it.
        let plus_layout = RequiredProperties::new([
            RequiredProperty::Alignment(AlignmentRequirement::natural_for(StorageScalar::F32)),
            RequiredProperty::StorageLayout(LayoutRequirement::DenseRowMajor),
        ])
        .unwrap();
        assert!(!plus_layout.is_no_stronger_than(&four));
        assert!(four.is_no_stronger_than(&plus_layout));

        // Incomparable encodings dominate in neither direction.
        let unpacked =
            RequiredProperties::new([RequiredProperty::StorageEncoding(StorageEncoding::Unpacked)])
                .unwrap();
        let packed = RequiredProperties::new([RequiredProperty::StorageEncoding(
            StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
        )])
        .unwrap();
        assert!(!unpacked.is_no_stronger_than(&packed));
        assert!(!packed.is_no_stronger_than(&unpacked));
    }

    #[test]
    fn guarantee_dominance_ranks_strength_and_not_cost() {
        let coarse = GuaranteedProperties::new([GuaranteedProperty::Alignment(
            AlignmentGuarantee::new(16).unwrap(),
        )])
        .unwrap();
        let fine = GuaranteedProperties::new([GuaranteedProperty::Alignment(
            AlignmentGuarantee::natural_for(StorageScalar::F32),
        )])
        .unwrap();
        assert!(coarse.is_at_least_as_strong_as(&fine));
        assert!(!fine.is_at_least_as_strong_as(&coarse));

        // A guarantee the other set does not make is extra strength, not a gap.
        let plus_visibility = GuaranteedProperties::new([
            GuaranteedProperty::Alignment(AlignmentGuarantee::new(16).unwrap()),
            GuaranteedProperty::Visibility(VisibilityGuarantee::CoherentOnProducingAffinity),
        ])
        .unwrap();
        assert!(plus_visibility.is_at_least_as_strong_as(&coarse));
        assert!(!coarse.is_at_least_as_strong_as(&plus_visibility));

        // A visibility state still owing a coherence action is the weaker one.
        let owes = GuaranteedProperties::new([GuaranteedProperty::Visibility(
            VisibilityGuarantee::RequiresExplicitCoherenceAction,
        )])
        .unwrap();
        let coherent = GuaranteedProperties::new([GuaranteedProperty::Visibility(
            VisibilityGuarantee::CoherentOnProducingAffinity,
        )])
        .unwrap();
        assert!(coherent.is_at_least_as_strong_as(&owes));
        assert!(!owes.is_at_least_as_strong_as(&coherent));
    }

    #[test]
    fn only_the_execution_affinity_propagates_to_a_child_requirement() {
        // A goal that constrains the region's output layout, encoding, alignment,
        // domain, availability, and visibility must not manufacture those
        // requirements on the region's input; only the affinity carries through,
        // because ADR 0047's initial profile places every stage on one affinity.
        let goal = bounded_requirements();
        let implementation = RequiredProperties::new([
            RequiredProperty::StorageLayout(LayoutRequirement::UnitStrideOnAxis {
                axis: Axis::new(1),
                rank: 2,
            }),
            RequiredProperty::Materialization(MaterializationForm::MaterializedBuffer),
        ])
        .unwrap();
        let derived = derive_child_requirements(&goal, &implementation).unwrap();

        let dimensions: Vec<BoundaryProperty> = derived
            .properties()
            .iter()
            .map(RequiredProperty::property)
            .collect();
        assert_eq!(
            dimensions,
            vec![
                BoundaryProperty::StorageLayout,
                BoundaryProperty::Materialization,
                BoundaryProperty::ExecutionAffinity,
            ],
            "only the affinity is inherited; the rest stay the implementation's own"
        );
        assert_eq!(
            derived.get(BoundaryProperty::StorageLayout),
            implementation.get(BoundaryProperty::StorageLayout),
            "the goal's dense-layout requirement must not overwrite the \
             implementation's unit-stride need"
        );
    }

    #[test]
    fn a_propagating_dimension_the_two_sides_disagree_on_fails_closed() {
        // The bounded profile has one affinity, so a disagreement cannot arise
        // end to end; the relation is exercised directly against a second
        // symbolic affinity. That is a measurement boundary on this test, not a
        // gap in the relation.
        let goal = RequiredProperties::new([RequiredProperty::ExecutionAffinity(
            ExecutionAffinity::PRIMARY,
        )])
        .unwrap();

        // A region silent on the affinity inherits the goal's.
        let silent = RequiredProperties::new([RequiredProperty::MemoryDomain(
            AdmittedMemoryDomains::new([MemoryDomainClass::Device]).unwrap(),
        )])
        .unwrap();
        let derived = derive_child_requirements(&goal, &silent).unwrap();
        assert_eq!(
            derived.get(BoundaryProperty::ExecutionAffinity),
            goal.get(BoundaryProperty::ExecutionAffinity)
        );

        // A region that reads from a different affinity cannot serve the goal.
        let conflicting =
            RequiredProperties::new([RequiredProperty::ExecutionAffinity(SECOND_AFFINITY)])
                .unwrap();
        let conflict = derive_child_requirements(&goal, &conflicting).unwrap_err();
        assert_eq!(conflict.property(), BoundaryProperty::ExecutionAffinity);
        assert_eq!(
            ChildRequirementConflict::REASON,
            "child-requirement-conflict"
        );
        assert_eq!(
            conflict.goal(),
            goal.get(BoundaryProperty::ExecutionAffinity).unwrap()
        );
        assert_eq!(
            conflict.implementation(),
            conflicting
                .get(BoundaryProperty::ExecutionAffinity)
                .unwrap()
        );
    }

    #[test]
    fn identity_is_independent_of_assembly_order_and_separates_distinct_contracts() {
        let forward = encode_property_identity(&bounded_requirements(), &bounded_guarantees());
        let reversed_required =
            RequiredProperties::new(bounded_requirements().properties().iter().rev().cloned())
                .unwrap();
        let reversed_guaranteed =
            GuaranteedProperties::new(bounded_guarantees().properties().iter().rev().cloned())
                .unwrap();
        let reverse = encode_property_identity(&reversed_required, &reversed_guaranteed);
        assert_eq!(forward, reverse);

        // A single differing dimension is a different identity.
        let coarser =
            GuaranteedProperties::new(bounded_guarantees().properties().iter().map(|property| {
                match property {
                    GuaranteedProperty::Alignment(_) => {
                        GuaranteedProperty::Alignment(AlignmentGuarantee::new(16).unwrap())
                    }
                    other => other.clone(),
                }
            }))
            .unwrap();
        assert_ne!(
            forward,
            encode_property_identity(&bounded_requirements(), &coarser)
        );

        // The domain tag comes first, so no property encoding can be read as
        // another subject's bytes.
        assert!(
            forward
                .as_bytes()
                .starts_with(b"tiler.compiler.boundary-property-set.v3\0")
        );
    }

    #[test]
    fn requirement_and_guarantee_sets_are_not_interchangeable_across_dimensions() {
        // A guarantee on one dimension never discharges a requirement on
        // another, however similar the values look.
        let guaranteed = GuaranteedProperties::new([GuaranteedProperty::StorageEncoding(
            StorageEncoding::Unpacked,
        )])
        .unwrap();
        let required = RequiredProperties::new([RequiredProperty::StorageLayout(
            LayoutRequirement::DenseRowMajor,
        )])
        .unwrap();
        let refused = unsatisfied_properties(&required, &guaranteed);
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].reason(), UnsatisfiedReason::NotGuaranteed);
    }

    #[test]
    fn every_dimension_has_a_distinct_key_and_tag_and_the_canonical_order_is_total() {
        let mut keys: Vec<&'static str> = CANONICAL_PROPERTIES
            .iter()
            .copied()
            .map(BoundaryProperty::key)
            .collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), CANONICAL_PROPERTIES.len());

        let mut tags: Vec<u8> = CANONICAL_PROPERTIES
            .iter()
            .copied()
            .map(BoundaryProperty::tag)
            .collect();
        tags.sort_unstable();
        tags.dedup();
        assert_eq!(tags.len(), CANONICAL_PROPERTIES.len());

        // The canonical order agrees with the derived ordering, so the encoding
        // order and the reporting order cannot drift apart.
        let mut sorted = CANONICAL_PROPERTIES;
        sorted.sort_unstable();
        assert_eq!(sorted, CANONICAL_PROPERTIES);
    }
}
