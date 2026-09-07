use quire_contract_ir::{
    AnchorName, Clause, ClauseId, ClauseKind, ClauseRef, ContractPackage, DependencyIdentity,
    DependencyKind, DependencyName, DependencySource, DiagnosticCode, ExecutionPoint, PackageId,
    ReferenceBody, Requirement, RequirementId, RequirementRef, RequirementRevision, SchemaVersion,
    SemanticIdentity, SourceDocumentId, SourceIdentity, SourceLocation, SourceRevision, SourceSpan,
    ValidationOptions,
};

const REGISTRY: &str = include_str!("../spec/contract/STD-001-diagnostic-registry.md");
const FORBIDDEN_PUBLIC_VOCABULARY: [&str; 8] = [
    "rust", "gumbo", "aadl", "hamr", "solver", "runtime", "usize", "pathbuf",
];

fn source(document: &str, revision: u64) -> SourceIdentity {
    SourceIdentity::new(
        SourceDocumentId::new(document).unwrap(),
        SourceRevision::new(revision).unwrap(),
    )
}

fn span_at(source: &SourceIdentity, start: u64, end: u64) -> SourceSpan {
    SourceSpan::new(
        SourceLocation::new(source.clone(), 1, start as u32 + 1, start).unwrap(),
        SourceLocation::new(source.clone(), 1, end as u32 + 1, end).unwrap(),
    )
    .unwrap()
}

fn requirement_ref(package: &str, requirement: &str, revision: u64) -> RequirementRef {
    RequirementRef::parse(package, requirement, revision).unwrap()
}

fn dependency(reference: RequirementRef, name: &str) -> DependencyIdentity {
    DependencyIdentity::new(
        reference,
        DependencyKind::Input,
        vec![DependencyName::new(name).unwrap()],
    )
    .unwrap()
}

fn valid_package_at(revision: u64) -> ContractPackage<ReferenceBody> {
    let package_id = PackageId::new("agent-ix/flight-contract").unwrap();
    let source = source("flight_contract", 7);
    let requirement_reference = requirement_ref(package_id.as_str(), "REQ_speed", revision);
    let speed = dependency(requirement_reference, "speed");
    let body = ReferenceBody::Composite {
        children: vec![
            ReferenceBody::Reference {
                identity: speed.clone(),
            },
            ReferenceBody::Composite {
                children: vec![ReferenceBody::Reference { identity: speed }],
            },
        ],
    };
    let clause = Clause::new(
        ClauseId::new("speed_positive").unwrap(),
        ClauseKind::Precondition,
        Some(ExecutionPoint::Pre {
            operation: AnchorName::new("update_speed").unwrap(),
        }),
        span_at(&source, 10, 25),
        body,
    )
    .unwrap();
    let requirement = Requirement::new(
        &package_id,
        RequirementId::new("REQ_speed").unwrap(),
        RequirementRevision::new(revision).unwrap(),
        span_at(&source, 0, 30),
        vec![clause],
    )
    .unwrap();
    let unrelated_clause = Clause::new(
        ClauseId::new("audit_note").unwrap(),
        ClauseKind::Information,
        None,
        span_at(&source, 31, 39),
        ReferenceBody::Literal,
    )
    .unwrap();
    let unrelated = Requirement::new(
        &package_id,
        RequirementId::new("REQ_audit").unwrap(),
        RequirementRevision::new(2).unwrap(),
        span_at(&source, 31, 40),
        vec![unrelated_clause],
    )
    .unwrap();
    ContractPackage::new(
        package_id,
        SchemaVersion::V1_0,
        source,
        vec![requirement, unrelated],
    )
    .unwrap()
}

fn valid_package() -> ContractPackage<ReferenceBody> {
    valid_package_at(3)
}

fn package_error(value: &serde_json::Value) -> Vec<quire_contract_ir::Diagnostic> {
    ContractPackage::from_json_str(
        &serde_json::to_string(value).unwrap(),
        ValidationOptions::strict(),
    )
    .unwrap_err()
}

fn diagnostic_code<T: std::fmt::Debug>(
    result: Result<T, quire_contract_ir::Diagnostic>,
) -> DiagnosticCode {
    result.unwrap_err().code
}

/// Tracing: TC-015
/// TC-015.
/// StR-001-VC-1.
/// StR-001-VC-2.
/// FR-011-AC-1.
/// FR-011-AC-2.
/// FR-011-AC-3.
/// FR-012-AC-1.
/// FR-012-AC-2.
/// FR-012-AC-3.
/// FR-012-AC-4.
/// FR-012-AC-6.
/// NFR-002-AC-3.
/// STD-001.
#[test]
fn tc_015_identity_anchor_dependency_and_reference_contract_conforms() {
    let package = valid_package();

    // FR-011-AC-1: valid issue #6 values round-trip structurally through JSON.
    let json = serde_json::to_string_pretty(&package).unwrap();
    let round_trip = ContractPackage::from_json_str(&json, ValidationOptions::strict()).unwrap();
    assert_eq!(round_trip, package);
    assert_eq!(package.schema_version(), SchemaVersion::new(1, 0).unwrap());
    assert_eq!(
        SchemaVersion::new(0, 1).unwrap_err().code,
        DiagnosticCode::InvalidSchemaVersion
    );

    // FR-012-AC-2: recursive traversal deduplicates and orders structural identities.
    let clause = &package.requirements()[0].clauses()[0];
    let dependencies = clause.dependencies();
    assert_eq!(dependencies.len(), 1);

    let requirement_reference = package.requirement_ref(&package.requirements()[0]);
    let expected_dependencies =
        std::iter::once(dependency(requirement_reference.clone(), "speed")).collect();
    assert_eq!(dependencies, expected_dependencies);
    let a = dependency(requirement_reference.clone(), "alpha");
    let z = dependency(requirement_reference.clone(), "zeta");
    let ordering_probe = ReferenceBody::Composite {
        children: vec![
            ReferenceBody::Reference {
                identity: z.clone(),
            },
            ReferenceBody::Reference {
                identity: a.clone(),
            },
            ReferenceBody::Reference {
                identity: z.clone(),
            },
        ],
    };
    assert_eq!(
        ordering_probe
            .dependencies()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![a.clone(), z]
    );

    // FR-011-AC-2: a revision is part of every downstream identity.
    let advanced = requirement_reference.advance(4).unwrap();
    assert_ne!(advanced, requirement_reference);
    let current_clause_ref = package.clause_ref(&package.requirements()[0], clause);
    let advanced_clause_ref = ClauseRef::new(advanced.clone(), current_clause_ref.clause().clone());
    assert_ne!(advanced_clause_ref, current_clause_ref);
    assert_ne!(dependency(advanced, "alpha"), a);
    let advanced_package = valid_package_at(4);
    assert_eq!(
        package.requirement_ref(&package.requirements()[1]),
        advanced_package.requirement_ref(&advanced_package.requirements()[1])
    );
    assert_eq!(
        package.clause_ref(
            &package.requirements()[1],
            &package.requirements()[1].clauses()[0]
        ),
        advanced_package.clause_ref(
            &advanced_package.requirements()[1],
            &advanced_package.requirements()[1].clauses()[0]
        )
    );
    assert_eq!(
        package.requirements()[1].clauses()[0].dependencies(),
        advanced_package.requirements()[1].clauses()[0].dependencies()
    );
    assert_eq!(
        diagnostic_code(requirement_reference.revision().advance(3)),
        DiagnosticCode::InvalidRequirementRevision
    );

    // FR-011-AC-3: all identity and revision failures are named.
    assert_eq!(
        PackageId::new("").unwrap_err().code,
        DiagnosticCode::InvalidPackageNamespace
    );
    assert_eq!(
        RequirementId::new("bad id").unwrap_err().code,
        DiagnosticCode::InvalidIdentifier
    );
    for diagnostic in [
        SourceDocumentId::new("9leading").unwrap_err(),
        RequirementId::new("").unwrap_err(),
        ClauseId::new("").unwrap_err(),
        AnchorName::new("bad name").unwrap_err(),
    ] {
        assert_eq!(diagnostic.code, DiagnosticCode::InvalidIdentifier);
        assert!(!diagnostic.path.contains("Id"));
        assert!(!diagnostic.path.contains("Name"));
    }
    for namespace in [
        "agent//contract",
        "agent/",
        "agent/../contract",
        "agent/./contract",
    ] {
        assert_eq!(
            PackageId::new(namespace).unwrap_err().code,
            DiagnosticCode::InvalidPackageNamespace
        );
    }
    assert_eq!(
        SourceRevision::new(0).unwrap_err().code,
        DiagnosticCode::InvalidSourceRevision
    );
    assert_eq!(
        RequirementRevision::new(0).unwrap_err().code,
        DiagnosticCode::InvalidRequirementRevision
    );

    let duplicate = package.requirements()[0].clone();
    let duplicate_error = ContractPackage::new(
        package.id().clone(),
        package.schema_version(),
        package.source().clone(),
        vec![package.requirements()[0].clone(), duplicate],
    )
    .unwrap_err();
    assert!(duplicate_error
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::DuplicateRequirement));

    let duplicate_clause_error = Requirement::new(
        package.id(),
        RequirementId::new("REQ_duplicate_clause").unwrap(),
        RequirementRevision::new(1).unwrap(),
        span_at(package.source(), 0, 30),
        vec![clause.clone(), clause.clone()],
    )
    .unwrap_err();
    assert_eq!(duplicate_clause_error.code, DiagnosticCode::DuplicateClause);
    assert_eq!(duplicate_clause_error.related.len(), 1);

    // FR-012-AC-1/4: the complete clause/anchor compatibility table is enforced.
    let source_span = span_at(package.source(), 40, 50);
    let literal = ReferenceBody::Literal;
    let floating = Clause::new(
        ClauseId::new("floating").unwrap(),
        ClauseKind::Assertion,
        None,
        source_span.clone(),
        literal.clone(),
    )
    .unwrap_err();
    assert_eq!(floating.code, DiagnosticCode::FloatingExecutableClause);
    assert_eq!(floating.span.as_deref(), Some(&source_span));

    let anchored_information = Clause::new(
        ClauseId::new("information").unwrap(),
        ClauseKind::Information,
        Some(ExecutionPoint::Handler {
            name: AnchorName::new("tick").unwrap(),
        }),
        source_span.clone(),
        literal.clone(),
    )
    .unwrap_err();
    assert_eq!(
        anchored_information.code,
        DiagnosticCode::InformationalClauseAnchored
    );

    let incompatible = Clause::new(
        ClauseId::new("bad_post").unwrap(),
        ClauseKind::Postcondition,
        Some(ExecutionPoint::Pre {
            operation: AnchorName::new("tick").unwrap(),
        }),
        source_span.clone(),
        literal.clone(),
    )
    .unwrap_err();
    assert_eq!(incompatible.code, DiagnosticCode::IncompatibleClauseAnchor);

    let clause_kinds = [
        ClauseKind::Precondition,
        ClauseKind::Postcondition,
        ClauseKind::Invariant,
        ClauseKind::Assertion,
        ClauseKind::Case,
        ClauseKind::Information,
    ];
    let anchor_kinds = [
        None,
        Some(ExecutionPoint::Initialization {
            name: AnchorName::new("boot").unwrap(),
        }),
        Some(ExecutionPoint::Handler {
            name: AnchorName::new("tick").unwrap(),
        }),
        Some(ExecutionPoint::Pre {
            operation: AnchorName::new("op").unwrap(),
        }),
        Some(ExecutionPoint::Post {
            operation: AnchorName::new("op").unwrap(),
        }),
    ];
    for (kind_index, kind) in clause_kinds.into_iter().enumerate() {
        for (anchor_index, anchor) in anchor_kinds.iter().cloned().enumerate() {
            let accepted = match kind {
                ClauseKind::Precondition => anchor_index == 3,
                ClauseKind::Postcondition => anchor_index == 4,
                ClauseKind::Invariant => matches!(anchor_index, 1 | 2),
                ClauseKind::Assertion => anchor_index != 0,
                ClauseKind::Case => anchor_index == 2,
                ClauseKind::Information => anchor_index == 0,
            };
            let result = Clause::new(
                ClauseId::new(format!("matrix_{kind_index}_{anchor_index}")).unwrap(),
                kind,
                anchor,
                source_span.clone(),
                literal.clone(),
            );
            if accepted {
                assert!(result.is_ok(), "rejected {kind:?} anchor {anchor_index}");
            } else {
                let expected = if kind == ClauseKind::Information {
                    DiagnosticCode::InformationalClauseAnchored
                } else if anchor_index == 0 {
                    DiagnosticCode::FloatingExecutableClause
                } else {
                    DiagnosticCode::IncompatibleClauseAnchor
                };
                assert_eq!(
                    result.unwrap_err().code,
                    expected,
                    "wrong result for {kind:?} anchor {anchor_index}"
                );
            }
        }
    }

    // FR-012-AC-6: invalid endpoints fail before entering a validated span.
    assert_eq!(
        SourceLocation::new(package.source().clone(), 0, 1, 0)
            .unwrap_err()
            .code,
        DiagnosticCode::InvalidSourceSpan
    );
    let later = SourceLocation::new(package.source().clone(), 2, 1, 20).unwrap();
    let earlier = SourceLocation::new(package.source().clone(), 1, 1, 10).unwrap();
    assert_eq!(
        SourceSpan::new(later, earlier).unwrap_err().code,
        DiagnosticCode::InvalidSourceSpan
    );
    let other_source = source("other_document", 1);
    assert_eq!(
        SourceSpan::new(
            SourceLocation::new(package.source().clone(), 1, 1, 0).unwrap(),
            SourceLocation::new(other_source, 1, 2, 1).unwrap(),
        )
        .unwrap_err()
        .code,
        DiagnosticCode::InvalidSourceSpan
    );

    // FR-012-AC-3: grammar, ownership, revision and existence resolve distinctly.
    assert_eq!(
        RequirementRef::parse(package.id().as_str(), "bad id", 1)
            .unwrap_err()
            .code,
        DiagnosticCode::InvalidIdentifier
    );
    // STD-001 precedence: grammar failures win before package/reference resolution.
    let invalid_package_reference = RequirementRef::parse("", "REQ_speed", 1).unwrap_err();
    assert_eq!(
        invalid_package_reference.code,
        DiagnosticCode::InvalidPackageNamespace
    );
    assert_eq!(invalid_package_reference.path, "reference.package");
    assert_eq!(
        ClauseId::new("").unwrap_err().code,
        DiagnosticCode::InvalidIdentifier
    );
    assert_eq!(
        DependencyIdentity::new(requirement_reference.clone(), DependencyKind::State, vec![])
            .unwrap_err()
            .code,
        DiagnosticCode::MalformedReference
    );

    let cross_package = requirement_ref("other/package", "REQ_speed", 3);
    let cross_package_diagnostic = package
        .resolve_requirement(&cross_package, Some(&source_span))
        .unwrap_err();
    assert_eq!(
        cross_package_diagnostic.code,
        DiagnosticCode::CrossPackageReference
    );
    assert_eq!(cross_package_diagnostic.span.as_deref(), Some(&source_span));
    let stale = requirement_ref(package.id().as_str(), "REQ_speed", 2);
    let stale_diagnostic = package
        .resolve_requirement(&stale, Some(&source_span))
        .unwrap_err();
    assert_eq!(
        stale_diagnostic.code,
        DiagnosticCode::StaleRequirementRevision
    );
    assert_eq!(stale_diagnostic.span.as_deref(), Some(&source_span));
    assert_eq!(stale_diagnostic.related.len(), 1);

    let orphaned_requirement = requirement_ref(package.id().as_str(), "REQ_missing", 1);
    let orphaned_requirement_diagnostic = package
        .resolve_requirement(&orphaned_requirement, Some(&source_span))
        .unwrap_err();
    assert_eq!(
        orphaned_requirement_diagnostic.code,
        DiagnosticCode::OrphanedRequirementReference
    );
    assert_eq!(
        orphaned_requirement_diagnostic.span.as_deref(),
        Some(&source_span)
    );
    let orphaned_clause = ClauseRef::new(
        requirement_reference,
        ClauseId::new("missing_clause").unwrap(),
    );
    let orphaned_clause_diagnostic = package
        .resolve_clause(&orphaned_clause, Some(&source_span))
        .unwrap_err();
    assert_eq!(
        orphaned_clause_diagnostic.code,
        DiagnosticCode::OrphanedClauseReference
    );
    assert_eq!(
        orphaned_clause_diagnostic.span.as_deref(),
        Some(&source_span)
    );

    // STD-001 and NFR-002-AC-3: exact codes are serialized, registered, and neutral.
    for code in DiagnosticCode::ALL {
        let row = format!("| `{}` |", code.as_str());
        assert_eq!(REGISTRY.matches(&row).count(), 1, "registry row {row}");
        let encoded = serde_json::to_string(code).unwrap();
        assert_eq!(encoded, format!("{:?}", code.as_str()));
        assert_eq!(
            serde_json::from_str::<DiagnosticCode>(&encoded).unwrap(),
            *code
        );
    }
    assert_eq!(
        REGISTRY
            .lines()
            .filter(|line| line.starts_with("| `"))
            .count(),
        DiagnosticCode::ALL.len()
    );
    let all_clause_kinds = [
        ClauseKind::Precondition,
        ClauseKind::Postcondition,
        ClauseKind::Invariant,
        ClauseKind::Assertion,
        ClauseKind::Case,
        ClauseKind::Information,
    ];
    let all_dependency_kinds = [
        DependencyKind::Input,
        DependencyKind::State,
        DependencyKind::Field,
        DependencyKind::EnumVariant,
        DependencyKind::PureFunction,
    ];
    let all_execution_points = [
        ExecutionPoint::Initialization {
            name: AnchorName::new("boot").unwrap(),
        },
        ExecutionPoint::Handler {
            name: AnchorName::new("tick").unwrap(),
        },
        ExecutionPoint::Pre {
            operation: AnchorName::new("update").unwrap(),
        },
        ExecutionPoint::Post {
            operation: AnchorName::new("update").unwrap(),
        },
    ];
    let current_requirement = package.requirement_ref(&package.requirements()[0]);
    let current_clause = package.clause_ref(
        &package.requirements()[0],
        &package.requirements()[0].clauses()[0],
    );
    let all_identity_kinds = [
        SemanticIdentity::Package {
            package: package.id().clone(),
        },
        SemanticIdentity::Requirement {
            reference: current_requirement.clone(),
        },
        SemanticIdentity::Clause {
            reference: current_clause,
        },
        SemanticIdentity::Dependency {
            identity: dependency(current_requirement, "speed"),
        },
    ];
    let serialized_vocabulary = serde_json::to_string(&(
        json,
        DiagnosticCode::ALL,
        all_clause_kinds,
        all_dependency_kinds,
        all_execution_points,
        all_identity_kinds,
    ))
    .unwrap()
    .to_ascii_lowercase();
    for forbidden in FORBIDDEN_PUBLIC_VOCABULARY {
        assert!(
            !serialized_vocabulary.contains(forbidden),
            "forbidden public vocabulary {forbidden}"
        );
    }
    let diagnostic_json = serde_json::to_string(&stale_diagnostic).unwrap();
    assert!(diagnostic_json.contains("\"code\":\"stale_requirement_revision\""));
    assert!(diagnostic_json.contains("\"path\":\"reference.revision\""));
    for forbidden in FORBIDDEN_PUBLIC_VOCABULARY {
        assert!(!diagnostic_json.to_ascii_lowercase().contains(forbidden));
    }
}

/// Tracing: TC-015
/// TC-015.
/// FR-011-AC-1.
/// FR-011-AC-3.
/// FR-012-AC-3.
/// FR-012-AC-4.
/// FR-012-AC-6.
/// STD-001.
#[test]
fn tc_015_untrusted_json_preserves_structured_failure_codes() {
    let package = valid_package();
    let valid = serde_json::to_value(&package).unwrap();

    let mut candidate = valid.clone();
    candidate["ignored"] = serde_json::json!([0]);
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InvalidWireFormat,
        "the public decoder and closed published schema must reject unknown fields"
    );

    let mut candidate = valid.clone();
    candidate["schema_version"]["major"] = 0.into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InvalidSchemaVersion
    );

    let mut candidate = valid.clone();
    candidate["source"]["revision"] = 0.into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InvalidSourceRevision
    );

    let mut candidate = valid.clone();
    candidate["id"] = "agent/../contract".into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InvalidPackageNamespace
    );

    let mut candidate = valid.clone();
    candidate["requirements"][0]["id"] = "bad id".into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InvalidIdentifier
    );

    let mut candidate = valid.clone();
    candidate["requirements"][0]["clauses"][0]["id"] = "".into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InvalidIdentifier
    );

    let mut candidate = valid.clone();
    candidate["requirements"][0]["source"]["start"]["byte_offset"] = 100.into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InvalidSourceSpan
    );

    let mut candidate = valid.clone();
    candidate["requirements"][0]["source"]["end"]["source"]["document"] = "other_document".into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InvalidSourceSpan
    );

    let dependency = &mut candidate_dependency(valid.clone());
    dependency["path"] = serde_json::json!([]);
    let malformed = package_error(&dependency_package(valid.clone(), dependency.clone()));
    assert_eq!(malformed[0].code, DiagnosticCode::MalformedReference);
    assert_eq!(
        malformed[0].span.as_deref(),
        Some(package.requirements()[0].clauses()[0].source())
    );

    let mut dependency = candidate_dependency(valid.clone());
    dependency["path"] = serde_json::json!(["bad name"]);
    let invalid_segment = package_error(&dependency_package(valid.clone(), dependency));
    assert_eq!(invalid_segment[0].code, DiagnosticCode::InvalidIdentifier);
    assert_eq!(invalid_segment[0].path, "dependency.path");

    let mut candidate = valid.clone();
    candidate["requirements"][0]["clauses"][0]["id"] = "".into();
    candidate["requirements"][0]["clauses"][0]["source"]["start"]["line"] = 0.into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InvalidIdentifier,
        "identity grammar must precede source-span validation"
    );

    let mut candidate = valid.clone();
    let mut duplicate = candidate["requirements"][0].clone();
    duplicate["revision"] = 4.into();
    duplicate["clauses"][0]["body"]["children"][0]["identity"]["requirement"]["revision"] =
        4.into();
    duplicate["clauses"][0]["body"]["children"][1]["children"][0]["identity"]["requirement"]
        ["revision"] = 4.into();
    candidate["requirements"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    let duplicate_requirement = &package_error(&candidate)[0];
    assert_eq!(
        duplicate_requirement.code,
        DiagnosticCode::DuplicateRequirement
    );
    assert_eq!(duplicate_requirement.related.len(), 1);
    assert_eq!(
        serde_json::to_value(&duplicate_requirement.related[0]).unwrap()["reference"]["revision"],
        3
    );

    let mut candidate = valid.clone();
    let duplicate = candidate["requirements"][0]["clauses"][0].clone();
    candidate["requirements"][0]["clauses"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    let duplicate_clause = &package_error(&candidate)[0];
    assert_eq!(duplicate_clause.code, DiagnosticCode::DuplicateClause);
    assert_eq!(duplicate_clause.related.len(), 1);

    let mut candidate = valid.clone();
    for endpoint in ["start", "end"] {
        candidate["requirements"][0]["source"][endpoint]["source"]["revision"] = 8.into();
    }
    let source_mismatch = &package_error(&candidate)[0];
    assert_eq!(source_mismatch.code, DiagnosticCode::InvalidSourceSpan);
    assert!(source_mismatch.span.is_some());

    let mut candidate = valid.clone();
    for endpoint in ["start", "end"] {
        candidate["requirements"][0]["clauses"][0]["source"][endpoint]["source"]["revision"] =
            8.into();
    }
    let source_mismatch = &package_error(&candidate)[0];
    assert_eq!(source_mismatch.code, DiagnosticCode::InvalidSourceSpan);
    assert!(source_mismatch.span.is_some());

    let mut candidate = valid.clone();
    candidate["requirements"][0]["clauses"][0]["body"]["children"][0]["identity"]["requirement"]
        ["package"] = "other/package".into();
    let cross_package = package_error(&candidate);
    assert_eq!(cross_package[0].code, DiagnosticCode::CrossPackageReference);
    assert_eq!(
        cross_package[0].span.as_deref(),
        Some(package.requirements()[0].clauses()[0].source())
    );

    let mut candidate = valid.clone();
    candidate["requirements"][0]["clauses"][0]["anchor"] = serde_json::Value::Null;
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::FloatingExecutableClause
    );

    let mut candidate = valid.clone();
    candidate["requirements"][0]["clauses"][0]["kind"] = "information".into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::InformationalClauseAnchored
    );

    let mut candidate = valid.clone();
    candidate["requirements"][0]["clauses"][0]["kind"] = "postcondition".into();
    assert_eq!(
        package_error(&candidate)[0].code,
        DiagnosticCode::IncompatibleClauseAnchor
    );

    let mut candidate = valid.clone();
    candidate["requirements"][0]["clauses"][0]["anchor"]["operation"] = "bad name".into();
    let invalid_anchor = &package_error(&candidate)[0];
    assert_eq!(invalid_anchor.code, DiagnosticCode::InvalidIdentifier);
    assert_eq!(invalid_anchor.path, "clause.anchor.name");

    assert_eq!(
        ContractPackage::from_json_str("{", ValidationOptions::strict()).unwrap_err()[0].code,
        DiagnosticCode::InvalidWireFormat
    );
    assert!(serde_json::from_str::<DiagnosticCode>("\"not_registered\"").is_err());

    // Exercise the direct convenience deserializers as defense in depth.
    assert!(serde_json::from_str::<PackageId>("\"agent//contract\"").is_err());
    assert!(serde_json::from_str::<SourceDocumentId>("\"bad document\"").is_err());
    assert!(serde_json::from_str::<SchemaVersion>(r#"{"major":0,"minor":1}"#).is_err());
    assert!(serde_json::from_str::<SourceRevision>("0").is_err());
    assert!(serde_json::from_str::<RequirementRevision>("0").is_err());

    let mut candidate = valid.clone();
    candidate["requirements"][0]["clauses"][0]["source"]["start"]["line"] = 0.into();
    let invalid_location = candidate["requirements"][0]["clauses"][0]["source"]["start"].clone();
    assert!(serde_json::from_value::<SourceLocation>(invalid_location).is_err());
    let invalid_span = candidate["requirements"][0]["clauses"][0]["source"].clone();
    assert!(serde_json::from_value::<SourceSpan>(invalid_span).is_err());

    let mut dependency_value = candidate_dependency(valid.clone());
    dependency_value["path"] = serde_json::json!([]);
    assert!(serde_json::from_value::<DependencyIdentity>(dependency_value).is_err());
}

fn candidate_dependency(mut package: serde_json::Value) -> serde_json::Value {
    package["requirements"][0]["clauses"][0]["body"]["children"][0]["identity"].take()
}

fn dependency_package(
    mut package: serde_json::Value,
    dependency: serde_json::Value,
) -> serde_json::Value {
    package["requirements"][0]["clauses"][0]["body"]["children"][0]["identity"] = dependency;
    package
}
