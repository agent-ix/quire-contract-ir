//! FR-014-AC-4: rechecking identical declarations, context, expression and expected type
//! produces structurally equal typed output, ordered diagnostics and dependency identities.

use ix_trace_rs::trace;
use quire_contract_model::{
    AnchorName, BooleanOperator, ComparisonOperator, DeclarationEnvironment, DependencyKind,
    Diagnostic, DiagnosticCode, EnumDeclaration, EnumVariantDeclaration, ExecutionPoint,
    Expression, ExpressionKind, FunctionParameter, IntegerDomain, IntegerType, NumericOperator,
    OverflowPolicy, PackageId, PureFunctionDeclaration, RecordDeclaration, RecordFieldDeclaration,
    RecordLiteralField, RequirementId, RequirementRef, RequirementRevision, SourceDocumentId,
    SourceIdentity, SourceLocation, SourceRevision, SourceSpan, StateObservation, SymbolName,
    TypeDeclaration, TypedExpression, ValueDeclaration, ValueDeclarationKind, ValueType,
};

fn name(value: &str) -> SymbolName {
    SymbolName::new(value).unwrap()
}

fn span(start: u64, end: u64) -> SourceSpan {
    let source = SourceIdentity::new(
        SourceDocumentId::new("recheck").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    SourceSpan::new(
        SourceLocation::new(source.clone(), 1, start as u32 + 1, start).unwrap(),
        SourceLocation::new(source, 1, end as u32 + 1, end).unwrap(),
    )
    .unwrap()
}

fn int_type() -> IntegerType {
    IntegerType::new(IntegerDomain::Signed, -10, 10, OverflowPolicy::Reject).unwrap()
}

fn int(value: i128, at: u64) -> Expression {
    Expression::new(
        ExpressionKind::IntegerLiteral {
            value,
            value_type: int_type(),
        },
        span(at, at + 1),
    )
}

fn value(declared: &str, observation: StateObservation, at: u64) -> Expression {
    Expression::new(
        ExpressionKind::ValueReference {
            name: name(declared),
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

fn pre() -> ExecutionPoint {
    ExecutionPoint::Pre {
        operation: AnchorName::new("update").unwrap().into(),
    }
}

/// Builds the declarations from scratch, so two calls give independent identical environments.
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
    let pair = PureFunctionDeclaration::new(
        name("pair"),
        vec![
            FunctionParameter::new(name("left"), ValueType::Boolean, span(19, 20)),
            FunctionParameter::new(name("right"), ValueType::Boolean, span(20, 21)),
        ],
        ValueType::Boolean,
        span(19, 22),
    )
    .unwrap();
    DeclarationEnvironment::new(
        RequirementRef::new(
            PackageId::new("agent-ix/contract").unwrap(),
            RequirementId::new("REQ_recheck").unwrap(),
            RequirementRevision::new(1).unwrap(),
        ),
        vec![
            TypeDeclaration::Enum { declaration: color },
            TypeDeclaration::Record {
                declaration: sensor,
            },
        ],
        values,
        vec![function, pair],
    )
    .unwrap()
}

struct Case {
    label: &'static str,
    expression: Expression,
    expected: ValueType,
    clause_root: bool,
    /// Whether the input is accepted, so the test also proves both outcomes are exercised.
    accepted: bool,
}

fn case(
    label: &'static str,
    expression: Expression,
    expected: ValueType,
    clause_root: bool,
    accepted: bool,
) -> Case {
    Case {
        label,
        expression,
        expected,
        clause_root,
        accepted,
    }
}

fn cases() -> Vec<Case> {
    let divisor = value("divisor", StateObservation::Current, 60);
    let nonzero = compare(
        ComparisonOperator::NotEqual,
        divisor.clone(),
        int(0, 61),
        60,
    );
    let bounded = compare(
        ComparisonOperator::LessEqual,
        divide(divisor, 63),
        int(10, 67),
        63,
    );
    let guarded = bool_op(
        BooleanOperator::ShortCircuitAnd,
        nonzero.clone(),
        bounded.clone(),
        60,
    );
    let total = bool_op(BooleanOperator::TotalAnd, nonzero, bounded, 60);
    let record = Expression::new(
        ExpressionKind::RecordLiteral {
            record: name("Sensor"),
            fields: vec![RecordLiteralField::new(name("reading"), int(2, 40))],
        },
        span(39, 42),
    );
    let enum_literal = Expression::new(
        ExpressionKind::EnumLiteral {
            enumeration: name("Color"),
            variant: name("red"),
        },
        span(42, 43),
    );
    let call = |arguments| {
        Expression::new(
            ExpressionKind::Call {
                function: name("identity"),
                arguments,
            },
            span(43, 46),
        )
    };
    let field_compare = compare(
        ComparisonOperator::Equal,
        Expression::new(
            ExpressionKind::FieldAccess {
                base: Box::new(value("sensor", StateObservation::Pre, 70)),
                field: name("reading"),
            },
            span(70, 72),
        ),
        call(vec![value("state_value", StateObservation::Pre, 73)]),
        70,
    );
    let int_ty = ValueType::integer(int_type());
    vec![
        case("guarded division", guarded, ValueType::Boolean, true, true),
        case(
            "pre-state field and call",
            field_compare,
            ValueType::Boolean,
            true,
            true,
        ),
        case(
            "record literal",
            record,
            ValueType::Record {
                name: name("Sensor"),
            },
            false,
            true,
        ),
        case(
            "enum literal",
            enum_literal,
            ValueType::Enum {
                name: name("Color"),
            },
            false,
            true,
        ),
        case(
            "pure call",
            call(vec![int(1, 44)]),
            int_ty.clone(),
            false,
            true,
        ),
        case(
            "total conjunction is potentially undefined",
            total,
            ValueType::Boolean,
            true,
            false,
        ),
        case(
            "two ill-typed call arguments give two ordered diagnostics",
            Expression::new(
                ExpressionKind::Call {
                    function: name("pair"),
                    arguments: vec![int(1, 56), int(2, 57)],
                },
                span(55, 58),
            ),
            ValueType::Boolean,
            false,
            false,
        ),
        case("call arity", call(vec![]), int_ty.clone(), false, false),
        case(
            "post observation at pre point",
            value("state_value", StateObservation::Post, 48),
            int_ty.clone(),
            false,
            false,
        ),
        case(
            "unbound local",
            Expression::new(
                ExpressionKind::LocalReference {
                    name: name("outside"),
                },
                span(49, 50),
            ),
            ValueType::Boolean,
            false,
            false,
        ),
        case(
            "non-Boolean clause root",
            int(1, 50),
            ValueType::Boolean,
            true,
            false,
        ),
        case(
            "result type mismatch",
            int(1, 51),
            ValueType::Boolean,
            false,
            false,
        ),
        case(
            "ill-typed comparison operands",
            compare(
                ComparisonOperator::Equal,
                int(1, 52),
                Expression::new(ExpressionKind::BooleanLiteral { value: true }, span(53, 54)),
                52,
            ),
            ValueType::Boolean,
            true,
            false,
        ),
    ]
}

/// Typed output (dependencies are a BTreeSet, so their order is the set's structural order and
/// set equality decides it) or the ordered diagnostics.
#[derive(Debug, PartialEq)]
enum Outcome {
    Typed { output: Box<TypedExpression> },
    Rejected { diagnostics: Vec<Diagnostic> },
}

fn check(environment: &DeclarationEnvironment, case: &Case) -> Outcome {
    // The expression is cloned per check: identical input, not the same allocation.
    let expression = case.expression.clone();
    match environment.check_expression(&expression, &case.expected, &pre(), case.clause_root) {
        Ok(output) => Outcome::Typed {
            output: Box::new(output),
        },
        Err(diagnostics) => Outcome::Rejected { diagnostics },
    }
}

#[trace("TC-016", "FR-014-AC-4")]
#[test]
fn rechecking_identical_input_gives_equal_typed_output_diagnostics_and_dependencies() {
    let shared = environment();
    let mut accepted = 0;
    let mut rejected = 0;
    for case in cases() {
        let first = check(&shared, &case);
        // Same environment again, then an independently rebuilt identical environment.
        let second = check(&shared, &case);
        let third = check(&environment(), &case);

        match (&first, &second, &third) {
            (
                Outcome::Typed { output: a },
                Outcome::Typed { output: b },
                Outcome::Typed { output: c },
            ) => {
                assert!(case.accepted, "{} unexpectedly accepted", case.label);
                accepted += 1;
                for other in [b, c] {
                    assert_eq!(a.expression(), other.expression(), "{}", case.label);
                    assert_eq!(a.value_type(), other.value_type(), "{}", case.label);
                    assert_eq!(a.nodes(), other.nodes(), "{}", case.label);
                    assert_eq!(a.obligations(), other.obligations(), "{}", case.label);
                    assert_eq!(a.dependencies(), other.dependencies(), "{}", case.label);
                }
            }
            (
                Outcome::Rejected { diagnostics: a },
                Outcome::Rejected { diagnostics: b },
                Outcome::Rejected { diagnostics: c },
            ) => {
                assert!(!case.accepted, "{} unexpectedly rejected", case.label);
                assert!(!a.is_empty(), "{}", case.label);
                if case.label.starts_with("two ill-typed") {
                    assert!(a.len() >= 2, "ordered comparison needs several diagnostics");
                }
                rejected += 1;
                // Vec equality compares code, message, path, span and order.
                assert_eq!(a, b, "{}", case.label);
                assert_eq!(a, c, "{}", case.label);
            }
            _ => panic!("{}: recheck changed accepted/rejected outcome", case.label),
        }
        assert_eq!(first, second, "{}", case.label);
        assert_eq!(first, third, "{}", case.label);
    }
    assert_eq!((accepted, rejected), (5, 8));

    // The compared dependency set is non-trivial: it carries more than one identity kind.
    let dependent = cases()
        .into_iter()
        .find(|case| case.label == "pre-state field and call")
        .unwrap();
    let Outcome::Typed { output, .. } = check(&shared, &dependent) else {
        panic!("dependent case must be accepted");
    };
    let kinds: Vec<_> = output.dependencies().iter().map(|d| d.kind()).collect();
    assert!(kinds.contains(&DependencyKind::Field));
    assert!(kinds.contains(&DependencyKind::PureFunction));
    assert!(output.dependencies().len() >= 3);

    let arity = cases()
        .into_iter()
        .find(|case| case.label == "call arity")
        .unwrap();
    let Outcome::Rejected { diagnostics } = check(&shared, &arity) else {
        panic!("arity case must be rejected");
    };
    assert_eq!(diagnostics[0].code, DiagnosticCode::ArityMismatch);
}
