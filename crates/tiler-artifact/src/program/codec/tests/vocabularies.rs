//! Governed tag tables and digest domains.

use super::super::super::expr::AvailabilityPhase;
use super::super::super::model::{
    BINDING_TARGET_INTERNAL, BINDING_TARGET_PROGRAM_INPUT, BINDING_TARGET_PROGRAM_OUTPUT,
    BindingTargetData, RoutingPolicy, address_space_from_tag, address_space_tag,
    buffer_access_from_tag, buffer_access_tag, element_type_from_tag, element_type_tag,
    exceptional_assumption_from_tag, exceptional_assumption_tag, permission_from_tag,
    permission_tag, storage_scalar_from_tag, storage_scalar_tag, subnormal_from_tag, subnormal_tag,
};
use super::super::model::{SectionDisposition, SectionKind};
use std::mem::variant_count;
use tiler_ir::schedule::{
    ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission, SubnormalMode,
    ValueDomainProvenance,
};
use tiler_ir::semantic::{InputKey, OutputKey};

#[test]
fn section_tag_tables_are_injective_and_inverse_complete() {
    const KINDS: [SectionKind; variant_count::<SectionKind>()] = [
        SectionKind::KernelProgramSubject,
        SectionKind::BackendPayloadMetadata,
        SectionKind::BackendPayloadCode,
    ];
    const DISPOSITIONS: [SectionDisposition; variant_count::<SectionDisposition>()] =
        [SectionDisposition::Required, SectionDisposition::Optional];

    super::super::super::tag_injectivity::assert_tag_table_with_inverse(
        "SectionKind",
        &KINDS,
        SectionKind::tag,
        SectionKind::from_tag,
    );

    super::super::super::tag_injectivity::assert_tag_table_with_inverse(
        "SectionDisposition",
        &DISPOSITIONS,
        SectionDisposition::tag,
        SectionDisposition::from_tag,
    );
}

/// Defines one payload-carrying enum's population from its exhaustive outer shape.
macro_rules! exhaustive_enum_population {
    ($name:ident: $ty:ty { $($pattern:pat => $contribution:expr),+ $(,)? }) => {
        const $name: usize = {
            const fn contribution(value: $ty) -> usize {
                match value {
                    $($pattern => $contribution),+
                }
            }

            let _ = contribution;
            0 $(+ $contribution)+
        };
    };
}

exhaustive_enum_population!(SUBNORMAL_MODE_POPULATION: SubnormalMode {
    SubnormalMode::Preserve => 1,
    SubnormalMode::FlushToZero { zero_sign: _ } => variant_count::<FlushedZeroSign>(),
});

/// Every subnormal treatment encoded by the governed tag table.
const SUBNORMAL_MODES: [SubnormalMode; SUBNORMAL_MODE_POPULATION] = [
    SubnormalMode::Preserve,
    SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    },
    SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::AlwaysPositive,
    },
];

exhaustive_enum_population!(EXCEPTIONAL_ASSUMPTION_POPULATION: ExceptionalValueAssumption {
    ExceptionalValueAssumption::MakeNoAssumption => 1,
    ExceptionalValueAssumption::AssumeAbsent { provenance: _ } =>
        variant_count::<ValueDomainProvenance>(),
});

/// Every exceptional-value assumption encoded by the governed tag table.
const EXCEPTIONAL_ASSUMPTIONS: [ExceptionalValueAssumption; EXCEPTIONAL_ASSUMPTION_POPULATION] = [
    ExceptionalValueAssumption::MakeNoAssumption,
    ExceptionalValueAssumption::AssumeAbsent {
        provenance: ValueDomainProvenance::CompilerProven,
    },
    ExceptionalValueAssumption::AssumeAbsent {
        provenance: ValueDomainProvenance::RuntimeValidated,
    },
    ExceptionalValueAssumption::AssumeAbsent {
        provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
    },
];

/// The separator is a prefix, so separation rests on the domains themselves.
///
/// `digest(domain, body)` hashes `domain || body`, which distinguishes two
/// subjects only when no admitted domain is a prefix of another — otherwise a
/// longer domain and a shorter one with leading body bytes would collide. Every
/// governed domain is a fixed constant of a crate that admits it, so the
/// property is checkable rather than assumed, and a new domain that violates it
/// fails a test instead of silently merging two subjects.
///
/// **This test covers the envelope's seven domains and not every domain the
/// workspace admits.** The property is global: one algorithm hashes the
/// envelope, the proof sidecar, the artifact program's identity encoding, and
/// the shared IR's layered identities in one process, so a domain added to any of
/// them could collide with one in another, and a check confined to the envelope
/// would report separation it had not established.
/// `crate::domains::no_governed_domain_of_this_crate_prefixes_another` checks the
/// union of this crate's twenty and is the authority for the property; this
/// test is the envelope-local half. Across crates, the accepted artifact ABI
/// contract establishes the no-prefix property by a spelling-and-terminator
/// argument over the observed IR population. `tiler-artifact` depends on
/// `tiler-ir`, so this crate could check the union if that population were
/// exported; the obstacle is that `tiler-ir` keeps its complete pin population
/// private and test-only. `tiler-digest` deliberately owns no subject domains.
/// `crate::domains` checks this crate's half rather than only stating it.
///
/// **The population is derived rather than listed.** It is
/// `GovernedDomain::of(DomainContainer::Envelope)`, so an envelope domain added
/// to the crate's enumeration appears here without this test being edited, and
/// one added to the crate and *not* enumerated is caught by that module's source
/// census. The previous shape — a hand-written array beside a hand-written count
/// — is exactly what let the manifest framing tag and both payload domains be
/// admitted with nothing failing.
///
/// It lives beside the codec rather than with the algorithm because a domain
/// belongs to the authority that decides what it names: `tiler-digest` owns the
/// algorithm and deliberately knows none of the domains it is called with.
#[test]
fn no_governed_domain_is_a_prefix_of_another() {
    let domains = crate::domains::GovernedDomain::of(crate::domains::DomainContainer::Envelope);
    assert_eq!(
        domains.len(),
        crate::domains::DomainContainer::ENVELOPE,
        "the envelope's derived domain population is {:?}",
        domains
            .iter()
            .map(|domain| String::from_utf8_lossy(domain.bytes()).into_owned())
            .collect::<Vec<_>>(),
    );
    for (index, left) in domains.iter().enumerate() {
        for right in domains.iter().skip(index + 1) {
            assert!(
                !left.bytes().starts_with(right.bytes())
                    && !right.bytes().starts_with(left.bytes()),
                "one governed digest domain prefixes another: {left:?} against {right:?}",
            );
        }
    }
}

// -------------------------------------------------------------------------
// One tag table per vocabulary
// -------------------------------------------------------------------------

#[test]
fn every_governed_tag_table_round_trips() {
    use tiler_ir::kernel::{AddressSpace, BufferAccess, KernelType};
    use tiler_ir::program::{StorageScalar, ValueRole};

    const ELEMENT_TYPES: [KernelType; variant_count::<KernelType>()] = [
        KernelType::Bool,
        KernelType::Index,
        KernelType::F32,
        KernelType::U8,
        KernelType::I32,
        KernelType::Bf16,
        KernelType::U32,
    ];
    const STORAGE_SCALARS: [StorageScalar; variant_count::<StorageScalar>()] = [
        StorageScalar::U8,
        StorageScalar::F32,
        StorageScalar::Bf16,
        StorageScalar::U32,
    ];
    for value in ELEMENT_TYPES {
        // The wildcard-free `match` is what keeps the array above honest. A bare
        // list would keep passing while covering one fewer variant than the
        // vocabulary has, which is the opposite of the exhaustive round trip
        // `docs/artifact-abi.md` claims pins each of these tables.
        match value {
            KernelType::Bool
            | KernelType::Index
            | KernelType::F32
            | KernelType::U8
            | KernelType::I32
            | KernelType::Bf16
            | KernelType::U32 => {}
        }
        assert_eq!(element_type_from_tag(element_type_tag(value)), Some(value),);
    }
    assert_eq!(element_type_tag(KernelType::Bool), 0x01);
    assert_eq!(element_type_tag(KernelType::Index), 0x02);
    assert_eq!(element_type_tag(KernelType::F32), 0x03);
    assert_eq!(element_type_tag(KernelType::U8), 0x04);
    assert_eq!(element_type_tag(KernelType::I32), 0x05);
    assert_eq!(element_type_tag(KernelType::Bf16), 0x06);
    assert_eq!(element_type_tag(KernelType::U32), 0x07);
    for value in STORAGE_SCALARS {
        match value {
            StorageScalar::U8 | StorageScalar::F32 | StorageScalar::Bf16 | StorageScalar::U32 => {}
        }
        assert_eq!(
            storage_scalar_from_tag(storage_scalar_tag(value)),
            Some(value),
        );
    }
    assert_eq!(storage_scalar_tag(StorageScalar::U8), 0x01);
    assert_eq!(storage_scalar_tag(StorageScalar::F32), 0x02);
    assert_eq!(storage_scalar_tag(StorageScalar::Bf16), 0x03);
    assert_eq!(storage_scalar_tag(StorageScalar::U32), 0x04);
    for value in [
        AddressSpace::Device,
        AddressSpace::Workgroup,
        AddressSpace::InvocationPrivate,
        AddressSpace::Constant,
    ] {
        assert_eq!(
            address_space_from_tag(address_space_tag(value)),
            Some(value),
        );
    }
    for value in [BufferAccess::Read, BufferAccess::Write] {
        assert_eq!(
            buffer_access_from_tag(buffer_access_tag(value)),
            Some(value),
        );
    }
    // A binding target carries data, so its inverse is the decoder rather than a
    // tag table. What a tag table would have pinned is pinned directly: the three
    // governed tags are pairwise distinct, and the program role each target
    // implies is distinct too — a collision in either would let a decoder read one
    // dispatch instruction as another.
    let tags = [
        BINDING_TARGET_PROGRAM_INPUT,
        BINDING_TARGET_PROGRAM_OUTPUT,
        BINDING_TARGET_INTERNAL,
    ];
    let mut sorted = tags;
    sorted.sort_unstable();
    assert!(
        sorted.windows(2).all(|pair| pair[0] != pair[1]),
        "the governed binding-target tags must be pairwise distinct",
    );
    for (target, role) in [
        (
            BindingTargetData::ProgramInput(InputKey::new("input").unwrap()),
            ValueRole::Input,
        ),
        (
            BindingTargetData::ProgramOutput(vec![OutputKey::new("result").unwrap()]),
            ValueRole::Output,
        ),
        (BindingTargetData::Internal, ValueRole::Temporary),
    ] {
        assert_eq!(target.value_role(), role);
    }
    // Both flush behaviours are enumerated: they name different zeros, so a
    // shared tag would decode one as the other.
    for value in SUBNORMAL_MODES {
        assert_eq!(subnormal_from_tag(subnormal_tag(value)), Some(value));
    }
    for value in [
        NumericalPermission::Forbidden,
        NumericalPermission::Permitted,
    ] {
        assert_eq!(permission_from_tag(permission_tag(value)), Some(value));
    }
    for value in EXCEPTIONAL_ASSUMPTIONS {
        assert_eq!(
            exceptional_assumption_from_tag(exceptional_assumption_tag(value)),
            Some(value),
        );
    }
    assert_eq!(
        RoutingPolicy::from_tag(RoutingPolicy::StablePriority.tag()),
        Some(RoutingPolicy::StablePriority),
    );
    assert_eq!(
        SectionKind::from_tag(SectionKind::KernelProgramSubject.tag()),
        Some(SectionKind::KernelProgramSubject),
    );
    for phase in [
        AvailabilityPhase::CompileProfile,
        AvailabilityPhase::ArtifactEvidence,
        AvailabilityPhase::LiveDevicePreflight,
        AvailabilityPhase::PreparedKernelPreflight,
        AvailabilityPhase::LaunchPreflight,
    ] {
        assert_eq!(AvailabilityPhase::from_tag(phase.tag()), Some(phase));
    }
}
