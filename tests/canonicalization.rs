use quire_contract_ir::{
    classify_coverage, migrate_reference_body, AnchorName, ArtifactId, ArtifactTrace,
    CanonicalDigest, CanonicalKind, CanonicalProfile, Clause, ClauseId, ClauseKind, CollectionType,
    ContractPackage, CoverageClass, DeclarationEnvironment, DiagnosticCode, EnumDeclaration,
    EnumVariantDeclaration, ExecutionPoint, Expression, ExpressionKind, FunctionParameter,
    OrphanReason, PackageId, PureFunctionDeclaration, RationalType, RecordDeclaration,
    RecordFieldDeclaration, RecordLiteralField, ReferenceBody, Requirement, RequirementId,
    RequirementRef, RequirementRevision, SchemaVersion, SourceDocumentId, SourceIdentity,
    SourceLocation, SourceRevision, SourceSpan, SymbolName, TypeDeclaration, ValidationOptions,
    ValueDeclaration, ValueDeclarationKind, ValueType,
};

fn source(document: &str) -> SourceIdentity {
    SourceIdentity::new(
        SourceDocumentId::new(document).unwrap(),
        SourceRevision::new(1).unwrap(),
    )
}

fn span(source: &SourceIdentity, start: u64, end: u64) -> SourceSpan {
    SourceSpan::new(
        SourceLocation::new(source.clone(), 1, start as u32 + 1, start).unwrap(),
        SourceLocation::new(source.clone(), 1, end as u32 + 1, end).unwrap(),
    )
    .unwrap()
}

fn requirement(
    package: &PackageId,
    source: &SourceIdentity,
    id: &str,
    revision: u64,
    body: ReferenceBody,
) -> Requirement<ReferenceBody> {
    Requirement::new(
        package,
        RequirementId::new(id).unwrap(),
        RequirementRevision::new(revision).unwrap(),
        span(source, 0, 10),
        vec![Clause::new(
            ClauseId::new("note").unwrap(),
            ClauseKind::Information,
            None,
            span(source, 1, 9),
            body,
        )
        .unwrap()],
    )
    .unwrap()
}

fn package_fixture(document: &str, reverse: bool, changed: bool) -> ContractPackage<ReferenceBody> {
    let package_id = PackageId::new("agent-ix/pkg").unwrap();
    let source = source(document);
    let first_body = if changed {
        ReferenceBody::Composite {
            children: vec![ReferenceBody::Literal],
        }
    } else {
        ReferenceBody::Literal
    };
    let first = requirement(&package_id, &source, "REQ_a", 1, first_body);
    let second = requirement(&package_id, &source, "REQ_b", 2, ReferenceBody::Literal);
    let requirements = if reverse {
        vec![second, first]
    } else {
        vec![first, second]
    };
    ContractPackage::new(package_id, SchemaVersion::V1_0, source, requirements).unwrap()
}

fn reference(package: &ContractPackage<ReferenceBody>, id: &str, revision: u64) -> RequirementRef {
    RequirementRef::parse(package.id().as_str(), id, revision).unwrap()
}

/// Tracing: TC-017.
/// FR-016-AC-1.
/// FR-016-AC-2.
/// FR-016-AC-3.
/// NFR-001-AC-1.
#[test]
fn tc_017_canonical_bytes_digests_ordering_and_resource_failure_conform() {
    let package = package_fixture("canonical_a", false, false);
    let permuted = package_fixture("canonical_b", true, false);
    let output = package.canonical_package(CanonicalProfile::V1).unwrap();
    let repeated = package.canonical_package(CanonicalProfile::V1).unwrap();
    let permuted_output = permuted.canonical_package(CanonicalProfile::V1).unwrap();

    assert_eq!(output, repeated);
    assert_eq!(output.kind(), CanonicalKind::Package);
    assert_eq!(output, permuted_output);
    assert_eq!(output.bytes().len(), output.bytes().as_slice().len() as u64);
    let text = std::str::from_utf8(output.bytes().as_slice()).unwrap();
    assert_eq!(
        text,
        "{\"kind\":\"package\",\"profile\":\"quire.contract.canonical-json/v1\",\"value\":{\"id\":\"agent-ix/pkg\",\"requirements\":[{\"clauses\":[{\"body\":{\"node\":\"literal\"},\"id\":\"note\",\"kind\":\"information\",\"requirement\":{\"package\":\"agent-ix/pkg\",\"requirement\":\"REQ_a\",\"revision\":1}}],\"id\":\"REQ_a\",\"package\":\"agent-ix/pkg\",\"revision\":1},{\"clauses\":[{\"body\":{\"node\":\"literal\"},\"id\":\"note\",\"kind\":\"information\",\"requirement\":{\"package\":\"agent-ix/pkg\",\"requirement\":\"REQ_b\",\"revision\":2}}],\"id\":\"REQ_b\",\"package\":\"agent-ix/pkg\",\"revision\":2}],\"schema_version\":{\"major\":1,\"minor\":0}}}"
    );
    assert!(!text.contains("canonical_a"));
    assert_eq!(
        output.digest().to_string(),
        "b6e1a7da8f8bdc86ffa723af858bb7a5bbced174c42ea4e542cfd76212133577"
    );
    assert_eq!(
        CanonicalDigest::parse(&output.digest().to_string()).unwrap(),
        output.digest()
    );
    assert_eq!(
        CanonicalDigest::parse("B6E1A7DA8F8BDC86FFA723AF858BB7A5BBCED174C42EA4E542CFD76212133577")
            .unwrap_err()
            .code,
        DiagnosticCode::InvalidWireFormat
    );

    let changed = package_fixture("canonical_c", false, true);
    assert_ne!(
        output.digest(),
        changed
            .canonical_package(CanonicalProfile::V1)
            .unwrap()
            .digest()
    );
    assert_ne!(
        package
            .canonical_requirement(&package.requirements()[0], CanonicalProfile::V1)
            .unwrap()
            .digest(),
        changed
            .canonical_requirement(&changed.requirements()[0], CanonicalProfile::V1)
            .unwrap()
            .digest()
    );
    assert_eq!(
        package
            .canonical_requirement(&package.requirements()[1], CanonicalProfile::V1)
            .unwrap()
            .digest(),
        changed
            .canonical_requirement(&changed.requirements()[1], CanonicalProfile::V1)
            .unwrap()
            .digest()
    );
    assert_eq!(
        package
            .canonical_requirement(&package.requirements()[0], CanonicalProfile::V1)
            .unwrap()
            .kind(),
        CanonicalKind::Requirement
    );
    assert_eq!(
        package
            .canonical_clause(
                &package.requirements()[0],
                &package.requirements()[0].clauses()[0],
                CanonicalProfile::V1,
            )
            .unwrap()
            .kind(),
        CanonicalKind::Clause
    );
    assert_eq!(
        package
            .canonical_requirement(&permuted.requirements()[0], CanonicalProfile::V1)
            .unwrap_err()
            .code,
        DiagnosticCode::MalformedReference
    );

    let unsupported = ContractPackage::new(
        package.id().clone(),
        SchemaVersion::new(1, 9).unwrap(),
        package.source().clone(),
        package.requirements().to_vec(),
    )
    .unwrap();
    assert_eq!(
        unsupported
            .canonical_package(CanonicalProfile::V1)
            .unwrap_err()
            .code,
        DiagnosticCode::UnregisteredMigration
    );

    let exhausted = package
        .canonical_package_with_limit(CanonicalProfile::V1, 0)
        .unwrap_err();
    assert_eq!(
        exhausted.code,
        DiagnosticCode::CanonicalizationResourceExhausted
    );
    let exhausted_requirement = package
        .canonical_requirement_with_limit(&package.requirements()[0], CanonicalProfile::V1, 0)
        .unwrap_err();
    assert_eq!(
        exhausted_requirement.code,
        DiagnosticCode::CanonicalizationResourceExhausted
    );
    assert_eq!(
        exhausted_requirement.span.as_deref(),
        Some(package.requirements()[0].source())
    );
    let exhausted_clause = package
        .canonical_clause_with_limit(
            &package.requirements()[0],
            &package.requirements()[0].clauses()[0],
            CanonicalProfile::V1,
            0,
        )
        .unwrap_err();
    assert_eq!(
        exhausted_clause.code,
        DiagnosticCode::CanonicalizationResourceExhausted
    );
    assert_eq!(
        exhausted_clause.span.as_deref(),
        Some(package.requirements()[0].clauses()[0].source())
    );
}

/// Tracing: TC-017.
/// FR-016-AC-1.
#[test]
fn tc_017_declaration_and_expression_projections_are_source_free_and_exact() {
    let owner = RequirementRef::parse("agent-ix/pkg", "REQ_a", 1).unwrap();
    let source_a = source("declarations_a");
    let source_b = source("declarations_b");
    let value_a = ValueDeclaration::new(
        SymbolName::new("alpha").unwrap(),
        ValueDeclarationKind::Input,
        ValueType::Text,
        span(&source_a, 1, 2),
    );
    let value_z = ValueDeclaration::new(
        SymbolName::new("zeta").unwrap(),
        ValueDeclarationKind::State,
        ValueType::Boolean,
        span(&source_a, 2, 3),
    );
    let enum_a = EnumDeclaration::new(
        SymbolName::new("Color").unwrap(),
        span(&source_a, 3, 4),
        vec![
            EnumVariantDeclaration::new(SymbolName::new("violet").unwrap(), span(&source_a, 4, 5)),
            EnumVariantDeclaration::new(SymbolName::new("amber").unwrap(), span(&source_a, 5, 6)),
        ],
    )
    .unwrap();
    let record_a = RecordDeclaration::new(
        SymbolName::new("Record").unwrap(),
        span(&source_a, 6, 7),
        vec![
            RecordFieldDeclaration::new(
                SymbolName::new("second").unwrap(),
                ValueType::Text,
                span(&source_a, 7, 8),
            ),
            RecordFieldDeclaration::new(
                SymbolName::new("first").unwrap(),
                ValueType::Boolean,
                span(&source_a, 8, 9),
            ),
        ],
    )
    .unwrap();
    let function_a = PureFunctionDeclaration::new(
        SymbolName::new("ordered").unwrap(),
        vec![
            FunctionParameter::new(
                SymbolName::new("param_z").unwrap(),
                ValueType::Boolean,
                span(&source_a, 9, 10),
            ),
            FunctionParameter::new(
                SymbolName::new("param_a").unwrap(),
                ValueType::Text,
                span(&source_a, 10, 11),
            ),
        ],
        ValueType::Boolean,
        span(&source_a, 9, 12),
    )
    .unwrap();
    let environment = DeclarationEnvironment::new(
        owner.clone(),
        vec![
            TypeDeclaration::Record {
                declaration: record_a,
            },
            TypeDeclaration::Enum {
                declaration: enum_a,
            },
        ],
        vec![value_z.clone(), value_a.clone()],
        vec![function_a],
    )
    .unwrap();
    let enum_b = EnumDeclaration::new(
        SymbolName::new("Color").unwrap(),
        span(&source_b, 3, 4),
        vec![
            EnumVariantDeclaration::new(SymbolName::new("amber").unwrap(), span(&source_b, 4, 5)),
            EnumVariantDeclaration::new(SymbolName::new("violet").unwrap(), span(&source_b, 5, 6)),
        ],
    )
    .unwrap();
    let record_b = RecordDeclaration::new(
        SymbolName::new("Record").unwrap(),
        span(&source_b, 6, 7),
        vec![
            RecordFieldDeclaration::new(
                SymbolName::new("first").unwrap(),
                ValueType::Boolean,
                span(&source_b, 7, 8),
            ),
            RecordFieldDeclaration::new(
                SymbolName::new("second").unwrap(),
                ValueType::Text,
                span(&source_b, 8, 9),
            ),
        ],
    )
    .unwrap();
    let function_b = PureFunctionDeclaration::new(
        SymbolName::new("ordered").unwrap(),
        vec![
            FunctionParameter::new(
                SymbolName::new("param_z").unwrap(),
                ValueType::Boolean,
                span(&source_b, 9, 10),
            ),
            FunctionParameter::new(
                SymbolName::new("param_a").unwrap(),
                ValueType::Text,
                span(&source_b, 10, 11),
            ),
        ],
        ValueType::Boolean,
        span(&source_b, 9, 12),
    )
    .unwrap();
    let environment_permuted = DeclarationEnvironment::new(
        owner,
        vec![
            TypeDeclaration::Enum {
                declaration: enum_b,
            },
            TypeDeclaration::Record {
                declaration: record_b,
            },
        ],
        vec![
            ValueDeclaration::new(
                value_a.name().clone(),
                value_a.kind(),
                value_a.value_type().clone(),
                span(&source_b, 10, 11),
            ),
            ValueDeclaration::new(
                value_z.name().clone(),
                value_z.kind(),
                value_z.value_type().clone(),
                span(&source_b, 11, 12),
            ),
        ],
        vec![function_b],
    )
    .unwrap();
    let declaration = environment
        .canonical_declaration(CanonicalProfile::V1)
        .unwrap();
    assert_eq!(
        environment
            .canonical_declaration_with_limit(CanonicalProfile::V1, 0)
            .unwrap_err()
            .code,
        DiagnosticCode::CanonicalizationResourceExhausted
    );
    assert_eq!(
        declaration,
        environment_permuted
            .canonical_declaration(CanonicalProfile::V1)
            .unwrap()
    );
    let declaration_text = std::str::from_utf8(declaration.bytes().as_slice()).unwrap();
    assert_eq!(declaration.kind(), CanonicalKind::Declaration);
    assert!(declaration_text.find("alpha").unwrap() < declaration_text.find("zeta").unwrap());
    assert!(declaration_text.find("amber").unwrap() < declaration_text.find("violet").unwrap());
    assert!(declaration_text.find("first").unwrap() < declaration_text.find("second").unwrap());
    assert!(declaration_text.find("param_z").unwrap() < declaration_text.find("param_a").unwrap());
    assert!(!declaration_text.contains("declarations_"));

    let expression_source = source("expression_source");
    let expression = Expression::new(
        ExpressionKind::TextLiteral {
            value: "\"\\\u{8}\t\n\u{c}\r\u{1}/é".to_owned(),
        },
        span(&expression_source, 0, 1),
    );
    let typed = DeclarationEnvironment::new(
        RequirementRef::parse("agent-ix/pkg", "REQ_text", 1).unwrap(),
        vec![],
        vec![],
        vec![],
    )
    .unwrap()
    .check_expression(
        &expression,
        &ValueType::Text,
        &ExecutionPoint::Pre {
            operation: AnchorName::new("render").unwrap(),
        },
        false,
    )
    .unwrap();
    let expression_output = typed.canonical_expression(CanonicalProfile::V1).unwrap();
    let exhausted_expression = typed
        .canonical_expression_with_limit(CanonicalProfile::V1, 0)
        .unwrap_err();
    assert_eq!(
        exhausted_expression.code,
        DiagnosticCode::CanonicalizationResourceExhausted
    );
    assert_eq!(
        exhausted_expression.span.as_deref(),
        Some(typed.expression().source())
    );
    let expression_text = std::str::from_utf8(expression_output.bytes().as_slice()).unwrap();
    assert_eq!(
        expression_text,
        "{\"kind\":\"expression\",\"profile\":\"quire.contract.canonical-json/v1\",\"value\":{\"result_type\":{\"kind\":\"text\"},\"tree\":{\"kind\":{\"node\":\"text_literal\",\"value\":\"\\\"\\\\\\b\\t\\n\\f\\r\\u0001/é\"}}}}"
    );
    assert_eq!(expression_output.kind(), CanonicalKind::Expression);
    assert!(!expression_text.contains("expression_source"));

    let record_expression = Expression::new(
        ExpressionKind::RecordLiteral {
            record: SymbolName::new("Record").unwrap(),
            fields: vec![
                RecordLiteralField::new(
                    SymbolName::new("second").unwrap(),
                    Expression::new(
                        ExpressionKind::TextLiteral {
                            value: "two".to_owned(),
                        },
                        span(&expression_source, 20, 21),
                    ),
                ),
                RecordLiteralField::new(
                    SymbolName::new("first").unwrap(),
                    Expression::new(
                        ExpressionKind::BooleanLiteral { value: true },
                        span(&expression_source, 21, 22),
                    ),
                ),
            ],
        },
        span(&expression_source, 20, 23),
    );
    let typed_record = environment
        .check_expression(
            &record_expression,
            &ValueType::Record {
                name: SymbolName::new("Record").unwrap(),
            },
            &ExecutionPoint::Pre {
                operation: AnchorName::new("render").unwrap(),
            },
            false,
        )
        .unwrap();
    let record_text = String::from_utf8(
        typed_record
            .canonical_expression(CanonicalProfile::V1)
            .unwrap()
            .bytes()
            .as_slice()
            .to_vec(),
    )
    .unwrap();
    assert!(record_text.find("first").unwrap() < record_text.find("second").unwrap());

    let scalar_environment = DeclarationEnvironment::new(
        RequirementRef::parse("agent-ix/pkg", "REQ_scalar", 1).unwrap(),
        vec![],
        vec![],
        vec![],
    )
    .unwrap();
    let rational_type = RationalType::new(-10, 10, 10).unwrap();
    let unreduced = Expression::new(
        ExpressionKind::RationalLiteral {
            numerator: 2,
            denominator: 4,
            value_type: rational_type.clone(),
        },
        span(&expression_source, 30, 31),
    );
    let reduced = Expression::new(
        ExpressionKind::RationalLiteral {
            numerator: 1,
            denominator: 2,
            value_type: rational_type.clone(),
        },
        span(&expression_source, 31, 32),
    );
    let typed_unreduced = scalar_environment
        .check_expression(
            &unreduced,
            &ValueType::rational(rational_type.clone()),
            &ExecutionPoint::Pre {
                operation: AnchorName::new("calculate").unwrap(),
            },
            false,
        )
        .unwrap();
    let typed_reduced = scalar_environment
        .check_expression(
            &reduced,
            &ValueType::rational(rational_type),
            &ExecutionPoint::Pre {
                operation: AnchorName::new("calculate").unwrap(),
            },
            false,
        )
        .unwrap();
    let rational_output = typed_unreduced
        .canonical_expression(CanonicalProfile::V1)
        .unwrap();
    assert_eq!(
        rational_output,
        typed_reduced
            .canonical_expression(CanonicalProfile::V1)
            .unwrap()
    );
    let rational_text = String::from_utf8(rational_output.bytes().as_slice().to_vec()).unwrap();
    assert!(rational_text.contains("\"denominator\":2"));
    assert!(rational_text.contains("\"numerator\":1"));

    let collection_type = CollectionType::new(ValueType::Text, 2).unwrap();
    let collection = Expression::new(
        ExpressionKind::CollectionLiteral {
            value_type: collection_type.clone(),
            items: vec![
                Expression::new(
                    ExpressionKind::TextLiteral {
                        value: "z_item".to_owned(),
                    },
                    span(&expression_source, 40, 41),
                ),
                Expression::new(
                    ExpressionKind::TextLiteral {
                        value: "a_item".to_owned(),
                    },
                    span(&expression_source, 41, 42),
                ),
            ],
        },
        span(&expression_source, 40, 43),
    );
    let typed_collection = scalar_environment
        .check_expression(
            &collection,
            &ValueType::collection(collection_type),
            &ExecutionPoint::Pre {
                operation: AnchorName::new("calculate").unwrap(),
            },
            false,
        )
        .unwrap();
    let collection_text = String::from_utf8(
        typed_collection
            .canonical_expression(CanonicalProfile::V1)
            .unwrap()
            .bytes()
            .as_slice()
            .to_vec(),
    )
    .unwrap();
    assert!(collection_text.find("z_item").unwrap() < collection_text.find("a_item").unwrap());
}

/// Tracing: TC-017.
/// FR-017-AC-1.
/// NFR-003-AC-1.
#[test]
fn tc_017_version_preflight_and_registered_migration_fail_closed() {
    let package = package_fixture("migration", false, false);
    let (migrated, receipt) =
        migrate_reference_body(package.clone(), SchemaVersion::V1_1, CanonicalProfile::V1).unwrap();
    assert_eq!(migrated.schema_version(), SchemaVersion::V1_1);
    assert_eq!(receipt.migration_id(), "reference_body_1_0_to_1_1");
    assert_eq!(receipt.source_version(), SchemaVersion::V1_0);
    assert_eq!(receipt.target_version(), SchemaVersion::V1_1);
    assert_ne!(
        receipt.source_package_digest(),
        receipt.target_package_digest()
    );
    assert_eq!(package.id(), migrated.id());
    assert_eq!(package.requirements(), migrated.requirements());
    assert_eq!(
        migrate_reference_body(package.clone(), SchemaVersion::V1_0, CanonicalProfile::V1,)
            .unwrap_err()[0]
            .code,
        DiagnosticCode::UnregisteredMigration
    );

    let mut wire = serde_json::to_value(&package).unwrap();
    wire["id"] = serde_json::json!("bad id");
    wire["schema_version"] = serde_json::json!({"major": 2, "minor": 0});
    assert_eq!(
        ContractPackage::from_json_str(&wire.to_string(), ValidationOptions::strict()).unwrap_err()
            [0]
        .code,
        DiagnosticCode::UnsupportedSchemaVersion
    );
    wire["schema_version"] = serde_json::json!({"major": 1, "minor": 9});
    assert_eq!(
        ContractPackage::from_json_str(&wire.to_string(), ValidationOptions::strict()).unwrap_err()
            [0]
        .code,
        DiagnosticCode::UnregisteredMigration
    );
    wire["schema_version"] = serde_json::json!({"major": 0, "minor": 1});
    assert_eq!(
        ContractPackage::from_json_str(&wire.to_string(), ValidationOptions::strict()).unwrap_err()
            [0]
        .code,
        DiagnosticCode::InvalidSchemaVersion
    );
    wire.as_object_mut().unwrap().remove("schema_version");
    assert_eq!(
        ContractPackage::from_json_str(&wire.to_string(), ValidationOptions::strict()).unwrap_err()
            [0]
        .code,
        DiagnosticCode::InvalidWireFormat
    );
}

/// Tracing: TC-017.
/// FR-017-AC-2.
/// NFR-003-AC-2.
#[test]
fn tc_017_coverage_classes_orphans_diagnostics_and_sorting_conform() {
    let package = package_fixture("coverage", false, false);
    let source = source("traces");
    let req_a = reference(&package, "REQ_a", 1);
    let req_b = reference(&package, "REQ_b", 2);
    let req_a_digest = package
        .canonical_requirement(&package.requirements()[0], CanonicalProfile::V1)
        .unwrap()
        .digest();
    let wrong_digest = quire_contract_ir::CanonicalDigest::parse(
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();
    let make_span = |at| span(&source, at, at + 1);
    let traces = vec![
        ArtifactTrace::shallow(
            ArtifactId::new("art_shallow").unwrap(),
            make_span(0),
            req_a.clone(),
            make_span(1),
        ),
        ArtifactTrace::deep(
            ArtifactId::new("art_deep").unwrap(),
            make_span(2),
            req_a.clone(),
            make_span(3),
            req_a_digest,
            make_span(4),
        ),
        ArtifactTrace::shallow(
            ArtifactId::new("art_cross").unwrap(),
            make_span(5),
            RequirementRef::parse("agent-ix/other", "REQ_a", 1).unwrap(),
            make_span(6),
        ),
        ArtifactTrace::shallow(
            ArtifactId::new("art_missing").unwrap(),
            make_span(7),
            reference(&package, "REQ_missing", 1),
            make_span(8),
        ),
        ArtifactTrace::shallow(
            ArtifactId::new("art_stale").unwrap(),
            make_span(9),
            reference(&package, "REQ_a", 9),
            make_span(10),
        ),
        ArtifactTrace::deep(
            ArtifactId::new("art_wrong_digest").unwrap(),
            make_span(11),
            req_a.clone(),
            make_span(12),
            wrong_digest,
            make_span(13),
        ),
        ArtifactTrace::shallow(
            ArtifactId::new("art_duplicate").unwrap(),
            make_span(14),
            req_b.clone(),
            make_span(15),
        ),
        ArtifactTrace::shallow(
            ArtifactId::new("art_duplicate").unwrap(),
            make_span(16),
            req_b,
            make_span(17),
        ),
    ];
    let result = classify_coverage(&package, &traces, CanonicalProfile::V1).unwrap();
    assert_eq!(result.report().requirements().len(), 2);
    assert_eq!(result.report().requirements()[0].reference(), &req_a);
    assert_eq!(
        result.report().requirements()[0].class(),
        CoverageClass::Deep
    );
    assert_eq!(
        result.report().requirements()[1].class(),
        CoverageClass::Uncovered
    );
    assert_eq!(result.report().artifacts().len(), 7);
    assert!(result
        .report()
        .artifacts()
        .windows(2)
        .all(|rows| rows[0].artifact_id() < rows[1].artifact_id()));
    let duplicate = result
        .report()
        .artifacts()
        .iter()
        .find(|row| row.artifact_id().as_str() == "art_duplicate")
        .unwrap();
    assert_eq!(duplicate.class(), CoverageClass::Orphaned);
    assert_eq!(
        duplicate.orphan_reason(),
        Some(OrphanReason::DuplicateArtifact)
    );
    assert_eq!(
        result
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            DiagnosticCode::CrossPackageReference,
            DiagnosticCode::OrphanedRequirementReference,
            DiagnosticCode::StaleRequirementRevision,
            DiagnosticCode::StaleTraceDigest,
            DiagnosticCode::DuplicateArtifactTrace,
        ]
    );
    assert_eq!(
        result.diagnostics()[3].span.as_deref(),
        Some(&make_span(13))
    );
    assert_eq!(
        result.diagnostics()[4].span.as_deref(),
        Some(&make_span(16))
    );
    let reasons = result
        .report()
        .artifacts()
        .iter()
        .filter_map(|row| row.orphan_reason())
        .collect::<Vec<_>>();
    assert!(reasons.contains(&OrphanReason::CrossPackage));
    assert!(reasons.contains(&OrphanReason::MissingRequirement));
    assert!(reasons.contains(&OrphanReason::StaleRevision));
    assert!(reasons.contains(&OrphanReason::DuplicateArtifact));
    assert!(reasons.contains(&OrphanReason::DigestMismatch));
}
