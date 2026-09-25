// SPDX-License-Identifier: AGPL-3.0-or-later
//! IT-001 SC-06..SC-09: B's reader for A's `quire.compiled-protocol/2` handoff.
//!
//! A publishes the handoff as a directory: the offered `/2` bytes, their
//! external artifact reference, the original unrecanonicalized producer inputs,
//! an independent selection sidecar, and a machine-readable mutation corpus.
//!
//! Every `Expected` value this module builds comes from the sidecar and the
//! original input files. Nothing is read out of the offered `/2` bytes: the
//! offer is carried to `admit_and_link_v2` untouched and has no authority to
//! select or repair any expectation.
//!
//! The sidecar and mutation-manifest record shapes are A's own published types
//! (`protocol_artifact::handoff`), decoded here rather than re-declared, and
//! refusal identities come from A's `Error::code`. This module declares no
//! protocol vocabulary of its own.

use std::{
    collections::BTreeSet,
    fmt, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use qsl_foundation::{ByteDigest, Source, SourceIdentity};
use quire_contract_model_owner::{
    SourceDocumentId, SourceIdentity as IrSourceIdentity, SourceRevision,
};
use quire_spec_language::{
    formal_source::FormalSource,
    model_source::{self, ModelSourceLimits},
    native_model::{ModelLimits, NativeModel},
    protocol_artifact::{self as artifact, handoff::write_v2, v2, wire as w},
};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Absence
// ---------------------------------------------------------------------------

/// Why the `/2` lane could not be assembled. Every variant is reported; none is
/// treated as a pass.
#[derive(Debug)]
pub enum Absence {
    /// The named directory does not exist.
    MissingRoot(PathBuf),
    /// A published member is absent.
    MissingMember { role: String, path: PathBuf },
    /// A published member does not deserialise into its envelope shape.
    Unreadable { path: PathBuf, cause: String },
    /// The mutation manifest declares an envelope version B does not read.
    UnknownMutationFormat { format: String },
    /// The mutation manifest's base members name files other than the ones B
    /// loaded, so its cases are stated against a different handoff.
    ManifestBase { member: String, named: String },
    /// The selection names an accounting contract whose counters B does not
    /// interpret, so its ceilings cannot be carried over.
    AccountingVersion { named: String },
    /// A selected ceiling exceeds A's own hard maximum for that counter. A's
    /// publisher clamps to that maximum, so a higher value did not come from
    /// it, and honouring it would run B under a budget A never selected.
    ExcessiveLimit {
        dimension: artifact::Dimension,
        selected: u64,
        hard: u64,
    },
    /// A published path is not a plain sequence of handoff-relative segments,
    /// so it could name bytes outside the handoff.
    EscapingMember { role: String, named: String },
    /// A handoff member is a symbolic link, so its bytes are not the published
    /// ones and its target is not covered by the checksum manifest.
    SymbolicLink { relative: String },
    /// The handoff nests deeper than B walks, so its coverage cannot be
    /// established.
    TooDeep { relative: String },
    /// A checksum manifest line is not `<64 hex>  <path>`.
    ChecksumLine { line: String },
    /// A published member does not hash to its checksum-manifest digest.
    Checksum {
        relative: String,
        listed: String,
        actual: String,
    },
    /// A file is present in the handoff but absent from the checksum manifest,
    /// so it carries no published integrity claim.
    Unlisted { relative: String },
    /// Two temporal selections name one clock identity.
    RepeatedClockIdentity { identity: String },
    /// A retained original input does not hash to the identity the sidecar
    /// independently selected for it. The handoff is internally inconsistent
    /// and no admission outcome may be reported from it.
    InputDigest {
        role: String,
        path: PathBuf,
        selected: String,
        actual: String,
    },
    /// The model source could not be re-admitted through A's public reader, so
    /// B cannot construct the `AdmittedModel` the expectation requires.
    Model { cause: String },
}

impl fmt::Display for Absence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRoot(path) => write!(f, "handoff root {} does not exist", path.display()),
            Self::MissingMember { role, path } => {
                write!(f, "published {role} member {} is absent", path.display())
            }
            Self::Unreadable { path, cause } => {
                write!(f, "{} is unreadable: {cause}", path.display())
            }
            Self::UnknownMutationFormat { format } => {
                write!(
                    f,
                    "mutation manifest declares format {format}, not {MUTATION_MANIFEST_FORMAT}"
                )
            }
            Self::ManifestBase { member, named } => write!(
                f,
                "mutation manifest's {member} names {named}, which is not the \
                 member B loaded"
            ),
            Self::AccountingVersion { named } => write!(
                f,
                "the independent selection names accounting contract {named}, \
                 not {}",
                artifact::ACCOUNTING_VERSION
            ),
            Self::ExcessiveLimit {
                dimension,
                selected,
                hard,
            } => write!(
                f,
                "the independent selection's {dimension:?} ceiling {selected} \
                 exceeds A's hard maximum {hard}"
            ),
            Self::EscapingMember { role, named } => write!(
                f,
                "published {role} path {named:?} is not a plain handoff-relative \
                 path"
            ),
            Self::SymbolicLink { relative } => {
                write!(f, "{relative} is a symbolic link, not published bytes")
            }
            Self::TooDeep { relative } => write!(
                f,
                "{relative} nests deeper than {MAX_DEPTH} directories below the \
                 handoff root"
            ),
            Self::InputDigest {
                role,
                path,
                selected,
                actual,
            } => write!(
                f,
                "retained {role} input {} hashes to {actual}, but the independent \
                 selection names {selected}",
                path.display()
            ),
            Self::ChecksumLine { line } => {
                write!(f, "{PUBLISHED_CHECKSUMS_FILE} line is malformed: {line:?}")
            }
            Self::Checksum {
                relative,
                listed,
                actual,
            } => write!(
                f,
                "{relative} hashes to {actual}, but {PUBLISHED_CHECKSUMS_FILE} lists {listed}"
            ),
            Self::Unlisted { relative } => {
                write!(
                    f,
                    "{relative} is present but not listed in {PUBLISHED_CHECKSUMS_FILE}"
                )
            }
            Self::RepeatedClockIdentity { identity } => {
                write!(f, "two temporal selections name clock identity {identity}")
            }
            Self::Model { cause } => {
                write!(f, "the model source could not be re-admitted: {cause}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// A's published handoff envelopes
// ---------------------------------------------------------------------------

// `expected-v2.json` and `mutations/manifest.json` are A's record shapes, and A
// publishes them as `protocol_artifact::handoff`. B decodes those types; it
// declares no envelope of its own, so a producer-side field change is a compile
// error here rather than a silent divergence.
pub use artifact::handoff::{
    MutationManifest, SelectedArtifactLimits, SelectedDeclaration, SelectedModel, SelectionV2,
    MUTATION_MANIFEST_FORMAT, PUBLISHED_ARTIFACT_REFERENCE_FILE, PUBLISHED_CHECKSUMS_FILE,
    PUBLISHED_MUTATION_MANIFEST_FILE, PUBLISHED_OFFER_FILE, PUBLISHED_SELECTION_FILE,
};

/// A's selected ceilings as this platform's `Limits`.
///
/// The published record is fixed-width `u64`, and A's own publisher clamps each
/// counter to that counter's hard maximum before writing it. A value above the
/// maximum therefore did not come from A's publisher, and a value too wide for
/// this platform saturates above it: both are refused rather than narrowed into
/// a budget A never selected.
pub fn selected_limits(selected: &SelectedArtifactLimits) -> Result<artifact::Limits, Absence> {
    let hard = artifact::Limits::default();
    macro_rules! native {
        ($field:ident, $dimension:expr) => {{
            let value = usize::try_from(selected.$field).unwrap_or(usize::MAX);
            if value > hard.$field {
                return Err(Absence::ExcessiveLimit {
                    dimension: $dimension,
                    selected: selected.$field,
                    hard: u64::try_from(hard.$field).unwrap_or(u64::MAX),
                });
            }
            value
        }};
    }
    Ok(artifact::Limits {
        payload_bytes: native!(payload_bytes, artifact::Dimension::PayloadBytes),
        output_bytes: native!(output_bytes, artifact::Dimension::OutputBytes),
        source_bytes: native!(source_bytes, artifact::Dimension::SourceBytes),
        content_bytes: native!(content_bytes, artifact::Dimension::ContentBytes),
        sources: native!(sources, artifact::Dimension::Sources),
        dependencies: native!(dependencies, artifact::Dimension::Dependencies),
        definitions: native!(definitions, artifact::Dimension::Definitions),
        models: native!(models, artifact::Dimension::Models),
        declarations: native!(declarations, artifact::Dimension::Declarations),
        entries: native!(entries, artifact::Dimension::Entries),
        references: native!(references, artifact::Dimension::References),
        byte_work: native!(byte_work, artifact::Dimension::ByteWork),
        depth: native!(depth, artifact::Dimension::Depth),
    })
}

// ---------------------------------------------------------------------------
// The producer's written handoff
// ---------------------------------------------------------------------------

/// The `/2` handoff directory, written once per test process.
///
/// It is published as `<target>/qsl-producer/handoff-<sha256 of SHA256SUMS>`:
/// equal names mean equal handoffs, so repeated test runs reuse one directory
/// instead of piling up copies. A published directory is never replaced, so a
/// process still reading one is undisturbed.
fn written_handoff() -> Result<&'static Path, Absence> {
    static WRITTEN: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    WRITTEN
        .get_or_init(write_handoff)
        .as_deref()
        .map_err(|cause| Absence::Unreadable {
            path: PathBuf::from("handoff::write_v2"),
            cause: cause.clone(),
        })
}

fn write_handoff() -> Result<PathBuf, String> {
    // `<target>/<profile>/deps/<test binary>`.
    let executable = std::env::current_exe()
        .map_err(|error| format!("cannot locate the test executable: {error}"))?;
    let base = executable
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(|target| target.join("qsl-producer"))
        .ok_or_else(|| format!("{} is not under a target directory", executable.display()))?;
    let fresh = base.join(format!("fresh-{}", std::process::id()));
    remove_dir_if_present(&fresh)?;
    fs::create_dir_all(&fresh)
        .map_err(|error| format!("cannot create {}: {error}", fresh.display()))?;
    write_v2(&fresh.join("v2")).map_err(|error| format!("QSL write_v2 failed: {error}"))?;
    let sums = fresh.join("v2").join(PUBLISHED_CHECKSUMS_FILE);
    let bytes =
        fs::read(&sums).map_err(|error| format!("cannot read {}: {error}", sums.display()))?;
    let name: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let published = base.join(format!("handoff-{name}"));
    if !published.is_dir() {
        // Losing a race to a concurrent process leaves the same content there.
        let _ = fs::rename(&fresh, &published);
    }
    remove_dir_if_present(&fresh)?;
    if published.is_dir() {
        Ok(published.join("v2"))
    } else {
        Err(format!(
            "cannot publish the handoff at {}",
            published.display()
        ))
    }
}

fn remove_dir_if_present(directory: &Path) -> Result<(), String> {
    match fs::remove_dir_all(directory) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(format!("cannot remove {}: {error}", directory.display()))
        }
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// The loaded handoff
// ---------------------------------------------------------------------------

/// A's handoff, with every original input retained as its published bytes.
#[derive(Debug)]
pub struct Handoff {
    offer: Vec<u8>,
    selection: SelectionV2,
    /// A's selected ceilings, narrowed to this platform once at load.
    limits: artifact::Limits,
    /// Original source text per `selection.inherited.sources`, in that order.
    source_text: Vec<String>,
    /// Original dependency bytes per `selection.inherited.dependencies`.
    dependency_bytes: Vec<Vec<u8>>,
    /// The model source's original bytes and its re-admitted native model.
    model_source_bytes: Vec<u8>,
    model: NativeModel,
}

impl Handoff {
    /// Load a complete handoff written by the pinned producer's public writer.
    ///
    /// QSL's committed `PUBLISHED_HANDOFF` carries no `dependencies/` bytes, so
    /// it cannot be admitted. `handoff::write_v2` (QSL feature `handoff-writer`,
    /// enabled on this crate's dev-dependency) writes the complete handoff.
    pub fn load() -> Result<Self, Absence> {
        Self::load_from(written_handoff()?)
    }

    /// Load the handoff published under `root`.
    pub fn load_from(root: &Path) -> Result<Self, Absence> {
        if !root.is_dir() {
            return Err(Absence::MissingRoot(root.to_path_buf()));
        }
        verify_checksums(root)?;
        let offer = read_member(root, PUBLISHED_OFFER_FILE, "offer")?;
        read_json::<w::ArtifactRef>(
            root,
            PUBLISHED_ARTIFACT_REFERENCE_FILE,
            "artifact reference",
        )?;
        let selection: SelectionV2 =
            read_json(root, PUBLISHED_SELECTION_FILE, "independent selection")?;
        let mutations: MutationManifest =
            read_json(root, PUBLISHED_MUTATION_MANIFEST_FILE, "mutation manifest")?;
        if mutations.format != MUTATION_MANIFEST_FORMAT {
            return Err(Absence::UnknownMutationFormat {
                format: mutations.format,
            });
        }
        // The manifest states its cases against named base members. If those are
        // not the members B loaded, the declared refusal identities belong to a
        // different handoff and cannot be asserted against this one.
        for (member, named, loaded) in [
            ("base_offer", &mutations.base_offer, PUBLISHED_OFFER_FILE),
            (
                "base_artifact",
                &mutations.base_artifact,
                PUBLISHED_ARTIFACT_REFERENCE_FILE,
            ),
            (
                "independent_selection",
                &mutations.independent_selection,
                PUBLISHED_SELECTION_FILE,
            ),
        ] {
            if named != loaded {
                return Err(Absence::ManifestBase {
                    member: member.to_owned(),
                    named: named.clone(),
                });
            }
        }
        // The ceilings below are counted under a named accounting contract. A
        // different contract reinterprets every counter, so it is refused rather
        // than carried across.
        if selection.limits.accounting_version != artifact::ACCOUNTING_VERSION {
            return Err(Absence::AccountingVersion {
                named: selection.limits.accounting_version.clone(),
            });
        }
        let limits = selected_limits(&selection.limits)?;

        // The sidecar is an independent selection, not an oracle: every identity
        // it names is checked against the original bytes A also published. A
        // disagreement is the handoff's defect, reported before any admission.
        let mut source_text = Vec::new();
        for selected in &selection.inherited.sources {
            let bytes = read_member(root, &selected.file, "source")?;
            check_digest(
                root,
                &selected.file,
                "source",
                &bytes,
                selected.source.artifact.digest,
            )?;
            source_text.push(
                String::from_utf8(bytes).map_err(|error| Absence::Unreadable {
                    path: root.join(&selected.file),
                    cause: error.to_string(),
                })?,
            );
        }

        let mut dependency_bytes = Vec::new();
        for selected in &selection.inherited.dependencies {
            let bytes = read_member(root, &selected.file, "dependency")?;
            check_digest(
                root,
                &selected.file,
                "dependency",
                &bytes,
                selected.artifact.digest,
            )?;
            dependency_bytes.push(bytes);
        }

        let model_file = selection.inherited.model.source_file.clone();
        let model_source_bytes = read_member(root, &model_file, "model source")?;
        check_digest(
            root,
            &model_file,
            "model source",
            &model_source_bytes,
            selection.inherited.model.source.artifact.digest,
        )?;

        let mut clock_identities: Vec<&str> = Vec::new();
        for selected in &selection.temporal {
            let bytes = read_member(root, &selected.clock_input.file, "clock input")?;
            let actual = ByteDigest::of(&bytes).to_string();
            if actual != selected.clock_input.digest {
                return Err(Absence::InputDigest {
                    role: "clock input".to_owned(),
                    path: root.join(&selected.clock_input.file),
                    selected: selected.clock_input.digest.clone(),
                    actual,
                });
            }
            // Two temporal rows selecting one clock identity would let one row's
            // clock outcome stand in for another's.
            let identity = selected.clock_input.identity.as_str();
            if clock_identities.contains(&identity) {
                return Err(Absence::RepeatedClockIdentity {
                    identity: identity.to_owned(),
                });
            }
            clock_identities.push(identity);
        }

        let model = admit_model(&selection.inherited.model, &model_source_bytes, limits)?;

        Ok(Self {
            offer,
            selection,
            limits,
            source_text,
            dependency_bytes,
            model_source_bytes,
            model,
        })
    }

    /// The offered `/2` bytes, exactly as published.
    pub fn offer(&self) -> &[u8] {
        &self.offer
    }

    /// A's accepted ceilings for this handoff.
    pub fn limits(&self) -> artifact::Limits {
        self.limits
    }

    /// Stage one: declaration tables and the foreign model source, borrowed
    /// from the retained originals.
    pub fn declarations(&self) -> Declarations<'_> {
        let sources = self
            .selection
            .inherited
            .sources
            .iter()
            .map(|selected| {
                selected
                    .declarations
                    .iter()
                    .map(expected_declaration)
                    .collect()
            })
            .collect();
        let temporal = self
            .selection
            .temporal
            .iter()
            .map(|selected| expected_declaration(&selected.declaration))
            .collect();
        Declarations {
            sources,
            temporal,
            foreign: artifact::ExpectedForeignSource {
                artifact: &self.selection.inherited.model.source.artifact,
                formal: &self.selection.inherited.model.source.formal,
                bytes: &self.model_source_bytes,
            },
        }
    }

    /// Stage two: the inventories `Expected` borrows.
    pub fn inventories<'a>(&'a self, declarations: &'a Declarations<'a>) -> Inventories<'a> {
        let sources = self
            .selection
            .inherited
            .sources
            .iter()
            .zip(&self.source_text)
            .zip(&declarations.sources)
            .map(|((selected, text), declared)| artifact::ExpectedSource {
                artifact: &selected.source.artifact,
                native: &selected.source.native,
                path: &selected.source.path,
                formal: &selected.source.formal,
                text,
                declarations: declared,
            })
            .collect();
        let dependencies = self
            .selection
            .inherited
            .dependencies
            .iter()
            .zip(&self.dependency_bytes)
            .map(|(selected, bytes)| artifact::SuppliedDependency {
                artifact: &selected.artifact,
                bytes,
                requires: &selected.requires,
            })
            .collect();
        let models = vec![artifact::AdmittedModel {
            artifact: &self.selection.inherited.model.artifact,
            model: &self.model,
            source: &declarations.foreign,
        }];
        let temporal = self
            .selection
            .temporal
            .iter()
            .zip(&declarations.temporal)
            .map(|(selected, declared)| v2::ExpectedTemporal {
                source: &selected.source,
                declaration: declared,
                definition: v2::ExpectedDefinition {
                    identity: &selected.definition_identity,
                    revision: &selected.definition_revision,
                    artifact: &selected.definition_artifact,
                },
                clock: &selected.clock_input.configuration,
            })
            .collect();
        Inventories {
            sources,
            dependencies,
            models,
            temporal,
        }
    }

    /// Stage three: the complete independent expectation for the base offer.
    pub fn expected<'a>(&'a self, inventories: &'a Inventories<'a>) -> v2::Expected<'a> {
        self.expected_for(inventories, &self.selection.inherited.artifact)
    }

    /// The same expectation with one substituted external artifact reference,
    /// as a mutation case's independently selected seal requires.
    pub fn expected_for<'a>(
        &'a self,
        inventories: &'a Inventories<'a>,
        seal: &'a w::ArtifactRef,
    ) -> v2::Expected<'a> {
        v2::Expected {
            inherited: artifact::Expected {
                artifact: seal,
                contract: &self.selection.inherited.contract,
                baseline: &self.selection.inherited.baseline,
                producer: &self.selection.inherited.producer,
                language: &self.selection.inherited.language,
                sources: &inventories.sources,
                dependencies: &inventories.dependencies,
                models: &inventories.models,
                domain_packages: &[],
            },
            temporal: &inventories.temporal,
        }
    }
}

/// Stage-one storage: nothing here borrows from stage two.
#[derive(Debug)]
pub struct Declarations<'a> {
    pub sources: Vec<Vec<artifact::ExpectedDeclaration<'a>>>,
    pub temporal: Vec<artifact::ExpectedDeclaration<'a>>,
    pub foreign: artifact::ExpectedForeignSource<'a>,
}

/// Stage-two storage: the four inventories `v2::Expected` borrows.
#[derive(Debug)]
pub struct Inventories<'a> {
    pub sources: Vec<artifact::ExpectedSource<'a>>,
    pub dependencies: Vec<artifact::SuppliedDependency<'a>>,
    pub models: Vec<artifact::AdmittedModel<'a>>,
    pub temporal: Vec<v2::ExpectedTemporal<'a>>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn expected_declaration(selected: &SelectedDeclaration) -> artifact::ExpectedDeclaration<'_> {
    artifact::ExpectedDeclaration {
        name: &selected.name,
        span: &selected.span,
        requirement: &selected.requirement,
        clause: &selected.clause,
        execution: &selected.execution,
    }
}

/// Resolve one handoff-relative member path.
///
/// Every path B joins onto the handoff root arrives from a published file — the
/// checksum manifest, the selection sidecar, the mutation manifest. A path that
/// is absolute, or that carries `..`, `.` or an empty segment, can name bytes
/// the handoff does not publish and that no coverage sweep over the root would
/// ever see, so only a plain sequence of named segments is accepted. Accepting
/// exactly one spelling per file also makes the manifest's paths and the walk's
/// paths directly comparable.
fn member_path(root: &Path, relative: &str, role: &str) -> Result<PathBuf, Absence> {
    let escaping = || Absence::EscapingMember {
        role: role.to_owned(),
        named: relative.to_owned(),
    };
    if relative.is_empty() || relative.contains('\\') {
        return Err(escaping());
    }
    let mut resolved = root.to_path_buf();
    let mut segments = 0usize;
    for segment in relative.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(escaping());
        }
        segments += 1;
        if segments > MAX_DEPTH {
            return Err(Absence::TooDeep {
                relative: relative.to_owned(),
            });
        }
        resolved.push(segment);
    }
    Ok(resolved)
}

/// Verify A's published checksum manifest before any member is trusted.
///
/// Every listed member must hash to its listed digest, and every file the
/// handoff carries must be listed: an unlisted file — a mutation fixture above
/// all — would otherwise be submitted to the reader with no published integrity
/// claim behind it.
///
/// This establishes that the directory is internally whole and self-consistent.
/// It does not establish provenance: the manifest is verified only against
/// itself. Provenance is anchored separately by loading only A's exported path
/// from the exactly pinned dependency source.
fn verify_checksums(root: &Path) -> Result<(), Absence> {
    // The walk runs first: it refuses symbolic links, whose bytes are not the
    // published ones, before any member is read through one.
    let present = published_files(root, root, 0)?;

    let text = read_member(root, PUBLISHED_CHECKSUMS_FILE, "checksum manifest")?;
    let text = String::from_utf8(text).map_err(|error| Absence::Unreadable {
        path: root.join(PUBLISHED_CHECKSUMS_FILE),
        cause: error.to_string(),
    })?;

    let mut listed = BTreeSet::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        // Lowercase hex only: that is the spelling `ByteDigest`'s `LowerHex`
        // renders, and accepting a second spelling would compare unequal.
        let (digest, name) = line
            .split_once("  ")
            .filter(|(digest, name)| {
                digest.len() == 64
                    && digest
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    && !name.is_empty()
            })
            .ok_or_else(|| Absence::ChecksumLine {
                line: line.to_owned(),
            })?;
        let relative = name.strip_prefix("./").unwrap_or(name).to_owned();
        let path = member_path(root, &relative, "checksummed")?;
        let bytes = fs::read(&path).map_err(|_| Absence::MissingMember {
            role: "checksummed".to_owned(),
            path,
        })?;
        let actual = format!("{:x}", ByteDigest::of(&bytes));
        if actual != digest {
            return Err(Absence::Checksum {
                relative,
                listed: digest.to_owned(),
                actual,
            });
        }
        listed.insert(relative);
    }

    for relative in present {
        if relative != PUBLISHED_CHECKSUMS_FILE && !listed.contains(&relative) {
            return Err(Absence::Unlisted { relative });
        }
    }
    Ok(())
}

/// How many directories below the handoff root B walks and joins.
const MAX_DEPTH: usize = 32;

/// Every file under `directory`, as a handoff-relative slash-separated path.
///
/// Symbolic links are refused rather than followed: a link's bytes are not the
/// ones the handoff publishes, its target need not lie under the root, and a
/// directory cycle would otherwise recurse without end.
fn published_files(root: &Path, directory: &Path, depth: usize) -> Result<Vec<String>, Absence> {
    let unreadable = |error: std::io::Error| Absence::Unreadable {
        path: directory.to_path_buf(),
        cause: error.to_string(),
    };
    let mut found = Vec::new();
    for entry in fs::read_dir(directory).map_err(unreadable)? {
        let path = entry.map_err(unreadable)?.path();
        let relative = relative_name(root, &path);
        if fs::symlink_metadata(&path)
            .map_err(unreadable)?
            .is_symlink()
        {
            return Err(Absence::SymbolicLink { relative });
        }
        if path.is_dir() {
            if depth + 1 > MAX_DEPTH {
                return Err(Absence::TooDeep { relative });
            }
            found.extend(published_files(root, &path, depth + 1)?);
        } else {
            found.push(relative);
        }
    }
    Ok(found)
}

/// `path` as a handoff-relative slash-separated name.
fn relative_name(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn read_member(root: &Path, relative: &str, role: &str) -> Result<Vec<u8>, Absence> {
    let path = member_path(root, relative, role)?;
    fs::read(&path).map_err(|_| Absence::MissingMember {
        role: role.to_owned(),
        path,
    })
}

fn read_json<T: serde::de::DeserializeOwned>(
    root: &Path,
    relative: &str,
    role: &str,
) -> Result<T, Absence> {
    let bytes = read_member(root, relative, role)?;
    serde_json::from_slice(&bytes).map_err(|error| Absence::Unreadable {
        path: root.join(relative),
        cause: error.to_string(),
    })
}

fn check_digest(
    root: &Path,
    relative: &str,
    role: &str,
    bytes: &[u8],
    selected: ByteDigest,
) -> Result<(), Absence> {
    let actual = ByteDigest::of(bytes);
    if actual == selected {
        return Ok(());
    }
    Err(Absence::InputDigest {
        role: role.to_owned(),
        path: root.join(relative),
        selected: selected.to_string(),
        actual: actual.to_string(),
    })
}

/// Re-admit the model through A's public reader. B holds no model of its own:
/// the `AdmittedModel` an expectation needs is constructor-private to A, so the
/// only way to supply it is to execute A's admission over the original bytes.
fn admit_model(
    selected: &SelectedModel,
    bytes: &[u8],
    limits: artifact::Limits,
) -> Result<NativeModel, Absence> {
    let source = Source::read(
        SourceIdentity {
            authority: selected.source.native.authority.clone(),
            revision_namespace: selected.source.native.revision_namespace.clone(),
            identity: selected.source.native.identity.clone(),
            revision: selected.source.native.revision.clone(),
        },
        selected.source.path.clone(),
        bytes,
        limits.source_bytes,
    )
    .map_err(|error| Absence::Model {
        cause: error.to_string(),
    })?;
    let document =
        SourceDocumentId::new(selected.source.formal.document.clone()).map_err(|error| {
            Absence::Model {
                cause: error.to_string(),
            }
        })?;
    let revision = selected
        .source
        .formal
        .revision
        .value
        .parse::<u64>()
        .map_err(|error| Absence::Model {
            cause: format!(
                "formal revision {} is not a positive integer: {error}",
                selected.source.formal.revision.value
            ),
        })?;
    let revision = SourceRevision::new(revision).map_err(|error| Absence::Model {
        cause: error.to_string(),
    })?;
    let formal = FormalSource::new(source, IrSourceIdentity::new(document, revision));
    model_source::read(
        formal,
        &selected.source_format,
        ModelSourceLimits::default(),
    )
    .map_err(|error| Absence::Model {
        cause: error.to_string(),
    })?
    .admit(ModelLimits::default())
    .map_err(|error| Absence::Model {
        cause: error.to_string(),
    })
}
