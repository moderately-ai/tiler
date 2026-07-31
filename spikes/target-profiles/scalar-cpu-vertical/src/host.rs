//! The live host execution context, and what binding to one proves.
//!
//! # Why a CPU vertical needs this stage at all
//!
//! `tiler-runtime` is device-free by design: it decides everything derivable
//! from bytes plus a *stated* execution environment, and a host that states its
//! environment wrongly gets a wrong answer. Metal's second stage is obvious —
//! `MTLDevice`, a library, a pipeline, a threadgroup capacity — and it is
//! tempting to conclude a CPU backend has no second stage, because the process
//! executing the loader is already the process that will run the kernel.
//!
//! That conclusion is wrong, and the spike exists partly to show it. A scalar
//! CPU realization depends on facts about the running process that no compiled
//! artifact can assert: the address width the host was built for, its byte
//! order, and — the one that decides bits rather than layout — whether this
//! process's floating-point environment preserves subnormals. An artifact
//! declaring `Preserve` executed in a process running with flush-to-zero
//! enabled returns numbers that are close and wrong, which is exactly the
//! failure the numerical contract exists to forbid.
//!
//! So the host binds a context by **measuring** those facts, compares them
//! against what the image and the profile declare, and refuses before the
//! routing commit. Nothing here is derived from `cfg!` alone: the arithmetic
//! answers come from arithmetic this process actually performs.
//!
//! # The measurement boundary
//!
//! Everything below is a fact about *this process on this host at this moment*.
//! A process can change its own floating-point control state after this probe
//! runs, so the binding is evidence about the interval it was taken in, and the
//! spike re-measures per run rather than caching. It is not a portable claim
//! about the target triple, and the profile in `super::profile` states its
//! arithmetic as an external IEEE-754 guarantee precisely so the two evidence
//! classes stay apart.

use std::hint::black_box;

use crate::image::{ImageNumerics, ImageSubnormals};

/// What this process measured about itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostExecutionContext {
    /// Architecture this binary was built for.
    pub arch: &'static str,
    /// Operating system this binary was built for.
    pub os: &'static str,
    /// Width of a host pointer, in bits.
    pub pointer_width_bits: u32,
    /// Byte order of host scalar storage.
    pub endianness: &'static str,
    /// Measured behaviour of a subnormal *operand*, or `None` when the probe
    /// returned a value no declared realization produces.
    pub input_subnormals: Option<ImageSubnormals>,
    /// Measured behaviour of a subnormal *result*, or `None` for the same reason.
    pub result_subnormals: Option<ImageSubnormals>,
}

impl HostExecutionContext {
    /// Binds a live execution context by measuring this process.
    ///
    /// The two arithmetic probes are the substance. `black_box` stands between
    /// the operands and the compiler on both sides of each operation, because a
    /// constant-folded probe measures rustc's compile-time arithmetic — which
    /// always preserves subnormals — rather than the floating-point unit this
    /// dispatch will run on. That would report a preserving host on a machine
    /// running with flush-to-zero enabled, which is the one wrong answer this
    /// stage exists to prevent.
    #[must_use]
    pub fn bind() -> Self {
        Self {
            arch: std::env::consts::ARCH,
            os: std::env::consts::OS,
            pointer_width_bits: usize::BITS,
            endianness: if cfg!(target_endian = "little") {
                "little"
            } else {
                "big"
            },
            input_subnormals: measure_input_subnormals(),
            result_subnormals: measure_result_subnormals(),
        }
    }

    /// Refuses a host that cannot honour what the image and profile declare.
    ///
    /// # Errors
    ///
    /// Returns the first [`HostRefusal`]. Every check here is decidable before
    /// any buffer is written and any dispatch is encoded, which is what keeps
    /// the routing commit that follows free of refusals.
    pub fn admits(
        &self,
        numerics: ImageNumerics,
        declared_address_bits: u8,
        declared_triple: &str,
    ) -> Result<(), HostRefusal> {
        let (arch, rest) = declared_triple
            .split_once('-')
            .ok_or(HostRefusal::UndeclaredTriple)?;
        if arch != self.arch {
            return Err(HostRefusal::ArchitectureMismatch {
                declared: arch.to_owned(),
                host: self.arch,
            });
        }
        // The declared triple's system component is `darwin`; Rust spells the
        // same system `macos`. The mapping is stated rather than assumed equal,
        // so a triple naming another system is refused instead of matching by
        // accident.
        let declared_os = match rest {
            "apple-darwin" => "macos",
            other => other,
        };
        if declared_os != self.os {
            return Err(HostRefusal::SystemMismatch {
                declared: declared_os.to_owned(),
                host: self.os,
            });
        }
        if u32::from(declared_address_bits) != self.pointer_width_bits {
            return Err(HostRefusal::AddressWidthMismatch {
                declared: declared_address_bits,
                host: self.pointer_width_bits,
            });
        }
        if self.endianness != "little" {
            return Err(HostRefusal::UnsupportedByteOrder {
                host: self.endianness,
            });
        }
        for (dimension, declared, measured) in [
            ("input", numerics.input_subnormals, self.input_subnormals),
            ("result", numerics.result_subnormals, self.result_subnormals),
        ] {
            let Some(measured) = measured else {
                return Err(HostRefusal::UnclassifiedArithmetic { dimension });
            };
            if measured != declared {
                return Err(HostRefusal::SubnormalRealizationMismatch {
                    dimension,
                    declared,
                    measured,
                });
            }
        }
        Ok(())
    }
}

/// Why this host cannot carry out a route it was offered.
///
/// Every variant is a *route miss* in `prototypes/serial-sum-run`'s
/// classification — another artifact, built for another declared profile, might
/// run here — and none is a systemic failure or an artifact defect. Keeping
/// that separate matters for the same reason it does on Metal: a host that
/// cannot tell them apart either abandons an artifact that had a working
/// variant or retries work that can never succeed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostRefusal {
    /// The declared triple is not of the form this host can compare against.
    UndeclaredTriple,
    /// The artifact was built for another architecture.
    ArchitectureMismatch {
        /// Architecture component of the declared triple.
        declared: String,
        /// Architecture this binary was built for.
        host: &'static str,
    },
    /// The artifact was built for another operating system.
    SystemMismatch {
        /// System the declared triple names, in Rust's spelling.
        declared: String,
        /// System this binary was built for.
        host: &'static str,
    },
    /// The declared address model is not this host's.
    AddressWidthMismatch {
        /// Declared address width in bits.
        declared: u8,
        /// Host pointer width in bits.
        host: u32,
    },
    /// The scalar image's storage encoding is little-endian only.
    UnsupportedByteOrder {
        /// Byte order this binary was built for.
        host: &'static str,
    },
    /// This process's arithmetic produced a value no declared realization does.
    UnclassifiedArithmetic {
        /// Which dimension the probe was taken for.
        dimension: &'static str,
    },
    /// This process's arithmetic does not deliver the declared subnormal behaviour.
    SubnormalRealizationMismatch {
        /// Which dimension disagreed.
        dimension: &'static str,
        /// What the image declares.
        declared: ImageSubnormals,
        /// What this process measured.
        measured: ImageSubnormals,
    },
}

impl std::fmt::Display for HostRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndeclaredTriple => formatter.write_str(
                "cpu.host.triple: the declared target triple has no architecture component",
            ),
            Self::ArchitectureMismatch { declared, host } => write!(
                formatter,
                "cpu.host.architecture: the artifact declares {declared} and this host is {host}",
            ),
            Self::SystemMismatch { declared, host } => write!(
                formatter,
                "cpu.host.system: the artifact declares {declared} and this host is {host}",
            ),
            Self::AddressWidthMismatch { declared, host } => write!(
                formatter,
                "cpu.host.address-width: the artifact declares {declared}-bit addresses and this \
                 host uses {host}-bit",
            ),
            Self::UnsupportedByteOrder { host } => write!(
                formatter,
                "cpu.host.byte-order: the scalar image encodes storage little-endian and this host \
                 is {host}-endian",
            ),
            Self::UnclassifiedArithmetic { dimension } => write!(
                formatter,
                "cpu.host.arithmetic: the {dimension}-subnormal probe returned a value no \
                 declared realization produces, so this host is not classified",
            ),
            Self::SubnormalRealizationMismatch {
                dimension,
                declared,
                measured,
            } => write!(
                formatter,
                "cpu.host.subnormals: the image declares {dimension} subnormals {} and this \
                 process measured {}",
                declared.as_str(),
                measured.as_str(),
            ),
        }
    }
}

impl std::error::Error for HostRefusal {}

/// Measures what this process does with a subnormal *operand*.
///
/// The least *negative* subnormal multiplied by one. A negative operand is what
/// makes the three realizations fully separable in one observation: preserving
/// returns the operand unchanged, sign-preserving flushing returns negative
/// zero, and always-positive flushing returns positive zero. A positive operand
/// would collapse the two flushing answers into one bit pattern and force this
/// probe to report the weaker of them.
fn measure_input_subnormals() -> Option<ImageSubnormals> {
    let operand = black_box(f32::from_bits(0x8000_0001));
    classify(operand * black_box(1.0_f32), 0x8000_0001)
}

/// Measures what this process does with a subnormal *result*.
///
/// Two least-subnormal units halved, negative for the same separability reason.
/// The exact result is the least negative subnormal, so this observes an
/// operation whose *result* crosses into the subnormal range and reports what
/// came back.
fn measure_result_subnormals() -> Option<ImageSubnormals> {
    let operand = black_box(f32::from_bits(0x8000_0002));
    classify(operand * black_box(0.5_f32), 0x8000_0001)
}

/// Classifies one probe result against the exact value IEEE-754 requires.
///
/// `None` for anything that is neither the exact subnormal nor a zero: there is
/// no realization to report, and inventing the nearest-looking one would let a
/// host whose arithmetic nobody understands execute a contract it was never
/// checked against. The caller refuses.
fn classify(observed: f32, exact_bits: u32) -> Option<ImageSubnormals> {
    match observed.to_bits() {
        bits if bits == exact_bits => Some(ImageSubnormals::Preserve),
        0x8000_0000 => Some(ImageSubnormals::FlushSignedZero),
        0x0000_0000 => Some(ImageSubnormals::FlushPositiveZero),
        _ => None,
    }
}
