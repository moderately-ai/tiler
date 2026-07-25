//! Runtime fact binding and availability-phase enforcement.
//!
//! ADR 0068 splits the ABI expression language in two. The domain type, its
//! roots, validation, canonical identity, and pure checked evaluation belong to
//! the executable-program IR; **binding runtime facts and enforcing the phase
//! at which each could legally be queried belong to this crate**. This module
//! is that half, and it stays here whichever way
//! `complete-program-identity-with-abi-guards-and-routing` resolves.
//!
//! A binder is monotonic in exactly one direction: it is opened at the phase
//! preparation has actually reached, and it refuses any fact that could not
//! have been observed by then. A consumer therefore cannot smuggle a
//! prepared-pipeline fact into a live-device guard, and evaluation never has to
//! ask when a value became true.

use tiler_ir::semantic::InputKey;
use tiler_ir::shape::{Axis, Shape};

use super::error::{ArtifactBuildError, ArtifactLimitKind, limit};
use super::expr::TargetPropertyKey;
use super::expr::{AbiFacts, AvailabilityPhase};

/// Maximum bound input-axis extents admitted by one fact environment.
pub const MAX_BOUND_INPUT_EXTENTS: usize = 4_096;
/// Maximum bound target properties admitted by one fact environment.
pub const MAX_BOUND_TARGET_PROPERTIES: usize = 256;

/// A binding-time failure while assembling an ABI fact environment.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AbiBindingError {
    /// A fact was offered that preparation has not reached the phase to observe.
    PhaseNotReached {
        /// Earliest phase at which the offered fact becomes observable.
        available_at: AvailabilityPhase,
        /// Phase preparation has actually reached.
        reached: AvailabilityPhase,
    },
    /// The same input axis was bound twice.
    DuplicateInputExtent {
        /// Interface key that was bound twice.
        key: InputKey,
        /// Axis that was bound twice.
        axis: Axis,
    },
    /// The same target property was bound twice.
    DuplicateTargetProperty {
        /// Governed property key that was bound twice.
        key: TargetPropertyKey,
    },
    /// A governed binding resource exceeded its limit.
    StructuralLimit {
        /// Governed resource.
        resource: ArtifactLimitKind,
        /// Attempted quantity.
        actual: usize,
        /// Maximum admitted quantity.
        limit: usize,
    },
}

impl std::fmt::Display for AbiBindingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for AbiBindingError {}

/// A transactional binder for one ABI fact environment.
///
/// Only [`AbiFactBinder::build`] produces an [`AbiFacts`], so an environment
/// an expression is evaluated against always went through phase enforcement.
#[derive(Clone, Debug)]
pub struct AbiFactBinder {
    reached: AvailabilityPhase,
    input_extents: Vec<(InputKey, Axis, u64)>,
    target_properties: Vec<(TargetPropertyKey, u64)>,
}

impl AbiFactBinder {
    /// Opens a binder at the phase preparation has actually reached.
    #[must_use]
    pub const fn new(reached: AvailabilityPhase) -> Self {
        Self {
            reached,
            input_extents: Vec::new(),
            target_properties: Vec::new(),
        }
    }

    /// Binds one axis extent of a named program input.
    ///
    /// Input extents become observable at
    /// [`AvailabilityPhase::LiveDevicePreflight`], when the semantic root
    /// bindings are constructed.
    ///
    /// # Errors
    ///
    /// Returns [`AbiBindingError::PhaseNotReached`] before live preflight,
    /// [`AbiBindingError::DuplicateInputExtent`] for a repeated axis, or a
    /// structural-limit error.
    pub fn bind_input_extent(
        &mut self,
        key: InputKey,
        axis: Axis,
        extent: u64,
    ) -> Result<(), AbiBindingError> {
        self.check_phase(AvailabilityPhase::LiveDevicePreflight)?;
        if self
            .input_extents
            .iter()
            .any(|(bound, bound_axis, _)| *bound == key && *bound_axis == axis)
        {
            return Err(AbiBindingError::DuplicateInputExtent { key, axis });
        }
        bind_limit(
            self.input_extents.len().saturating_add(1),
            MAX_BOUND_INPUT_EXTENTS,
            ArtifactLimitKind::BoundInputExtents,
        )?;
        self.input_extents.push((key, axis, extent));
        Ok(())
    }

    /// Binds every axis extent of one named program input from its shape.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`AbiFactBinder::bind_input_extent`].
    pub fn bind_input_shape(
        &mut self,
        key: &InputKey,
        shape: &Shape,
    ) -> Result<(), AbiBindingError> {
        for (axis, extent) in shape.extents().iter().enumerate() {
            let axis = u32::try_from(axis).map_err(|_| AbiBindingError::StructuralLimit {
                resource: ArtifactLimitKind::BoundInputExtents,
                actual: axis,
                limit: MAX_BOUND_INPUT_EXTENTS,
            })?;
            self.bind_input_extent(key.clone(), Axis::new(axis), extent.get())?;
        }
        Ok(())
    }

    /// Binds one governed target property observed at a declared phase.
    ///
    /// # Errors
    ///
    /// Returns [`AbiBindingError::PhaseNotReached`] when the property could not
    /// have been observed yet, [`AbiBindingError::DuplicateTargetProperty`] for
    /// a repeated key, or a structural-limit error.
    pub fn bind_target_property(
        &mut self,
        key: TargetPropertyKey,
        available_at: AvailabilityPhase,
        value: u64,
    ) -> Result<(), AbiBindingError> {
        self.check_phase(available_at)?;
        if self
            .target_properties
            .iter()
            .any(|(bound, _)| *bound == key)
        {
            return Err(AbiBindingError::DuplicateTargetProperty { key });
        }
        bind_limit(
            self.target_properties.len().saturating_add(1),
            MAX_BOUND_TARGET_PROPERTIES,
            ArtifactLimitKind::BoundTargetProperties,
        )?;
        self.target_properties.push((key, value));
        Ok(())
    }

    /// Freezes the bound facts into a resolved evaluation environment.
    #[must_use]
    pub fn build(mut self) -> AbiFacts {
        self.input_extents
            .sort_unstable_by(|left, right| (&left.0, left.1).cmp(&(&right.0, right.1)));
        self.target_properties
            .sort_unstable_by(|left, right| left.0.cmp(&right.0));
        AbiFacts::new(self.reached, self.input_extents, self.target_properties)
    }

    fn check_phase(&self, available_at: AvailabilityPhase) -> Result<(), AbiBindingError> {
        if available_at > self.reached {
            return Err(AbiBindingError::PhaseNotReached {
                available_at,
                reached: self.reached,
            });
        }
        Ok(())
    }
}

fn bind_limit(
    actual: usize,
    maximum: usize,
    resource: ArtifactLimitKind,
) -> Result<(), AbiBindingError> {
    limit(actual, maximum, resource).map_err(|error| match error {
        ArtifactBuildError::StructuralLimit {
            resource,
            actual,
            limit,
        } => AbiBindingError::StructuralLimit {
            resource,
            actual,
            limit,
        },
        _ => unreachable!("the shared limit helper returns only a structural-limit error"),
    })
}
