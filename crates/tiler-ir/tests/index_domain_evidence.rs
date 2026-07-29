//! Public index-domain evidence custody tests.

use tiler_ir::index::{
    DomainRole, FrozenScalarRegistry, IndexDomainEvidence, IndexDomainPredicate,
    IndexDomainSoundProof, IndexExtentRef, IndexRegionBuilder, TensorRole,
    VerifiedIndexHandleError,
};
use tiler_ir::semantic::F32;
use tiler_ir::shape::{Extent, Shape};

fn verified_copy() -> tiler_ir::index::VerifiedIndexRegion {
    let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap()).unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(5))
        .unwrap();
    let coordinate = builder.dimension_expr(dimension).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::new([Extent::new(5)]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::new([Extent::new(5)]),
        )
        .unwrap();
    let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
    let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
    builder.output(write, value).unwrap();
    builder.build().unwrap()
}

#[test]
fn downstream_can_inspect_each_exact_discharged_predicate() {
    let region = verified_copy();
    let records = region
        .discharged_index_domain_predicates()
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 4);

    for access in region.accesses() {
        let expression = access.coordinates().next().unwrap();
        for predicate in [
            IndexDomainPredicate::NonNegative { expression },
            IndexDomainPredicate::LessThanExtent {
                expression,
                extent: IndexExtentRef::TensorAxis {
                    tensor: access.tensor(),
                    axis: 0,
                },
            },
        ] {
            let record = region
                .index_domain_evidence(access.id(), predicate)
                .unwrap()
                .expect("the verified copy discharges both coordinate bounds");
            assert_eq!(record.subject(), access.id());
            assert_eq!(record.predicate(), predicate);
            assert_eq!(
                record.evidence(),
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval)
            );
        }
    }
}

#[test]
fn lookup_refuses_foreign_subjects_and_predicates() {
    let region = verified_copy();
    let foreign = verified_copy();
    let local_access = region.accesses().next().unwrap();
    let subject = local_access.id();
    let local_expression = local_access.coordinates().next().unwrap();
    let foreign_access = foreign.accesses().next().unwrap();
    let foreign_expression = foreign_access.coordinates().next().unwrap();

    assert!(matches!(
        region.index_domain_evidence(
            foreign_access.id(),
            IndexDomainPredicate::NonNegative {
                expression: local_expression,
            },
        ),
        Err(VerifiedIndexHandleError::ForeignRegion { .. })
    ));
    assert!(matches!(
        region.index_domain_evidence(
            subject,
            IndexDomainPredicate::NonNegative {
                expression: foreign_expression,
            },
        ),
        Err(VerifiedIndexHandleError::ForeignRegion { .. })
    ));
    assert!(matches!(
        region.index_domain_evidence(
            subject,
            IndexDomainPredicate::LessThanExtent {
                expression: local_expression,
                extent: IndexExtentRef::TensorAxis {
                    tensor: foreign_access.tensor(),
                    axis: 0,
                },
            },
        ),
        Err(VerifiedIndexHandleError::ForeignRegion { .. })
    ));
}
