//! Parametric broadcast access: one sourced relation over its whole domain.
//!
//! [`LogicalAccess::ParametricBroadcast`] is an **accepted public surface**.
//! Tom accepted this exact spelling on 2026-08-13 under
//! [`accept-the-parametric-broadcast-access-surface`]. It is deliberately not
//! [`LogicalAccess::BroadcastReplication`] and not [`LogicalAccess::ReindexBijection`]:
//! those remain exact over their concrete subjects. Consumers match this carrier
//! explicitly.
//!
//! [`accept-the-parametric-broadcast-access-surface`]: ../../../../../tickets/accept-the-parametric-broadcast-access-surface.md
//!
//! The carrier names the authored mapping and the environment identity needed
//! to interpret it. It does not bind an extent, select a concrete neighbour, or
//! introduce a runtime fallback. Replication-only fusion and costing are
//! admitted only when that environment proves every model actually widens.

use std::fmt;
use std::sync::Arc;

use crate::semantic::{BroadcastAxisMapping, BroadcastMappingError};
use crate::shape::{ExtentSources, ShapeEnv, ShapeEnvIdentity, SourcedExtent, SourcedShape};

use super::model::LogicalAccess;

/// One named refusal of a parametric broadcast access relation.
///
/// The four substitution classes the carrier's acceptance names are distinct
/// variants so a forged zero-capable mapping, a foreign environment, an
/// unproved equality, and a concrete-variant stand-in cannot share a rule.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ParametricBroadcastRule {
    /// A many-to-one extent is not proved at least one, including any interval
    /// whose lower bound is zero.
    ZeroCapable {
        /// The result axis whose many-to-one extent is not proved positive.
        result_axis: usize,
        /// The extent the mapping declared there.
        declared: SourcedExtent,
    },
    /// The access names an environment that is not the one offered to interpret it,
    /// or a symbol that environment does not declare.
    ForeignEnvironment,
    /// A `from-operand` pair is not proved to be one extent.
    ExtentsNotProvedEqual {
        /// The result axis whose correspondence failed.
        result_axis: usize,
        /// The extent the mapping declares.
        declared: SourcedExtent,
        /// The operand extent behind it.
        operand: SourcedExtent,
    },
    /// A concrete [`LogicalAccess::BroadcastReplication`] or
    /// [`LogicalAccess::ReindexBijection`] was offered where the parametric
    /// carrier is required, or a wholly literal mapping was stored on it.
    ConcreteVariant,
    /// A stretch names an operand extent the environment does not prove is one.
    StretchSourceNotProvedUnit {
        /// The named operand axis.
        operand_axis: crate::shape::Axis,
        /// The operand extent the mapping asked about.
        extent: SourcedExtent,
    },
    /// The mapping does not apply against this operand and environment for a
    /// reason other than the four substitution classes.
    Mapping(BroadcastMappingError),
}

impl ParametricBroadcastRule {
    /// Returns the stable rule identifier for this refusal.
    #[must_use]
    pub const fn rule(&self) -> &'static str {
        match self {
            Self::ZeroCapable { .. } => "parametric-broadcast.zero-capable",
            Self::ForeignEnvironment => "parametric-broadcast.foreign-environment",
            Self::ExtentsNotProvedEqual { .. } => "parametric-broadcast.extents-not-proved-equal",
            Self::ConcreteVariant => "parametric-broadcast.concrete-variant",
            Self::StretchSourceNotProvedUnit { .. } => {
                "parametric-broadcast.stretch-source-not-proved-unit"
            }
            Self::Mapping(_) => "parametric-broadcast.mapping",
        }
    }
}

impl fmt::Display for ParametricBroadcastRule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroCapable {
                result_axis,
                declared,
            } => write!(
                formatter,
                "result axis {result_axis} declares {declared}, and a parametric broadcast requires this environment to prove that many-to-one extent is at least one"
            ),
            Self::ForeignEnvironment => formatter.write_str(
                "the parametric broadcast names an environment that is not the one offered to interpret it",
            ),
            Self::ExtentsNotProvedEqual {
                result_axis,
                declared,
                operand,
            } => write!(
                formatter,
                "result axis {result_axis} declares {declared} and reads operand extent {operand}, and this environment does not prove they are one extent"
            ),
            Self::ConcreteVariant => formatter.write_str(
                "a concrete broadcast replication or reindex bijection cannot stand in for the parametric broadcast carrier",
            ),
            Self::StretchSourceNotProvedUnit {
                operand_axis,
                extent,
            } => write!(
                formatter,
                "operand axis {} names {extent}, and this environment does not prove that extent is one",
                operand_axis.get()
            ),
            Self::Mapping(source) => write!(formatter, "{source}"),
        }
    }
}

impl std::error::Error for ParametricBroadcastRule {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Mapping(source) => Some(source),
            _ => None,
        }
    }
}

/// How fusion and costing may treat one broadcast-shaped access.
///
/// Replication-only transforms are legal only for
/// [`Self::ConcreteReplication`] and [`Self::ParametricProvedWidening`]. A
/// parametric mapping whose interval still includes one is
/// [`Self::ParametricUnprovedWidening`] and must be declined.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BroadcastTransformClass {
    /// Concrete [`LogicalAccess::BroadcastReplication`]. Already proved to widen.
    ConcreteReplication,
    /// Concrete [`LogicalAccess::ReindexBijection`]. Not a replication.
    ConcreteBijection,
    /// Parametric carrier whose environment proves every model widens.
    ParametricProvedWidening,
    /// Parametric carrier that may bind to one. Replication-only reasoning declines.
    ParametricUnprovedWidening,
}

/// Returns whether `mapping` names at least one symbol on the operand or result.
#[must_use]
pub fn mapping_names_a_symbol(operand: &SourcedShape, mapping: &BroadcastAxisMapping) -> bool {
    operand.extents().any(|extent| extent.symbol().is_some())
        || mapping
            .result_extents()
            .iter()
            .any(|extent| extent.symbol().is_some())
}

/// Returns whether every model of `environment` assigns at least one many-to-one
/// result extent a value of two or more.
///
/// One-sided: `false` means *not proved*, including a symbol whose interval is
/// `[1, upper]` and therefore includes the bijective binding.
#[must_use]
pub fn environment_proves_actual_widening(
    mapping: &BroadcastAxisMapping,
    environment: &ShapeEnv,
) -> bool {
    let sources = ExtentSources::new(Arc::new(environment.clone()));
    mapping
        .sources()
        .iter()
        .zip(mapping.result_extents())
        .any(|(source, extent)| source.is_many_to_one() && proves_at_least_two(&sources, extent))
}

/// Interprets `access` as the parametric broadcast carrier against `environment`.
///
/// # Errors
///
/// Returns a distinct [`ParametricBroadcastRule`] for each forged substitution
/// class: a zero-capable many-to-one extent, a foreign or undeclared
/// environment, an unproved `from-operand` equality, and a concrete variant
/// (or a wholly literal mapping stored on this carrier).
pub fn interpret_parametric_broadcast(
    access: &LogicalAccess,
    environment: &ShapeEnv,
) -> Result<(), ParametricBroadcastRule> {
    let LogicalAccess::ParametricBroadcast {
        operand_shape,
        mapping,
        environment: named,
    } = access
    else {
        return Err(ParametricBroadcastRule::ConcreteVariant);
    };
    if named != environment.identity() {
        return Err(ParametricBroadcastRule::ForeignEnvironment);
    }
    if !mapping_names_a_symbol(operand_shape, mapping) {
        return Err(ParametricBroadcastRule::ConcreteVariant);
    }
    let sources = ExtentSources::new(Arc::new(environment.clone()));
    mapping
        .apply(operand_shape, Some(&sources))
        .map(|_| ())
        .map_err(map_apply_error)
}

/// Classifies `access` for replication-only fusion and costing.
///
/// `None` for an access that is not a broadcast- or reindex-shaped relation.
#[must_use]
pub fn classify_broadcast_transform(
    access: &LogicalAccess,
    environment: Option<&ShapeEnv>,
) -> Option<BroadcastTransformClass> {
    match access {
        LogicalAccess::BroadcastReplication { .. } => {
            Some(BroadcastTransformClass::ConcreteReplication)
        }
        LogicalAccess::ReindexBijection { .. } => Some(BroadcastTransformClass::ConcreteBijection),
        LogicalAccess::ParametricBroadcast { mapping, .. } => {
            let environment = environment?;
            if interpret_parametric_broadcast(access, environment).is_err() {
                return Some(BroadcastTransformClass::ParametricUnprovedWidening);
            }
            if environment_proves_actual_widening(mapping, environment) {
                Some(BroadcastTransformClass::ParametricProvedWidening)
            } else {
                Some(BroadcastTransformClass::ParametricUnprovedWidening)
            }
        }
        _ => None,
    }
}

/// Returns whether a replication-only fusion or cost path may treat `access` as
/// a widening.
#[must_use]
pub fn replication_only_transform_is_admitted(
    access: &LogicalAccess,
    environment: &ShapeEnv,
) -> bool {
    matches!(
        classify_broadcast_transform(access, Some(environment)),
        Some(
            BroadcastTransformClass::ConcreteReplication
                | BroadcastTransformClass::ParametricProvedWidening
        )
    )
}

/// Returns whether `access` is an admissible pointwise parametric-broadcast read
/// of an iteration domain whose rank is `iteration_rank`.
///
/// Structural only: the environment proof is [`interpret_parametric_broadcast`].
/// A wholly literal mapping is refused here so it cannot become a second
/// spelling of [`LogicalAccess::BroadcastReplication`].
#[must_use]
pub fn parametric_broadcast_read_is_admissible(
    access: &LogicalAccess,
    iteration_rank: usize,
) -> bool {
    let LogicalAccess::ParametricBroadcast {
        operand_shape,
        mapping,
        ..
    } = access
    else {
        return false;
    };
    mapping_names_a_symbol(operand_shape, mapping)
        && mapping.result_extents().len() == iteration_rank
        && mapping
            .sources()
            .iter()
            .filter(|source| source.operand_axis().is_some())
            .count()
            == operand_shape.rank()
}

fn proves_at_least_two(sources: &ExtentSources, extent: &SourcedExtent) -> bool {
    match extent {
        SourcedExtent::Static(value) => value.get() >= 2,
        SourcedExtent::Symbol(_) => sources
            .interval(extent)
            .is_some_and(|interval| interval.lower >= 2),
    }
}

fn map_apply_error(error: BroadcastMappingError) -> ParametricBroadcastRule {
    match error {
        BroadcastMappingError::ExtentNotProvedPositive {
            result_axis,
            declared,
        } => ParametricBroadcastRule::ZeroCapable {
            result_axis,
            declared,
        },
        BroadcastMappingError::UndeclaredSymbol { .. }
        | BroadcastMappingError::SourceTooLate { .. } => {
            ParametricBroadcastRule::ForeignEnvironment
        }
        BroadcastMappingError::ExtentsNotProvedEqual {
            result_axis,
            declared,
            operand,
        } => ParametricBroadcastRule::ExtentsNotProvedEqual {
            result_axis,
            declared,
            operand,
        },
        BroadcastMappingError::StretchSourceNotProvedUnit {
            operand_axis,
            extent,
        } => ParametricBroadcastRule::StretchSourceNotProvedUnit {
            operand_axis,
            extent,
        },
        other => ParametricBroadcastRule::Mapping(other),
    }
}

/// Builds one parametric broadcast access, refusing a wholly literal mapping.
///
/// # Errors
///
/// Returns [`ParametricBroadcastRule::ConcreteVariant`] when neither the operand
/// nor the mapping names a symbol, [`ParametricBroadcastRule::Mapping`] when the
/// operand rank disagrees with the mapping, and
/// [`ParametricBroadcastRule::Mapping`] wrapping a shape-vocabulary refusal.
pub fn parametric_broadcast(
    operand_extents: impl IntoIterator<Item = SourcedExtent>,
    mapping: BroadcastAxisMapping,
    environment: ShapeEnvIdentity,
) -> Result<LogicalAccess, ParametricBroadcastRule> {
    let operand_shape = SourcedShape::sourced(operand_extents.into_iter().collect())
        .map_err(BroadcastMappingError::ResultShape)
        .map_err(ParametricBroadcastRule::Mapping)?;
    let consumed = mapping
        .sources()
        .iter()
        .filter(|source| source.operand_axis().is_some())
        .count();
    if consumed != operand_shape.rank() {
        return Err(ParametricBroadcastRule::Mapping(
            BroadcastMappingError::OperandAxesUnconsumed {
                consumed,
                rank: operand_shape.rank(),
            },
        ));
    }
    if !mapping_names_a_symbol(&operand_shape, &mapping) {
        return Err(ParametricBroadcastRule::ConcreteVariant);
    }
    Ok(LogicalAccess::ParametricBroadcast {
        operand_shape,
        mapping,
        environment,
    })
}

#[cfg(test)]
pub(crate) fn encode_logical_access_bytes(access: &LogicalAccess) -> Vec<u8> {
    let mut bytes = Vec::new();
    super::model::push_logical_access_for_test(&mut bytes, access);
    bytes
}

#[cfg(test)]
mod tests {
    use std::mem::variant_count;

    use super::*;
    use crate::schedule::model::push_bounds_proof_for_test;
    use crate::schedule::{
        AccessOrdinal, AxisDecode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
        LogicalAccess, TensorRole,
    };
    use crate::semantic::{BroadcastAxisSource, F32Broadcast, InputKey, OutputKey};
    use crate::shape::{
        Axis, BindingSource, EXTENT_PHASE_CEILING, Extent, ExtentRelation, ExtentTerm,
        FactProvenance, RootBinding, SemanticInputConstraint, Shape, ShapeEnvBuilder, ShapeSymbol,
        SymbolScope,
    };

    fn hex(bytes: &[u8]) -> String {
        let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
        for byte in bytes {
            use std::fmt::Write as _;
            let _ = write!(encoded, "{byte:02x}");
        }
        encoded
    }

    fn scope() -> SymbolScope {
        SymbolScope::new("parametric-broadcast/0").unwrap()
    }

    fn sym(name: &str) -> ShapeSymbol {
        ShapeSymbol::new(scope(), name).unwrap()
    }

    fn axis_binding(input: &str, axis: u32) -> RootBinding {
        RootBinding::new(
            BindingSource::InputDimension {
                input: InputKey::new(input).expect("a valid key"),
                axis: Axis::new(axis),
            },
            EXTENT_PHASE_CEILING,
            FactProvenance::RuntimeValidated,
        )
        .unwrap()
    }

    fn env_with(relations: &[ExtentRelation], names: &[&str]) -> Arc<ShapeEnv> {
        let mut draft = ShapeEnvBuilder::new();
        for (index, name) in names.iter().enumerate() {
            let declared = sym(name);
            draft.declare(declared.clone()).unwrap();
            draft
                .bind(
                    &declared,
                    axis_binding("operand", u32::try_from(index).unwrap()),
                )
                .unwrap();
        }
        for relation in relations {
            draft
                .require(SemanticInputConstraint::new(
                    relation.clone(),
                    FactProvenance::FrontendRequired,
                ))
                .unwrap();
        }
        Arc::new(draft.build().unwrap())
    }

    fn interval(name: &str, lower: u64, upper: u64) -> ExtentRelation {
        ExtentRelation::interval(ExtentTerm::Symbol(sym(name)), lower, upper).unwrap()
    }

    fn equal_to(name: &str, value: u64) -> ExtentRelation {
        ExtentRelation::equal(ExtentTerm::Symbol(sym(name)), ExtentTerm::Constant(value))
    }

    fn pad_mapping(result: Vec<SourcedExtent>) -> BroadcastAxisMapping {
        BroadcastAxisMapping::new(
            result,
            [
                BroadcastAxisSource::Replicate,
                BroadcastAxisSource::FromOperand(Axis::new(0)),
            ],
        )
        .expect("a rank-pad mapping is context-free")
    }

    fn carrier(
        environment: &ShapeEnv,
        operand: SourcedExtent,
        pad: SourcedExtent,
    ) -> LogicalAccess {
        parametric_broadcast(
            [operand],
            pad_mapping(vec![pad, operand_extent_n()]),
            environment.identity().clone(),
        )
        .expect("a symbolic rank pad is parametric")
    }

    fn operand_extent_n() -> SourcedExtent {
        SourcedExtent::Symbol(sym("n"))
    }

    fn pad_t() -> SourcedExtent {
        SourcedExtent::Symbol(sym("t"))
    }

    fn admitted_env(t_relation: ExtentRelation) -> Arc<ShapeEnv> {
        env_with(&[t_relation, interval("n", 2, 64)], &["n", "t"])
    }

    #[test]
    fn the_same_carrier_verifies_at_one_two_ten_and_the_admitted_upper_bound() {
        let t = pad_t();
        let n = operand_extent_n();
        for relation in [
            interval("t", 1, 32_768),
            interval("t", 2, 32_768),
            equal_to("t", 1),
            equal_to("t", 2),
            equal_to("t", 10),
            equal_to("t", 32_768),
        ] {
            let environment = admitted_env(relation);
            let access = carrier(&environment, n.clone(), t.clone());
            interpret_parametric_broadcast(&access, &environment)
                .unwrap_or_else(|error| panic!("{}: {error}", error.rule()));
        }
    }

    #[test]
    fn forged_zero_capable_foreign_environment_wrong_equality_and_concrete_variant_fail_distinctly()
    {
        let t = pad_t();
        let n = operand_extent_n();
        let admitted = admitted_env(interval("t", 1, 64));
        let access = carrier(&admitted, n.clone(), t.clone());
        interpret_parametric_broadcast(&access, &admitted).unwrap();

        let zero_capable = admitted_env(interval("t", 0, 64));
        let zero_access = LogicalAccess::ParametricBroadcast {
            operand_shape: SourcedShape::sourced(vec![n.clone()]).unwrap(),
            mapping: pad_mapping(vec![t.clone(), n.clone()]),
            environment: zero_capable.identity().clone(),
        };
        let error = interpret_parametric_broadcast(&zero_access, &zero_capable).unwrap_err();
        assert_eq!(error.rule(), "parametric-broadcast.zero-capable");
        assert!(
            error.to_string().contains("at least one"),
            "zero-capable refusal: {error}"
        );

        let error = interpret_parametric_broadcast(&access, &zero_capable).unwrap_err();
        assert_eq!(error.rule(), "parametric-broadcast.foreign-environment");
        assert!(
            error.to_string().contains("not the one offered"),
            "foreign-environment refusal: {error}"
        );

        let unequal = env_with(
            &[
                interval("n", 2, 64),
                interval("m", 2, 64),
                interval("t", 2, 64),
            ],
            &["n", "m", "t"],
        );
        let wrong_equality = parametric_broadcast(
            [n.clone()],
            BroadcastAxisMapping::new(
                vec![t.clone(), SourcedExtent::Symbol(sym("m"))],
                [
                    BroadcastAxisSource::Replicate,
                    BroadcastAxisSource::FromOperand(Axis::new(0)),
                ],
            )
            .unwrap(),
            unequal.identity().clone(),
        )
        .unwrap();
        let error = interpret_parametric_broadcast(&wrong_equality, &unequal).unwrap_err();
        assert_eq!(
            error.rule(),
            "parametric-broadcast.extents-not-proved-equal"
        );
        assert!(
            error
                .to_string()
                .contains("does not prove they are one extent"),
            "wrong-equality refusal: {error}"
        );

        let concrete = LogicalAccess::BroadcastReplication {
            operand_shape: Shape::from_dims([4]),
            result_shape: Shape::from_dims([8, 4]),
            axes: vec![AxisDecode::read(1, 4)],
        };
        let error = interpret_parametric_broadcast(&concrete, &admitted).unwrap_err();
        assert_eq!(error.rule(), "parametric-broadcast.concrete-variant");
        assert!(
            error.to_string().contains("cannot stand in"),
            "concrete-variant refusal: {error}"
        );

        let reindex = LogicalAccess::ReindexBijection {
            operand_shape: Shape::from_dims([4]),
            result_shape: Shape::from_dims([4]),
            axes: vec![AxisDecode::read(1, 4)],
        };
        assert_eq!(
            interpret_parametric_broadcast(&reindex, &admitted)
                .unwrap_err()
                .rule(),
            "parametric-broadcast.concrete-variant"
        );
    }

    #[test]
    fn replication_only_fusion_and_cost_decline_when_widening_is_unproved() {
        let t = pad_t();
        let n = operand_extent_n();
        let unproved = admitted_env(interval("t", 1, 64));
        let proved = admitted_env(interval("t", 2, 64));
        let unproved_access = carrier(&unproved, n.clone(), t.clone());
        let proved_access = carrier(&proved, n, t);

        assert!(
            !replication_only_transform_is_admitted(&unproved_access, &unproved),
            "T in [1, 64] includes the bijective binding"
        );
        assert!(
            replication_only_transform_is_admitted(&proved_access, &proved),
            "T in [2, 64] proves actual widening"
        );
        assert_eq!(
            classify_broadcast_transform(&unproved_access, Some(&unproved)),
            Some(BroadcastTransformClass::ParametricUnprovedWidening)
        );
        assert_eq!(
            classify_broadcast_transform(&proved_access, Some(&proved)),
            Some(BroadcastTransformClass::ParametricProvedWidening)
        );

        let concrete = LogicalAccess::BroadcastReplication {
            operand_shape: Shape::from_dims([4]),
            result_shape: Shape::from_dims([8, 4]),
            axes: vec![AxisDecode::read(1, 4)],
        };
        assert!(replication_only_transform_is_admitted(&concrete, &proved));
        let reindex = LogicalAccess::ReindexBijection {
            operand_shape: Shape::from_dims([4]),
            result_shape: Shape::from_dims([4]),
            axes: vec![AxisDecode::read(1, 4)],
        };
        assert!(!replication_only_transform_is_admitted(&reindex, &proved));
    }

    #[test]
    fn existing_concrete_reindex_and_broadcast_canonical_bytes_are_unchanged() {
        let reindex = LogicalAccess::ReindexBijection {
            operand_shape: Shape::from_dims([2, 3]),
            result_shape: Shape::from_dims([3, 2]),
            axes: vec![AxisDecode::read(1, 2), AxisDecode::read(2, 3)],
        };
        let broadcast = LogicalAccess::BroadcastReplication {
            operand_shape: Shape::from_dims([2]),
            result_shape: Shape::from_dims([2, 2]),
            axes: vec![AxisDecode::read(1, 2)],
        };
        let reindex_bytes = encode_logical_access_bytes(&reindex);
        let broadcast_bytes = encode_logical_access_bytes(&broadcast);
        assert_eq!(reindex_bytes.first().copied(), Some(0x06));
        assert_eq!(broadcast_bytes.first().copied(), Some(0x07));
        let reindex_again = encode_logical_access_bytes(&LogicalAccess::ReindexBijection {
            operand_shape: Shape::from_dims([2, 3]),
            result_shape: Shape::from_dims([3, 2]),
            axes: vec![AxisDecode::read(1, 2), AxisDecode::read(2, 3)],
        });
        let broadcast_again = encode_logical_access_bytes(&LogicalAccess::BroadcastReplication {
            operand_shape: Shape::from_dims([2]),
            result_shape: Shape::from_dims([2, 2]),
            axes: vec![AxisDecode::read(1, 2)],
        });
        assert_eq!(reindex_bytes, reindex_again);
        assert_eq!(broadcast_bytes, broadcast_again);
        assert_ne!(reindex_bytes, broadcast_bytes);
        assert_eq!(
            hex(&reindex_bytes),
            "06000000000000000200000000000000020000000000000003000000000000000200000000000000030000000000000002000000000000000200000000000000010000000000000002000000000000000002000000000000000300"
        );
        assert_eq!(
            hex(&broadcast_bytes),
            "070000000000000001000000000000000200000000000000020000000000000002000000000000000200000000000000010000000000000001000000000000000200"
        );
    }

    /// The gather relation's exact canonical bytes, field by field.
    ///
    /// A tag census shows the leading byte is fresh; it says nothing about the
    /// order of what follows. Two gathers that exchanged their source and index
    /// shapes, or their axis and ordinal, would carry distinct tags and equal
    /// lengths, so only a byte pin distinguishes them. The accepted surface
    /// requires exactly this: "gather field-order injectivity".
    ///
    /// The frame is tag, framed source shape, framed result shape, axis,
    /// index-access ordinal, framed index shape — every field recoverable at a
    /// position the frames determine.
    #[test]
    fn the_gather_relation_canonical_bytes_are_pinned_field_by_field() {
        let gather = LogicalAccess::GatherSource {
            source_shape: Shape::from_dims([8, 3]),
            result_shape: Shape::from_dims([2, 3]),
            axis: Axis::new(0),
            index_access: AccessOrdinal::new(1),
            index_shape: Shape::from_dims([2]),
        };
        let encoded = |access: &LogicalAccess| hex(&encode_logical_access_bytes(access));
        assert_eq!(
            encoded(&gather),
            concat!(
                "0c",
                "000000000000000200000000000000080000000000000003",
                "000000000000000200000000000000020000000000000003",
                "00000000",
                "00000001",
                "00000000000000010000000000000002",
            ),
        );
        // The two shapes are not interchangeable: a gather whose source and
        // index shapes are exchanged is a different relation and must not share
        // these bytes.
        let exchanged = LogicalAccess::GatherSource {
            source_shape: Shape::from_dims([2]),
            result_shape: Shape::from_dims([2, 3]),
            axis: Axis::new(0),
            index_access: AccessOrdinal::new(1),
            index_shape: Shape::from_dims([8, 3]),
        };
        assert_ne!(encoded(&gather), encoded(&exchanged));
        // Nor are the axis and the ordinal, which are adjacent fixed-width
        // fields and would otherwise be silently transposable.
        let transposed = LogicalAccess::GatherSource {
            source_shape: Shape::from_dims([8, 3]),
            result_shape: Shape::from_dims([2, 3]),
            axis: Axis::new(1),
            index_access: AccessOrdinal::new(0),
            index_shape: Shape::from_dims([2]),
        };
        assert_ne!(encoded(&gather), encoded(&transposed));
    }

    /// Every access relation's tag, sized from the enum rather than by hand.
    ///
    /// This is the whole-vocabulary injectivity test the tag derivations in
    /// `model.rs` refer to. It was a five-of-twelve sample until the gather
    /// relation landed; a sample cannot show that `0x0C` is free, which is
    /// exactly the fact a new relation needs. Sizing the array from
    /// [`variant_count`] makes a widened vocabulary a length type error here
    /// instead of a census that has quietly stopped covering its own domain.
    ///
    /// The expected list is written out rather than merely deduplicated,
    /// because distinctness alone would admit a relation that silently moved
    /// onto another's retired value — `0x09` is retired-and-never-reused, and
    /// no entry here may take it.
    #[test]
    fn every_access_relation_tag_is_distinct_and_pinned() {
        let environment = admitted_env(interval("t", 1, 64));
        let relations: [LogicalAccess; variant_count::<LogicalAccess>()] = [
            LogicalAccess::LinearIdentity,
            LogicalAccess::ScalarBroadcast,
            LogicalAccess::PackedU4LsbZeroTail {
                logical_elements: 4,
            },
            LogicalAccess::ReductionContributor {
                input_shape: Shape::from_dims([2, 3]),
                output_shape: Shape::from_dims([2]),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            LogicalAccess::ContractionOperand {
                operand_shape: Shape::from_dims([2, 3]),
                output_shape: Shape::from_dims([2]),
                contracted_shape: Shape::from_dims([3]),
                sources: Vec::new(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            LogicalAccess::ReindexBijection {
                operand_shape: Shape::from_dims([2, 3]),
                result_shape: Shape::from_dims([3, 2]),
                axes: vec![AxisDecode::read(1, 2), AxisDecode::read(2, 3)],
            },
            LogicalAccess::BroadcastReplication {
                operand_shape: Shape::from_dims([2]),
                result_shape: Shape::from_dims([2, 2]),
                axes: vec![AxisDecode::read(1, 2)],
            },
            carrier(&environment, operand_extent_n(), pad_t()),
            LogicalAccess::LiveRowMajorSource {
                inner_axis: Axis::new(0),
            },
            LogicalAccess::LiveRowMajor,
            LogicalAccess::PartitionedCopySource,
            LogicalAccess::GatherSource {
                source_shape: Shape::from_dims([8, 3]),
                result_shape: Shape::from_dims([2, 3]),
                axis: Axis::new(0),
                index_access: AccessOrdinal::new(1),
                index_shape: Shape::from_dims([2]),
            },
        ];
        let tags: Vec<u8> = relations
            .iter()
            .map(|relation| encode_logical_access_bytes(relation)[0])
            .collect();
        let mut unique = tags.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), tags.len(), "access tags collided: {tags:?}");
        assert_eq!(
            tags,
            [
                0x01, 0x03, 0x04, 0x02, 0x05, 0x06, 0x07, 0x08, 0x0A, 0x0B, 0x0D, 0x0C,
            ],
            "the pinned access-relation tag assignment moved",
        );
        assert!(
            !tags.contains(&0x09),
            "0x09 is the retired live-row-major relation and is never reused",
        );
    }

    /// Every bounds-proof kind's tag, sized from the enum.
    ///
    /// The proof tags occupy their own `0x1X` family run, which is a separate
    /// frame from the access-relation space above: `0x01` means
    /// `LinearIdentity` in one and nothing at all in the other. That is why the
    /// gather proof takes `0x13` rather than the `0x03` its packet named — the
    /// run convention, not a byte-level collision.
    #[test]
    fn every_bounds_proof_kind_tag_is_distinct_and_pinned() {
        let kinds: [BoundsProofKind; variant_count::<BoundsProofKind>()] = [
            BoundsProofKind::LinearRange { element_count: 4 },
            BoundsProofKind::ReductionDomain {
                input_shape: Shape::from_dims([2, 3]),
                output_shape: Shape::from_dims([2]),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            BoundsProofKind::GatherSource {
                source_shape: Shape::from_dims([8, 3]),
                result_shape: Shape::from_dims([2, 3]),
                axis: Axis::new(0),
                index_access: AccessOrdinal::new(1),
                index_shape: Shape::from_dims([2]),
                proof: Box::new(crate::schedule::builder::gather_tests::static_gather_proof()),
            },
        ];
        let tags: Vec<u8> = kinds
            .iter()
            .map(|kind| {
                let mut bytes = Vec::new();
                push_bounds_proof_for_test(
                    &mut bytes,
                    &BoundsProof {
                        id: BoundsWitnessId::new(0),
                        tensor: TensorRole::Input,
                        component_role: None,
                        kind: kind.clone(),
                    },
                );
                // id (4) + tensor role (1) + component role (1) then the kind tag.
                bytes[6]
            })
            .collect();
        let mut unique = tags.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), tags.len(), "proof tags collided: {tags:?}");
        assert_eq!(
            tags,
            [0x11, 0x12, 0x13],
            "the bounds-proof family run is 0x1X and its assignment moved",
        );
    }

    #[test]
    fn a_literal_mapping_cannot_wear_the_parametric_carrier() {
        let environment = admitted_env(interval("t", 1, 64));
        let error = parametric_broadcast(
            [SourcedExtent::Static(Extent::new(4))],
            BroadcastAxisMapping::new(
                [Extent::new(8), Extent::new(4)],
                [
                    BroadcastAxisSource::Replicate,
                    BroadcastAxisSource::FromOperand(Axis::new(0)),
                ],
            )
            .unwrap(),
            environment.identity().clone(),
        )
        .unwrap_err();
        assert_eq!(error.rule(), "parametric-broadcast.concrete-variant");
    }

    #[test]
    fn semantic_apply_is_not_required_to_construct_the_carrier() {
        let environment = admitted_env(interval("t", 1, 64));
        let mut builder =
            crate::semantic::SemanticProgramBuilder::try_standard_with_shape_environment(
                Arc::clone(&environment),
            )
            .unwrap();
        let input = builder
            .input_sourced::<crate::semantic::F32>(
                InputKey::new("operand").unwrap(),
                vec![operand_extent_n()],
            )
            .unwrap();
        let mapping = pad_mapping(vec![pad_t(), operand_extent_n()]);
        let widened = F32Broadcast::apply(&mut builder, &mapping, input).unwrap();
        builder
            .output(OutputKey::new("widened").unwrap(), widened)
            .unwrap();
        let program = builder.build().unwrap();
        assert_eq!(program.operation_count(), 1);
        let access = carrier(&environment, operand_extent_n(), pad_t());
        interpret_parametric_broadcast(&access, &environment).unwrap();
    }
}
