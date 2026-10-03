// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-274 part B: every byte and digest of the v1 canonical objects comes from
//! `quire-canonical`, and the eight integer members are decimal strings in
//! every canonical object (FR-016-AC-5 through FR-016-AC-8, FR-011-AC-3).
//!
//! Every expected byte string below is written out member by member from
//! RFC 8785 (members in UTF-16 order, minimal escapes), not computed by a call
//! into this crate or into the encoder.

use crate::checked_package_v2_identity_digests::{production_source, repository_path};
use ix_trace_rs::trace;
use quire_contract_ir::{
    AnchorName, CanonicalKind, CanonicalOutput, CanonicalProfile, Clause, ClauseId, ClauseKind,
    CollectionType, ContractPackage, DeclarationEnvironment, DiagnosticCode, ExecutionPoint,
    Expression, ExpressionKind, IntegerDomain, IntegerType, OverflowPolicy, PackageId,
    RationalType, ReferenceBody, Requirement, RequirementId, RequirementRef, RequirementRevision,
    SchemaVersion, SourceDocumentId, SourceIdentity, SourceLocation, SourceRevision, SourceSpan,
    SymbolName, TypedExpression, ValueDeclaration, ValueDeclarationKind, ValueType,
};
use sha2::{Digest as _, Sha256};
use std::fs;

const PROFILE: &str = "quire.contract.canonical-json/v1";

fn source() -> SourceIdentity {
    SourceIdentity::new(
        SourceDocumentId::new("exact").unwrap(),
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

fn owner() -> RequirementRef {
    RequirementRef::parse("agent-ix/pkg", "REQ_a", 1).unwrap()
}

fn package_with_revision(revision: u64) -> ContractPackage<ReferenceBody> {
    let package = PackageId::new("agent-ix/pkg").unwrap();
    let requirement = Requirement::new(
        &package,
        RequirementId::new("REQ_a").unwrap(),
        RequirementRevision::new(revision).unwrap(),
        span(0, 10),
        vec![Clause::new(
            ClauseId::new("note").unwrap(),
            ClauseKind::Information,
            None,
            span(1, 9),
            ReferenceBody::Literal,
        )
        .unwrap()],
    )
    .unwrap();
    ContractPackage::new(package, SchemaVersion::V1_1, source(), vec![requirement]).unwrap()
}

fn typed(expression: Expression, expected: &ValueType) -> TypedExpression {
    DeclarationEnvironment::new(owner(), vec![], vec![], vec![])
        .unwrap()
        .check_expression(
            &expression,
            expected,
            &ExecutionPoint::Pre {
                operation: AnchorName::new("render").unwrap(),
            },
            false,
        )
        .unwrap()
}

fn widest_integer() -> IntegerType {
    IntegerType::new(
        IntegerDomain::Signed,
        i64::MIN,
        i64::MAX,
        OverflowPolicy::Reject,
    )
    .unwrap()
}

fn widest_rational() -> RationalType {
    RationalType::new(i64::MIN, i64::MAX, i64::MAX as u64).unwrap()
}

fn text(output: &CanonicalOutput) -> &str {
    std::str::from_utf8(output.bytes().as_slice()).unwrap()
}

/// SHA-256 over the domain prefix `quire-contract-ir`, a zero byte, the
/// profile, a zero byte, the kind, a zero byte and the expected bytes.
fn expected_digest(kind: &str, bytes: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"quire-contract-ir\0");
    hasher.update(PROFILE.as_bytes());
    hasher.update(b"\0");
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(bytes.as_bytes());
    format!("{:x}", hasher.finalize())
}

const PACKAGE_BYTES: &str = concat!(
    "{\"kind\":\"package\",\"profile\":\"quire.contract.canonical-json/v1\",\"value\":",
    "{\"id\":\"agent-ix/pkg\",\"requirements\":[{\"clauses\":[{\"body\":{\"node\":\"literal\"},",
    "\"id\":\"note\",\"kind\":\"information\",\"requirement\":{\"package\":\"agent-ix/pkg\",",
    "\"requirement\":\"REQ_a\",\"revision\":1}}],\"id\":\"REQ_a\",\"package\":\"agent-ix/pkg\",",
    "\"revision\":1}],\"schema_version\":{\"major\":1,\"minor\":1}}}"
);

const REQUIREMENT_BYTES: &str = concat!(
    "{\"kind\":\"requirement\",\"profile\":\"quire.contract.canonical-json/v1\",\"value\":",
    "{\"clauses\":[{\"body\":{\"node\":\"literal\"},\"id\":\"note\",\"kind\":\"information\",",
    "\"requirement\":{\"package\":\"agent-ix/pkg\",\"requirement\":\"REQ_a\",\"revision\":1}}],",
    "\"id\":\"REQ_a\",\"package\":\"agent-ix/pkg\",\"revision\":1}}"
);

const CLAUSE_BYTES: &str = concat!(
    "{\"kind\":\"clause\",\"profile\":\"quire.contract.canonical-json/v1\",\"value\":",
    "{\"body\":{\"node\":\"literal\"},\"id\":\"note\",\"kind\":\"information\",",
    "\"requirement\":{\"package\":\"agent-ix/pkg\",\"requirement\":\"REQ_a\",\"revision\":1}}}"
);

const DECLARATION_BYTES: &str = concat!(
    "{\"kind\":\"declaration\",\"profile\":\"quire.contract.canonical-json/v1\",\"value\":",
    "{\"functions\":[],\"owner\":{\"package\":\"agent-ix/pkg\",\"requirement\":\"REQ_a\",",
    "\"revision\":1},\"types\":[],\"values\":[{\"kind\":\"input\",\"name\":\"n\",",
    "\"value_type\":{\"kind\":\"integer\",\"value\":{\"domain\":\"signed\",",
    "\"maximum\":\"9223372036854775807\",\"minimum\":\"-9223372036854775808\",",
    "\"overflow\":\"reject\"}}},{\"kind\":\"state\",\"name\":\"q\",\"value_type\":",
    "{\"kind\":\"rational\",\"value\":{\"maximum_denominator\":\"9223372036854775807\",",
    "\"numerator_maximum\":\"9223372036854775807\",",
    "\"numerator_minimum\":\"-9223372036854775808\"}}}]}}"
);

const EXPRESSION_BYTES: &str = concat!(
    "{\"kind\":\"expression\",\"profile\":\"quire.contract.canonical-json/v1\",\"value\":",
    "{\"result_type\":{\"kind\":\"integer\",\"value\":{\"domain\":\"signed\",",
    "\"maximum\":\"9223372036854775807\",\"minimum\":\"-9223372036854775808\",",
    "\"overflow\":\"reject\"}},\"tree\":{\"kind\":{\"node\":\"integer_literal\",",
    "\"value\":\"-1\",\"value_type\":{\"domain\":\"signed\",",
    "\"maximum\":\"9223372036854775807\",\"minimum\":\"-9223372036854775808\",",
    "\"overflow\":\"reject\"}}}}}"
);

fn declaration_fixture() -> DeclarationEnvironment {
    DeclarationEnvironment::new(
        owner(),
        vec![],
        vec![
            ValueDeclaration::new(
                SymbolName::new("q").unwrap(),
                ValueDeclarationKind::State,
                ValueType::rational(widest_rational()),
                span(2, 3),
            ),
            ValueDeclaration::new(
                SymbolName::new("n").unwrap(),
                ValueDeclarationKind::Input,
                ValueType::integer(widest_integer()),
                span(1, 2),
            ),
        ],
        vec![],
    )
    .unwrap()
}

fn expression_fixture() -> TypedExpression {
    typed(
        Expression::new(
            ExpressionKind::IntegerLiteral {
                value: -1,
                value_type: widest_integer(),
            },
            span(0, 1),
        ),
        &ValueType::integer(widest_integer()),
    )
}

/// One fixture of each of the five kinds, with the bytes it must produce and
/// a closure that asks for them under a byte limit.
type Budgeted<'a> = Box<dyn Fn(u64) -> Result<CanonicalOutput, quire_contract_ir::Diagnostic> + 'a>;

/// Tracing: TC-017.
/// ACs: FR-016-AC-5, FR-016-AC-6
#[trace("TC-017", "FR-016-AC-5", "FR-016-AC-6")]
#[test]
fn tc_017_the_five_kinds_equal_their_written_out_bytes_digests_and_limits() {
    let package = package_with_revision(1);
    let requirement = &package.requirements()[0];
    let clause = &requirement.clauses()[0];
    let environment = declaration_fixture();
    let expression = expression_fixture();
    let cases: [(&str, &str, Budgeted<'_>); 5] = [
        (
            "package",
            PACKAGE_BYTES,
            Box::new(|limit| package.canonical_package_with_limit(CanonicalProfile::V1, limit)),
        ),
        (
            "requirement",
            REQUIREMENT_BYTES,
            Box::new(|limit| {
                package.canonical_requirement_with_limit(requirement, CanonicalProfile::V1, limit)
            }),
        ),
        (
            "clause",
            CLAUSE_BYTES,
            Box::new(|limit| {
                package.canonical_clause_with_limit(
                    requirement,
                    clause,
                    CanonicalProfile::V1,
                    limit,
                )
            }),
        ),
        (
            "declaration",
            DECLARATION_BYTES,
            Box::new(|limit| {
                environment.canonical_declaration_with_limit(CanonicalProfile::V1, limit)
            }),
        ),
        (
            "expression",
            EXPRESSION_BYTES,
            Box::new(|limit| {
                expression.canonical_expression_with_limit(CanonicalProfile::V1, limit)
            }),
        ),
    ];
    for (kind, expected, ask) in cases {
        let output = ask(u64::MAX).unwrap();
        assert_eq!(output.kind().as_str(), kind);
        assert_eq!(text(&output), expected, "{kind}: bytes");
        assert_eq!(
            output.digest().to_string(),
            expected_digest(kind, expected),
            "{kind}: digest"
        );
        // A limit of the exact length returns the bytes; one byte lower is the
        // registered resource refusal, with no bytes and no digest.
        let length = u64::try_from(expected.len()).unwrap();
        assert_eq!(ask(length).unwrap(), output, "{kind}: exact limit");
        let refusal = ask(length - 1).unwrap_err();
        assert_eq!(
            refusal.code,
            DiagnosticCode::CanonicalizationResourceExhausted,
            "{kind}: one byte under"
        );
    }
}

/// Tracing: TC-017.
/// ACs: FR-016-AC-5
#[trace("TC-017", "FR-016-AC-5")]
#[test]
fn tc_017_the_eight_integer_members_are_strings_and_the_rest_stay_numbers() {
    // An integer type of 0 and 0 holds "0" and "0".
    let zero = IntegerType::new(IntegerDomain::Unsigned, 0, 0, OverflowPolicy::Saturate).unwrap();
    let zero_output = typed(
        Expression::new(
            ExpressionKind::IntegerLiteral {
                value: 0,
                value_type: zero.clone(),
            },
            span(0, 1),
        ),
        &ValueType::integer(zero),
    )
    .canonical_expression(CanonicalProfile::V1)
    .unwrap();
    assert_eq!(
        text(&zero_output),
        concat!(
            "{\"kind\":\"expression\",\"profile\":\"quire.contract.canonical-json/v1\",",
            "\"value\":{\"result_type\":{\"kind\":\"integer\",\"value\":{\"domain\":\"unsigned\",",
            "\"maximum\":\"0\",\"minimum\":\"0\",\"overflow\":\"saturate\"}},",
            "\"tree\":{\"kind\":{\"node\":\"integer_literal\",\"value\":\"0\",",
            "\"value_type\":{\"domain\":\"unsigned\",\"maximum\":\"0\",\"minimum\":\"0\",",
            "\"overflow\":\"saturate\"}}}}}"
        )
    );

    // A rational literal -3/4 holds "-3" and "4", under the widest rational
    // type.
    let rational = typed(
        Expression::new(
            ExpressionKind::RationalLiteral {
                numerator: -3,
                denominator: 4,
                value_type: widest_rational(),
            },
            span(0, 1),
        ),
        &ValueType::rational(widest_rational()),
    )
    .canonical_expression(CanonicalProfile::V1)
    .unwrap();
    assert_eq!(
        text(&rational),
        concat!(
            "{\"kind\":\"expression\",\"profile\":\"quire.contract.canonical-json/v1\",",
            "\"value\":{\"result_type\":{\"kind\":\"rational\",\"value\":",
            "{\"maximum_denominator\":\"9223372036854775807\",",
            "\"numerator_maximum\":\"9223372036854775807\",",
            "\"numerator_minimum\":\"-9223372036854775808\"}},",
            "\"tree\":{\"kind\":{\"denominator\":\"4\",\"node\":\"rational_literal\",",
            "\"numerator\":\"-3\",\"value_type\":",
            "{\"maximum_denominator\":\"9223372036854775807\",",
            "\"numerator_maximum\":\"9223372036854775807\",",
            "\"numerator_minimum\":\"-9223372036854775808\"}}}}}"
        )
    );

    // `maximum_items` and `revision` of the same objects stay JSON numbers.
    let collection = CollectionType::new(ValueType::integer(widest_integer()), 2).unwrap();
    let items = typed(
        Expression::new(
            ExpressionKind::CollectionLiteral {
                value_type: collection.clone(),
                items: vec![],
            },
            span(0, 1),
        ),
        &ValueType::collection(collection),
    )
    .canonical_expression(CanonicalProfile::V1)
    .unwrap();
    assert!(
        text(&items).contains("\"maximum_items\":2"),
        "{}",
        text(&items)
    );
    assert!(text(&items).contains("\"element\":{\"kind\":\"integer\""));
    assert!(text(&items).contains("\"minimum\":\"-9223372036854775808\""));
    assert!(PACKAGE_BYTES.contains("\"revision\":1}"));
    assert!(text(
        &package_with_revision(7)
            .canonical_package(CanonicalProfile::V1)
            .unwrap()
    )
    .contains("\"revision\":7}"));

    // No recorded canonical fixture spells any of the eight members as a
    // number: no `"<name>":` is followed by a digit or a minus sign.
    let names = [
        "minimum",
        "maximum",
        "numerator_minimum",
        "numerator_maximum",
        "maximum_denominator",
        "value",
        "numerator",
        "denominator",
    ];
    let directory = repository_path("corpus/contract-v0.1/canonical");
    let mut scanned = 0_usize;
    let mut holding = 0_usize;
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let bytes = fs::read_to_string(&path).unwrap();
        scanned += 1;
        for name in names {
            let needle = format!("\"{name}\":");
            for (at, _) in bytes.match_indices(&needle) {
                let next = bytes[at + needle.len()..].chars().next().unwrap();
                assert!(
                    !(next.is_ascii_digit() || next == '-'),
                    "{}: `{needle}` is followed by `{next}`",
                    path.display()
                );
                if next == '"' && name != "value" {
                    holding += 1;
                }
            }
        }
    }
    assert!(
        scanned >= 75,
        "the scan read the published corpus: {scanned}"
    );
    assert!(holding > 0, "the scan found the members it is looking for");
}

/// Tracing: TC-017.
/// ACs: FR-016-AC-6
#[trace("TC-017", "FR-016-AC-6")]
#[test]
fn tc_017_the_canonical_source_holds_no_encoder_of_its_own() {
    for file in ["canonical.rs", "binding.rs", "output_mapping.rs"] {
        let text = fs::read_to_string(repository_path(&format!(
            "crates/quire-contract-model/src/{file}"
        )))
        .unwrap();
        let production = production_source(&text);
        for symbol in [
            "CanonicalWriter",
            "canonical_envelope_bytes",
            "digest_json",
            "serde_json_canonicalizer",
            "serde_json::to_vec",
            "serde_json::to_value",
        ] {
            assert_eq!(production.matches(symbol).count(), 0, "{file}: `{symbol}`");
        }
    }
    for manifest in ["Cargo.toml", "crates/quire-contract-model/Cargo.toml"] {
        let text = fs::read_to_string(repository_path(manifest)).unwrap();
        assert_eq!(
            text.matches("serde_json_canonicalizer").count(),
            0,
            "{manifest}"
        );
    }
}

/// The source of the named `struct` or `enum`, from its declaration to its
/// closing brace (or its `;` for a tuple struct).
fn definition(source: &str, name: &str) -> String {
    let lines = source.lines().collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| {
            [
                format!("pub struct {name} "),
                format!("pub struct {name}<"),
                format!("pub struct {name}("),
                format!("pub enum {name} "),
                format!("struct {name} "),
                format!("struct {name}<"),
            ]
            .iter()
            .any(|prefix| line.starts_with(prefix.as_str()))
        })
        .unwrap_or_else(|| panic!("no definition of {name}"));
    let mut block = String::new();
    for line in &lines[start..] {
        block.push_str(line);
        block.push('\n');
        if *line == "}" || line.ends_with(';') {
            break;
        }
    }
    block
}

/// Tracing: TC-017.
/// ACs: FR-016-AC-7, FR-011-AC-3
#[trace("TC-017", "FR-016-AC-7", "FR-011-AC-3")]
#[test]
fn tc_017_no_canonical_type_can_hold_a_null_a_float_or_a_value() {
    let read = |file: &str| {
        production_source(
            &fs::read_to_string(repository_path(&format!(
                "crates/quire-contract-model/src/{file}"
            )))
            .unwrap(),
        )
    };
    let expression = read("expression.rs");
    let canonical = read("canonical.rs");
    let types = [
        "ValueType",
        "IntegerType",
        "RationalType",
        "CollectionType",
        "Expression",
        "ExpressionKind",
        "RecordLiteralField",
        "TypedExpression",
        "DeclarationEnvironment",
        "EnumDeclaration",
        "EnumVariantDeclaration",
        "RecordDeclaration",
        "RecordFieldDeclaration",
        "TypeDeclaration",
        "ValueDeclaration",
        "FunctionParameter",
        "PureFunctionDeclaration",
    ];
    let projections = [
        "PackageProjection",
        "RequirementProjection",
        "ClauseProjection",
    ];
    let blocks = types
        .iter()
        .map(|name| (*name, definition(&expression, name)))
        .chain(
            projections
                .iter()
                .map(|name| (*name, definition(&canonical, name))),
        )
        .collect::<Vec<_>>();
    for (name, block) in &blocks {
        assert!(block.len() > 20, "{name}: the scan read its definition");
        for token in block.split(|c: char| !c.is_alphanumeric() && c != '_') {
            assert!(
                !matches!(token, "f32" | "f64" | "Value"),
                "{name} declares `{token}`"
            );
        }
        assert!(
            !block.contains("Option<"),
            "{name} declares an Option member"
        );
    }

    // A requirement revision of 2^53 canonicalizes as the number, and the
    // types refuse one above it.
    let at_bound = package_with_revision(9_007_199_254_740_992);
    let output = at_bound.canonical_package(CanonicalProfile::V1).unwrap();
    assert!(
        text(&output).contains("\"revision\":9007199254740992}"),
        "{}",
        text(&output)
    );
    assert_eq!(output.kind(), CanonicalKind::Package);
    assert_eq!(
        RequirementRevision::new(9_007_199_254_740_993)
            .unwrap_err()
            .code,
        DiagnosticCode::InvalidRequirementRevision
    );
}

/// Tracing: TC-017.
/// ACs: FR-016-AC-8
#[trace("TC-017", "FR-016-AC-8")]
#[test]
fn tc_017_the_digest_is_the_explicit_domain_prefix_not_the_encoder_domain_framing() {
    let package = package_with_revision(1);
    let output = package.canonical_package(CanonicalProfile::V1).unwrap();
    assert_eq!(
        output.digest().to_string(),
        expected_digest("package", PACKAGE_BYTES)
    );
    // `sha256_with_domain` frames the label with its length and hashes the
    // same envelope, so even with the same label it is a different identity.
    let document = quire_canonical::read(PACKAGE_BYTES.as_bytes(), 1 << 20).unwrap();
    let framed = quire_canonical::sha256_with_domain(
        b"quire-contract-ir\0quire.contract.canonical-json/v1\0package\0",
        &document,
        quire_canonical::Limits::new(1 << 20),
    )
    .unwrap();
    assert_ne!(output.digest().to_string(), framed.to_string());
    let bare = quire_canonical::sha256(&document, quire_canonical::Limits::new(1 << 20)).unwrap();
    assert_ne!(output.digest().to_string(), bare.to_string());
}
