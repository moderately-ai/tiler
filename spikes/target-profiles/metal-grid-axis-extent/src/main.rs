//! Measures the grid-axis thread extent this Apple9 macOS row dispatches correctly.
//!
//! `establish-an-upper-bound-authority-for-the-metal-grid-axis-row` has to
//! replace a compile-guarantee number that no authority states. The row it
//! replaces reads four, and its own comment records why: the macOS SDK's
//! `dispatchThreads:threadsPerThreadgroup:` contract proves an extent is
//! *representable* and states no maximum at all, so four was chosen to cover the
//! bounded serial-sum program rather than derived from anything.
//!
//! # What this spike can and cannot establish
//!
//! `CapabilityAxis::GridAxisThreads` is consumed as a **guarantee**: a plan is
//! feasible when its required extent is no greater than the declared bound. So
//! the row needs an authority that says *extents up to N work*, which is a lower
//! bound on capability. The normative sources go the other way — they cap the
//! space without licensing any value inside it — so only a measurement can
//! supply the number, and it supplies it for exactly one environment.
//!
//! This spike therefore reports what a bounded ladder of extents did on one
//! host under one toolchain. It is not a portable guarantee, it does not
//! establish that the next extent up fails, and it makes no performance claim of
//! any kind: nothing here is timed.
//!
//! # Design
//!
//! One invocation per grid point writes `tid ^ salt` into its own slot of a
//! buffer poisoned before every dispatch, so "the invocation did not run" and
//! "the invocation wrote its own value" are distinguishable, and the salt
//! arrives at dispatch time so no fill the host could have performed reproduces
//! the expected pattern. Every slot is then checked, not sampled: the claim is
//! about the whole grid.
//!
//! Three things are held to the profile's own choices rather than to whatever is
//! convenient. The kernel is compiled offline for `air64-apple-macos26.0` under
//! `-std=metal4.0`, which is the compilation the authority ledger's rows are
//! scoped to. It declares `uint tid [[thread_position_in_grid]]`, which is the
//! launch-index realization the profile selects. And it is dispatched through
//! `dispatchThreads:threadsPerThreadgroup:` with an `MTLSize`, which is the
//! route `prototypes/serial-sum-run` encodes.
//!
//! Every extent is run at three threadgroup widths, because the tail case — a
//! grid extent that is not a multiple of the threadgroup width — is the one
//! place a wide grid could plausibly go wrong while a narrow one does not, and
//! `dispatchThreads:` is precisely the entry point that admits it.
//!
//! # Running it
//!
//! From this directory, with the toolchain the authority ledger records:
//!
//! ```sh
//! DEVELOPER_DIR=/Applications/Xcode.app cargo run --release > results/<date>-<host>/extent.tsv
//! ```
//!
//! `DEVELOPER_DIR` selects the offline toolchain for this invocation only; it
//! mutates nothing. The harness records whatever toolchain answered, so a run
//! under a different one is self-describing rather than silently mislabelled.
//!
//! No `make` target reaches here, per `spikes/README.md`.

mod readback;

use std::ffi::c_void;
use std::process::Command;

use metal::{
    Buffer, CommandBufferRef, ComputePipelineDescriptor, ComputePipelineState, Device,
    MTLCommandBufferStatus, MTLGPUFamily, MTLResourceOptions, MTLSize,
};

/// The value every slot holds before a dispatch, and must not hold after one.
///
/// Any bit pattern would do provided it is not a value the kernel can write.
/// `tid ^ SALT` reaches this only when `tid == POISON ^ SALT`, which is far
/// above every extent this ladder runs, and the run asserts that.
const POISON: u32 = 0xDEAD_BEEF;

/// Mixed into every written value so no host-side fill can forge the result.
const SALT: u32 = 0x9E37_79B9;

/// The largest extent the ladder runs, and therefore the largest value this
/// spike can support declaring.
///
/// **This is the experiment's stop condition, and it is a design choice rather
/// than an observed limit** — every rung below it passed, so nothing here says
/// the next one would fail. Two considerations set it. MSL 4.0 Table 5.8 types
/// `[[thread_position_in_grid]]` as `ushort` or `uint` and offers nothing wider,
/// so `2^32` is a hard ceiling on what any kernel in this language can address;
/// and complete verification costs four bytes of device memory per thread, so
/// the run has to fit a buffer it can also check slot by slot.
///
/// `2^28` sits sixteen times below the language ceiling and covers the widest
/// single tensor in the corpus this project measures against — Qwen3-0.6B's
/// embedding matrix is 151,936 x 1,024, about `2^27.2` elements — so the
/// declared row does not refuse a plan for a tensor the project already handles.
/// The verification buffer is 1 GiB against this device's reported
/// `maxBufferLength` of roughly 22.6 GB.
const MAX_EXTENT: u64 = 1 << 28;

/// Elements verified per host chunk.
///
/// Verification reads the device buffer back in fixed-size pieces so that
/// checking the widest rung costs 16 MiB of host memory rather than a second
/// copy of the whole gigabyte-scale buffer. It changes the cost of the check,
/// never its completeness: every slot is still compared.
const VERIFY_CHUNK: usize = 1 << 22;

/// Extents below which every single extent is run, not just a sampled ladder.
///
/// Under this bound the evidence is exhaustive over the integers rather than
/// sampled, which is what makes the small end of the declared guarantee — the
/// end every current Tiler plan actually lands in — a complete finite check
/// rather than an interpolation between rungs.
const EXHAUSTIVE_BELOW: u64 = 2_049;

fn main() {
    let device = Device::system_default().expect("this host exposes a default Metal device");
    let toolchain = Toolchain::observe();
    let object = compile_probe();
    let pipeline = pipeline_for(&device, &object);
    let queue = device.new_command_queue();

    let widths = threadgroup_widths(&pipeline);
    let extents = ladder();
    let largest = *extents.last().expect("the ladder is nonempty");
    assert!(
        POISON ^ SALT > u32::try_from(largest).expect("the ladder stays inside u32"),
        "the poison pattern is reachable by a thread this ladder launches",
    );

    let buffer = device.new_buffer(largest * 4, MTLResourceOptions::StorageModeShared);
    let salt = device.new_buffer_with_data(
        std::ptr::from_ref(&SALT).cast::<c_void>(),
        4,
        MTLResourceOptions::StorageModeShared,
    );

    print_environment(&device, &toolchain, &pipeline, largest);
    println!("extent\tthreadgroup_width\tstatus\tverified_slots\tfirst_mismatch\tobserved");

    let mut ceiling = u64::MAX;
    for width in widths {
        let mut widest_verified = 0;
        for extent in &extents {
            let row = run_extent(&device, &queue, &pipeline, &buffer, &salt, *extent, width);
            println!("{row}");
            if row.verified() {
                widest_verified = *extent;
            } else {
                break;
            }
        }
        ceiling = ceiling.min(widest_verified);
    }

    println!();
    println!("# widest extent verified at every tested threadgroup width\t{ceiling}");
    println!(
        "# exhaustive below\t{EXHAUSTIVE_BELOW}\t# sampled above, so the guarantee between rungs \
         is an interpolation and is stated as one"
    );
}

/// One dispatched rung and what it observed.
struct Row {
    extent: u64,
    width: u64,
    status: &'static str,
    verified_slots: u64,
    first_mismatch: Option<u64>,
    observed: Option<u32>,
}

impl Row {
    const fn verified(&self) -> bool {
        self.first_mismatch.is_none() && self.verified_slots == self.extent
    }
}

impl std::fmt::Display for Row {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}\t{}\t{}\t{}\t{}\t{}",
            self.extent,
            self.width,
            self.status,
            self.verified_slots,
            self.first_mismatch
                .map_or_else(|| "none".to_owned(), |index| index.to_string()),
            self.observed
                .map_or_else(|| "none".to_owned(), |value| format!("{value:08x}")),
        )
    }
}

/// Poisons, dispatches, waits for a terminal status, and checks every slot.
///
/// The command buffer's terminal state is inspected **before** anything is read
/// back, and the only accepted state is `Completed`. A failed submission leaves
/// the buffer holding the poison, and reporting that as a verification failure
/// rather than as a dispatch failure would attribute the wrong cause.
fn run_extent(
    device: &Device,
    queue: &metal::CommandQueue,
    pipeline: &ComputePipelineState,
    buffer: &Buffer,
    salt: &Buffer,
    extent: u64,
    width: u64,
) -> Row {
    let count = usize::try_from(extent).expect("an extent fits a usize on this host");
    readback::poison(buffer, count, POISON);
    let _ = device;

    let command_buffer = queue.new_command_buffer();
    let encoder = command_buffer.new_compute_command_encoder();
    encoder.set_compute_pipeline_state(pipeline);
    encoder.set_buffer(0, Some(buffer), 0);
    encoder.set_buffer(1, Some(salt), 0);
    encoder.dispatch_threads(MTLSize::new(extent, 1, 1), MTLSize::new(width, 1, 1));
    encoder.end_encoding();
    command_buffer.commit();
    command_buffer.wait_until_completed();

    let status = terminal_status(command_buffer);
    if status != "completed" {
        return Row {
            extent,
            width,
            status,
            verified_slots: 0,
            first_mismatch: None,
            observed: None,
        };
    }

    let mut chunk = vec![0_u32; VERIFY_CHUNK.min(count)];
    let mut base = 0_usize;
    while base < count {
        let span = VERIFY_CHUNK.min(count - base);
        readback::read_u32_into(buffer, base, &mut chunk[..span]);
        for (offset, value) in chunk[..span].iter().enumerate() {
            let index = base + offset;
            let expected = u32::try_from(index).expect("the ladder stays inside u32") ^ SALT;
            if *value != expected {
                let index = u64::try_from(index).expect("an index fits a u64");
                return Row {
                    extent,
                    width,
                    status,
                    verified_slots: index,
                    first_mismatch: Some(index),
                    observed: Some(*value),
                };
            }
        }
        base += span;
    }
    Row {
        extent,
        width,
        status,
        verified_slots: extent,
        first_mismatch: None,
        observed: None,
    }
}

/// Names the command buffer's terminal state, never assuming success.
fn terminal_status(command_buffer: &CommandBufferRef) -> &'static str {
    match command_buffer.status() {
        MTLCommandBufferStatus::Completed => "completed",
        MTLCommandBufferStatus::Error => "error",
        MTLCommandBufferStatus::NotEnqueued => "not-enqueued",
        MTLCommandBufferStatus::Enqueued => "enqueued",
        MTLCommandBufferStatus::Committed => "committed",
        MTLCommandBufferStatus::Scheduled => "scheduled",
    }
}

/// The extents this run dispatches, ascending and deduplicated.
///
/// Exhaustive below [`EXHAUSTIVE_BELOW`], then powers of two with both
/// neighbours of each. The neighbours are the point of the sampled half: a
/// power-of-two-only ladder would never present a grid extent that is not a
/// multiple of any tested threadgroup width, which is exactly the case
/// `dispatchThreads:` exists to admit and therefore the one worth checking.
fn ladder() -> Vec<u64> {
    let mut extents: Vec<u64> = (1..EXHAUSTIVE_BELOW.min(MAX_EXTENT + 1)).collect();
    let mut power = EXHAUSTIVE_BELOW.next_power_of_two();
    while power <= MAX_EXTENT {
        extents.push(power - 1);
        extents.push(power);
        if power < MAX_EXTENT {
            extents.push(power + 1);
        }
        power *= 2;
    }
    extents.sort_unstable();
    extents.dedup();
    extents
}

/// The threadgroup widths every extent is run at.
///
/// One is the width every current Tiler independent-invocation region declares,
/// the SIMD execution width is the shape a realistic kernel uses, and the
/// pipeline's own reported maximum is the widest this function can legally ask
/// for. All three are read from the prepared pipeline rather than from the
/// feature tables, because a prepared function's capacity is the only number
/// that binds a dispatch — the same distinction the authority ledger's
/// workgroup row is a deferred query for.
fn threadgroup_widths(pipeline: &ComputePipelineState) -> Vec<u64> {
    let mut widths = vec![
        1,
        pipeline.thread_execution_width(),
        pipeline.max_total_threads_per_threadgroup(),
    ];
    widths.sort_unstable();
    widths.dedup();
    widths
}

/// Compiles the probe offline under the profile's own flags.
///
/// Offline rather than through `newLibraryWithSource:`, because Tiler's AOT
/// route supplies no source and ADR 0086 excludes the runtime compiler by name
/// from anything this profile is built from.
fn compile_probe() -> Vec<u8> {
    let directory = std::env::temp_dir().join(format!(
        "tiler-grid-axis-extent-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("the scratch directory is creatable");
    let air = directory.join("probe.air");
    let metallib = directory.join("probe.metallib");
    run(
        "xcrun",
        &[
            "--sdk",
            "macosx",
            "metal",
            "-std=metal4.0",
            "-target",
            "air64-apple-macos26.0",
            "-c",
            "probe.metal",
            "-o",
            air.to_str().expect("a UTF-8 scratch path"),
        ],
    );
    run(
        "xcrun",
        &[
            "--sdk",
            "macosx",
            "metallib",
            air.to_str().expect("a UTF-8 scratch path"),
            "-o",
            metallib.to_str().expect("a UTF-8 scratch path"),
        ],
    );
    let object = std::fs::read(&metallib).expect("the linked object is readable");
    std::fs::remove_dir_all(&directory).expect("the scratch directory is removable");
    object
}

/// Builds the compute pipeline for the probe's single entry point.
fn pipeline_for(device: &Device, object: &[u8]) -> ComputePipelineState {
    let library = device
        .new_library_with_data(object)
        .expect("the offline object loads");
    let function = library
        .get_function("grid_extent_probe", None)
        .expect("the probe entry point is present");
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    device
        .new_compute_pipeline_state(&descriptor)
        .expect("the probe prepares")
}

/// The exact toolchain and platform components this run observed.
struct Toolchain {
    metal: String,
    linker: String,
    xcode: String,
    sdk_version: String,
    sdk_build: String,
    platform_version: String,
    platform_build: String,
    architecture: String,
}

impl Toolchain {
    /// Reads every component from the tool that owns it.
    ///
    /// Nothing here is defaulted or inferred: a component whose command fails
    /// aborts the run, because a measurement whose environment is half-known
    /// cannot source a profile row.
    fn observe() -> Self {
        Self {
            metal: first_line(&run("xcrun", &["--sdk", "macosx", "metal", "--version"])),
            linker: first_line(&run("xcrun", &["--sdk", "macosx", "metallib", "-version"])),
            xcode: run("xcodebuild", &["-version"]).replace('\n', " ").trim().to_owned(),
            sdk_version: first_line(&run("xcrun", &["--sdk", "macosx", "--show-sdk-version"])),
            sdk_build: first_line(&run(
                "xcrun",
                &["--sdk", "macosx", "--show-sdk-build-version"],
            )),
            platform_version: first_line(&run("sw_vers", &["-productVersion"])),
            platform_build: first_line(&run("sw_vers", &["-buildVersion"])),
            architecture: first_line(&run("uname", &["-m"])),
        }
    }
}

/// Writes the provenance header every retained row is scoped by.
fn print_environment(
    device: &Device,
    toolchain: &Toolchain,
    pipeline: &ComputePipelineState,
    largest: u64,
) {
    println!("# offline.metal\t{}", toolchain.metal);
    println!("# offline.linker\t{}", toolchain.linker);
    println!("# offline.xcode\t{}", toolchain.xcode);
    println!(
        "# offline.sdk\t{} ({})",
        toolchain.sdk_version, toolchain.sdk_build
    );
    println!("# offline.std\t-std=metal4.0");
    println!("# offline.target\tair64-apple-macos26.0");
    println!(
        "# execution.platform\tmacos {} ({})",
        toolchain.platform_version, toolchain.platform_build
    );
    println!("# execution.architecture\t{}", toolchain.architecture);
    println!("# execution.device\t{}", device.name());
    println!(
        "# execution.apple9\t{}",
        device.supports_family(MTLGPUFamily::Apple9)
    );
    println!("# execution.max_buffer_length\t{}", device.max_buffer_length());
    println!(
        "# prepared.max_total_threads_per_threadgroup\t{}",
        pipeline.max_total_threads_per_threadgroup()
    );
    println!(
        "# prepared.thread_execution_width\t{}",
        pipeline.thread_execution_width()
    );
    println!("# ladder.max_extent\t{largest}");
    println!("# verification\tevery slot of a poisoned buffer, never sampled");
}

/// Runs one command, refusing anything but a clean exit.
fn run(program: &str, arguments: &[&str]) -> String {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("{program} {arguments:?} is runnable: {error}"));
    assert!(
        output.status.success(),
        "{program} {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    if text.trim().is_empty() {
        return String::from_utf8_lossy(&output.stderr).into_owned();
    }
    text
}

/// The first nonempty line of a captured output.
fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .to_owned()
}
