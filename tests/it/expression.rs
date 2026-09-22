use quire_contract_ir::{
    AnchorName, BooleanOperator, CollectionType, ComparisonOperator, DeclarationEnvironment,
    DefinednessObligationKind, DependencyKind, DiagnosticCode, EnumDeclaration,
    EnumVariantDeclaration, ExecutionPoint, Expression, ExpressionKind, FunctionParameter,
    IntegerDomain, IntegerType, NumericOperator, OverflowPolicy, PackageId,
    PureFunctionDeclaration, QuantifierDomain, QuantifierKind, RationalType, RecordDeclaration,
    RecordFieldDeclaration, RecordLiteralField, RequirementRef, SourceDocumentId, SourceIdentity,
    SourceLocation, SourceRevision, SourceSpan, StateObservation, SymbolName, TypeDeclaration,
    ValueDeclaration, ValueDeclarationKind, ValueType, MAX_EXPRESSION_DEPTH, MAX_EXPRESSION_NODES,
    MAX_TEXT_LENGTH,
};

fn name(value: &str) -> SymbolName {
    SymbolName::new(value).unwrap()
}

fn source() -> SourceIdentity {
    SourceIdentity::new(
        SourceDocumentId::new("expressions").unwrap(),
        SourceRevision::new(1).unwrap(),
    )
}

fn span(start: u64, end: u64) -> SourceSpan {
    let source = source();
    SourceSpan::new(
        SourceLocation::new(source.clone(), 1, start as u32 + 1, start).unwrap(),
        SourceLocation::new(source, 1, end as u32 + 1, end).unwrap(),
    )
    .unwrap()
}

fn integer(minimum: i64, maximum: i64, overflow: OverflowPolicy) -> IntegerType {
    IntegerType::new(IntegerDomain::Signed, minimum, maximum, overflow).unwrap()
}

fn int_type() -> IntegerType {
    integer(-10, 10, OverflowPolicy::Reject)
}

fn index_type(maximum: u32) -> IntegerType {
    IntegerType::new(
        IntegerDomain::Unsigned,
        0,
        i64::from(maximum),
        OverflowPolicy::Reject,
    )
    .unwrap()
}

fn int(value: i64, at: u64) -> Expression {
    Expression::new(
        ExpressionKind::IntegerLiteral {
            value,
            value_type: int_type(),
        },
        span(at, at + 1),
    )
}

fn value(name_value: &str, observation: StateObservation, at: u64) -> Expression {
    Expression::new(
        ExpressionKind::ValueReference {
            name: name(name_value),
            observation,
        },
        span(at, at + 1),
    )
}

fn compare(
    operator: ComparisonOperator,
    left: Expression,
    right: Expression,
    at: u64,
) -> Expression {
    Expression::new(
        ExpressionKind::Compare {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        },
        span(at, at + 3),
    )
}

fn bool_op(operator: BooleanOperator, left: Expression, right: Expression, at: u64) -> Expression {
    Expression::new(
        ExpressionKind::Boolean {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        },
        span(at, at + 8),
    )
}

fn divide(divisor: Expression, at: u64) -> Expression {
    Expression::new(
        ExpressionKind::Numeric {
            operator: NumericOperator::Divide,
            left: Box::new(int(10, at + 1)),
            right: Box::new(divisor),
        },
        span(at, at + 4),
    )
}

fn boolean(value: bool, at: u64) -> Expression {
    Expression::new(ExpressionKind::BooleanLiteral { value }, span(at, at + 1))
}

fn assert_code(
    result: Result<quire_contract_ir::TypedExpression, Vec<quire_contract_ir::Diagnostic>>,
    code: DiagnosticCode,
) {
    assert_eq!(result.unwrap_err()[0].code, code);
}

fn environment() -> DeclarationEnvironment {
    let color = EnumDeclaration::new(
        name("Color"),
        span(0, 2),
        vec![
            EnumVariantDeclaration::new(name("red"), span(1, 2)),
            EnumVariantDeclaration::new(name("blue"), span(2, 3)),
        ],
    )
    .unwrap();
    let sensor = RecordDeclaration::new(
        name("Sensor"),
        span(3, 8),
        vec![RecordFieldDeclaration::new(
            name("reading"),
            ValueType::integer(int_type()),
            span(4, 5),
        )],
    )
    .unwrap();
    let collection = CollectionType::new(ValueType::integer(int_type()), 4).unwrap();
    let values = vec![
        ValueDeclaration::new(
            name("divisor"),
            ValueDeclarationKind::Input,
            ValueType::integer(int_type()),
            span(10, 11),
        ),
        ValueDeclaration::new(
            name("state_value"),
            ValueDeclarationKind::State,
            ValueType::integer(int_type()),
            span(11, 12),
        ),
        ValueDeclaration::new(
            name("maybe"),
            ValueDeclarationKind::State,
            ValueType::option(ValueType::integer(int_type())),
            span(12, 13),
        ),
        ValueDeclaration::new(
            name("items"),
            ValueDeclarationKind::State,
            ValueType::collection(collection),
            span(13, 14),
        ),
        ValueDeclaration::new(
            name("position"),
            ValueDeclarationKind::Input,
            ValueType::integer(index_type(4)),
            span(14, 15),
        ),
        ValueDeclaration::new(
            name("sensor"),
            ValueDeclarationKind::State,
            ValueType::Record {
                name: name("Sensor"),
            },
            span(15, 16),
        ),
    ];
    let function = PureFunctionDeclaration::new(
        name("identity"),
        vec![FunctionParameter::new(
            name("value"),
            ValueType::integer(int_type()),
            span(16, 17),
        )],
        ValueType::integer(int_type()),
        span(16, 18),
    )
    .unwrap();
    DeclarationEnvironment::new(
        RequirementRef::new(
            PackageId::new("agent-ix/contract").unwrap(),
            quire_contract_ir::RequirementId::new("REQ_expression").unwrap(),
            quire_contract_ir::RequirementRevision::new(1).unwrap(),
        ),
        vec![
            TypeDeclaration::Enum { declaration: color },
            TypeDeclaration::Record {
                declaration: sensor,
            },
        ],
        values,
        vec![function],
    )
    .unwrap()
}

fn pre() -> ExecutionPoint {
    ExecutionPoint::Pre {
        operation: AnchorName::new("update").unwrap(),
    }
}

/// Tracing: TC-016
/// TC-016.
/// StR-001-VC-1.
/// FR-012-AC-5.
/// FR-013-AC-1.
/// FR-013-AC-2.
/// FR-013-AC-3.
/// FR-013-AC-4.
/// FR-014-AC-1.
/// FR-014-AC-2.
/// FR-014-AC-3.
/// FR-014-AC-4.
/// FR-014-AC-6.
/// NFR-002-AC-4.
/// STD-001.
#[test]
fn tc_016_closed_types_expressions_scopes_and_dependencies_conform() {
    let environment = environment();

    let same = DeclarationEnvironment::new(
        environment.owner().clone(),
        environment.types().to_vec(),
        environment.values().to_vec(),
        environment.functions().to_vec(),
    )
    .unwrap();
    assert_eq!(same, environment);

    assert_eq!(
        IntegerType::new(IntegerDomain::Unsigned, -1, 4, OverflowPolicy::Reject)
            .unwrap_err()
            .code,
        DiagnosticCode::InvalidNumericBounds
    );
    assert_eq!(
        RationalType::new(-2, 2, 0).unwrap_err().code,
        DiagnosticCode::InvalidNumericBounds
    );
    assert_eq!(
        CollectionType::new(ValueType::Boolean, 0).unwrap_err().code,
        DiagnosticCode::UnboundedCollection
    );

    let empty_enum = EnumDeclaration::new(name("Empty"), span(20, 21), vec![]).unwrap_err();
    assert_eq!(empty_enum[0].code, DiagnosticCode::EmptyEnum);
    let duplicate_enum = EnumDeclaration::new(
        name("Duplicate"),
        span(20, 23),
        vec![
            EnumVariantDeclaration::new(name("same"), span(21, 22)),
            EnumVariantDeclaration::new(name("same"), span(22, 23)),
        ],
    )
    .unwrap_err();
    assert_eq!(duplicate_enum[0].code, DiagnosticCode::DuplicateVariant);
    let duplicate_record = RecordDeclaration::new(
        name("DuplicateRecord"),
        span(23, 26),
        vec![
            RecordFieldDeclaration::new(name("same"), ValueType::Boolean, span(24, 25)),
            RecordFieldDeclaration::new(name("same"), ValueType::Boolean, span(25, 26)),
        ],
    )
    .unwrap_err();
    assert_eq!(duplicate_record[0].code, DiagnosticCode::DuplicateField);
    let duplicate_parameter = PureFunctionDeclaration::new(
        name("duplicate_parameters"),
        vec![
            FunctionParameter::new(name("same"), ValueType::Boolean, span(26, 27)),
            FunctionParameter::new(name("same"), ValueType::Boolean, span(27, 28)),
        ],
        ValueType::Boolean,
        span(26, 29),
    )
    .unwrap_err();
    assert_eq!(
        duplicate_parameter[0].code,
        DiagnosticCode::DuplicateParameter
    );

    let cycle = RecordDeclaration::new(
        name("Cycle"),
        span(30, 33),
        vec![RecordFieldDeclaration::new(
            name("next"),
            ValueType::option(ValueType::Record {
                name: name("Cycle"),
            }),
            span(31, 32),
        )],
    )
    .unwrap();
    let cycle_error = DeclarationEnvironment::new(
        environment.owner().clone(),
        vec![TypeDeclaration::Record { declaration: cycle }],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert_eq!(cycle_error[0].code, DiagnosticCode::RecursiveType);

    let orphan = ValueDeclaration::new(
        name("orphan"),
        ValueDeclarationKind::Input,
        ValueType::Record {
            name: name("Missing"),
        },
        span(33, 34),
    );
    assert_eq!(
        DeclarationEnvironment::new(environment.owner().clone(), vec![], vec![orphan], vec![],)
            .unwrap_err()[0]
            .code,
        DiagnosticCode::OrphanedTypeReference
    );

    let record_literal = Expression::new(
        ExpressionKind::RecordLiteral {
            record: name("Sensor"),
            fields: vec![RecordLiteralField::new(name("reading"), int(2, 40))],
        },
        span(39, 42),
    );
    let record_checked = environment
        .check_expression(
            &record_literal,
            &ValueType::Record {
                name: name("Sensor"),
            },
            &pre(),
            false,
        )
        .unwrap();
    assert!(record_checked
        .dependencies()
        .iter()
        .any(|dependency| dependency.kind() == DependencyKind::Field));

    let enum_literal = Expression::new(
        ExpressionKind::EnumLiteral {
            enumeration: name("Color"),
            variant: name("red"),
        },
        span(42, 43),
    );
    assert!(environment
        .check_expression(
            &enum_literal,
            &ValueType::Enum {
                name: name("Color"),
            },
            &pre(),
            false,
        )
        .unwrap()
        .dependencies()
        .iter()
        .any(|dependency| dependency.kind() == DependencyKind::EnumVariant));

    let call = Expression::new(
        ExpressionKind::Call {
            function: name("identity"),
            arguments: vec![int(1, 44)],
        },
        span(43, 46),
    );
    let checked = environment
        .check_expression(&call, &ValueType::integer(int_type()), &pre(), false)
        .unwrap();
    assert!(checked
        .dependencies()
        .iter()
        .any(|dependency| dependency.kind() == DependencyKind::PureFunction));
    let arity = Expression::new(
        ExpressionKind::Call {
            function: name("identity"),
            arguments: vec![],
        },
        span(46, 47),
    );
    assert_eq!(
        environment
            .check_expression(&arity, &ValueType::integer(int_type()), &pre(), false)
            .unwrap_err()[0]
            .code,
        DiagnosticCode::ArityMismatch
    );

    let invalid_observation = value("state_value", StateObservation::Post, 48);
    assert_eq!(
        environment
            .check_expression(
                &invalid_observation,
                &ValueType::integer(int_type()),
                &pre(),
                false,
            )
            .unwrap_err()[0]
            .code,
        DiagnosticCode::InvalidStateObservation
    );

    let missing_local = Expression::new(
        ExpressionKind::LocalReference {
            name: name("outside"),
        },
        span(49, 50),
    );
    assert_eq!(
        environment
            .check_expression(&missing_local, &ValueType::Boolean, &pre(), false,)
            .unwrap_err()[0]
            .code,
        DiagnosticCode::InvalidScope
    );

    assert_eq!(
        environment
            .check_expression(&int(1, 50), &ValueType::Boolean, &pre(), true)
            .unwrap_err()[0]
            .code,
        DiagnosticCode::NonBooleanClauseRoot
    );

    let operators = [
        BooleanOperator::ShortCircuitAnd,
        BooleanOperator::ShortCircuitOr,
        BooleanOperator::TotalAnd,
        BooleanOperator::TotalOr,
        BooleanOperator::Implication,
    ];
    let serialized = serde_json::to_string(&operators).unwrap();
    for spelling in [
        "short_circuit_and",
        "short_circuit_or",
        "total_and",
        "total_or",
        "implication",
    ] {
        assert!(serialized.contains(spelling));
    }

    let public =
        serde_json::to_string(&(environment, record_literal, enum_literal, call, operators))
            .unwrap()
            .to_ascii_lowercase();
    for forbidden in [
        "rust", "gumbo", "aadl", "hamr", "solver", "runtime", "usize", "pathbuf",
    ] {
        assert!(
            !public.contains(forbidden),
            "forbidden vocabulary {forbidden}"
        );
    }
}

/// Tracing: TC-016
/// TC-016.
/// FR-014-AC-1.
/// FR-014-AC-4.
/// FR-015-AC-1.
/// FR-015-AC-2.
/// FR-015-AC-3.
/// FR-015-AC-4.
/// FR-015-AC-5.
/// FR-015-AC-6.
/// FR-015-AC-7.
#[test]
fn tc_016_definedness_guards_ranges_and_quantifiers_fail_closed() {
    let environment = environment();
    let divisor = value("divisor", StateObservation::Current, 60);
    let nonzero = compare(
        ComparisonOperator::NotEqual,
        divisor.clone(),
        int(0, 61),
        60,
    );
    let division_is_bounded = compare(
        ComparisonOperator::LessEqual,
        divide(divisor.clone(), 63),
        int(10, 67),
        63,
    );
    let guarded = bool_op(
        BooleanOperator::ShortCircuitAnd,
        nonzero.clone(),
        division_is_bounded.clone(),
        60,
    );
    let checked = environment
        .check_expression(&guarded, &ValueType::Boolean, &pre(), true)
        .unwrap();
    assert!(checked
        .obligations()
        .iter()
        .any(|obligation| { obligation.kind() == DefinednessObligationKind::NonZeroDivisor }));

    let total = bool_op(BooleanOperator::TotalAnd, nonzero, division_is_bounded, 60);
    let diagnostic = &environment
        .check_expression(&total, &ValueType::Boolean, &pre(), true)
        .unwrap_err()[0];
    assert_eq!(diagnostic.code, DiagnosticCode::PotentiallyUndefined);
    assert_eq!(
        diagnostic.obligation_kind,
        Some(DefinednessObligationKind::NonZeroDivisor)
    );

    let maybe = value("maybe", StateObservation::Pre, 70);
    let present = Expression::new(
        ExpressionKind::IsPresent {
            option: Box::new(maybe.clone()),
        },
        span(70, 72),
    );
    let unwrap = Expression::new(
        ExpressionKind::Unwrap {
            option: Box::new(maybe),
        },
        span(73, 75),
    );
    let unwrap_check = compare(ComparisonOperator::GreaterEqual, unwrap, int(-10, 76), 73);
    assert!(environment
        .check_expression(
            &bool_op(BooleanOperator::Implication, present, unwrap_check, 70),
            &ValueType::Boolean,
            &pre(),
            true,
        )
        .is_ok());

    let collection = value("items", StateObservation::Pre, 80);
    let position = value("position", StateObservation::Current, 81);
    let length = Expression::new(
        ExpressionKind::Length {
            collection: Box::new(collection.clone()),
        },
        span(82, 84),
    );
    let in_bounds = compare(ComparisonOperator::Less, position.clone(), length, 81);
    let indexed = Expression::new(
        ExpressionKind::Index {
            collection: Box::new(collection.clone()),
            index: Box::new(position),
        },
        span(85, 88),
    );
    let indexed_check = compare(ComparisonOperator::GreaterEqual, indexed, int(-10, 89), 85);
    assert!(environment
        .check_expression(
            &bool_op(
                BooleanOperator::ShortCircuitAnd,
                in_bounds,
                indexed_check,
                81,
            ),
            &ValueType::Boolean,
            &pre(),
            true,
        )
        .is_ok());

    let local = name("index");
    let local_reference = Expression::new(
        ExpressionKind::LocalReference {
            name: local.clone(),
        },
        span(92, 93),
    );
    let indexed = Expression::new(
        ExpressionKind::Index {
            collection: Box::new(collection.clone()),
            index: Box::new(local_reference),
        },
        span(91, 94),
    );
    let predicate = compare(ComparisonOperator::GreaterEqual, indexed, int(-10, 95), 91);
    let quantifier = Expression::new(
        ExpressionKind::Quantifier {
            quantifier: QuantifierKind::ForAll,
            domain: QuantifierDomain::Indices,
            collection: Box::new(collection),
            local,
            local_source: span(90, 91),
            predicate: Box::new(predicate),
        },
        span(90, 96),
    );
    assert!(environment
        .check_expression(&quantifier, &ValueType::Boolean, &pre(), true)
        .is_ok());

    let rational = RationalType::new(-10, 10, 8).unwrap();
    let half = Expression::new(
        ExpressionKind::RationalLiteral {
            numerator: 2,
            denominator: 4,
            value_type: rational.clone(),
        },
        span(97, 98),
    );
    let rational_add = Expression::new(
        ExpressionKind::Numeric {
            operator: NumericOperator::Add,
            left: Box::new(half.clone()),
            right: Box::new(half),
        },
        span(97, 100),
    );
    assert!(environment
        .check_expression(&rational_add, &ValueType::rational(rational), &pre(), false,)
        .is_ok());
}

/// Tracing: TC-016
/// TC-016.
/// FR-014-AC-5.
/// NFR-002-AC-4.
#[test]
fn tc_016_expression_limits_preflight_before_recursive_typing() {
    let environment = environment();
    let mut at_limit = Expression::new(
        ExpressionKind::BooleanLiteral { value: true },
        span(100, 101),
    );
    for _ in 1..MAX_EXPRESSION_DEPTH {
        at_limit = Expression::new(
            ExpressionKind::BooleanNot {
                operand: Box::new(at_limit),
            },
            span(100, 101),
        );
    }
    assert!(environment
        .check_expression(&at_limit, &ValueType::Boolean, &pre(), false)
        .is_ok());

    let beyond = Expression::new(
        ExpressionKind::BooleanNot {
            operand: Box::new(at_limit),
        },
        span(100, 101),
    );
    assert_eq!(
        environment
            .check_expression(&beyond, &ValueType::Boolean, &pre(), false)
            .unwrap_err()[0]
            .code,
        DiagnosticCode::ExpressionTooLarge
    );

    let item = Expression::new(
        ExpressionKind::BooleanLiteral { value: true },
        span(102, 103),
    );
    let at_node_limit = Expression::new(
        ExpressionKind::CollectionLiteral {
            value_type: CollectionType::new(ValueType::Boolean, MAX_EXPRESSION_NODES).unwrap(),
            items: vec![item.clone(); MAX_EXPRESSION_NODES as usize - 1],
        },
        span(102, 104),
    );
    let collection_type = match at_node_limit.kind() {
        ExpressionKind::CollectionLiteral { value_type, .. } => value_type.clone(),
        _ => unreachable!(),
    };
    assert!(environment
        .check_expression(
            &at_node_limit,
            &ValueType::collection(collection_type.clone()),
            &pre(),
            false,
        )
        .is_ok());

    let beyond_node_limit = Expression::new(
        ExpressionKind::CollectionLiteral {
            value_type: collection_type.clone(),
            items: vec![item; MAX_EXPRESSION_NODES as usize],
        },
        span(102, 104),
    );
    assert_eq!(
        environment
            .check_expression(
                &beyond_node_limit,
                &ValueType::collection(collection_type),
                &pre(),
                false,
            )
            .unwrap_err()[0]
            .code,
        DiagnosticCode::ExpressionTooLarge
    );

    let too_long = Expression::new(
        ExpressionKind::TextLiteral {
            value: "x".repeat(MAX_TEXT_LENGTH as usize + 1),
        },
        span(105, 106),
    );
    assert_eq!(
        environment
            .check_expression(&too_long, &ValueType::Text, &pre(), false)
            .unwrap_err()[0]
            .code,
        DiagnosticCode::TextBoundExceeded
    );
}

/// Tracing: TC-016.
/// FR-013-AC-1.
/// FR-013-AC-3.
/// FR-013-AC-4.
/// FR-014-AC-1.
/// STD-001.
#[test]
fn tc_016_declaration_and_literal_failures_are_distinct_and_ordered() {
    let base = environment();
    let duplicated_type = base.types()[0].clone();
    assert_eq!(
        DeclarationEnvironment::new(
            base.owner().clone(),
            vec![duplicated_type.clone(), duplicated_type],
            vec![],
            vec![],
        )
        .unwrap_err()[0]
            .code,
        DiagnosticCode::DuplicateTypeDeclaration
    );

    let duplicated_value = base.values()[0].clone();
    assert_eq!(
        DeclarationEnvironment::new(
            base.owner().clone(),
            base.types().to_vec(),
            vec![duplicated_value.clone(), duplicated_value],
            vec![],
        )
        .unwrap_err()[0]
            .code,
        DiagnosticCode::DuplicateValueDeclaration
    );

    let duplicated_function = base.functions()[0].clone();
    assert_eq!(
        DeclarationEnvironment::new(
            base.owner().clone(),
            base.types().to_vec(),
            vec![],
            vec![duplicated_function.clone(), duplicated_function],
        )
        .unwrap_err()[0]
            .code,
        DiagnosticCode::DuplicateFunctionDeclaration
    );

    let pair = RecordDeclaration::new(
        name("Pair"),
        span(110, 115),
        vec![
            RecordFieldDeclaration::new(
                name("left"),
                ValueType::integer(int_type()),
                span(111, 112),
            ),
            RecordFieldDeclaration::new(
                name("right"),
                ValueType::integer(int_type()),
                span(112, 113),
            ),
        ],
    )
    .unwrap();
    let mut types = base.types().to_vec();
    types.push(TypeDeclaration::Record { declaration: pair });
    let environment = DeclarationEnvironment::new(
        base.owner().clone(),
        types,
        base.values().to_vec(),
        base.functions().to_vec(),
    )
    .unwrap();

    let duplicate_fields = Expression::new(
        ExpressionKind::RecordLiteral {
            record: name("MissingRecord"),
            fields: vec![
                RecordLiteralField::new(name("same"), int(1, 116)),
                RecordLiteralField::new(name("same"), int(2, 117)),
            ],
        },
        span(115, 119),
    );
    let diagnostics = environment
        .check_expression(
            &duplicate_fields,
            &ValueType::Record {
                name: name("MissingRecord"),
            },
            &pre(),
            false,
        )
        .unwrap_err();
    assert_eq!(diagnostics[0].code, DiagnosticCode::DuplicateField);
    assert_eq!(diagnostics[0].span.as_deref(), Some(&span(117, 118)));

    let two_bad_fields = Expression::new(
        ExpressionKind::RecordLiteral {
            record: name("Pair"),
            fields: vec![
                RecordLiteralField::new(name("left"), boolean(true, 120)),
                RecordLiteralField::new(name("right"), boolean(false, 121)),
            ],
        },
        span(119, 123),
    );
    let diagnostics = environment
        .check_expression(
            &two_bad_fields,
            &ValueType::Record { name: name("Pair") },
            &pre(),
            false,
        )
        .unwrap_err();
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code == DiagnosticCode::IllTypedExpression));

    let missing_field = Expression::new(
        ExpressionKind::RecordLiteral {
            record: name("Pair"),
            fields: vec![RecordLiteralField::new(name("left"), int(1, 124))],
        },
        span(123, 126),
    );
    assert_code(
        environment.check_expression(
            &missing_field,
            &ValueType::Record { name: name("Pair") },
            &pre(),
            false,
        ),
        DiagnosticCode::IllTypedExpression,
    );

    let unknown_field = Expression::new(
        ExpressionKind::RecordLiteral {
            record: name("Pair"),
            fields: vec![
                RecordLiteralField::new(name("left"), int(1, 127)),
                RecordLiteralField::new(name("unknown"), int(2, 128)),
            ],
        },
        span(126, 130),
    );
    assert_code(
        environment.check_expression(
            &unknown_field,
            &ValueType::Record { name: name("Pair") },
            &pre(),
            false,
        ),
        DiagnosticCode::IllTypedExpression,
    );

    let collection_type = CollectionType::new(ValueType::Boolean, 1).unwrap();
    let excessive = Expression::new(
        ExpressionKind::CollectionLiteral {
            value_type: collection_type.clone(),
            items: vec![boolean(true, 131), boolean(false, 132)],
        },
        span(130, 134),
    );
    assert_code(
        environment.check_expression(
            &excessive,
            &ValueType::collection(collection_type.clone()),
            &pre(),
            false,
        ),
        DiagnosticCode::CollectionBoundExceeded,
    );
    let wrong_item = Expression::new(
        ExpressionKind::CollectionLiteral {
            value_type: collection_type.clone(),
            items: vec![int(1, 135)],
        },
        span(134, 137),
    );
    assert_code(
        environment.check_expression(
            &wrong_item,
            &ValueType::collection(collection_type),
            &pre(),
            false,
        ),
        DiagnosticCode::IllTypedExpression,
    );

    for (expression, expected, code) in [
        (
            Expression::new(
                ExpressionKind::EnumLiteral {
                    enumeration: name("MissingEnum"),
                    variant: name("value"),
                },
                span(138, 139),
            ),
            ValueType::Enum {
                name: name("MissingEnum"),
            },
            DiagnosticCode::OrphanedTypeReference,
        ),
        (
            value("missing_value", StateObservation::Current, 139),
            ValueType::Boolean,
            DiagnosticCode::OrphanedValueReference,
        ),
        (
            Expression::new(
                ExpressionKind::Call {
                    function: name("missing_function"),
                    arguments: vec![],
                },
                span(140, 141),
            ),
            ValueType::Boolean,
            DiagnosticCode::OrphanedFunctionReference,
        ),
    ] {
        assert_code(
            environment.check_expression(&expression, &expected, &pre(), false),
            code,
        );
    }
}

/// Tracing: TC-016.
/// FR-014-AC-4.
/// FR-015-AC-1.
/// FR-015-AC-4.
/// FR-015-AC-5.
/// FR-015-AC-7.
#[test]
fn tc_016_numeric_policies_normalization_and_proof_spans_conform() {
    let base = environment();
    let saturating = integer(-10, 10, OverflowPolicy::Saturate);
    let values = vec![ValueDeclaration::new(
        name("saturating_divisor"),
        ValueDeclarationKind::Input,
        ValueType::integer(saturating.clone()),
        span(150, 151),
    )];
    let environment = DeclarationEnvironment::new(
        base.owner().clone(),
        base.types().to_vec(),
        values,
        base.functions().to_vec(),
    )
    .unwrap();
    let literal = |value, at| {
        Expression::new(
            ExpressionKind::IntegerLiteral {
                value,
                value_type: saturating.clone(),
            },
            span(at, at + 1),
        )
    };
    let divide = Expression::new(
        ExpressionKind::Numeric {
            operator: NumericOperator::Divide,
            left: Box::new(literal(-10, 151)),
            right: Box::new(literal(-1, 152)),
        },
        span(151, 154),
    );
    let checked = environment
        .check_expression(
            &divide,
            &ValueType::integer(saturating.clone()),
            &pre(),
            false,
        )
        .unwrap();
    assert_eq!(checked.obligations().len(), 1);
    assert_eq!(
        checked.obligations()[0].kind(),
        DefinednessObligationKind::NonZeroDivisor
    );
    assert_eq!(checked.obligations()[0].proof_span(), &span(152, 153));

    let saturating_add = Expression::new(
        ExpressionKind::Numeric {
            operator: NumericOperator::Add,
            left: Box::new(literal(10, 155)),
            right: Box::new(literal(10, 156)),
        },
        span(155, 158),
    );
    assert!(environment
        .check_expression(
            &saturating_add,
            &ValueType::integer(saturating),
            &pre(),
            false,
        )
        .unwrap()
        .obligations()
        .is_empty());

    let rational_type = RationalType::new(-100, 100, 64).unwrap();
    let rational = |numerator, denominator, at| {
        Expression::new(
            ExpressionKind::RationalLiteral {
                numerator,
                denominator,
                value_type: rational_type.clone(),
            },
            span(at, at + 1),
        )
    };
    let normalized = environment
        .check_expression(
            &rational(2, 4, 160),
            &ValueType::rational(rational_type.clone()),
            &pre(),
            false,
        )
        .unwrap();
    assert!(matches!(
        normalized.expression().kind(),
        ExpressionKind::RationalLiteral {
            numerator: 1,
            denominator: 2,
            ..
        }
    ));

    for (operator, left, right) in [
        (NumericOperator::Add, (1, 2), (1, 3)),
        (NumericOperator::Subtract, (1, 2), (1, 3)),
        (NumericOperator::Multiply, (1, 2), (1, 3)),
        (NumericOperator::Divide, (1, 2), (1, 3)),
    ] {
        let expression = Expression::new(
            ExpressionKind::Numeric {
                operator,
                left: Box::new(rational(left.0, left.1, 161)),
                right: Box::new(rational(right.0, right.1, 162)),
            },
            span(161, 164),
        );
        let checked = environment
            .check_expression(
                &expression,
                &ValueType::rational(rational_type.clone()),
                &pre(),
                false,
            )
            .unwrap();
        assert!(checked
            .obligations()
            .iter()
            .any(|obligation| obligation.kind() == DefinednessObligationKind::CheckedRange));
    }

    let negate = Expression::new(
        ExpressionKind::NumericNegate {
            operand: Box::new(rational(1, 2, 165)),
        },
        span(165, 167),
    );
    assert!(environment
        .check_expression(&negate, &ValueType::rational(rational_type), &pre(), false,)
        .unwrap()
        .obligations()
        .iter()
        .any(|obligation| obligation.kind() == DefinednessObligationKind::CheckedRange));
}

/// Tracing: TC-016.
/// FR-014-AC-3.
/// FR-015-AC-3.
/// FR-015-AC-5.
/// FR-015-AC-6.
/// STD-001.
#[test]
fn tc_016_scope_guard_flow_call_ranges_and_diagnostic_invariants_conform() {
    let environment = environment();
    let collection = value("items", StateObservation::Pre, 170);
    let repeated = name("repeated");
    let inner = Expression::new(
        ExpressionKind::Quantifier {
            quantifier: QuantifierKind::Exists,
            domain: QuantifierDomain::Elements,
            collection: Box::new(collection.clone()),
            local: repeated.clone(),
            local_source: span(173, 174),
            predicate: Box::new(boolean(true, 174)),
        },
        span(172, 176),
    );
    let nested = Expression::new(
        ExpressionKind::Quantifier {
            quantifier: QuantifierKind::ForAll,
            domain: QuantifierDomain::Elements,
            collection: Box::new(collection.clone()),
            local: repeated,
            local_source: span(171, 172),
            predicate: Box::new(inner),
        },
        span(170, 177),
    );
    let diagnostics = environment
        .check_expression(&nested, &ValueType::Boolean, &pre(), true)
        .unwrap_err();
    assert_eq!(diagnostics[0].code, DiagnosticCode::InvalidScope);
    assert_eq!(diagnostics[0].span.as_deref(), Some(&span(173, 174)));

    let divisor = value("divisor", StateObservation::Current, 180);
    let equals_zero = compare(ComparisonOperator::Equal, int(0, 181), divisor.clone(), 180);
    let division_check = compare(
        ComparisonOperator::LessEqual,
        divide(divisor.clone(), 183),
        int(10, 187),
        183,
    );
    let guarded_or = bool_op(
        BooleanOperator::ShortCircuitOr,
        equals_zero.clone(),
        division_check.clone(),
        180,
    );
    assert!(environment
        .check_expression(&guarded_or, &ValueType::Boolean, &pre(), true)
        .is_ok());

    let not_zero = Expression::new(
        ExpressionKind::BooleanNot {
            operand: Box::new(equals_zero),
        },
        span(189, 191),
    );
    let guarded_not = bool_op(
        BooleanOperator::ShortCircuitAnd,
        not_zero,
        division_check,
        189,
    );
    assert!(environment
        .check_expression(&guarded_not, &ValueType::Boolean, &pre(), true)
        .is_ok());

    let reversed_collection = value("items", StateObservation::Pre, 188);
    let reversed_position = value("position", StateObservation::Current, 189);
    let reversed_length = Expression::new(
        ExpressionKind::Length {
            collection: Box::new(reversed_collection.clone()),
        },
        span(188, 190),
    );
    let reversed_bound = compare(
        ComparisonOperator::Greater,
        reversed_length,
        reversed_position.clone(),
        188,
    );
    let reversed_index = Expression::new(
        ExpressionKind::Index {
            collection: Box::new(reversed_collection),
            index: Box::new(reversed_position),
        },
        span(190, 193),
    );
    let reversed_index_check = compare(
        ComparisonOperator::GreaterEqual,
        reversed_index,
        int(-10, 193),
        190,
    );
    assert!(environment
        .check_expression(
            &bool_op(
                BooleanOperator::ShortCircuitAnd,
                reversed_bound,
                reversed_index_check,
                188,
            ),
            &ValueType::Boolean,
            &pre(),
            true,
        )
        .is_ok());

    let local = name("element");
    let local_reference = Expression::new(
        ExpressionKind::LocalReference {
            name: local.clone(),
        },
        span(192, 193),
    );
    let predicate = compare(
        ComparisonOperator::NotEqual,
        local_reference,
        int(0, 193),
        192,
    );
    let quantifier = Expression::new(
        ExpressionKind::Quantifier {
            quantifier: QuantifierKind::ForAll,
            domain: QuantifierDomain::Elements,
            collection: Box::new(collection),
            local,
            local_source: span(191, 192),
            predicate: Box::new(predicate),
        },
        span(191, 195),
    );
    let unguarded_after_quantifier = bool_op(
        BooleanOperator::ShortCircuitAnd,
        quantifier,
        compare(
            ComparisonOperator::LessEqual,
            divide(divisor.clone(), 196),
            int(10, 200),
            196,
        ),
        191,
    );
    let diagnostic = environment
        .check_expression(
            &unguarded_after_quantifier,
            &ValueType::Boolean,
            &pre(),
            true,
        )
        .unwrap_err()
        .remove(0);
    assert_eq!(diagnostic.code, DiagnosticCode::PotentiallyUndefined);
    assert_eq!(
        diagnostic.obligation_kind,
        Some(DefinednessObligationKind::NonZeroDivisor)
    );
    assert!(serde_json::to_string(&diagnostic).is_ok());

    let mut missing_kind = diagnostic.clone();
    missing_kind.obligation_kind = None;
    assert!(serde_json::to_string(&missing_kind).is_err());
    let invalid_wire = r#"{
        "code":"ill_typed_expression",
        "severity":"error",
        "message":"invalid",
        "path":"expression",
        "obligation_kind":"checked_range"
    }"#;
    assert!(serde_json::from_str::<quire_contract_ir::Diagnostic>(invalid_wire).is_err());

    let call = |at| {
        Expression::new(
            ExpressionKind::Call {
                function: name("identity"),
                arguments: vec![int(1, at + 1)],
            },
            span(at, at + 2),
        )
    };
    let call_plus_one = |at| {
        Expression::new(
            ExpressionKind::Numeric {
                operator: NumericOperator::Add,
                left: Box::new(call(at + 1)),
                right: Box::new(int(1, at + 3)),
            },
            span(at, at + 5),
        )
    };
    assert_code(
        environment.check_expression(
            &call_plus_one(205),
            &ValueType::integer(int_type()),
            &pre(),
            false,
        ),
        DiagnosticCode::PotentiallyUndefined,
    );
    let call_bound = compare(ComparisonOperator::LessEqual, call(211), int(9, 214), 211);
    let guarded_call = bool_op(
        BooleanOperator::ShortCircuitAnd,
        call_bound,
        compare(
            ComparisonOperator::LessEqual,
            call_plus_one(216),
            int(10, 221),
            216,
        ),
        211,
    );
    assert!(environment
        .check_expression(&guarded_call, &ValueType::Boolean, &pre(), true)
        .is_ok());
}

/// Tracing: TC-016.
/// FR-012-AC-5.
/// FR-014-AC-6.
#[test]
fn tc_016_mixed_dependencies_implement_the_structural_source_contract() {
    let environment = environment();
    let input = compare(
        ComparisonOperator::Equal,
        value("divisor", StateObservation::Current, 230),
        int(0, 231),
        230,
    );
    let state = compare(
        ComparisonOperator::Equal,
        value("state_value", StateObservation::Pre, 233),
        int(0, 234),
        233,
    );
    let field = Expression::new(
        ExpressionKind::FieldAccess {
            base: Box::new(value("sensor", StateObservation::Pre, 236)),
            field: name("reading"),
        },
        span(236, 238),
    );
    let field_check = compare(ComparisonOperator::Equal, field, int(0, 239), 236);
    let enum_left = Expression::new(
        ExpressionKind::EnumLiteral {
            enumeration: name("Color"),
            variant: name("red"),
        },
        span(241, 242),
    );
    let enum_right = Expression::new(
        ExpressionKind::EnumLiteral {
            enumeration: name("Color"),
            variant: name("blue"),
        },
        span(242, 243),
    );
    let enum_check = compare(ComparisonOperator::NotEqual, enum_left, enum_right, 241);
    let call = Expression::new(
        ExpressionKind::Call {
            function: name("identity"),
            arguments: vec![int(1, 245)],
        },
        span(244, 247),
    );
    let call_check = compare(ComparisonOperator::Equal, call, int(1, 247), 244);
    let combined = [state, field_check, enum_check, call_check]
        .into_iter()
        .fold(input, |left, right| {
            bool_op(BooleanOperator::TotalAnd, left, right, 230)
        });
    let checked = environment
        .check_expression(&combined, &ValueType::Boolean, &pre(), true)
        .unwrap();

    assert_eq!(
        checked.dependencies(),
        &quire_contract_ir::DependencySource::dependencies(&checked)
    );
    let actual = checked
        .dependencies()
        .iter()
        .map(|dependency| {
            (
                dependency.kind(),
                dependency.observation(),
                dependency
                    .path()
                    .iter()
                    .map(|segment| segment.as_str())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            (
                DependencyKind::Input,
                Some(StateObservation::Current),
                vec!["divisor"],
            ),
            (
                DependencyKind::State,
                Some(StateObservation::Pre),
                vec!["sensor"],
            ),
            (
                DependencyKind::State,
                Some(StateObservation::Pre),
                vec!["state_value"],
            ),
            (DependencyKind::Field, None, vec!["Sensor", "reading"]),
            (DependencyKind::EnumVariant, None, vec!["Color", "blue"],),
            (DependencyKind::EnumVariant, None, vec!["Color", "red"],),
            (DependencyKind::PureFunction, None, vec!["identity"]),
        ]
    );
}

/// Tracing: TC-016.
/// FR-015-AC-3.
/// FR-015-AC-5.
/// FR-015-AC-6.
/// FR-015-AC-7.
#[test]
fn tc_016_rational_literal_guards_refine_nonzero_and_ordered_ranges() {
    let base = environment();
    let unit_type = RationalType::new(-1, 1, 1).unwrap();
    let bounded_type = RationalType::new(-10, 10, 1).unwrap();
    let environment = DeclarationEnvironment::new(
        base.owner().clone(),
        base.types().to_vec(),
        vec![
            ValueDeclaration::new(
                name("rational_unit"),
                ValueDeclarationKind::Input,
                ValueType::rational(unit_type.clone()),
                span(250, 251),
            ),
            ValueDeclaration::new(
                name("rational_bound"),
                ValueDeclarationKind::Input,
                ValueType::rational(bounded_type.clone()),
                span(251, 252),
            ),
        ],
        vec![],
    )
    .unwrap();
    let rational = |numerator, value_type: &RationalType, at| {
        Expression::new(
            ExpressionKind::RationalLiteral {
                numerator,
                denominator: 1,
                value_type: value_type.clone(),
            },
            span(at, at + 1),
        )
    };
    let unit_reference = Expression::new(
        ExpressionKind::ValueReference {
            name: name("rational_unit"),
            observation: StateObservation::Current,
        },
        span(253, 254),
    );
    let nonzero = compare(
        ComparisonOperator::NotEqual,
        rational(0, &unit_type, 254),
        unit_reference.clone(),
        253,
    );
    let division = Expression::new(
        ExpressionKind::Numeric {
            operator: NumericOperator::Divide,
            left: Box::new(rational(1, &unit_type, 257)),
            right: Box::new(unit_reference.clone()),
        },
        span(257, 260),
    );
    assert_code(
        environment.check_expression(
            &division,
            &ValueType::rational(unit_type.clone()),
            &pre(),
            false,
        ),
        DiagnosticCode::PotentiallyUndefined,
    );
    let division_check = compare(
        ComparisonOperator::GreaterEqual,
        division,
        rational(-1, &unit_type, 261),
        257,
    );
    let checked = environment
        .check_expression(
            &bool_op(
                BooleanOperator::ShortCircuitAnd,
                nonzero,
                division_check,
                253,
            ),
            &ValueType::Boolean,
            &pre(),
            true,
        )
        .unwrap();
    assert!(checked
        .obligations()
        .iter()
        .any(|obligation| obligation.kind() == DefinednessObligationKind::NonZeroDivisor));

    let bounded_reference = || {
        Expression::new(
            ExpressionKind::ValueReference {
                name: name("rational_bound"),
                observation: StateObservation::Current,
            },
            span(265, 266),
        )
    };
    let add_one = || {
        Expression::new(
            ExpressionKind::Numeric {
                operator: NumericOperator::Add,
                left: Box::new(bounded_reference()),
                right: Box::new(rational(1, &bounded_type, 267)),
            },
            span(265, 269),
        )
    };
    assert_code(
        environment.check_expression(
            &add_one(),
            &ValueType::rational(bounded_type.clone()),
            &pre(),
            false,
        ),
        DiagnosticCode::PotentiallyUndefined,
    );
    let upper_bound = compare(
        ComparisonOperator::LessEqual,
        bounded_reference(),
        rational(9, &bounded_type, 270),
        265,
    );
    let bounded_add_check = compare(
        ComparisonOperator::LessEqual,
        add_one(),
        rational(10, &bounded_type, 274),
        265,
    );
    assert!(environment
        .check_expression(
            &bool_op(
                BooleanOperator::ShortCircuitAnd,
                upper_bound,
                bounded_add_check,
                265,
            ),
            &ValueType::Boolean,
            &pre(),
            true,
        )
        .is_ok());
}
