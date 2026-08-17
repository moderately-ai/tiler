//! Consumer-owned loading of one pinned Qwen3 checkpoint as F32 program inputs.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Instant;

use safetensors::Dtype;
use safetensors::tensor::Metadata;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tiler::value::{
    AdapterCapability, ResultRequest, StorageScalar, Tensor, TensorAdapter, ValueMetadata,
};

const CHECKPOINT_SHA256: &str = "cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba";
const MANIFEST_SHA256: &str = "7044ad5173ee123d8970f7a8f782fc24b607d19628a3af5b036995109de250ee";
const WIDENED_BYTES_SHA256: &str =
    "d2abe344f7a4e4c0ea79c4a3c524ca851b095d930064e086d980972fe95c8437";
const CHECKPOINT_REVISION: &str = "da87bfb608c14b7cf20ba1ce41287e8de496c0cd";
const CHECKPOINT_BYTES: u64 = 1_192_135_096;
const SAFETENSORS_HEADER_BYTES: u64 = 35_248;
const TENSOR_COUNT: usize = 310;
const WIDEN_CHUNK_BYTES: usize = 1024 * 1024;

/// One dense consumer buffer carrying bytes the F32 program input stores.
#[derive(Debug)]
struct DenseF32 {
    bytes: Vec<u8>,
    extents: Vec<u64>,
    scalar: StorageScalar,
}

/// An adapter over this consumer's dense, host-owned F32 buffers.
struct CheckpointAdapter;

#[derive(Debug, Eq, PartialEq)]
struct AdapterError(&'static str);

impl std::fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for AdapterError {}

impl TensorAdapter for CheckpointAdapter {
    type Value = DenseF32;
    type Context = ();
    type Error = AdapterError;

    fn supports(capability: AdapterCapability) -> bool {
        match capability {
            AdapterCapability::DenseRowMajorStorage => true,
            AdapterCapability::ResultConstruction => false,
        }
    }

    fn metadata(value: &DenseF32) -> Result<ValueMetadata, AdapterError> {
        Ok(ValueMetadata::new(
            value.scalar,
            value.extents.iter().copied(),
        ))
    }

    fn build((): &(), _: &ResultRequest<'_>) -> Result<DenseF32, AdapterError> {
        Err(AdapterError(
            "this read-only checkpoint adapter builds no results",
        ))
    }
}

#[derive(Debug)]
struct ProgramInput {
    checkpoint_tensor: String,
    qualified_slot: String,
    interface_key: String,
    tensor: Tensor<CheckpointAdapter>,
}

#[derive(Debug)]
struct LoadedCheckpoint {
    inputs: Vec<ProgramInput>,
    widened_sha256: String,
    census: Census,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Census {
    nan: u64,
    infinite: u64,
    subnormal: u64,
}

#[derive(Deserialize)]
struct Manifest {
    schema: String,
    checkpoint: ManifestCheckpoint,
    bindings: Vec<ManifestBinding>,
}

#[derive(Deserialize)]
struct ManifestCheckpoint {
    bytes: u64,
    dtype: String,
    revision: String,
    safetensors_header_bytes: u64,
    sha256: String,
    tensor_count: usize,
}

#[derive(Clone, Deserialize)]
struct ManifestBinding {
    checkpoint_tensor: String,
    expected_shape: Vec<u64>,
    expected_storage_scalar: String,
    interface_key: String,
    qualified_slot: String,
}

fn main() -> std::process::ExitCode {
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: cargo run --release -- <external-path-to-model.safetensors>");
        return std::process::ExitCode::FAILURE;
    };

    let started = Instant::now();
    match load_checkpoint(&path) {
        Ok(loaded) => {
            let resident_bytes = resident_bytes_at_completion()
                .map_or_else(|| "unavailable".to_owned(), |value| value.to_string());
            println!("checkpoint tensors: {}", loaded.inputs.len());
            let widened_bytes: usize = loaded
                .inputs
                .iter()
                .map(|input| input.tensor.value().bytes.len())
                .sum();
            let first_slot = loaded
                .inputs
                .first()
                .map_or("none", |input| input.qualified_slot.as_str());
            let last_tensor = loaded
                .inputs
                .last()
                .map_or("none", |input| input.checkpoint_tensor.as_str());
            let interface_keys: BTreeSet<_> = loaded
                .inputs
                .iter()
                .map(|input| input.interface_key.as_str())
                .collect();
            println!("retained F32 bytes: {widened_bytes}");
            println!("first qualified slot: {first_slot}; final checkpoint tensor: {last_tensor}");
            println!("distinct bare interface keys: {}", interface_keys.len());
            println!("widened bytes sha256: {}", loaded.widened_sha256);
            println!(
                "widened census: nan={} infinite={} subnormal={}",
                loaded.census.nan, loaded.census.infinite, loaded.census.subnormal
            );
            println!("elapsed milliseconds: {}", started.elapsed().as_millis());
            println!("resident bytes at retained-load completion: {resident_bytes}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("CHECKPOINT INPUT STOP: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn load_checkpoint(path: &Path) -> Result<LoadedCheckpoint, String> {
    let manifest = read_and_verify_manifest()?;
    let source_digest = sha256_file(path)?;
    require_digest("checkpoint", &source_digest, CHECKPOINT_SHA256)?;

    let (data_start, metadata) = read_metadata(path)?;
    verify_manifest_against_header(&manifest, &metadata, data_start, path)?;

    let mut file = File::open(path).map_err(|error| format!("opening checkpoint: {error}"))?;
    let mut digest = Sha256::new();
    let mut census = Census::default();
    let mut pending = Vec::with_capacity(manifest.bindings.len());

    for binding in &manifest.bindings {
        let info = metadata.info(&binding.checkpoint_tensor).ok_or_else(|| {
            format!(
                "header has no tensor `{}` after manifest validation",
                binding.checkpoint_tensor
            )
        })?;
        let start = u64::try_from(info.data_offsets.0).map_err(|_| "tensor offset exceeds u64")?;
        let end = u64::try_from(info.data_offsets.1).map_err(|_| "tensor offset exceeds u64")?;
        let bytes = widen_tensor(
            &mut file,
            data_start + start,
            end - start,
            &mut digest,
            &mut census,
        )?;
        pending.push((binding, bytes));
    }

    refuse_nonfinite(census)?;
    let widened_sha256 = hex_digest(digest.finalize());
    require_digest("widened payload", &widened_sha256, WIDENED_BYTES_SHA256)?;

    let inputs = pending
        .into_iter()
        .map(|(binding, bytes)| ProgramInput {
            checkpoint_tensor: binding.checkpoint_tensor.clone(),
            qualified_slot: binding.qualified_slot.clone(),
            interface_key: binding.interface_key.clone(),
            tensor: Tensor::new(
                DenseF32 {
                    bytes,
                    extents: binding.expected_shape.clone(),
                    scalar: StorageScalar::F32,
                },
                (),
            ),
        })
        .collect();

    Ok(LoadedCheckpoint {
        inputs,
        widened_sha256,
        census,
    })
}

fn read_and_verify_manifest() -> Result<Manifest, String> {
    let directory = manifest_directory();
    let manifest_path = directory.join("manifest.json");
    let digest_path = directory.join("manifest.sha256");
    let bytes =
        std::fs::read(&manifest_path).map_err(|error| format!("reading manifest: {error}"))?;
    let actual = hex_digest(Sha256::digest(&bytes));
    require_digest("manifest", &actual, MANIFEST_SHA256)?;
    let declared = std::fs::read_to_string(digest_path)
        .map_err(|error| format!("reading manifest digest: {error}"))?
        .split_whitespace()
        .next()
        .ok_or_else(|| "manifest digest file is empty".to_owned())?
        .to_owned();
    require_digest("manifest digest file", &declared, MANIFEST_SHA256)?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|error| format!("parsing manifest: {error}"))?;
    validate_manifest_ownership(&manifest)?;
    Ok(manifest)
}

fn manifest_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../qwen3-conformance-fixture/results/2026-08-17-qwen3-0.6b-base-da87bfb6-weight-bindings",
    )
}

fn validate_manifest_ownership(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != "tiler-research/qwen3-weight-binding-manifest/v1" {
        return Err(format!("unexpected manifest schema `{}`", manifest.schema));
    }
    if manifest.bindings.len() != TENSOR_COUNT || manifest.checkpoint.tensor_count != TENSOR_COUNT {
        return Err(format!("manifest tensor count is not {TENSOR_COUNT}"));
    }
    let mut names = BTreeSet::new();
    let mut slots = BTreeSet::new();
    for binding in &manifest.bindings {
        if !names.insert(&binding.checkpoint_tensor) || !slots.insert(&binding.qualified_slot) {
            return Err("manifest has a duplicate checkpoint tensor or qualified slot".to_owned());
        }
        if binding.expected_storage_scalar != "F32" {
            return Err(format!(
                "manifest binds `{}` with non-F32 storage",
                binding.checkpoint_tensor
            ));
        }
        let (expected_slot, expected_key) = expected_program_binding(&binding.checkpoint_tensor)?;
        if binding.qualified_slot != expected_slot || binding.interface_key != expected_key {
            return Err(format!(
                "manifest mapping mismatch for `{}`: expected `{expected_slot}` / `{expected_key}`, got `{}` / `{}`",
                binding.checkpoint_tensor, binding.qualified_slot, binding.interface_key
            ));
        }
    }
    Ok(())
}

fn expected_program_binding(name: &str) -> Result<(String, &'static str), String> {
    if name == "model.embed_tokens.weight" {
        return Ok(("P1+P3.shared.W_embed".to_owned(), "W_embed"));
    }
    if name == "model.norm.weight" {
        return Ok(("P3.w_norm".to_owned(), "w_norm"));
    }
    let suffix = name
        .strip_prefix("model.layers.")
        .ok_or_else(|| format!("foreign tensor `{name}`"))?;
    let (layer, role) = suffix
        .split_once('.')
        .ok_or_else(|| format!("missing layer role in `{name}`"))?;
    let layer: u8 = layer
        .parse()
        .map_err(|_| format!("invalid layer in `{name}`"))?;
    if layer >= 28 {
        return Err(format!("out-of-range layer in `{name}`"));
    }
    let key = match role {
        "input_layernorm.weight" => "w_input_layernorm",
        "post_attention_layernorm.weight" => "w_post_attention_layernorm",
        "self_attn.q_norm.weight" => "w_q_norm",
        "self_attn.k_norm.weight" => "w_k_norm",
        "self_attn.q_proj.weight" => "W_q",
        "self_attn.k_proj.weight" => "W_k",
        "self_attn.v_proj.weight" => "W_v",
        "self_attn.o_proj.weight" => "W_o",
        "mlp.gate_proj.weight" => "W_gate",
        "mlp.up_proj.weight" => "W_up",
        "mlp.down_proj.weight" => "W_down",
        _ => return Err(format!("foreign layer role in `{name}`")),
    };
    Ok((format!("P2.layer-{layer:02}.{key}"), key))
}

fn read_metadata(path: &Path) -> Result<(u64, Metadata), String> {
    let mut file =
        File::open(path).map_err(|error| format!("opening checkpoint header: {error}"))?;
    let mut length = [0_u8; 8];
    file.read_exact(&mut length)
        .map_err(|error| format!("reading header length: {error}"))?;
    let header_len = u64::from_le_bytes(length);
    let header_len_usize =
        usize::try_from(header_len).map_err(|_| "header length exceeds usize")?;
    let mut header = vec![0_u8; header_len_usize];
    file.read_exact(&mut header)
        .map_err(|error| format!("reading header: {error}"))?;
    let metadata: Metadata = serde_json::from_slice(&header)
        .map_err(|error| format!("validating safetensors header: {error}"))?;
    Ok((8 + header_len, metadata))
}

fn verify_manifest_against_header(
    manifest: &Manifest,
    metadata: &Metadata,
    data_start: u64,
    path: &Path,
) -> Result<(), String> {
    let checkpoint = &manifest.checkpoint;
    if checkpoint.sha256 != CHECKPOINT_SHA256
        || checkpoint.revision != CHECKPOINT_REVISION
        || checkpoint.bytes != CHECKPOINT_BYTES
        || checkpoint.safetensors_header_bytes != SAFETENSORS_HEADER_BYTES
        || checkpoint.dtype != "BF16"
    {
        return Err(
            "manifest checkpoint pin does not match this consumer's pinned identity".to_owned(),
        );
    }
    if data_start != 8 + SAFETENSORS_HEADER_BYTES {
        return Err(format!(
            "header is {data_start} bytes including framing, not the pinned size"
        ));
    }
    let file_len = std::fs::metadata(path)
        .map_err(|error| format!("reading checkpoint metadata: {error}"))?
        .len();
    let payload_len =
        u64::try_from(metadata.data_len()).map_err(|_| "payload length exceeds u64")?;
    if file_len != data_start + payload_len || file_len != CHECKPOINT_BYTES {
        return Err(format!(
            "safetensors framing does not cover the pinned file size {file_len}"
        ));
    }
    if metadata.tensors().len() != TENSOR_COUNT {
        return Err(format!(
            "header declares {} tensors, not {TENSOR_COUNT}",
            metadata.tensors().len()
        ));
    }
    let header_names: BTreeSet<_> = metadata.tensors().into_keys().collect();
    for binding in &manifest.bindings {
        let info = metadata.info(&binding.checkpoint_tensor).ok_or_else(|| {
            format!(
                "manifest tensor `{}` is absent from safetensors header",
                binding.checkpoint_tensor
            )
        })?;
        if info.dtype != Dtype::BF16 {
            return Err(format!(
                "header tensor `{}` is not BF16",
                binding.checkpoint_tensor
            ));
        }
        let shape: Result<Vec<u64>, _> = info.shape.iter().copied().map(u64::try_from).collect();
        if shape.map_err(|_| "header extent exceeds u64")? != binding.expected_shape {
            return Err(format!(
                "header shape mismatch for `{}`",
                binding.checkpoint_tensor
            ));
        }
    }
    let manifest_names: BTreeSet<_> = manifest
        .bindings
        .iter()
        .map(|binding| binding.checkpoint_tensor.clone())
        .collect();
    if header_names != manifest_names {
        return Err("manifest names are not exactly the safetensors header inventory".to_owned());
    }
    Ok(())
}

fn widen_tensor<R: Read + Seek>(
    file: &mut R,
    offset: u64,
    bf16_bytes: u64,
    digest: &mut Sha256,
    census: &mut Census,
) -> Result<Vec<u8>, String> {
    if !bf16_bytes.is_multiple_of(2) {
        return Err("BF16 tensor has an odd byte length".to_owned());
    }
    let output_len = usize::try_from(bf16_bytes.checked_mul(2).ok_or("F32 length overflow")?)
        .map_err(|_| "F32 buffer length exceeds usize")?;
    let mut output = Vec::with_capacity(output_len);
    let mut source = vec![0_u8; WIDEN_CHUNK_BYTES];
    file.seek(SeekFrom::Start(offset))
        .map_err(|error| format!("seeking tensor: {error}"))?;
    let mut remaining =
        usize::try_from(bf16_bytes).map_err(|_| "BF16 buffer length exceeds usize")?;
    while remaining != 0 {
        let chunk_len = remaining.min(source.len());
        file.read_exact(&mut source[..chunk_len])
            .map_err(|error| format!("reading BF16 tensor: {error}"))?;
        let (pairs, []) = source[..chunk_len].as_chunks::<2>() else {
            return Err("BF16 chunk has an odd byte length".to_owned());
        };
        for pair in pairs {
            let widened = f32::from_bits(u32::from(u16::from_le_bytes([pair[0], pair[1]])) << 16);
            if widened.is_nan() {
                census.nan += 1;
            } else if widened.is_infinite() {
                census.infinite += 1;
            } else if widened.is_subnormal() {
                census.subnormal += 1;
            }
            output.extend_from_slice(&widened.to_le_bytes());
        }
        remaining -= chunk_len;
    }
    digest.update(&output);
    Ok(output)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        File::open(path).map_err(|error| format!("opening checkpoint for digest: {error}"))?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; WIDEN_CHUNK_BYTES].into_boxed_slice();
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("digesting checkpoint: {error}"))?;
        if read == 0 {
            return Ok(hex_digest(digest.finalize()));
        }
        digest.update(&buffer[..read]);
    }
}

fn require_digest(subject: &str, actual: &str, expected: &str) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{subject} digest mismatch: expected {expected}, got {actual}"
        ))
    }
}

fn refuse_nonfinite(census: Census) -> Result<(), String> {
    if census.nan == 0 && census.infinite == 0 {
        Ok(())
    } else {
        Err(format!(
            "refusing widened checkpoint: {} NaN, {} infinite, {} subnormal values",
            census.nan, census.infinite, census.subnormal
        ))
    }
}

fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    let bytes = bytes.as_ref();
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(result, "{byte:02x}").expect("writing to String cannot fail");
    }
    result
}

fn resident_bytes_at_completion() -> Option<u64> {
    let process = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    if !process.status.success() {
        return None;
    }
    let kibibytes: u64 = std::str::from_utf8(&process.stdout)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    kibibytes.checked_mul(1024)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiler::__private::{OperandExtent, OperandFacts, RegionFacts, ResultFacts};
    use tiler::value::BindError;

    const F32_REGION: RegionFacts = RegionFacts {
        operands: &[OperandFacts {
            key: "weight",
            storage_scalar: StorageScalar::F32,
            extents: &[OperandExtent::Literal(1)],
        }],
        symbols: &[],
        capabilities: &[AdapterCapability::DenseRowMajorStorage],
        result: ResultFacts {
            key: "out",
            storage_scalar: StorageScalar::F32,
            axes: &[],
        },
    };

    #[test]
    fn retained_manifest_has_the_complete_f32_population() {
        let manifest = read_and_verify_manifest().expect("retained manifest verifies");
        assert_eq!(manifest.bindings.len(), TENSOR_COUNT);
        assert!(
            manifest
                .bindings
                .iter()
                .all(|binding| binding.expected_storage_scalar == "F32")
        );
    }

    #[test]
    fn same_shape_slot_permutation_is_refused_even_after_rehashing() {
        let mut manifest = read_and_verify_manifest().expect("retained manifest verifies");
        let left = manifest
            .bindings
            .iter()
            .position(|binding| {
                binding.checkpoint_tensor == "model.layers.0.self_attn.k_proj.weight"
            })
            .expect("K projection exists");
        let right = manifest
            .bindings
            .iter()
            .position(|binding| {
                binding.checkpoint_tensor == "model.layers.0.self_attn.v_proj.weight"
            })
            .expect("V projection exists");
        assert_eq!(
            manifest.bindings[left].expected_shape,
            manifest.bindings[right].expected_shape
        );
        let left_slot = manifest.bindings[left].qualified_slot.clone();
        let right_slot = manifest.bindings[right].qualified_slot.clone();
        manifest.bindings[left].qualified_slot = right_slot;
        manifest.bindings[right].qualified_slot = left_slot;
        let error = validate_manifest_ownership(&manifest).unwrap_err();
        eprintln!("CONTROL STOP: {error}");
        assert!(error.contains("manifest mapping mismatch"));
    }

    #[test]
    fn changed_checkpoint_digest_is_refused() {
        let error = require_digest("checkpoint", "00", CHECKPOINT_SHA256).unwrap_err();
        eprintln!("CONTROL STOP: {error}");
        assert_eq!(
            error,
            format!("checkpoint digest mismatch: expected {CHECKPOINT_SHA256}, got 00")
        );
    }

    #[test]
    fn bf16_infinity_is_counted_and_refused() {
        let mut census = Census::default();
        let mut digest = Sha256::new();
        let mut file = std::io::Cursor::new(0x7f80_u16.to_le_bytes().to_vec());
        let bytes = widen_tensor(&mut file, 0, 2, &mut digest, &mut census).expect("widen subject");
        assert_eq!(bytes, f32::INFINITY.to_le_bytes());
        assert_eq!(
            census,
            Census {
                nan: 0,
                infinite: 1,
                subnormal: 0
            }
        );
        let error = refuse_nonfinite(census).unwrap_err();
        eprintln!("CONTROL STOP: {error}");
        assert_eq!(
            error,
            "refusing widened checkpoint: 0 NaN, 1 infinite, 0 subnormal values"
        );
    }

    #[test]
    fn bf16_nan_is_counted_and_refused() {
        let mut census = Census::default();
        let mut digest = Sha256::new();
        let mut file = std::io::Cursor::new(0x7fc0_u16.to_le_bytes().to_vec());
        widen_tensor(&mut file, 0, 2, &mut digest, &mut census).expect("widen subject");
        assert_eq!(
            census,
            Census {
                nan: 1,
                infinite: 0,
                subnormal: 0
            }
        );
        let error = refuse_nonfinite(census).unwrap_err();
        eprintln!("CONTROL STOP: {error}");
        assert_eq!(
            error,
            "refusing widened checkpoint: 1 NaN, 0 infinite, 0 subnormal values"
        );
    }

    #[test]
    fn widened_payload_digest_reaches_the_widening_pass() {
        let mut original_census = Census::default();
        let mut original_digest = Sha256::new();
        let mut original_file = std::io::Cursor::new(0x3f80_u16.to_le_bytes().to_vec());
        widen_tensor(
            &mut original_file,
            0,
            2,
            &mut original_digest,
            &mut original_census,
        )
        .expect("widen original subject");
        let original = hex_digest(original_digest.finalize());

        let mut changed_census = Census::default();
        let mut changed_digest = Sha256::new();
        let mut changed_file = std::io::Cursor::new(0x4000_u16.to_le_bytes().to_vec());
        widen_tensor(
            &mut changed_file,
            0,
            2,
            &mut changed_digest,
            &mut changed_census,
        )
        .expect("widen changed subject");
        let changed = hex_digest(changed_digest.finalize());
        assert_ne!(
            original, changed,
            "the widened payload digest must not ignore value bytes"
        );
        let error = require_digest("widened payload", &changed, &original).unwrap_err();
        eprintln!("CONTROL STOP: {error}");
    }

    #[test]
    fn subnormal_is_counted_independently() {
        let mut census = Census::default();
        let mut digest = Sha256::new();
        let mut file = std::io::Cursor::new(0x0001_u16.to_le_bytes().to_vec());
        widen_tensor(&mut file, 0, 2, &mut digest, &mut census).expect("widen subject");
        assert_eq!(
            census,
            Census {
                nan: 0,
                infinite: 0,
                subnormal: 1
            }
        );
    }

    #[test]
    fn bf16_storage_is_refused_by_an_f32_program_input() {
        let tensor = Tensor::<CheckpointAdapter>::new(
            DenseF32 {
                bytes: vec![0; 2],
                extents: vec![1],
                scalar: StorageScalar::Bf16,
            },
            (),
        );
        let error = tiler::__private::bind_region::<CheckpointAdapter>(&F32_REGION, &[&tensor])
            .expect_err("Bf16 storage must not bind to a declared F32 input");
        eprintln!("CONTROL STOP: {error}");
        assert_eq!(
            error,
            BindError::StorageScalarMismatch {
                input: "weight",
                declared: StorageScalar::F32,
                actual: StorageScalar::Bf16,
            }
        );
    }
}
