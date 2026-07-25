//! The preflight stage and the one-way routing commit.
//!
//! # The two stages are two types, so the order is not a convention
//!
//! ADR 0051 requires routing to commit one way, before program work, and
//! forbids falling back after it. That is enforced here by construction rather
//! than by documentation. Every obligation that can refuse lives in
//! [`super::DecodedProgram::preflight`], which returns a [`Preflight`] or a
//! typed rejection; [`Preflight::commit`] consumes that value and is
//! **infallible**. So there is nothing left that can fail after the commit, and
//! no way to hold both a committed route and an uncommitted one — the value the
//! fallback would have needed is gone.
//!
//! A caller that wants a fallback takes it by not calling [`Preflight::commit`],
//! which is exactly ADR 0051's "fallback only before program work".
//!
//! # What this crate commits to, and what it cannot
//!
//! A committed route names one carried object and the descriptor that
//! identifies it. It does **not** name an entry symbol, a binding-to-buffer
//! correspondence, or an evaluated launch extent, because a decoded envelope
//! publishes none of those: the payload-metadata section has no public parser,
//! `BindingData` carries no value reference, and every expression accessor
//! hangs off a `VerifiedArtifactProgram` that no decode produces. A host
//! therefore still supplies those from the program it compiled, bound to these
//! bytes by the identity check preflight performed.
//!
//! That is a real limit and it is stated rather than worked around.
//! `carry-reconstructable-kernel-programs-in-the-neutral-envelope` owns closing
//! it, and until it does, a caller that does not hold the program it compiled
//! cannot dispatch from an artifact alone.

use tiler_artifact::program::{
    ArtifactExecutionPolicy, BackendPayloadDescriptor, CanonicalArtifactProgramIdentity,
};

/// One artifact that passed every obligation this loader can decide.
///
/// Deliberately neither [`Clone`] nor [`Copy`]. A route that could be duplicated
/// could be committed twice, and "committed once" is the property ADR 0051
/// asks for.
#[derive(Debug)]
#[must_use = "a preflight that is neither committed nor abandoned decides nothing"]
pub struct Preflight<'a> {
    pub(super) identity: CanonicalArtifactProgramIdentity,
    pub(super) payload: &'a BackendPayloadDescriptor,
    pub(super) object: &'a [u8],
}

impl<'a> Preflight<'a> {
    /// Returns the identity of the artifact this route would execute.
    #[must_use]
    pub const fn identity(&self) -> &CanonicalArtifactProgramIdentity {
        &self.identity
    }

    /// Returns the descriptor of the payload this route selected.
    #[must_use]
    pub const fn payload(&self) -> &'a BackendPayloadDescriptor {
        self.payload
    }

    /// Commits to executing this route. One way, and infallible.
    ///
    /// There is no `Result` here on purpose. Every decidable obligation was
    /// discharged by the preflight that produced this value, so a failure at
    /// this point would mean an obligation was checked in the wrong stage.
    /// Consuming `self` is what makes the commit one-way: the caller cannot
    /// afterwards hold this value to fall back to.
    #[must_use]
    pub fn commit(self) -> RoutedDispatch<'a> {
        let Self {
            identity,
            payload,
            object,
        } = self;
        RoutedDispatch {
            identity,
            payload,
            object,
        }
    }
}

/// A committed route: the exact object bytes a host may now load.
///
/// Reaching this type is the boundary ADR 0051 draws. Everything before it may
/// be abandoned for a fallback; everything after it is program work, and a
/// failure there is reported rather than retried on another route.
/// `Clone` here is deliberate and is not the permission [`Preflight`] withholds.
/// Cloning a route that is already committed cannot un-commit it or produce a
/// second choice; it only lets a host hand the committed decision to the code
/// that encodes it.
#[derive(Clone, Debug)]
pub struct RoutedDispatch<'a> {
    identity: CanonicalArtifactProgramIdentity,
    payload: &'a BackendPayloadDescriptor,
    object: &'a [u8],
}

impl<'a> RoutedDispatch<'a> {
    /// Returns the identity of the artifact being executed.
    #[must_use]
    pub const fn identity(&self) -> &CanonicalArtifactProgramIdentity {
        &self.identity
    }

    /// Returns the descriptor of the payload this route committed to.
    #[must_use]
    pub const fn payload(&self) -> &'a BackendPayloadDescriptor {
        self.payload
    }

    /// Returns how the committed object reaches an executable state.
    ///
    /// Always [`ArtifactExecutionPolicy::NativeImage`] in this build — preflight
    /// refuses anything else — and returned rather than assumed, so a host does
    /// not hard-code the assumption at its own load site.
    #[must_use]
    pub const fn execution_policy(&self) -> ArtifactExecutionPolicy {
        self.payload.execution_policy
    }

    /// Returns the exact emitted object bytes the artifact carries.
    ///
    /// These are the bytes the producer handed to `push_carried_payload`,
    /// byte for byte: the envelope's framing is stripped by the decoder and the
    /// section body is the object itself. A host loads *these* and nothing it
    /// held before, which is what makes the envelope load-bearing rather than
    /// descriptive.
    #[must_use]
    pub const fn object(&self) -> &'a [u8] {
        self.object
    }
}
