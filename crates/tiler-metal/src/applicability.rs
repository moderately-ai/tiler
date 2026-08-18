//! Whether a live host is the exact macOS row the first Metal profile was measured on.
//!
//! This module is two pure functions over Apple's own vocabulary. Neither
//! touches a device, spawns a process, reads an environment variable, or
//! consults an artifact: a platform adapter observes the host, and
//! [`evaluate_metal_host_applicability`](crate::applicability::evaluate_metal_host_applicability)
//! decides. The split is what lets every policy case — including the ones no
//! machine in this project can produce — run in the ordinary test gate without
//! Metal hardware.
//!
//! The second function,
//! [`try_observe_highest_gpu_family`](crate::applicability::try_observe_highest_gpu_family),
//! is on the observing side of that split and is still device-free: it walks
//! [`MetalGpuFamily::ALL`](crate::applicability::MetalGpuFamily::ALL) and asks
//! the caller one fallible yes-or-no question per family, so the *population*
//! and abort-the-whole-walk rule are this crate's rather than tables and side
//! channels written beside each device call.
//!
//! # It refuses on every host, and that is the decision rather than a gap
//!
//! [ADR 0086](../../../docs/decisions/0086-require-attributable-or-attested-native-translation.md)
//! decides that native device translation of a metallib during pipeline
//! creation is a typed capability fact whose authority and provenance are
//! `Unknown` on every macOS row currently observable, and that a positive
//! host-applicability receipt requires either an attributable identity for the
//! private translating component or exact host attestation. Neither exists on
//! current APIs, so [`evaluate_metal_host_applicability`](crate::applicability::evaluate_metal_host_applicability) returns
//! [`MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority`](crate::applicability::MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority) even for
//! an observation that matches the measured row in every public field.
//!
//! That refusal is ADR 0043's existing disposal of `Unknown` applied, not a new
//! outcome class: it names the exact unsatisfied predicate
//! ([`MetalHostPredicate::NativeTranslationAuthority`](crate::applicability::MetalHostPredicate::NativeTranslationAuthority)) so explain output can
//! tell it from a disproved hard predicate and from a cost disadvantage.
//!
//! The environment predicates are still evaluated, and still refused precisely,
//! because they are the validity scope every future positive receipt will also
//! need. ADR 0086 item 3 is explicit that the measured row remains *necessary*
//! and is not *sufficient*.
//!
//! # The measured row
//!
//! The required values of [`MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9`](crate::applicability::MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9)
//! are transcribed from two retained records, which agree field for field:
//!
//! - `spikes/apple-targets/aot-runtime-compiler-observer/results/2026-07-31-macos27-m4max/clean-1.tsv`,
//!   the run ADR 0086 cites, records `environment.os_version=27.0`,
//!   `environment.os_build=26A5388g`, `environment.architecture=arm64`,
//!   `environment.device=Apple M4 Max`, and
//!   `environment.device_apple9_support=supported`.
//! - `spikes/apple-targets/results/2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`,
//!   the unified MSL 4 measurement this ticket consumes (the 2026-07-30 record
//!   it replaces is retained beside it as the previous row and agrees on every
//!   field named here), records
//!   `probe.required_gpu_family apple9`, `environment.os_version 27.0`,
//!   `environment.os_build 26A5388g`, `environment.machine arm64`,
//!   `environment.family.macos.device Apple M4 Max`, and
//!   `environment.family.macos.device_apple9_support supported`.
//!
//! The `macos26` in that second directory name is the *deployment minimum* of
//! the offline request (`air64-apple-macos26.0`), not the host OS version. The
//! host ran macOS 27.0, and a reader reconciling the two names should not
//! "correct" either.
//!
//! # What is deliberately not an input
//!
//! - **The device registry ID.** ADR 0086 excludes it by name: the retained
//!   records report `4294968621` and `4294968452` for the same named Apple M4
//!   Max, so a predicate keyed on the 2026-07-25 value would have rejected the
//!   same machine on 2026-07-27. The [Apple GPU numerical
//!   behaviour](../../../docs/research/apple-targets/numerical-behaviour.md)
//!   record owns the measurement and its lifetime bound.
//! - **`Compilation`, a target profile reference, decoded artifact bytes,
//!   offline compiler provenance, and the source-JIT compiler identity.** The
//!   first two would make the check a tautology — the host would earn
//!   eligibility from the producer's own declaration — and the last three are
//!   eliminated by name in ADR 0086 item 4. None of them can reach this
//!   function: its two parameters are the two types declared below, whose fields
//!   are `&'static str`, `String`, and this module's own enums. `tiler-metal`
//!   also cannot *name* a `Compilation` or an offline compiler identity, because
//!   its `[dependencies]` are exactly `tiler-artifact` and `tiler-ir` and
//!   `tiler-metal-aot` is a development dependency;
//!   `crate::applicability_tests::the_dependency_set_keeps_producer_types_unnameable`
//!   is the check, and `crate::applicability_tests` also pins the two argument
//!   positions against a `TargetProfileRef` and against artifact bytes.
//!
//! # Why this lives in `tiler-metal`
//!
//! Both consumers reach it with no new dependency edge: `tiler-build` owns the
//! profile declaration this receipt will eventually bind to, and
//! `prototypes/serial-sum-run` hosts the first adapter, and both already depend
//! on this crate. `tiler-runtime` is eliminated because it is backend-neutral by
//! charter and `tiler-build` does not depend on it; `tiler-compiler` and
//! `tiler-artifact` are eliminated for the same neutrality reason;
//! `tiler-metal-aot` is eliminated because it owns the offline compiler
//! provenance ADR 0086 excludes from this decision. The full elimination is
//! recorded in `tickets/validate-macos-metal-profile-host-applicability.md`.
//!
//! This is a *live-host* fact class, so it is a sibling of
//! [`crate::target`] rather than a member of it: that module states it holds
//! compile-time target facts only, and a host observation is not one.
//!
//! # The normalization an adapter owes
//!
//! The policy compares exact bytes, so an adapter must hand over the spellings
//! the retained records use rather than whatever its own API happens to return:
//!
//! - **OS family** as `std::env::consts::OS` spells it — `macos`.
//! - **Architecture** as the records spell it — `arm64`. `std::env::consts::ARCH`
//!   says `aarch64` for the same machine, so an adapter reading it must map that
//!   one spelling and pass everything else through unchanged, so that an
//!   unexpected architecture is refused by name rather than renamed into one.
//! - **OS version** and **OS build** exactly as the platform reports them
//!   (`27.0`, `26A5388g`).
//! - **Device name** exactly as `MTLDevice.name` reports it (`Apple M4 Max`).
//!
//! The **GPU family** is the one field an adapter does not spell for itself.
//! It calls [`try_observe_highest_gpu_family`](crate::applicability::try_observe_highest_gpu_family)
//! and forwards each enumerator to its own `supportsFamily`, so the families
//! probed and the family named are one authority rather than two.
//!
//! Every public item here is an accepted boundary, and this module's
//! provenance predates the whole-crate acceptance: Tom accepted the exact
//! host-applicability packet at `6c1cd1e`
//! (`tickets/validate-macos-metal-profile-host-applicability.md`), the
//! exhaustive-family/raw-constant correction is recorded by
//! `close-the-metal-gpu-family-out-of-crate-total-map`, and the fallible
//! observer by `decide-the-unnameable-gpu-enumerator-channel`. The 2026-08-18
//! whole-facade acceptance
//! (`tickets/decide-the-tiler-metal-public-facade-surface.md`) preserved this
//! surface byte for byte. Accepted is not stabilized — ADR 0075's pre-alpha
//! posture keeps a later source break cheap, explicit, and reviewed.

use core::fmt;
use std::error::Error;

/// One Apple GPU family a device may report supporting.
///
/// The set is bounded by what the retained measurements needed, and it is
/// narrower than Apple's: `MTLDevice.h` in the macOS 26.5 SDK (build `25F70`)
/// declares `MTLGPUFamilyApple1 = 1001` through `MTLGPUFamilyApple10 = 1010`,
/// and this vocabulary names five of those ten. A family Apple ships and this
/// enum does not name is not observable through
/// [`try_observe_highest_gpu_family`] and is reported as
/// [`MetalGpuFamilySupport::NoneNamed`] or as a lower family, which is a
/// deliberate consequence of scoping the vocabulary to measured rows rather
/// than an oversight. Widening to `Apple10` is deferred until a retained
/// measurement observes a device that reports `MTLGPUFamilyApple10` and the
/// `metal`-binding gap in `prototypes/serial-sum-run` is closed;
/// `widen-the-metal-gpu-family-vocabulary-to-apple10` records the deferral,
/// its grounds, and that trigger.
///
/// **An ADR 0074 convention 5b type, deliberately exhaustive.** It carries no
/// `#[non_exhaustive]`, and the reason is the opposite of the one this
/// declaration used to state. It previously read "no consumer outside this
/// crate classifies it by exhaustive match", which was false:
/// `prototypes/serial-sum-run` pairs every variant with its Apple counterpart,
/// and `prototypes/candle-metal-adapter` did the same until
/// [`try_observe_highest_gpu_family`] gave it a probe that names no family at all.
/// A pairing like that is 5b's total map — every variant must contribute the
/// Apple constant it alone determines, and there is no constant a wildcard
/// could return — so the attribute forced such a consumer to choose between an
/// arm that cannot be written correctly and a hand-written table, and the table
/// is what it chose. A table has no arm to be missing, so a family added here
/// left the device unprobed with the tree green: convention 5c's
/// "fail-closed but silently incomplete" reached without the attribute ever
/// being consulted.
///
/// Growth is therefore announced by the compiler. Adding a family is a build
/// error at [`Self::as_str`], at [`Self::apple_constant`], and — through
/// [`Self::ALL`]'s completeness assertion — at the list every probe is driven
/// from.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MetalGpuFamily {
    /// `MTLGPUFamilyApple5`.
    Apple5,
    /// `MTLGPUFamilyApple6`.
    Apple6,
    /// `MTLGPUFamilyApple7`.
    Apple7,
    /// `MTLGPUFamilyApple8`.
    Apple8,
    /// `MTLGPUFamilyApple9`.
    Apple9,
}

impl MetalGpuFamily {
    /// Every Apple family this vocabulary names, lowest first.
    ///
    /// Ascending order is load-bearing: [`try_observe_highest_gpu_family`] walks
    /// this list in reverse so a cumulative device answers the most specific
    /// true statement about itself on the first query, and the const assertion
    /// below rejects a member inserted out of order.
    pub const ALL: [Self; core::mem::variant_count::<Self>()] = [
        Self::Apple5,
        Self::Apple6,
        Self::Apple7,
        Self::Apple8,
        Self::Apple9,
    ];

    /// How many Apple families this vocabulary names.
    pub const COUNT: usize = Self::ALL.len();

    /// Returns the spelling Apple's `MTLGPUFamily` constant uses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Apple5 => "Apple5",
            Self::Apple6 => "Apple6",
            Self::Apple7 => "Apple7",
            Self::Apple8 => "Apple8",
            Self::Apple9 => "Apple9",
        }
    }

    /// Returns the `MTLGPUFamily` enumerator Apple declares for this family.
    ///
    /// This is the Apple-side authority for the whole workspace, and it lives
    /// here because a total map from this vocabulary belongs in the crate that
    /// defines it: written as this exhaustive wildcard-free match, a family
    /// added to the enum is an `E0004` here, in a crate the lint gate and the
    /// test gate both cover. Written in a consumer it was a hand-written table
    /// outside both.
    ///
    /// The values are transcribed from `MTLDevice.h` in the macOS 26.5 SDK
    /// (`$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h:237-241`),
    /// and `crate::applicability_tests` pins each one so a silent edit is not
    /// silent.
    #[must_use]
    pub const fn apple_constant(self) -> AppleGpuFamilyConstant {
        AppleGpuFamilyConstant(match self {
            Self::Apple5 => 1005,
            Self::Apple6 => 1006,
            Self::Apple7 => 1007,
            Self::Apple8 => 1008,
            Self::Apple9 => 1009,
        })
    }
}

/// Compiles only while [`MetalGpuFamily::ALL`] ascends, lowest family first.
///
/// **Completeness is not asserted here, because it cannot be.** It is delivered
/// by the declaration: `ALL`'s length is written as `variant_count`, so a family
/// added to the enum and left out of the list is an array-length `E0308` at the
/// declaration itself. That is the check that can say no about a *population* —
/// every other site that has to know about a family is an exhaustive match,
/// which `rustc` closes on its own, but an array is a hand-written list and a
/// family left out of it is a device silently never probed for that family. An
/// `assert!` restating the declared length would compare `variant_count` against
/// itself and could never fail; one stood here, and a reader who mistook it for
/// the guard would look for a const-eval failure rather than for the type of
/// `ALL`. The same sizing carries [`MetalHostPredicate::ALL`] and the three
/// vocabularies in [`crate::target`], none of which needs an assertion either.
///
/// The order half is a real check and is what remains: it compares Apple's own
/// enumerators rather than this type's derived `Ord`, because `Ord` follows
/// declaration order and would agree with a misordered `ALL` for exactly the
/// reason that made it wrong.
const _: () = {
    let mut index = 1;
    while index < MetalGpuFamily::ALL.len() {
        assert!(
            MetalGpuFamily::ALL[index - 1].apple_constant().value()
                < MetalGpuFamily::ALL[index].apple_constant().value(),
            "MetalGpuFamily::ALL must ascend, lowest Apple family first",
        );
        index += 1;
    }
};

impl fmt::Display for MetalGpuFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One Apple `MTLGPUFamily` enumerator, as the raw value a caller passes to its
/// own Metal binding.
///
/// Deliberately not an `MTLGPUFamily`: this crate is pure lowering and target
/// metadata and names no Metal runtime type, and a raw enumerator crosses to
/// either shape the ecosystem uses — `objc2-metal` 0.3.2 models `MTLGPUFamily`
/// as `MTLGPUFamily(pub NSInteger)`, which takes this value directly, while
/// `metal` 0.33.0 models it as a `#[repr(i64)]` Rust enum, which a caller names
/// back itself.
///
/// `isize` because that is what `NSInteger` is: `MTLDevice.h` declares the
/// enumeration as `NS_ENUM(NSInteger, MTLGPUFamily)`, and `NSInteger` is
/// pointer-sized. Spelling it `i64` would have been correct on every target
/// Tiler supports and would still have made the one binding that names the
/// Apple type exactly insert a fallible conversion.
///
/// Opaque under ADR 0074 convention 2: the field is private, so a caller cannot
/// mint an enumerator for a family [`MetalGpuFamily`] does not name and hand it
/// back as though this vocabulary had produced it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AppleGpuFamilyConstant(isize);

impl AppleGpuFamilyConstant {
    /// Returns the raw `MTLGPUFamily` value, for the caller's own binding.
    #[must_use]
    pub const fn value(self) -> isize {
        self.0
    }
}

impl fmt::Display for AppleGpuFamilyConstant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Observes the highest Apple family a device reports supporting.
///
/// The caller supplies one thing — whether the bound device supports the
/// enumerator it is handed, or why it could not ask — and this walks
/// [`MetalGpuFamily::ALL`] in reverse, stopping at the first supported family or
/// the first failed query. Highest first because Apple's families are
/// cumulative, so the highest supported one is the most specific true statement
/// a device makes about itself, and the walk costs one device query on a device
/// that supports the newest family this vocabulary names.
///
/// # Why the walk is here and not at the call site
///
/// Every consumer that wrote this itself wrote it as a table pairing each
/// variant with its Apple constant, and a table is not a match: a family added
/// to [`MetalGpuFamily`] compiled cleanly at every such site, the device was
/// never asked about it, and the applicability policy then refused a machine
/// that satisfied it while naming the wrong observed family. Driving the walk
/// from `ALL` removes the population from the call site entirely, so the only
/// way to under-probe a device is to under-populate `ALL` — which the assertion
/// beside it rejects at compile time.
///
/// ```
/// use tiler_metal::applicability::{
///     MetalGpuFamily, MetalGpuFamilySupport, try_observe_highest_gpu_family,
/// };
///
/// // A device that claims Apple7 and everything below it.
/// let observed = try_observe_highest_gpu_family::<core::convert::Infallible>(|family| {
///     Ok(family.value() <= 1007)
/// });
/// assert_eq!(observed, Ok(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple7)));
///
/// // A device that claims none of them is a different answer from a device
/// // nobody asked, and this is the first of the two.
/// assert_eq!(
///     try_observe_highest_gpu_family::<core::convert::Infallible>(|_| Ok(false)),
///     Ok(MetalGpuFamilySupport::NoneNamed),
/// );
///
/// // A binding that cannot name one enumerator has no family observation. The
/// // error aborts the whole highest-first walk rather than being treated as a
/// // device answer of `false`.
/// assert_eq!(
///     try_observe_highest_gpu_family(|family| {
///         (family.value() != 1009).then_some(false).ok_or(family)
///     }),
///     Err(MetalGpuFamily::Apple9.apple_constant()),
/// );
/// ```
///
/// # Errors
///
/// Returns the first error produced by `supports_family` and asks no lower
/// family afterward. One failed query invalidates the entire highest-family
/// observation: continuing could report a lower family as though every higher
/// family had answered `false`.
pub fn try_observe_highest_gpu_family<E>(
    mut supports_family: impl FnMut(AppleGpuFamilyConstant) -> Result<bool, E>,
) -> Result<MetalGpuFamilySupport, E> {
    for family in MetalGpuFamily::ALL.into_iter().rev() {
        if supports_family(family.apple_constant())? {
            return Ok(MetalGpuFamilySupport::Highest(family));
        }
    }
    Ok(MetalGpuFamilySupport::NoneNamed)
}

/// What a device reported about the Apple GPU families it supports.
///
/// A device that names none of them is a different answer from a device nobody
/// asked, which is why this type exists rather than an `Option`: the first is an
/// observation that refuses, the second is a missing observation that refuses
/// under [`MetalHostApplicabilityRefusal::Unobserved`].
///
/// **An ADR 0074 convention 5b type, deliberately exhaustive.** The answer set
/// is closed by construction — a device either names a highest supported family
/// or names none — so growth belongs to [`MetalGpuFamily`] rather than here.
/// Out-of-crate consumers do map both arms (the adapter in
/// `prototypes/serial-sum-run` reports them), and a wildcard there could only
/// invent a description of an answer this type does not have.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MetalGpuFamilySupport {
    /// The highest named Apple family this device reports supporting.
    ///
    /// Highest rather than a set, because Apple's families are cumulative and
    /// the highest supported one is the most specific true statement a device
    /// makes about itself.
    Highest(MetalGpuFamily),
    /// The device reported none of the families [`MetalGpuFamily`] names.
    NoneNamed,
}

/// One predicate a host-applicability policy evaluates.
///
/// Used to name the unsatisfied predicate in a refusal — both for a missing
/// observation and for the ADR 0086 translation authority, which has no
/// observed value to report at all.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MetalHostPredicate {
    /// The operating-system family the host runs.
    OsFamily,
    /// The operating-system marketing version.
    OsVersion,
    /// The exact operating-system build identifier.
    OsBuild,
    /// The host architecture.
    Architecture,
    /// The exact name the Metal device reports for itself.
    DeviceName,
    /// The Apple GPU family the device reports supporting.
    GpuFamily,
    /// An attributable identity for the component that translates a metallib
    /// during pipeline creation, or exact host attestation binding the
    /// execution environment.
    ///
    /// `Unknown` on every macOS row currently observable, per ADR 0086.
    NativeTranslationAuthority,
}

impl MetalHostPredicate {
    /// Every predicate a policy evaluates, in evaluation order.
    ///
    /// The declared length is `variant_count`, and this list needs that more
    /// than its siblings do rather than less: no code reads it — it is cited by
    /// [`evaluate_metal_host_applicability`] as the stated evaluation order and
    /// by `crate::applicability_tests` as the population its cases enumerate —
    /// so nothing else would notice a predicate left out of it. [`Self::COUNT`]
    /// is what those tests count against, and a short `ALL` would shrink the
    /// expected population to match the cases that were written.
    pub const ALL: [Self; core::mem::variant_count::<Self>()] = [
        Self::OsFamily,
        Self::OsVersion,
        Self::OsBuild,
        Self::Architecture,
        Self::DeviceName,
        Self::GpuFamily,
        Self::NativeTranslationAuthority,
    ];

    /// How many predicates a policy evaluates.
    pub const COUNT: usize = Self::ALL.len();

    /// Returns a stable lowercase identifier for this predicate.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OsFamily => "os-family",
            Self::OsVersion => "os-version",
            Self::OsBuild => "os-build",
            Self::Architecture => "architecture",
            Self::DeviceName => "device-name",
            Self::GpuFamily => "gpu-family",
            Self::NativeTranslationAuthority => "native-translation-authority",
        }
    }
}

impl fmt::Display for MetalHostPredicate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One normalized observation of a live host, stated field by field.
///
/// Starts from [`Self::unobserved`] and gains one field per `observing_*` call,
/// so a field an adapter could not answer stays missing rather than defaulting
/// to a value that would be compared. See this module's documentation for the
/// exact spellings an adapter owes.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct MetalHostObservation {
    os_family: Option<String>,
    os_version: Option<String>,
    os_build: Option<String>,
    architecture: Option<String>,
    device_name: Option<String>,
    gpu_family: Option<MetalGpuFamilySupport>,
}

impl MetalHostObservation {
    /// An observation that states nothing: every predicate is unobserved.
    #[must_use]
    pub const fn unobserved() -> Self {
        Self {
            os_family: None,
            os_version: None,
            os_build: None,
            architecture: None,
            device_name: None,
            gpu_family: None,
        }
    }

    /// Returns this observation with the host's OS family stated.
    #[must_use]
    pub fn observing_os_family(mut self, os_family: impl Into<String>) -> Self {
        self.os_family = Some(os_family.into());
        self
    }

    /// Returns this observation with the host's OS marketing version stated.
    #[must_use]
    pub fn observing_os_version(mut self, os_version: impl Into<String>) -> Self {
        self.os_version = Some(os_version.into());
        self
    }

    /// Returns this observation with the host's exact OS build stated.
    #[must_use]
    pub fn observing_os_build(mut self, os_build: impl Into<String>) -> Self {
        self.os_build = Some(os_build.into());
        self
    }

    /// Returns this observation with the host's architecture stated.
    #[must_use]
    pub fn observing_architecture(mut self, architecture: impl Into<String>) -> Self {
        self.architecture = Some(architecture.into());
        self
    }

    /// Returns this observation with the device's own reported name stated.
    #[must_use]
    pub fn observing_device_name(mut self, device_name: impl Into<String>) -> Self {
        self.device_name = Some(device_name.into());
        self
    }

    /// Returns this observation with the device's Apple family answer stated.
    #[must_use]
    pub fn observing_gpu_family(mut self, support: MetalGpuFamilySupport) -> Self {
        self.gpu_family = Some(support);
        self
    }

    /// Returns the observed OS family, or `None` where nothing observed it.
    #[must_use]
    pub fn os_family(&self) -> Option<&str> {
        self.os_family.as_deref()
    }

    /// Returns the observed OS marketing version, or `None`.
    #[must_use]
    pub fn os_version(&self) -> Option<&str> {
        self.os_version.as_deref()
    }

    /// Returns the observed exact OS build, or `None`.
    #[must_use]
    pub fn os_build(&self) -> Option<&str> {
        self.os_build.as_deref()
    }

    /// Returns the observed architecture, or `None`.
    #[must_use]
    pub fn architecture(&self) -> Option<&str> {
        self.architecture.as_deref()
    }

    /// Returns the device's own reported name, or `None`.
    #[must_use]
    pub fn device_name(&self) -> Option<&str> {
        self.device_name.as_deref()
    }

    /// Returns the device's Apple family answer, or `None`.
    #[must_use]
    pub const fn gpu_family(&self) -> Option<MetalGpuFamilySupport> {
        self.gpu_family
    }
}

/// The exact measured row one host-applicability policy requires.
///
/// A *closed* set in the sense of ADR 0074 convention 2: there is no
/// constructor, so a caller cannot mint a policy for a row nobody measured. The
/// only value is [`Self::FIRST_MACOS_APPLE9`], and widening it to another OS
/// build, Apple family, or device is a new measurement rather than a new
/// argument.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MetalHostApplicabilityPolicy {
    id: &'static str,
    os_family: &'static str,
    os_version: &'static str,
    os_build: &'static str,
    architecture: &'static str,
    device_name: &'static str,
    gpu_family: MetalGpuFamily,
}

impl MetalHostApplicabilityPolicy {
    /// The row the unified MSL 4 Apple9 F32 measurement and the AOT
    /// runtime-compiler observer both ran on.
    ///
    /// See this module's documentation for the two retained records these
    /// values are transcribed from, and for why the deployment minimum in one
    /// record's directory name is not the host OS version.
    pub const FIRST_MACOS_APPLE9: Self = Self {
        id: "tiler.metal.host-applicability.macos-27.0-26A5388g-arm64-m4max-apple9.v1",
        os_family: "macos",
        os_version: "27.0",
        os_build: "26A5388g",
        architecture: "arm64",
        device_name: "Apple M4 Max",
        gpu_family: MetalGpuFamily::Apple9,
    };

    /// Returns the versioned identifier a receipt or refusal is scoped to.
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.id
    }

    /// Returns the OS family this policy requires.
    #[must_use]
    pub const fn os_family(self) -> &'static str {
        self.os_family
    }

    /// Returns the OS marketing version this policy requires.
    #[must_use]
    pub const fn os_version(self) -> &'static str {
        self.os_version
    }

    /// Returns the exact OS build this policy requires.
    #[must_use]
    pub const fn os_build(self) -> &'static str {
        self.os_build
    }

    /// Returns the architecture this policy requires.
    #[must_use]
    pub const fn architecture(self) -> &'static str {
        self.architecture
    }

    /// Returns the exact device name this policy requires.
    #[must_use]
    pub const fn device_name(self) -> &'static str {
        self.device_name
    }

    /// Returns the Apple GPU family this policy requires.
    #[must_use]
    pub const fn gpu_family(self) -> MetalGpuFamily {
        self.gpu_family
    }
}

/// Evidence that ADR 0086's native-translation authority is satisfied.
///
/// # There is no value of this type
///
/// Its one field is a private empty enum, so the type is uninhabited: not
/// merely unconstructed by this crate, but impossible to construct anywhere,
/// including inside this crate. That is the structural form of ADR 0086's
/// finding — the strongest available foreign observation returns loaded-image
/// *membership* rather than attribution, and a negative control produced
/// membership and a readable `metalfe-*` build string without producing
/// attribution — so there is nothing to name and nothing to mint.
///
/// A [`MetalHostEligibility`] holds one of these, which is what makes a
/// positive receipt unreachable by construction rather than by a test nobody
/// re-runs. When one of ADR 0086's reconsideration triggers supplies the
/// missing authority, this type gains an inhabitant and a checked constructor
/// under a superseding decision, and the receipt becomes reachable at exactly
/// that point and no earlier.
///
/// No public path mints one. The field is private, so a struct literal does not
/// compile:
///
/// ```compile_fail,E0451
/// use tiler_metal::applicability::NativeTranslationAuthority;
///
/// fn forge() -> NativeTranslationAuthority {
///     NativeTranslationAuthority { evidence: loop {} }
/// }
/// ```
///
/// And there is no constructor to reach for either:
///
/// ```compile_fail,E0599
/// use tiler_metal::applicability::NativeTranslationAuthority;
///
/// let _absent_constructor = NativeTranslationAuthority::new;
/// ```
#[derive(Debug)]
pub struct NativeTranslationAuthority {
    #[allow(
        dead_code,
        reason = "the field is what makes this type uninhabited; nothing reads it because \
                  no value of it exists"
    )]
    evidence: NoAdmissibleAuthority,
}

/// The empty set of admissible native-translation authorities on current APIs.
///
/// Private and uninhabited on purpose: ADR 0086 item 4 excludes every candidate
/// by name — the source-JIT build, the offline compiler and linker identity, the
/// OS build, the Xcode build, a framework present on disk, loaded-image
/// membership, a membership delta, a readable build string, a hard-coded
/// framework path, and the device registry ID — so there is no variant to write
/// that would not be one of those relabelled.
#[derive(Debug)]
enum NoAdmissibleAuthority {}

/// A checked receipt that one exact host satisfies one exact policy.
///
/// Unreachable while ADR 0086's translation-authority predicate is `Unknown`,
/// structurally: it holds a [`NativeTranslationAuthority`], and no value of that
/// type exists. The type is declared now because the refusal has to name what it
/// is refusing to produce, and because
/// `construct-and-bind-the-first-authoritative-metal-compile-profile` binds
/// against this exact input.
///
/// It carries the versioned policy and the exact observation that satisfied it,
/// and deliberately carries **no** target profile key or descriptor: the parent
/// ticket owns that declaration, and returning one here would make a dependency
/// mint the value its dependent creates.
#[derive(Debug)]
pub struct MetalHostEligibility {
    policy: MetalHostApplicabilityPolicy,
    observation: MetalHostObservation,
    #[allow(
        dead_code,
        reason = "holding the authority is the point; it is unreadable because no value of \
                  it exists, which is what makes this receipt unreachable"
    )]
    authority: NativeTranslationAuthority,
}

impl MetalHostEligibility {
    /// Returns the versioned policy this receipt is scoped to.
    #[must_use]
    pub const fn policy(&self) -> MetalHostApplicabilityPolicy {
        self.policy
    }

    /// Returns the exact normalized observation that satisfied that policy.
    #[must_use]
    pub const fn observation(&self) -> &MetalHostObservation {
        &self.observation
    }
}

/// Why one host did not earn an eligibility receipt.
///
/// One variant per predicate, rather than one variant carrying a predicate tag,
/// because a caller repairs each of these differently: a wrong OS build is a
/// host to update, a wrong device is a host to change, an unobserved field is an
/// adapter to fix, and the translation authority is a decision to reopen.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a later predicate lands
/// additively, and no consumer outside this crate classifies this by exhaustive
/// match.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MetalHostApplicabilityRefusal {
    /// The observation states nothing about a predicate the policy requires.
    ///
    /// Distinct from every mismatch below: nothing was compared, so this is a
    /// gap in the adapter rather than a fact about the host.
    Unobserved {
        /// The predicate no observation answered.
        predicate: MetalHostPredicate,
    },
    /// The host runs an operating-system family this policy was not measured on.
    OsFamilyMismatch {
        /// The OS family the policy requires.
        required: &'static str,
        /// The OS family the host reported.
        observed: String,
    },
    /// The host runs an OS version this policy was not measured on.
    OsVersionMismatch {
        /// The OS marketing version the policy requires.
        required: &'static str,
        /// The OS marketing version the host reported.
        observed: String,
    },
    /// The host runs an OS build this policy was not measured on.
    ///
    /// Separate from [`Self::OsVersionMismatch`] because the version can agree
    /// while the build moves underneath it, and the retained measurement is
    /// scoped to the build.
    OsBuildMismatch {
        /// The exact OS build the policy requires.
        required: &'static str,
        /// The exact OS build the host reported.
        observed: String,
    },
    /// The host runs an architecture this policy was not measured on.
    ArchitectureMismatch {
        /// The architecture the policy requires.
        required: &'static str,
        /// The architecture the host reported.
        observed: String,
    },
    /// The device reports a name this policy was not measured on.
    DeviceNameMismatch {
        /// The exact device name the policy requires.
        required: &'static str,
        /// The exact device name the device reported.
        observed: String,
    },
    /// The device does not report the exact Apple family this policy requires.
    ///
    /// Exact rather than "at least": the policy's whole validity is the measured
    /// row, and admitting a higher family would extend a bounded measurement to
    /// hardware nobody ran it on — which is the substitution ADR 0086 item 3
    /// forbids for the row as a whole.
    GpuFamilyMismatch {
        /// The Apple family the policy requires.
        required: MetalGpuFamily,
        /// What the device reported instead.
        observed: MetalGpuFamilySupport,
    },
    /// Every environment predicate matched and the translation authority did not
    /// exist.
    ///
    /// This is the refusal ADR 0086 decides for every macOS row currently
    /// observable. It reports no observed value because there is none to report:
    /// the predicate is `Unknown`, which is neither proved nor disproved, and
    /// ADR 0043's disposal of `Unknown` is what keeps the candidate out of an
    /// executable frontier.
    UnknownNativeTranslationAuthority {
        /// The versioned policy whose authority could not be established.
        policy: &'static str,
    },
}

impl MetalHostApplicabilityRefusal {
    /// Returns the exact predicate this refusal names.
    #[must_use]
    pub const fn predicate(&self) -> MetalHostPredicate {
        match self {
            Self::Unobserved { predicate } => *predicate,
            Self::OsFamilyMismatch { .. } => MetalHostPredicate::OsFamily,
            Self::OsVersionMismatch { .. } => MetalHostPredicate::OsVersion,
            Self::OsBuildMismatch { .. } => MetalHostPredicate::OsBuild,
            Self::ArchitectureMismatch { .. } => MetalHostPredicate::Architecture,
            Self::DeviceNameMismatch { .. } => MetalHostPredicate::DeviceName,
            Self::GpuFamilyMismatch { .. } => MetalHostPredicate::GpuFamily,
            Self::UnknownNativeTranslationAuthority { .. } => {
                MetalHostPredicate::NativeTranslationAuthority
            }
        }
    }

    /// Returns the stable rule identifier for this refusal.
    ///
    /// Two rules over seven variants, because a caller routing on the text needs
    /// the `Unknown` disposal told apart from a disproved predicate; which
    /// predicate it was is [`Self::predicate`].
    #[must_use]
    pub const fn rule(&self) -> &'static str {
        match self {
            Self::Unobserved { .. } => "metal.host-applicability.unobserved-predicate",
            Self::OsFamilyMismatch { .. }
            | Self::OsVersionMismatch { .. }
            | Self::OsBuildMismatch { .. }
            | Self::ArchitectureMismatch { .. }
            | Self::DeviceNameMismatch { .. }
            | Self::GpuFamilyMismatch { .. } => "metal.host-applicability.outside-measured-row",
            Self::UnknownNativeTranslationAuthority { .. } => {
                "metal.host-applicability.unknown-translation-authority"
            }
        }
    }
}

impl fmt::Display for MetalHostApplicabilityRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unobserved { predicate } => {
                write!(formatter, "{}: {predicate}", self.rule())
            }
            Self::OsFamilyMismatch { required, observed }
            | Self::OsVersionMismatch { required, observed }
            | Self::OsBuildMismatch { required, observed }
            | Self::ArchitectureMismatch { required, observed }
            | Self::DeviceNameMismatch { required, observed } => write!(
                formatter,
                "{}: {} requires {required:?} and this host reports {observed:?}",
                self.rule(),
                self.predicate(),
            ),
            Self::GpuFamilyMismatch { required, observed } => write!(
                formatter,
                "{}: {} requires {required} and this device reports {}",
                self.rule(),
                self.predicate(),
                match observed {
                    MetalGpuFamilySupport::Highest(family) => family.as_str(),
                    MetalGpuFamilySupport::NoneNamed => "no named Apple family",
                },
            ),
            Self::UnknownNativeTranslationAuthority { policy } => write!(
                formatter,
                "{}: {} is unknown for {policy}; ADR 0086 requires an attributable private \
                 native-translation identity or exact host attestation, and neither exists on \
                 current APIs",
                self.rule(),
                self.predicate(),
            ),
        }
    }
}

impl Error for MetalHostApplicabilityRefusal {}

/// Decides whether one observed host satisfies one measured applicability row.
///
/// Deterministic and pure: the same policy and observation always reach the same
/// answer, and nothing here reads a device, a process, an environment variable,
/// or an artifact.
///
/// # Order of refusal
///
/// The environment predicates are evaluated in [`MetalHostPredicate::ALL`]
/// order, each reporting a missing observation before a mismatch, and the
/// translation authority is evaluated last. Last rather than first, deliberately:
/// a host that is also on the wrong OS build should be told the thing it can act
/// on, and the ADR 0086 refusal is what an otherwise-matching host receives.
///
/// # Errors
///
/// Always, on every host observable today. A matching observation returns
/// [`MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority`] naming
/// [`MetalHostPredicate::NativeTranslationAuthority`]; every other observation
/// returns the first predicate it failed.
///
/// ```
/// use tiler_metal::applicability::{
///     MetalGpuFamily, MetalGpuFamilySupport, MetalHostApplicabilityPolicy,
///     MetalHostApplicabilityRefusal, MetalHostObservation, MetalHostPredicate,
///     evaluate_metal_host_applicability,
/// };
///
/// let policy = MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9;
/// let measured = MetalHostObservation::unobserved()
///     .observing_os_family("macos")
///     .observing_os_version("27.0")
///     .observing_os_build("26A5388g")
///     .observing_architecture("arm64")
///     .observing_device_name("Apple M4 Max")
///     .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9));
///
/// // Every public environment predicate matches the retained row, and the
/// // receipt is still refused, naming the predicate ADR 0086 left `Unknown`.
/// let refusal = evaluate_metal_host_applicability(policy, &measured)
///     .expect_err("ADR 0086 refuses every current host");
/// assert_eq!(
///     refusal,
///     MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority { policy: policy.id() },
/// );
/// assert_eq!(refusal.predicate(), MetalHostPredicate::NativeTranslationAuthority);
///
/// // A host outside the measured row is told which predicate put it there.
/// let other_build = measured.clone().observing_os_build("26A5389x");
/// assert_eq!(
///     evaluate_metal_host_applicability(policy, &other_build)
///         .expect_err("a different OS build is outside the measured row")
///         .predicate(),
///     MetalHostPredicate::OsBuild,
/// );
/// ```
///
/// # The producer's own declaration is not an admissible argument
///
/// Reading a profile reference from an artifact, or rebuilding one from a local
/// compilation, and then calling that host eligibility is a tautology: the host
/// would earn eligibility from the declaration it is supposed to be checked
/// against. The second parameter is a [`MetalHostObservation`] and nothing else,
/// so neither route compiles.
///
/// A declared target profile is not an observation of a host (`E0308`):
///
/// ```compile_fail,E0308
/// use tiler_artifact::program::TargetProfileRef;
/// use tiler_metal::applicability::{
///     MetalHostApplicabilityPolicy, evaluate_metal_host_applicability,
/// };
///
/// fn eligibility_from_a_declared_profile(declared: &TargetProfileRef) {
///     let _ = evaluate_metal_host_applicability(
///         MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9,
///         declared,
///     );
/// }
/// ```
///
/// Neither are the artifact's own bytes (`E0308`):
///
/// ```compile_fail,E0308
/// use tiler_metal::applicability::{
///     MetalHostApplicabilityPolicy, evaluate_metal_host_applicability,
/// };
///
/// fn eligibility_from_artifact_bytes(encoded: &[u8]) {
///     let _ = evaluate_metal_host_applicability(
///         MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9,
///         encoded,
///     );
/// }
/// ```
pub fn evaluate_metal_host_applicability(
    policy: MetalHostApplicabilityPolicy,
    observation: &MetalHostObservation,
) -> Result<MetalHostEligibility, MetalHostApplicabilityRefusal> {
    let os_family = observed(observation.os_family(), MetalHostPredicate::OsFamily)?;
    if os_family != policy.os_family {
        return Err(MetalHostApplicabilityRefusal::OsFamilyMismatch {
            required: policy.os_family,
            observed: os_family.to_owned(),
        });
    }
    let os_version = observed(observation.os_version(), MetalHostPredicate::OsVersion)?;
    if os_version != policy.os_version {
        return Err(MetalHostApplicabilityRefusal::OsVersionMismatch {
            required: policy.os_version,
            observed: os_version.to_owned(),
        });
    }
    let os_build = observed(observation.os_build(), MetalHostPredicate::OsBuild)?;
    if os_build != policy.os_build {
        return Err(MetalHostApplicabilityRefusal::OsBuildMismatch {
            required: policy.os_build,
            observed: os_build.to_owned(),
        });
    }
    let architecture = observed(observation.architecture(), MetalHostPredicate::Architecture)?;
    if architecture != policy.architecture {
        return Err(MetalHostApplicabilityRefusal::ArchitectureMismatch {
            required: policy.architecture,
            observed: architecture.to_owned(),
        });
    }
    let device_name = observed(observation.device_name(), MetalHostPredicate::DeviceName)?;
    if device_name != policy.device_name {
        return Err(MetalHostApplicabilityRefusal::DeviceNameMismatch {
            required: policy.device_name,
            observed: device_name.to_owned(),
        });
    }
    let gpu_family = observation
        .gpu_family()
        .ok_or(MetalHostApplicabilityRefusal::Unobserved {
            predicate: MetalHostPredicate::GpuFamily,
        })?;
    if gpu_family != MetalGpuFamilySupport::Highest(policy.gpu_family) {
        return Err(MetalHostApplicabilityRefusal::GpuFamilyMismatch {
            required: policy.gpu_family,
            observed: gpu_family,
        });
    }

    // The whole measured row matched, and that is a validity scope rather than
    // an authority. ADR 0086 item 3: an opaque translator can change while the
    // observed OS/build/device row stays identical, so the row cannot stand in
    // for the fact that a component was named.
    let authority = native_translation_authority().ok_or(
        MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority { policy: policy.id },
    )?;
    Ok(MetalHostEligibility {
        policy,
        observation: observation.clone(),
        authority,
    })
}

/// Returns one observed field, or the refusal naming the predicate it answers.
fn observed(
    value: Option<&str>,
    predicate: MetalHostPredicate,
) -> Result<&str, MetalHostApplicabilityRefusal> {
    value.ok_or(MetalHostApplicabilityRefusal::Unobserved { predicate })
}

/// Returns the authority ADR 0086 requires, if any exists on current APIs.
///
/// It never does, and it cannot: [`NativeTranslationAuthority`] is uninhabited,
/// so `None` is the only value this function could return no matter what it
/// tried to do. This is the single site an accepted superseding decision would
/// change, and changing it alone would not be enough — the authority type would
/// have to gain an inhabitant first, which is the point.
const fn native_translation_authority() -> Option<NativeTranslationAuthority> {
    None
}

#[cfg(test)]
mod structural_unreachability {
    use super::{MetalHostApplicabilityRefusal, MetalHostEligibility};

    /// Compiles only while a positive receipt is impossible to construct.
    ///
    /// The `match` has no `Ok` arm and is still exhaustive, because
    /// [`MetalHostEligibility`](super::MetalHostEligibility) holds an
    /// uninhabited authority and this module can see that it does. Inhabiting
    /// the authority type — which is what an accepted superseding decision under
    /// ADR 0086 would do — makes this function stop compiling, so the structural
    /// claim cannot decay quietly into "no test happens to reach it".
    ///
    /// Kept beside the policy rather than in `crate::applicability_tests`
    /// deliberately: uninhabitedness is only visible where the private empty
    /// enum is, and a sibling module would see an opaque struct and demand the
    /// `Ok` arm this proof is about.
    fn every_outcome_is_a_refusal(
        outcome: Result<MetalHostEligibility, MetalHostApplicabilityRefusal>,
    ) -> MetalHostApplicabilityRefusal {
        match outcome {
            Err(refusal) => refusal,
        }
    }

    #[test]
    fn a_receipt_has_no_arm_to_match() {
        let refusal = MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority {
            policy: super::MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9.id(),
        };
        assert_eq!(
            every_outcome_is_a_refusal(Err(refusal.clone())),
            refusal,
            "the only inhabited variant is the refusal",
        );
    }
}
