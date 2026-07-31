//! An independent adapter, written outside the crate that defines the seam.
//!
//! This file can see nothing but `tiler`'s public surface, which is what makes
//! it evidence rather than an in-crate exercise: if [`TensorAdapter`] needed
//! anything private, or if supplying one needed a registration step or a change
//! inside `tiler`, none of this would compile. Two adapters are defined below —
//! one complete, one declining a capability — because "an arbitrary consumer can
//! supply the adapter" is a claim about more than one implementation.
//!
//! The compile-pass fixtures under `tests/facade/pass/` carry the same evidence
//! one step further out, as separate crates. What lives here instead is the
//! failure half: every check `bind_region` and `build_result` perform, each with
//! an accepting neighbour differing in exactly the input under test.

use tiler::__private::{
    AxisRef, OperandFacts, RegionFacts, ResultAxis, ResultFacts, SymbolFacts, bind_region,
    build_result,
};
use tiler::value::{
    AdapterCapability, BindError, OperandAxis, ResultRequest, StorageScalar, Tensor, TensorAdapter,
    ValueMetadata,
};

/// The integration's own value. Nothing in `tiler` can look inside it.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Held {
    scalar: StorageScalar,
    extents: Vec<u64>,
    /// Set to make [`Complete::metadata`] fail, so the adapter's own error can
    /// be observed travelling through `tiler` unchanged.
    unreadable: bool,
}

impl Held {
    fn f32(extents: impl IntoIterator<Item = u64>) -> Self {
        Self {
            scalar: StorageScalar::F32,
            extents: extents.into_iter().collect(),
            unreadable: false,
        }
    }
}

/// The integration's own error.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Refused(&'static str);

impl core::fmt::Display for Refused {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for Refused {}

/// An adapter offering every capability the profile requires.
struct Complete;

impl TensorAdapter for Complete {
    type Value = Held;
    type Context = &'static str;
    type Error = Refused;

    fn supports(capability: AdapterCapability) -> bool {
        match capability {
            AdapterCapability::DenseRowMajorStorage | AdapterCapability::ResultConstruction => true,
        }
    }

    fn metadata(value: &Held) -> Result<ValueMetadata, Refused> {
        if value.unreadable {
            return Err(Refused("this value's metadata is not readable"));
        }
        Ok(ValueMetadata::new(
            value.scalar,
            value.extents.iter().copied(),
        ))
    }

    fn build(context: &&'static str, request: &ResultRequest<'_>) -> Result<Held, Refused> {
        assert_eq!(
            *context, "device",
            "the caller's context reaches the adapter"
        );
        Ok(Held {
            scalar: request.storage_scalar(),
            extents: request.extents().to_vec(),
            unreadable: false,
        })
    }
}

/// An adapter over a borrowed tensor type that cannot construct new values.
///
/// A real integration in this position exists: an adapter over a read-only view
/// has nothing to allocate into. The point of the capability is that it says so
/// instead of failing later.
struct ReadOnly;

impl TensorAdapter for ReadOnly {
    type Value = Held;
    type Context = ();
    type Error = Refused;

    fn supports(capability: AdapterCapability) -> bool {
        match capability {
            AdapterCapability::DenseRowMajorStorage => true,
            AdapterCapability::ResultConstruction => false,
        }
    }

    fn metadata(value: &Held) -> Result<ValueMetadata, Refused> {
        Ok(ValueMetadata::new(
            value.scalar,
            value.extents.iter().copied(),
        ))
    }

    fn build((): &(), _: &ResultRequest<'_>) -> Result<Held, Refused> {
        unreachable!("`ResultConstruction` is declined, so nothing may call this")
    }
}

/// `sym n; in a: f32[n], b: f32[n, 4]; out d: f32[n]`.
///
/// Two operands of different rank, so a rank check cannot pass by accident, and
/// one symbol occurring on both, so the equality obligation is exercised.
const REGION: RegionFacts = RegionFacts {
    operands: &[
        OperandFacts {
            key: "a",
            storage_scalar: StorageScalar::F32,
            rank: 1,
        },
        OperandFacts {
            key: "b",
            storage_scalar: StorageScalar::F32,
            rank: 2,
        },
    ],
    symbols: &[SymbolFacts {
        name: "n",
        source: AxisRef {
            operand: 0,
            axis: 0,
        },
        obligations: &[AxisRef {
            operand: 1,
            axis: 0,
        }],
    }],
    capabilities: &[
        AdapterCapability::DenseRowMajorStorage,
        AdapterCapability::ResultConstruction,
    ],
    result: ResultFacts {
        key: "d",
        storage_scalar: StorageScalar::F32,
        axes: &[ResultAxis::Symbol(0)],
    },
};

fn wrap(value: Held) -> Tensor<Complete> {
    Tensor::new(value, "device")
}

/// The accepting case every refusal below is a neighbour of.
#[test]
fn a_matching_invocation_binds_its_symbol_and_returns_the_declared_result() {
    let a = wrap(Held::f32([7]));
    let b = wrap(Held::f32([7, 4]));

    let bound = bind_region::<Complete>(&REGION, &[&a, &b]).expect("both operands agree");
    assert_eq!(bound.values(), [7]);
    assert_eq!(bound.get(0), Some(7));
    assert_eq!(bound.get(1), None);

    let result = build_result::<Complete>(&REGION, &bound, a.context())
        .expect("the adapter constructs results");
    assert_eq!(result, Held::f32([7]));
}

/// Two axes naming one symbol must agree, and the refusal names both.
#[test]
fn a_repeated_extent_that_disagrees_is_refused_naming_both_axes() {
    let a = wrap(Held::f32([7]));
    let b = wrap(Held::f32([8, 4]));

    assert_eq!(
        bind_region::<Complete>(&REGION, &[&a, &b]).expect_err("`b` axis 0 owes `a` axis 0"),
        BindError::InconsistentExtent {
            symbol: "n",
            source: OperandAxis {
                input: "a",
                axis: 0
            },
            source_extent: 7,
            conflicting: OperandAxis {
                input: "b",
                axis: 0
            },
            conflicting_extent: 8,
        },
    );

    // The message names the operands a consumer wrote, not their positions.
    let rendered = bind_region::<Complete>(&REGION, &[&a, &b])
        .expect_err("still refused")
        .to_string();
    assert!(rendered.contains("`a` axis 0"), "{rendered}");
    assert!(rendered.contains("`b` axis 0"), "{rendered}");
}

/// A rank the region did not declare is refused, naming both ranks.
#[test]
fn a_rank_the_region_did_not_declare_is_refused() {
    let a = wrap(Held::f32([7, 7]));
    let b = wrap(Held::f32([7, 4]));

    assert_eq!(
        bind_region::<Complete>(&REGION, &[&a, &b]).expect_err("`a` is declared rank 1"),
        BindError::RankMismatch {
            input: "a",
            declared: 1,
            actual: 2,
        },
    );
}

/// A stored scalar the region did not declare is refused.
#[test]
fn a_stored_scalar_the_region_did_not_declare_is_refused() {
    let a = wrap(Held {
        scalar: StorageScalar::U8,
        extents: vec![7],
        unreadable: false,
    });
    let b = wrap(Held::f32([7, 4]));

    assert_eq!(
        bind_region::<Complete>(&REGION, &[&a, &b]).expect_err("`a` is declared f32"),
        BindError::StorageScalarMismatch {
            input: "a",
            declared: StorageScalar::F32,
            actual: StorageScalar::U8,
        },
    );
}

/// An adapter that declines a required capability refuses the region outright.
///
/// Before any shape is read: a region an adapter cannot serve at all should not
/// report a shape mismatch it would also have had.
#[test]
fn an_adapter_declining_a_required_capability_is_refused_first() {
    let a = Tensor::<ReadOnly>::new(Held::f32([7]), ());
    // Deliberately the wrong rank as well, so the assertion below is about
    // ordering rather than about there being only one thing wrong.
    let b = Tensor::<ReadOnly>::new(Held::f32([7]), ());

    assert_eq!(
        bind_region::<ReadOnly>(&REGION, &[&a, &b])
            .expect_err("this adapter cannot construct results"),
        BindError::UnsupportedCapability {
            capability: AdapterCapability::ResultConstruction,
        },
    );

    // `build_result` checks it again rather than trusting that `bind_region`
    // ran: the two are separate entry points and generated code calls both.
    let bound =
        bind_region::<Complete>(&REGION, &[&wrap(Held::f32([7])), &wrap(Held::f32([7, 4]))])
            .expect("the complete adapter binds");
    assert_eq!(
        build_result::<ReadOnly>(&REGION, &bound, &()).expect_err("still declined"),
        BindError::UnsupportedCapability {
            capability: AdapterCapability::ResultConstruction,
        },
    );
}

/// Supplying the wrong number of operands is refused before any is read.
#[test]
fn an_operand_count_the_region_did_not_declare_is_refused() {
    let a = wrap(Held::f32([7]));
    assert_eq!(
        bind_region::<Complete>(&REGION, &[&a]).expect_err("the region declares two operands"),
        BindError::OperandCountMismatch {
            declared: 2,
            supplied: 1,
        },
    );
}

/// The adapter's own error travels through unchanged.
#[test]
fn the_adapters_own_error_is_carried_rather_than_replaced() {
    let a = wrap(Held {
        scalar: StorageScalar::F32,
        extents: vec![7],
        unreadable: true,
    });
    let b = wrap(Held::f32([7, 4]));

    let error = bind_region::<Complete>(&REGION, &[&a, &b]).expect_err("the adapter refused");
    assert_eq!(
        error,
        BindError::Adapter(Refused("this value's metadata is not readable")),
    );
    // The concrete type survives to `source`, per ADR 0074 convention 1.
    let source = std::error::Error::source(&error).expect("the adapter's error is the source");
    assert!(source.downcast_ref::<Refused>().is_some());
}

/// Facts that disagree with themselves fail closed instead of panicking.
///
/// A defect in an expansion, not in an invocation — but a panic inside generated
/// code aborts the consumer's process, so the refusal is typed and returned.
#[test]
fn facts_that_disagree_with_themselves_are_refused_rather_than_indexed_past() {
    /// A symbol sourced from an axis its own declared rank does not have.
    const OUT_OF_RANGE_SOURCE: RegionFacts = RegionFacts {
        operands: &[OperandFacts {
            key: "a",
            storage_scalar: StorageScalar::F32,
            rank: 1,
        }],
        symbols: &[SymbolFacts {
            name: "n",
            source: AxisRef {
                operand: 0,
                axis: 3,
            },
            obligations: &[],
        }],
        capabilities: &[],
        result: ResultFacts {
            key: "d",
            storage_scalar: StorageScalar::F32,
            axes: &[ResultAxis::Symbol(0)],
        },
    };
    /// A result axis naming a symbol index the region does not declare.
    const OUT_OF_RANGE_RESULT: RegionFacts = RegionFacts {
        operands: &[OperandFacts {
            key: "a",
            storage_scalar: StorageScalar::F32,
            rank: 1,
        }],
        symbols: &[],
        capabilities: &[],
        result: ResultFacts {
            key: "d",
            storage_scalar: StorageScalar::F32,
            axes: &[ResultAxis::Symbol(0)],
        },
    };

    let a = wrap(Held::f32([7]));
    assert!(matches!(
        bind_region::<Complete>(&OUT_OF_RANGE_SOURCE, &[&a])
            .expect_err("axis 3 of a rank-1 operand does not exist"),
        BindError::MalformedRegionFacts { .. }
    ));

    let bound =
        bind_region::<Complete>(&OUT_OF_RANGE_RESULT, &[&a]).expect("the operand itself matches");
    assert!(matches!(
        build_result::<Complete>(&OUT_OF_RANGE_RESULT, &bound, a.context())
            .expect_err("the result names a symbol the region does not declare"),
        BindError::MalformedRegionFacts { .. }
    ));
}

/// A wrapper hands the integration its own value back.
///
/// The conversion out of the wrapper is the integration's, and it is the
/// identity: `build_result` returns `A::Value`, so a region's result is the
/// consumer's own tensor type rather than something they must unwrap.
#[test]
fn the_wrapper_returns_the_integrations_own_value() {
    let held = Held::f32([2, 3]);
    let wrapped = wrap(held.clone());
    assert_eq!(wrapped.value(), &held);
    assert_eq!(*wrapped.context(), "device");
    let (value, context) = wrapped.into_parts();
    assert_eq!(value, held);
    assert_eq!(context, "device");
    assert_eq!(wrap(held.clone()).into_value(), held);
}

/// Metadata reports what the adapter was given, unchanged.
#[test]
fn reported_metadata_is_what_the_adapter_stated() {
    let metadata = ValueMetadata::new(StorageScalar::U8, [4, 0, 9]);
    assert_eq!(metadata.storage_scalar(), StorageScalar::U8);
    assert_eq!(metadata.rank(), 3);
    assert_eq!(metadata.extents(), [4, 0, 9]);
    // A zero extent is an empty axis, not an error: the shape vocabulary admits
    // it and a binding check has nothing to say about it.
    assert_eq!(ValueMetadata::new(StorageScalar::F32, []).rank(), 0);
}

/// Every capability has a stable diagnostic name.
#[test]
fn every_capability_names_itself() {
    for capability in [
        AdapterCapability::DenseRowMajorStorage,
        AdapterCapability::ResultConstruction,
    ] {
        assert!(!capability.as_str().is_empty());
        assert_eq!(capability.to_string(), capability.as_str());
    }
    assert_ne!(
        AdapterCapability::DenseRowMajorStorage.as_str(),
        AdapterCapability::ResultConstruction.as_str(),
    );
}
