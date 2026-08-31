use std::{collections::BTreeMap, fmt};

use serde::Serialize;

use crate::{
    CanonicalBody, CanonicalDigest, ContractPackage, Diagnostic, DiagnosticCode, RequirementRef,
    SourceSpan,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ArtifactId(String);

impl ArtifactId {
    pub fn new(value: impl Into<String>) -> Result<Self, Diagnostic> {
        let value = value.into();
        let mut characters = value.chars();
        let valid = matches!(characters.next(), Some(first) if first.is_ascii_alphabetic())
            && characters.all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
            });
        if valid {
            Ok(Self(value))
        } else {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidIdentifier,
                "artifact ID violates the contract identifier grammar",
                "artifact.id",
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtifactId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceDepth {
    Shallow,
    Deep {
        requirement_digest: CanonicalDigest,
        digest_span: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactTrace {
    artifact_id: ArtifactId,
    source: SourceSpan,
    target: RequirementRef,
    target_span: SourceSpan,
    depth: TraceDepth,
}

impl ArtifactTrace {
    pub fn shallow(
        artifact_id: ArtifactId,
        source: SourceSpan,
        target: RequirementRef,
        target_span: SourceSpan,
    ) -> Self {
        Self {
            artifact_id,
            source,
            target,
            target_span,
            depth: TraceDepth::Shallow,
        }
    }

    pub fn deep(
        artifact_id: ArtifactId,
        source: SourceSpan,
        target: RequirementRef,
        target_span: SourceSpan,
        requirement_digest: CanonicalDigest,
        digest_span: SourceSpan,
    ) -> Self {
        Self {
            artifact_id,
            source,
            target,
            target_span,
            depth: TraceDepth::Deep {
                requirement_digest,
                digest_span,
            },
        }
    }

    pub fn artifact_id(&self) -> &ArtifactId {
        &self.artifact_id
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }

    pub fn target(&self) -> &RequirementRef {
        &self.target
    }

    pub fn target_span(&self) -> &SourceSpan {
        &self.target_span
    }

    pub fn depth(&self) -> &TraceDepth {
        &self.depth
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageClass {
    Shallow,
    Deep,
    Uncovered,
    Orphaned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrphanReason {
    CrossPackage,
    MissingRequirement,
    StaleRevision,
    DuplicateArtifact,
    DigestMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RequirementCoverageRow {
    reference: RequirementRef,
    class: CoverageClass,
}

impl RequirementCoverageRow {
    pub fn reference(&self) -> &RequirementRef {
        &self.reference
    }

    pub const fn class(&self) -> CoverageClass {
        self.class
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArtifactCoverageRow {
    artifact_id: ArtifactId,
    target: RequirementRef,
    class: CoverageClass,
    orphan_reason: Option<OrphanReason>,
}

impl ArtifactCoverageRow {
    pub fn artifact_id(&self) -> &ArtifactId {
        &self.artifact_id
    }

    pub fn target(&self) -> &RequirementRef {
        &self.target
    }

    pub const fn class(&self) -> CoverageClass {
        self.class
    }

    pub const fn orphan_reason(&self) -> Option<OrphanReason> {
        self.orphan_reason
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CoverageReport {
    requirements: Vec<RequirementCoverageRow>,
    artifacts: Vec<ArtifactCoverageRow>,
}

impl CoverageReport {
    pub fn requirements(&self) -> &[RequirementCoverageRow] {
        &self.requirements
    }

    pub fn artifacts(&self) -> &[ArtifactCoverageRow] {
        &self.artifacts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageResult {
    report: CoverageReport,
    diagnostics: Vec<Diagnostic>,
}

impl CoverageResult {
    pub fn report(&self) -> &CoverageReport {
        &self.report
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

pub fn classify_coverage<B: CanonicalBody>(
    package: &ContractPackage<B>,
    traces: &[ArtifactTrace],
) -> Result<CoverageResult, Diagnostic> {
    let mut requirement_rows = package
        .requirements()
        .iter()
        .map(|requirement| {
            (
                package.requirement_ref(requirement),
                CoverageClass::Uncovered,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let counts = traces.iter().fold(BTreeMap::new(), |mut counts, trace| {
        *counts.entry(trace.artifact_id.clone()).or_insert(0_usize) += 1;
        counts
    });
    let mut artifact_rows = BTreeMap::new();
    let mut diagnostics = Vec::new();

    for trace in traces {
        if counts.get(trace.artifact_id()).copied().unwrap_or(0) > 1 {
            if artifact_rows.contains_key(trace.artifact_id()) {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateArtifactTrace,
                        "artifact ID is duplicated in this classification input",
                        "artifact.id",
                    )
                    .at_span(trace.source()),
                );
            } else {
                artifact_rows.insert(
                    trace.artifact_id.clone(),
                    artifact_row(
                        trace,
                        CoverageClass::Orphaned,
                        Some(OrphanReason::DuplicateArtifact),
                    ),
                );
            }
            continue;
        }

        let requirement =
            match package.resolve_requirement(trace.target(), Some(trace.target_span())) {
                Ok(requirement) => requirement,
                Err(diagnostic) => {
                    let Some(reason) = orphan_reason(diagnostic.code) else {
                        return Err(diagnostic);
                    };
                    diagnostics.push(diagnostic);
                    artifact_rows.insert(
                        trace.artifact_id.clone(),
                        artifact_row(trace, CoverageClass::Orphaned, Some(reason)),
                    );
                    continue;
                }
            };

        let class = match trace.depth() {
            TraceDepth::Shallow => CoverageClass::Shallow,
            TraceDepth::Deep {
                requirement_digest,
                digest_span,
            } => {
                let current = package.canonical_requirement(requirement)?.digest();
                if current != *requirement_digest {
                    diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::StaleTraceDigest,
                            "trace digest differs from the current requirement digest",
                            "artifact.depth.digest",
                        )
                        .at_span(digest_span),
                    );
                    artifact_rows.insert(
                        trace.artifact_id.clone(),
                        artifact_row(
                            trace,
                            CoverageClass::Orphaned,
                            Some(OrphanReason::DigestMismatch),
                        ),
                    );
                    continue;
                }
                CoverageClass::Deep
            }
        };

        if let Some(current) = requirement_rows.get_mut(trace.target()) {
            if class == CoverageClass::Deep || *current == CoverageClass::Uncovered {
                *current = class;
            }
        }
        artifact_rows.insert(trace.artifact_id.clone(), artifact_row(trace, class, None));
    }

    Ok(CoverageResult {
        report: CoverageReport {
            requirements: requirement_rows
                .into_iter()
                .map(|(reference, class)| RequirementCoverageRow { reference, class })
                .collect(),
            artifacts: artifact_rows.into_values().collect(),
        },
        diagnostics,
    })
}

fn artifact_row(
    trace: &ArtifactTrace,
    class: CoverageClass,
    orphan_reason: Option<OrphanReason>,
) -> ArtifactCoverageRow {
    ArtifactCoverageRow {
        artifact_id: trace.artifact_id.clone(),
        target: trace.target.clone(),
        class,
        orphan_reason,
    }
}

fn orphan_reason(code: DiagnosticCode) -> Option<OrphanReason> {
    match code {
        DiagnosticCode::CrossPackageReference => Some(OrphanReason::CrossPackage),
        DiagnosticCode::OrphanedRequirementReference => Some(OrphanReason::MissingRequirement),
        DiagnosticCode::StaleRequirementRevision => Some(OrphanReason::StaleRevision),
        _ => None,
    }
}
