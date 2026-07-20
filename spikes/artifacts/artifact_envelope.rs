//! Dependency-free spike for the target-neutral artifact envelope.
//!
//! Run with:
//! `rustc --edition 2021 --test spikes/artifacts/artifact_envelope.rs -o /tmp/tiler-artifact-envelope-spike && /tmp/tiler-artifact-envelope-spike`

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

const MAGIC: &[u8; 8] = b"TLRART01";
const HEADER_LEN: usize = 60;
const ENVELOPE_MAJOR: u16 = 1;
const ENVELOPE_MINOR: u16 = 0;
const MAX_MANIFEST_BYTES: usize = 1 << 20;
const MAX_SECTIONS: usize = 64;
const MAX_ITEMS: usize = 256;
const MAX_STRING_BYTES: usize = 1024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Digest([u8; 32]);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ProgramId(u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EntryId(u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PayloadId(u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SectionId(u32);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Version {
    major: u16,
    minor: u16,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SchemaUse {
    key: String,
    version: Version,
    required: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ProviderIdentity {
    key: String,
    capability_api_version: u16,
    revision: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProgramDescriptor {
    id: ProgramId,
    semantic_digest: Digest,
    numerical_contract_digest: Digest,
    kernel_program_digest: Digest,
    neutral_program_section: SectionId,
    entries: Vec<EntryId>,
    selected_providers: Vec<ProviderIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EntryDescriptor {
    id: EntryId,
    program: ProgramId,
    payload: PayloadId,
    scheduled_region_digest: Digest,
    abi_digest: Digest,
    launch_digest: Digest,
    requirements_digest: Digest,
    backend_entry_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PayloadDescriptor {
    id: PayloadId,
    backend_key: String,
    payload_schema_key: String,
    payload_schema_version: Version,
    target_profile_digest: Digest,
    execution_contract_key: String,
    metadata_section: SectionId,
    code_sections: Vec<SectionId>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SectionKind {
    NeutralProgram = 1,
    BackendMetadata = 2,
    BackendCode = 3,
    Evidence = 4,
    Diagnostics = 5,
}

impl SectionKind {
    fn parse(value: u8) -> Result<Self, Error> {
        match value {
            1 => Ok(Self::NeutralProgram),
            2 => Ok(Self::BackendMetadata),
            3 => Ok(Self::BackendCode),
            4 => Ok(Self::Evidence),
            5 => Ok(Self::Diagnostics),
            _ => Err(Error::UnknownRequiredMeaning),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SectionDescriptor {
    id: SectionId,
    kind: SectionKind,
    schema: SchemaUse,
    exact_len: u64,
    digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Manifest {
    schema_version: Version,
    required_features: Vec<String>,
    component_schemas: Vec<SchemaUse>,
    programs: Vec<ProgramDescriptor>,
    entries: Vec<EntryDescriptor>,
    payloads: Vec<PayloadDescriptor>,
    sections: Vec<SectionDescriptor>,
    compiler_fingerprint: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Section {
    id: SectionId,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LoadedEnvelope {
    manifest: Manifest,
    sections: BTreeMap<SectionId, Vec<u8>>,
    envelope_digest: Digest,
}

#[derive(Clone, Debug)]
struct ReaderCapabilities {
    envelope_major: u16,
    max_envelope_minor: u16,
    max_manifest_minor: u16,
    features: BTreeSet<String>,
    component_schemas: BTreeMap<(String, u16), u16>,
    backend_schemas: BTreeMap<(String, String, u16), u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Error {
    Truncated,
    BadMagic,
    UnsupportedEnvelopeVersion,
    UnsupportedManifestVersion,
    UnsupportedFeature,
    UnsupportedComponentSchema,
    UnsupportedBackendSchema,
    BoundsExceeded,
    LengthMismatch,
    ManifestDigestMismatch,
    SectionDigestMismatch(SectionId),
    NonCanonicalManifest,
    NonCanonicalSectionOrder,
    DuplicateId,
    DuplicateProvider,
    MissingReference,
    WrongSectionKind,
    UnknownRequiredMeaning,
    UnreferencedSection(SectionId),
    TrailingBytes,
}

fn domain_digest(domain: &[u8], bytes: &[u8]) -> Digest {
    let mut input = Vec::with_capacity(domain.len() + bytes.len());
    input.extend_from_slice(domain);
    input.extend_from_slice(bytes);
    Digest(sha256(&input))
}

fn manifest_digest(bytes: &[u8]) -> Digest {
    domain_digest(b"tiler.artifact.manifest.v1\0", bytes)
}

fn section_digest(bytes: &[u8]) -> Digest {
    domain_digest(b"tiler.artifact.section.v1\0", bytes)
}

fn envelope_digest(bytes: &[u8]) -> Digest {
    domain_digest(b"tiler.artifact.envelope.v1\0", bytes)
}

fn encode_bundle(manifest: &Manifest, sections: &[Section]) -> Vec<u8> {
    encode_bundle_with_manifest(manifest.canonical_bytes(), sections)
}

fn encode_bundle_with_manifest(manifest_bytes: Vec<u8>, sections: &[Section]) -> Vec<u8> {
    encode_bundle_in_order(manifest_bytes, sections, true)
}

fn encode_bundle_in_order(
    manifest_bytes: Vec<u8>,
    sections: &[Section],
    canonicalize_section_order: bool,
) -> Vec<u8> {
    let mut encoded_sections = sections.to_vec();
    if canonicalize_section_order {
        encoded_sections.sort_by_key(|section| section.id);
    }

    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    put_u16(&mut out, ENVELOPE_MAJOR);
    put_u16(&mut out, ENVELOPE_MINOR);
    put_u64(&mut out, 0); // total length, filled below
    put_u32(&mut out, manifest_bytes.len() as u32);
    put_u32(&mut out, encoded_sections.len() as u32);
    out.extend_from_slice(&manifest_digest(&manifest_bytes).0);
    debug_assert_eq!(out.len(), HEADER_LEN);
    out.extend_from_slice(&manifest_bytes);
    for section in encoded_sections {
        put_u32(&mut out, section.id.0);
        put_u64(&mut out, section.bytes.len() as u64);
        out.extend_from_slice(&section.bytes);
    }
    let total = out.len() as u64;
    out[12..20].copy_from_slice(&total.to_le_bytes());
    out
}

fn load(bytes: &[u8], capabilities: &ReaderCapabilities) -> Result<LoadedEnvelope, Error> {
    if bytes.len() < HEADER_LEN {
        return Err(Error::Truncated);
    }
    let mut cursor = Cursor::new(bytes);
    if cursor.take(8)? != MAGIC {
        return Err(Error::BadMagic);
    }
    let envelope_major = cursor.u16()?;
    let envelope_minor = cursor.u16()?;
    if envelope_major != capabilities.envelope_major
        || envelope_minor > capabilities.max_envelope_minor
    {
        return Err(Error::UnsupportedEnvelopeVersion);
    }
    let total_len = usize::try_from(cursor.u64()?).map_err(|_| Error::BoundsExceeded)?;
    if total_len != bytes.len() {
        return Err(Error::LengthMismatch);
    }
    let manifest_len = cursor.u32()? as usize;
    let section_count = cursor.u32()? as usize;
    if manifest_len > MAX_MANIFEST_BYTES || section_count > MAX_SECTIONS {
        return Err(Error::BoundsExceeded);
    }
    let expected_manifest_digest = Digest(cursor.array_32()?);
    let manifest_bytes = cursor.take(manifest_len)?;
    if manifest_digest(manifest_bytes) != expected_manifest_digest {
        return Err(Error::ManifestDigestMismatch);
    }
    let manifest = Manifest::parse(manifest_bytes)?;
    if manifest.canonical_bytes() != manifest_bytes {
        return Err(Error::NonCanonicalManifest);
    }
    validate_manifest(&manifest, capabilities)?;
    if manifest.sections.len() != section_count {
        return Err(Error::LengthMismatch);
    }

    let descriptors: BTreeMap<_, _> = manifest
        .sections
        .iter()
        .map(|descriptor| (descriptor.id, descriptor))
        .collect();
    let mut sections = BTreeMap::new();
    let mut previous_section = None;
    for _ in 0..section_count {
        let id = SectionId(cursor.u32()?);
        if previous_section.is_some_and(|previous| previous >= id) {
            return Err(Error::NonCanonicalSectionOrder);
        }
        previous_section = Some(id);
        let len = usize::try_from(cursor.u64()?).map_err(|_| Error::BoundsExceeded)?;
        let descriptor = descriptors.get(&id).ok_or(Error::MissingReference)?;
        if descriptor.exact_len != len as u64 {
            return Err(Error::LengthMismatch);
        }
        let section_bytes = cursor.take(len)?.to_vec();
        if section_digest(&section_bytes) != descriptor.digest {
            return Err(Error::SectionDigestMismatch(id));
        }
        if sections.insert(id, section_bytes).is_some() {
            return Err(Error::DuplicateId);
        }
    }
    if cursor.remaining() != 0 {
        return Err(Error::TrailingBytes);
    }
    if sections.len() != descriptors.len() {
        return Err(Error::MissingReference);
    }

    Ok(LoadedEnvelope {
        manifest,
        sections,
        envelope_digest: envelope_digest(bytes),
    })
}

fn validate_manifest(manifest: &Manifest, capabilities: &ReaderCapabilities) -> Result<(), Error> {
    if manifest.schema_version.major != 1
        || manifest.schema_version.minor > capabilities.max_manifest_minor
    {
        return Err(Error::UnsupportedManifestVersion);
    }
    for feature in &manifest.required_features {
        if !capabilities.features.contains(feature) {
            return Err(Error::UnsupportedFeature);
        }
    }
    for schema in &manifest.component_schemas {
        let supported_minor = capabilities
            .component_schemas
            .get(&(schema.key.clone(), schema.version.major));
        if schema.required
            && !matches!(supported_minor, Some(minor) if *minor >= schema.version.minor)
        {
            return Err(Error::UnsupportedComponentSchema);
        }
    }

    unique(manifest.programs.iter().map(|program| program.id))?;
    unique(manifest.entries.iter().map(|entry| entry.id))?;
    unique(manifest.payloads.iter().map(|payload| payload.id))?;
    unique(manifest.sections.iter().map(|section| section.id))?;
    unique(manifest.required_features.iter())?;
    unique(manifest.component_schemas.iter().map(|schema| &schema.key))?;

    let programs: BTreeMap<_, _> = manifest
        .programs
        .iter()
        .map(|program| (program.id, program))
        .collect();
    let entries: BTreeMap<_, _> = manifest
        .entries
        .iter()
        .map(|entry| (entry.id, entry))
        .collect();
    let payloads: BTreeMap<_, _> = manifest
        .payloads
        .iter()
        .map(|payload| (payload.id, payload))
        .collect();
    let sections: BTreeMap<_, _> = manifest
        .sections
        .iter()
        .map(|section| (section.id, section))
        .collect();

    let mut referenced_sections = BTreeSet::new();
    let mut referenced_entries = BTreeSet::new();
    for program in &manifest.programs {
        if program.selected_providers.is_empty() {
            return Err(Error::MissingReference);
        }
        unique(program.entries.iter())?;
        unique(
            program
                .selected_providers
                .iter()
                .map(|provider| &provider.key),
        )
        .map_err(|_| Error::DuplicateProvider)?;
        require_kind(
            &sections,
            program.neutral_program_section,
            SectionKind::NeutralProgram,
        )?;
        if sections[&program.neutral_program_section].digest != program.kernel_program_digest {
            return Err(Error::MissingReference);
        }
        referenced_sections.insert(program.neutral_program_section);
        for entry_id in &program.entries {
            let entry = entries.get(entry_id).ok_or(Error::MissingReference)?;
            if entry.program != program.id {
                return Err(Error::MissingReference);
            }
            referenced_entries.insert(*entry_id);
        }
    }
    if referenced_entries.len() != entries.len() {
        return Err(Error::MissingReference);
    }

    for entry in &manifest.entries {
        if !programs.contains_key(&entry.program) || !payloads.contains_key(&entry.payload) {
            return Err(Error::MissingReference);
        }
        if entry.backend_entry_key.is_empty() {
            return Err(Error::MissingReference);
        }
    }

    for payload in &manifest.payloads {
        let supported_minor = capabilities.backend_schemas.get(&(
            payload.backend_key.clone(),
            payload.payload_schema_key.clone(),
            payload.payload_schema_version.major,
        ));
        if !matches!(supported_minor, Some(minor) if *minor >= payload.payload_schema_version.minor)
        {
            return Err(Error::UnsupportedBackendSchema);
        }
        require_kind(
            &sections,
            payload.metadata_section,
            SectionKind::BackendMetadata,
        )?;
        let metadata = sections[&payload.metadata_section];
        if metadata.schema.key != payload.payload_schema_key
            || metadata.schema.version != payload.payload_schema_version
        {
            return Err(Error::WrongSectionKind);
        }
        referenced_sections.insert(payload.metadata_section);
        unique(payload.code_sections.iter())?;
        if payload.code_sections.is_empty() {
            return Err(Error::MissingReference);
        }
        for section in &payload.code_sections {
            require_kind(&sections, *section, SectionKind::BackendCode)?;
            referenced_sections.insert(*section);
        }
    }

    for descriptor in &manifest.sections {
        match descriptor.kind {
            SectionKind::Evidence | SectionKind::Diagnostics if !descriptor.schema.required => {}
            _ if !referenced_sections.contains(&descriptor.id) => {
                return Err(Error::UnreferencedSection(descriptor.id))
            }
            _ => {}
        }
    }
    Ok(())
}

fn unique<T: Ord>(values: impl IntoIterator<Item = T>) -> Result<(), Error> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(Error::DuplicateId);
        }
    }
    Ok(())
}

fn require_kind(
    sections: &BTreeMap<SectionId, &SectionDescriptor>,
    id: SectionId,
    kind: SectionKind,
) -> Result<(), Error> {
    if sections.get(&id).ok_or(Error::MissingReference)?.kind != kind {
        return Err(Error::WrongSectionKind);
    }
    Ok(())
}

impl Manifest {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut normalized = self.clone();
        normalized.required_features.sort();
        normalized.component_schemas.sort();
        normalized.programs.sort_by_key(|program| program.id);
        normalized.entries.sort_by_key(|entry| entry.id);
        normalized.payloads.sort_by_key(|payload| payload.id);
        normalized.sections.sort_by_key(|section| section.id);
        for program in &mut normalized.programs {
            program.entries.sort();
            program.selected_providers.sort();
        }
        for payload in &mut normalized.payloads {
            payload.code_sections.sort();
        }
        normalized.bytes_in_current_order()
    }

    fn bytes_in_current_order(&self) -> Vec<u8> {
        let mut out = Vec::new();
        put_version(&mut out, &self.schema_version);
        put_vec(&mut out, &self.required_features, |out, feature| {
            put_string(out, feature)
        });
        put_vec(&mut out, &self.component_schemas, put_schema);
        put_vec(&mut out, &self.programs, |out, program| {
            put_u32(out, program.id.0);
            put_digest(out, program.semantic_digest);
            put_digest(out, program.numerical_contract_digest);
            put_digest(out, program.kernel_program_digest);
            put_u32(out, program.neutral_program_section.0);
            put_vec(out, &program.entries, |out, entry| put_u32(out, entry.0));
            put_vec(out, &program.selected_providers, |out, provider| {
                put_string(out, &provider.key);
                put_u16(out, provider.capability_api_version);
                put_digest(out, provider.revision);
            });
        });
        put_vec(&mut out, &self.entries, |out, entry| {
            put_u32(out, entry.id.0);
            put_u32(out, entry.program.0);
            put_u32(out, entry.payload.0);
            put_digest(out, entry.scheduled_region_digest);
            put_digest(out, entry.abi_digest);
            put_digest(out, entry.launch_digest);
            put_digest(out, entry.requirements_digest);
            put_string(out, &entry.backend_entry_key);
        });
        put_vec(&mut out, &self.payloads, |out, payload| {
            put_u32(out, payload.id.0);
            put_string(out, &payload.backend_key);
            put_string(out, &payload.payload_schema_key);
            put_version(out, &payload.payload_schema_version);
            put_digest(out, payload.target_profile_digest);
            put_string(out, &payload.execution_contract_key);
            put_u32(out, payload.metadata_section.0);
            put_vec(out, &payload.code_sections, |out, section| {
                put_u32(out, section.0)
            });
        });
        put_vec(&mut out, &self.sections, |out, section| {
            put_u32(out, section.id.0);
            out.push(section.kind as u8);
            put_schema(out, &section.schema);
            put_u64(out, section.exact_len);
            put_digest(out, section.digest);
        });
        put_digest(&mut out, self.compiler_fingerprint);
        out
    }

    fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let mut cursor = Cursor::new(bytes);
        let schema_version = cursor.version()?;
        let required_features = cursor.vec(|cursor| cursor.string())?;
        let component_schemas = cursor.vec(|cursor| cursor.schema())?;
        let programs = cursor.vec(|cursor| {
            Ok(ProgramDescriptor {
                id: ProgramId(cursor.u32()?),
                semantic_digest: cursor.digest()?,
                numerical_contract_digest: cursor.digest()?,
                kernel_program_digest: cursor.digest()?,
                neutral_program_section: SectionId(cursor.u32()?),
                entries: cursor.vec(|cursor| Ok(EntryId(cursor.u32()?)))?,
                selected_providers: cursor.vec(|cursor| {
                    Ok(ProviderIdentity {
                        key: cursor.string()?,
                        capability_api_version: cursor.u16()?,
                        revision: cursor.digest()?,
                    })
                })?,
            })
        })?;
        let entries = cursor.vec(|cursor| {
            Ok(EntryDescriptor {
                id: EntryId(cursor.u32()?),
                program: ProgramId(cursor.u32()?),
                payload: PayloadId(cursor.u32()?),
                scheduled_region_digest: cursor.digest()?,
                abi_digest: cursor.digest()?,
                launch_digest: cursor.digest()?,
                requirements_digest: cursor.digest()?,
                backend_entry_key: cursor.string()?,
            })
        })?;
        let payloads = cursor.vec(|cursor| {
            Ok(PayloadDescriptor {
                id: PayloadId(cursor.u32()?),
                backend_key: cursor.string()?,
                payload_schema_key: cursor.string()?,
                payload_schema_version: cursor.version()?,
                target_profile_digest: cursor.digest()?,
                execution_contract_key: cursor.string()?,
                metadata_section: SectionId(cursor.u32()?),
                code_sections: cursor.vec(|cursor| Ok(SectionId(cursor.u32()?)))?,
            })
        })?;
        let sections = cursor.vec(|cursor| {
            Ok(SectionDescriptor {
                id: SectionId(cursor.u32()?),
                kind: SectionKind::parse(cursor.u8()?)?,
                schema: cursor.schema()?,
                exact_len: cursor.u64()?,
                digest: cursor.digest()?,
            })
        })?;
        let compiler_fingerprint = cursor.digest()?;
        if cursor.remaining() != 0 {
            return Err(Error::TrailingBytes);
        }
        Ok(Self {
            schema_version,
            required_features,
            component_schemas,
            programs,
            entries,
            payloads,
            sections,
            compiler_fingerprint,
        })
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(Error::BoundsExceeded)?;
        let result = self.bytes.get(self.position..end).ok_or(Error::Truncated)?;
        self.position = end;
        Ok(result)
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, Error> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn array_32(&mut self) -> Result<[u8; 32], Error> {
        Ok(self.take(32)?.try_into().unwrap())
    }

    fn digest(&mut self) -> Result<Digest, Error> {
        Ok(Digest(self.array_32()?))
    }

    fn version(&mut self) -> Result<Version, Error> {
        Ok(Version {
            major: self.u16()?,
            minor: self.u16()?,
        })
    }

    fn string(&mut self) -> Result<String, Error> {
        let len = self.u16()? as usize;
        if len > MAX_STRING_BYTES {
            return Err(Error::BoundsExceeded);
        }
        String::from_utf8(self.take(len)?.to_vec()).map_err(|_| Error::UnknownRequiredMeaning)
    }

    fn vec<T>(
        &mut self,
        mut parse: impl FnMut(&mut Cursor<'a>) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        let len = self.u16()? as usize;
        if len > MAX_ITEMS {
            return Err(Error::BoundsExceeded);
        }
        (0..len).map(|_| parse(self)).collect()
    }

    fn schema(&mut self) -> Result<SchemaUse, Error> {
        Ok(SchemaUse {
            key: self.string()?,
            version: self.version()?,
            required: match self.u8()? {
                0 => false,
                1 => true,
                _ => return Err(Error::UnknownRequiredMeaning),
            },
        })
    }
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_digest(out: &mut Vec<u8>, digest: Digest) {
    out.extend_from_slice(&digest.0);
}

fn put_version(out: &mut Vec<u8>, version: &Version) {
    put_u16(out, version.major);
    put_u16(out, version.minor);
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    assert!(value.len() <= u16::MAX as usize);
    put_u16(out, value.len() as u16);
    out.extend_from_slice(value.as_bytes());
}

fn put_vec<T>(out: &mut Vec<u8>, values: &[T], mut put: impl FnMut(&mut Vec<u8>, &T)) {
    assert!(values.len() <= u16::MAX as usize);
    put_u16(out, values.len() as u16);
    for value in values {
        put(out, value);
    }
}

fn put_schema(out: &mut Vec<u8>, schema: &SchemaUse) {
    put_string(out, &schema.key);
    put_version(out, &schema.version);
    out.push(schema.required as u8);
}

// Small in-file SHA-256 implementation keeps the spike dependency-free. The
// envelope contract still governs the production algorithm by an explicit key.
fn sha256(input: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut data = input.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for chunk in data.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(word.try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut out = [0u8; 32];
    for (chunk, value) in out.chunks_exact_mut(4).zip(h) {
        chunk.copy_from_slice(&value.to_be_bytes());
    }
    out
}

fn digest_of(label: &str) -> Digest {
    Digest(sha256(label.as_bytes()))
}

fn section(id: u32, bytes: &[u8]) -> Section {
    Section {
        id: SectionId(id),
        bytes: bytes.to_vec(),
    }
}

fn descriptor(id: u32, kind: SectionKind, schema: &str, section: &Section) -> SectionDescriptor {
    SectionDescriptor {
        id: SectionId(id),
        kind,
        schema: SchemaUse {
            key: schema.into(),
            version: Version { major: 1, minor: 0 },
            required: true,
        },
        exact_len: section.bytes.len() as u64,
        digest: section_digest(&section.bytes),
    }
}

fn sample() -> (Manifest, Vec<Section>, ReaderCapabilities) {
    let neutral = section(1, b"canonical KernelProgram bytes");
    // Metal-specific symbol and platform data are intentionally opaque bytes
    // to the neutral loader.
    let metal_metadata = section(
        2,
        b"platform=macos;symbol=tiler_entry_0;buffer_index=0;msl=3.2",
    );
    let metallib = section(3, b"opaque metallib bytes");
    let manifest = Manifest {
        schema_version: Version { major: 1, minor: 0 },
        required_features: vec![
            "tiler.feature.typed-feasibility".into(),
            "tiler.feature.routing-commit".into(),
        ],
        component_schemas: vec![
            SchemaUse {
                key: "tiler.kernel-program".into(),
                version: Version { major: 1, minor: 0 },
                required: true,
            },
            SchemaUse {
                key: "tiler.abi-expr".into(),
                version: Version { major: 1, minor: 0 },
                required: true,
            },
        ],
        programs: vec![ProgramDescriptor {
            id: ProgramId(10),
            semantic_digest: digest_of("semantic"),
            numerical_contract_digest: digest_of("numerics"),
            kernel_program_digest: section_digest(&neutral.bytes),
            neutral_program_section: neutral.id,
            entries: vec![EntryId(100)],
            selected_providers: vec![ProviderIdentity {
                key: "tiler.provider.metal.elementwise".into(),
                capability_api_version: 1,
                revision: digest_of("provider revision 7"),
            }],
        }],
        entries: vec![EntryDescriptor {
            id: EntryId(100),
            program: ProgramId(10),
            payload: PayloadId(20),
            scheduled_region_digest: digest_of("scheduled"),
            abi_digest: digest_of("abi"),
            launch_digest: digest_of("launch"),
            requirements_digest: digest_of("requirements"),
            backend_entry_key: "elementwise.scalar".into(),
        }],
        payloads: vec![PayloadDescriptor {
            id: PayloadId(20),
            backend_key: "tiler.backend.metal".into(),
            payload_schema_key: "tiler.payload.metal".into(),
            payload_schema_version: Version { major: 1, minor: 0 },
            target_profile_digest: digest_of("macos target profile"),
            execution_contract_key: "tiler.exec.target-ir-runtime-translation".into(),
            metadata_section: metal_metadata.id,
            code_sections: vec![metallib.id],
        }],
        sections: vec![
            descriptor(
                1,
                SectionKind::NeutralProgram,
                "tiler.kernel-program",
                &neutral,
            ),
            descriptor(
                2,
                SectionKind::BackendMetadata,
                "tiler.payload.metal",
                &metal_metadata,
            ),
            descriptor(
                3,
                SectionKind::BackendCode,
                "tiler.code.metallib",
                &metallib,
            ),
        ],
        compiler_fingerprint: digest_of("compiler toolchain and selected options"),
    };
    let capabilities = ReaderCapabilities {
        envelope_major: 1,
        max_envelope_minor: 0,
        max_manifest_minor: 0,
        features: manifest.required_features.iter().cloned().collect(),
        component_schemas: manifest
            .component_schemas
            .iter()
            .map(|schema| {
                (
                    (schema.key.clone(), schema.version.major),
                    schema.version.minor,
                )
            })
            .collect(),
        backend_schemas: BTreeMap::from([(
            (
                "tiler.backend.metal".into(),
                "tiler.payload.metal".into(),
                1,
            ),
            0,
        )]),
    };
    (
        manifest,
        vec![neutral, metal_metadata, metallib],
        capabilities,
    )
}

#[test]
fn sha256_matches_standard_vector() {
    assert_eq!(
        sha256(b"abc"),
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ]
    );
}

#[test]
fn valid_metal_envelope_round_trips_and_is_content_addressed() {
    let (manifest, sections, capabilities) = sample();
    let bytes = encode_bundle(&manifest, &sections);
    let loaded = load(&bytes, &capabilities).unwrap();
    assert_eq!(
        loaded.manifest.canonical_bytes(),
        manifest.canonical_bytes()
    );
    assert_eq!(loaded.envelope_digest, envelope_digest(&bytes));
    assert_eq!(loaded.sections[&SectionId(3)], b"opaque metallib bytes");
}

#[test]
fn canonical_encoding_ignores_set_insertion_order() {
    let (manifest, sections, _) = sample();
    let mut shuffled_manifest = manifest.clone();
    shuffled_manifest.required_features.reverse();
    shuffled_manifest.component_schemas.reverse();
    shuffled_manifest.sections.reverse();
    let mut shuffled_sections = sections.clone();
    shuffled_sections.reverse();
    assert_eq!(
        encode_bundle(&manifest, &sections),
        encode_bundle(&shuffled_manifest, &shuffled_sections)
    );
}

#[test]
fn validly_digested_but_noncanonical_manifest_is_rejected() {
    let (mut manifest, sections, capabilities) = sample();
    manifest.required_features.reverse();
    let bytes = encode_bundle_with_manifest(manifest.bytes_in_current_order(), &sections);
    assert_eq!(
        load(&bytes, &capabilities),
        Err(Error::NonCanonicalManifest)
    );
}

#[test]
fn noncanonical_physical_section_order_is_rejected() {
    let (manifest, mut sections, capabilities) = sample();
    sections.reverse();
    let bytes = encode_bundle_in_order(manifest.canonical_bytes(), &sections, false);
    assert_eq!(
        load(&bytes, &capabilities),
        Err(Error::NonCanonicalSectionOrder)
    );
}

#[test]
fn exact_section_corruption_and_truncation_are_rejected() {
    let (manifest, sections, capabilities) = sample();
    let mut corrupted = encode_bundle(&manifest, &sections);
    *corrupted.last_mut().unwrap() ^= 1;
    assert_eq!(
        load(&corrupted, &capabilities),
        Err(Error::SectionDigestMismatch(SectionId(3)))
    );

    let mut truncated = encode_bundle(&manifest, &sections);
    truncated.pop();
    assert_eq!(load(&truncated, &capabilities), Err(Error::LengthMismatch));
}

#[test]
fn duplicate_and_missing_cross_references_are_rejected() {
    let (mut manifest, sections, capabilities) = sample();
    manifest.programs.push(manifest.programs[0].clone());
    let bytes = encode_bundle(&manifest, &sections);
    assert_eq!(load(&bytes, &capabilities), Err(Error::DuplicateId));

    let (mut manifest, sections, capabilities) = sample();
    manifest.entries[0].payload = PayloadId(999);
    let bytes = encode_bundle(&manifest, &sections);
    assert_eq!(load(&bytes, &capabilities), Err(Error::MissingReference));
}

#[test]
fn unreferenced_executable_section_is_rejected() {
    let (mut manifest, mut sections, capabilities) = sample();
    let hidden = section(4, b"unadvertised executable");
    manifest.sections.push(descriptor(
        4,
        SectionKind::BackendCode,
        "tiler.code.metallib",
        &hidden,
    ));
    sections.push(hidden);
    let bytes = encode_bundle(&manifest, &sections);
    assert_eq!(
        load(&bytes, &capabilities),
        Err(Error::UnreferencedSection(SectionId(4)))
    );
}

#[test]
fn unsupported_required_feature_and_backend_schema_fail_closed() {
    let (manifest, sections, mut capabilities) = sample();
    capabilities
        .features
        .remove("tiler.feature.typed-feasibility");
    let bytes = encode_bundle(&manifest, &sections);
    assert_eq!(load(&bytes, &capabilities), Err(Error::UnsupportedFeature));

    let (manifest, sections, mut capabilities) = sample();
    capabilities.backend_schemas.clear();
    let bytes = encode_bundle(&manifest, &sections);
    assert_eq!(
        load(&bytes, &capabilities),
        Err(Error::UnsupportedBackendSchema)
    );
}

#[test]
fn incompatible_envelope_and_component_schema_versions_fail_closed() {
    let (manifest, sections, capabilities) = sample();
    let mut bytes = encode_bundle(&manifest, &sections);
    bytes[8..10].copy_from_slice(&2u16.to_le_bytes());
    assert_eq!(
        load(&bytes, &capabilities),
        Err(Error::UnsupportedEnvelopeVersion)
    );

    let (manifest, sections, mut capabilities) = sample();
    capabilities.component_schemas.clear();
    let bytes = encode_bundle(&manifest, &sections);
    assert_eq!(
        load(&bytes, &capabilities),
        Err(Error::UnsupportedComponentSchema)
    );
}

#[test]
fn backend_metadata_schema_must_match_payload_descriptor() {
    let (mut manifest, sections, capabilities) = sample();
    manifest.sections[1].schema.version.minor = 1;
    let bytes = encode_bundle(&manifest, &sections);
    assert_eq!(load(&bytes, &capabilities), Err(Error::WrongSectionKind));
}

#[test]
fn selected_provider_and_compiler_provenance_change_exact_identity() {
    let (manifest, sections, _) = sample();
    let first = encode_bundle(&manifest, &sections);

    let mut changed_provider = manifest.clone();
    changed_provider.programs[0].selected_providers[0].revision = digest_of("revision 8");
    let second = encode_bundle(&changed_provider, &sections);
    assert_ne!(envelope_digest(&first), envelope_digest(&second));

    let mut changed_compiler = manifest.clone();
    changed_compiler.compiler_fingerprint = digest_of("different compiler");
    let third = encode_bundle(&changed_compiler, &sections);
    assert_ne!(envelope_digest(&first), envelope_digest(&third));
}

#[test]
fn trailing_bytes_are_not_an_unhashed_extension_channel() {
    let (manifest, sections, capabilities) = sample();
    let mut bytes = encode_bundle(&manifest, &sections);
    bytes.extend_from_slice(b"hidden");
    // The fixed total length was not updated, so framing rejects before any
    // attempt to interpret trailing data.
    assert_eq!(load(&bytes, &capabilities), Err(Error::LengthMismatch));
}
