use super::*;
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};

/// [`read_semantic_ir`] under a work limit nothing reaches.
fn read(document: &Value) -> Result<DomainModel, ModelRefusal> {
    let mut meter = WorkMeter::new(u64::MAX);
    match read_semantic_ir(document, &mut Budget::new(&mut meter, 0)) {
        Ok(model) => Ok(model),
        Err(ModelFailure::Refused(refusal)) => Err(refusal),
        Err(ModelFailure::Limit(_)) => panic!("no limit is reached"),
    }
}

/// [`DomainModel::resolve`] under a work limit nothing reaches.
fn resolve<'m>(
    model: &'m DomainModel,
    node: &str,
    kind: MemberKind,
    name: &str,
) -> Result<Resolved<'m>, ModelRefusal> {
    let mut meter = WorkMeter::new(u64::MAX);
    match model.resolve(node, kind, name, &mut Budget::new(&mut meter, 0)) {
        Ok(resolved) => Ok(resolved),
        Err(ModelFailure::Refused(refusal)) => Err(refusal),
        Err(ModelFailure::Limit(_)) => panic!("no limit is reached"),
    }
}

/// QSL FR-092 golden keys (quire-spec-language
/// `spec/functional/FR-092-key-type-parameter-and-declared-nodes.md`), the
/// anonymous structural nodes a member type names.
const T1_BOOLEAN: &str = "9964390677844ad66b781babdbfa95933bc2b16ef1e86f67005966b77e6db3aa";
const T2_INTEGER: &str = "07f6dca966d22bde13d3bb198f12610e57d8e1e04d0476bbab03f405d2b04e32";
const T4_INT_0_9: &str = "652cc5b63910aca98b8c91b1c1ba42a8da1f568517c70f37de083414cd192477";
const T5_OPTION_INT_0_9: &str = "7bacf8b16f079a2352aac1a83b88a5925ed3f8c00e3a02735e8d3c646bb6461e";
const T6_SEQUENCE_INT_0_9: &str =
    "27355f7b7768c0596876c2173262651d3d36946f3e8e53cef42d51551c804956";
const T7_SEQUENCE_INT_0_9_0_5: &str =
    "515cb5664eae047ff9a2568809e02e6936cc6c8ab63fe54864f1e8f546e5f2c2";

const DIGIT: IntegerBounds = IntegerBounds { lower: 0, upper: 9 };

/// A member type's node key is the key QSL FR-092 gives the same type.
///
/// Tracing: TC-048, FR-038-AC-29
#[test]
fn tc_048_member_type_keys_are_qsl_fr_092_structural_keys() {
    let digit = || Box::new(MemberType::IntRange(DIGIT));
    let sequence = |bounds| MemberType::Collection {
        kind: CollectionKind::Sequence,
        element: digit(),
        bounds,
    };
    for (member_type, key) in [
        (MemberType::Boolean, T1_BOOLEAN),
        (MemberType::Integer, T2_INTEGER),
        (MemberType::IntRange(DIGIT), T4_INT_0_9),
        (MemberType::Option(digit()), T5_OPTION_INT_0_9),
        (sequence(None), T6_SEQUENCE_INT_0_9),
        (sequence(Some((0, 5))), T7_SEQUENCE_INT_0_9_0_5),
    ] {
        assert_eq!(member_type.node_key(), key, "{member_type:?}");
    }
}

fn multiplicity(lower: u64, upper: Option<u64>) -> Value {
    let mut value = json!({"lower": lower, "ordered": false, "unique": true});
    if let Some(upper) = upper {
        value["upper"] = json!(upper);
    }
    value
}

fn field(owner: &str, name: &str, type_ref: &str) -> Value {
    json!({
        "identity": format!("{owner}/{name}"), "name": name, "typeRef": type_ref,
        "presence": "required", "nullable": false, "defaultKind": "none",
        "multiplicity": multiplicity(1, Some(1)),
    })
}

fn object_type(node: &str, supertypes: &[&str], fields: Vec<Value>) -> Value {
    json!({
        "identity": node, "displayName": node, "kind": {"module": "acme/orders", "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes, "fields": fields, "operations": [],
    })
}

/// A Semantic IR 2.0.0 document of `acme/orders` `1.0.0` declaring `types`.
fn document(types: Vec<Value>) -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": "acme/orders", "version": "1.0.0"},
        "constructs": [
            {"kind": {"module": "acme/orders", "name": "entity"},
             "construct": {"meaning": "quire.meaning.model.object-type/v1"}},
            {"kind": {"module": "acme/orders", "name": "digit"},
             "construct": {"meaning": "quire.meaning.model.value-type/v1"}},
        ],
        "types": types,
    })
}

const WIDGET: &str = "ix://acme/orders/Widget";
const GADGET: &str = "ix://acme/orders/Gadget";

/// FR-154 over a Semantic IR document: an inherited field resolves on the
/// subtype and a subtype conforms to its supertype, decided in either order.
///
/// Tracing: TC-048, FR-038-AC-29
#[test]
fn tc_048_a_semantic_ir_document_reads_inherited_members_and_conformance() {
    let model = read(&document(vec![
        object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "code", "ix://quire/native/Integer")],
        ),
        object_type(GADGET, &[WIDGET], vec![]),
    ]))
    .expect("the document reads");
    let Resolved::Field(code) = resolve(&model, GADGET, MemberKind::Field, "code")
        .expect("code resolves on the subtype")
    else {
        panic!("a field");
    };
    assert_eq!(code.identity.as_ref(), "ix://acme/orders/Widget/code");
    assert_eq!(model.field_type(code), Some(MemberType::Integer));
    let conforms = |a, b| {
        let mut meter = WorkMeter::new(u64::MAX);
        model
            .conforms(a, b, &mut Budget::new(&mut meter, 0))
            .expect("no limit is reached")
    };
    assert!(conforms(GADGET, WIDGET));
    assert!(
        conforms(WIDGET, GADGET),
        "conformance is decided in either order"
    );
    assert_eq!(
        resolve(&model, GADGET, MemberKind::Operation, "code").map(|_| ()),
        Err(ModelRefusal::ineligible())
    );
}

/// An FCD value type `scalar: integer` with `min` and `max` constraints is
/// the element type `Int[lo, hi]`.
///
/// Tracing: TC-048, FR-038-AC-29
#[test]
fn tc_048_a_semantic_ir_integer_value_type_is_an_integer_range() {
    let digit = json!({
        "identity": "ix://acme/orders/Digit", "displayName": "Digit",
        "kind": {"module": "acme/orders", "name": "digit"}, "roles": [], "extensions": [],
        "unknownPolicy": "reject", "scalar": "integer",
        "constraints": [
            {"keyword": "min", "operands": {"value": 0}},
            {"keyword": "max", "operands": {"value": "9"}},
        ],
    });
    let model = read(&document(vec![
        digit,
        object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "digit", "ix://acme/orders/Digit")],
        ),
    ]))
    .expect("the document reads");
    let Resolved::Field(digit) = resolve(&model, WIDGET, MemberKind::Field, "digit").expect("digit")
    else {
        panic!("a field");
    };
    assert_eq!(model.field_type(digit), Some(MemberType::IntRange(DIGIT)));
}

/// FR-154's declaration refusals the reader draws, each for one defect.
///
/// Tracing: TC-048, FR-038-AC-28
#[test]
fn tc_048_semantic_ir_declaration_defects_refuse_with_their_fr_154_cause() {
    let refused = |types| read(&document(types)).map(|_| ()).expect_err("refused");
    let widget = || object_type(WIDGET, &[], vec![]);
    assert_eq!(
        refused(vec![object_type(
            GADGET,
            &["ix://acme/orders/Nope"],
            vec![]
        )]),
        ModelRefusal::new(Code::MissingDeclaration, Cause::MissingName)
    );
    assert_eq!(
        refused(vec![object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "x", "ix://acme/orders/Nope")]
        )]),
        ModelRefusal::new(Code::MissingDeclaration, Cause::MissingName)
    );
    assert_eq!(
        refused(vec![object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "x", "ix://quire/native/Uuid")]
        )]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration)
    );
    assert_eq!(
        refused(vec![widget(), widget()]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::ConflictingBinding)
    );
    assert_eq!(
        refused(vec![object_type("ix://acme/orders/sys-pump", &[], vec![])]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration)
    );
    let mut backwards = field(WIDGET, "x", "ix://quire/native/Integer");
    backwards["multiplicity"] = multiplicity(3, Some(1));
    assert_eq!(
        refused(vec![object_type(WIDGET, &[], vec![backwards])]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::UnpreservedModelMeaning)
    );
    let mut unknown_kind = widget();
    unknown_kind["kind"] = json!({"module": "acme/orders", "name": "nope"});
    assert_eq!(
        refused(vec![unknown_kind]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration)
    );
}

/// FR-322 step 1 over a Semantic IR document: the digest is recomputed from
/// the bytes and the document's own `package` identity and version compared.
///
/// Tracing: TC-048, FR-038-AC-27
#[test]
fn tc_048_a_selection_admits_only_the_document_it_names() {
    let document = document(vec![object_type(WIDGET, &[], vec![])]);
    let bytes = serde_json::to_vec(&document).expect("bytes");
    let digest = digest_json(&document).expect("digest");
    let selection = |version: &str, digest: &str| CheckedDomainPackageRef {
        identity: "acme/orders".into(),
        version: version.into(),
        digest_domain: DOMAIN_PACKAGE_DIGEST.into(),
        digest: digest.into(),
    };
    let mut evidence = CheckedPackageEvidence::new();
    evidence.insert_domain_package_document(digest.clone(), bytes.clone());
    let admit = |selection: &CheckedDomainPackageRef, evidence: &CheckedPackageEvidence| {
        let mut meter = WorkMeter::new(u64::MAX);
        admit_selection(selection, evidence, &mut Budget::new(&mut meter, 0))
    };
    let admitted = admit(&selection("1.0.0", &digest), &evidence).expect("admitted");
    assert!(admitted.object_types.contains_key(WIDGET));
    let refusal = |selection: &CheckedDomainPackageRef, evidence: &CheckedPackageEvidence| {
        match admit(selection, evidence) {
            Err(SelectionFailure::Refused(refusal)) => refusal,
            other => panic!("expected a refusal, got {other:?}"),
        }
    };
    assert_eq!(
        refusal(&selection("1.0.1", &digest), &evidence),
        SelectionRefusal::at(
            Code::InvalidModelBinding,
            Cause::WrongModelSelection,
            Some("version")
        )
    );
    assert_eq!(
        refusal(&selection("1.0.0", &"0".repeat(64)), &evidence),
        SelectionRefusal::at(Code::MissingImport, Cause::MissingSelection, Some("digest"))
    );
    let mut forged = CheckedPackageEvidence::new();
    forged.insert_domain_package_document(digest.clone(), b"{}".to_vec());
    assert_eq!(
        refusal(&selection("1.0.0", &digest), &forged),
        SelectionRefusal::at(
            Code::StaleDependency,
            Cause::ByteDigestMismatch,
            Some("digest")
        )
    );
}

const ZETA: &str = "ix://acme/orders/Zeta";
const BAD_ID: &str = "ix://acme/orders/bad-id";

/// FR-154: a reference to a node refused for its own object id is not
/// reported again as `missing-name`; the refused node's own refusal stands,
/// whether the referencing node is read before or after it.
///
/// Tracing: TC-048, FR-038-AC-28
#[test]
fn tc_048_a_reference_to_a_refused_node_leaves_that_nodes_own_refusal() {
    let malformed = ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration);
    let bad = || object_type(BAD_ID, &[], vec![]);
    // `Gadget` sorts before `bad-id`; `Zeta` after `Widget` but before it too.
    for referencing in [
        object_type(GADGET, &[BAD_ID], vec![]),
        object_type(
            GADGET,
            &[],
            vec![field(GADGET, "peer", BAD_ID)],
        ),
        object_type(WIDGET, &[BAD_ID], vec![]),
    ] {
        assert_eq!(
            read(&document(vec![referencing, bad()])).map(|_| ()),
            Err(malformed)
        );
    }
    // A node refused for a duplicated identity stands for its references too.
    assert_eq!(
        read(&document(vec![
            object_type(GADGET, &[WIDGET], vec![]),
            object_type(WIDGET, &[], vec![]),
            object_type(WIDGET, &[], vec![]),
        ]))
        .map(|_| ()),
        Err(ModelRefusal::new(
            Code::InvalidModelBinding,
            Cause::ConflictingBinding
        ))
    );
}

/// FR-154: within one node the failures report in the declaration-refusal
/// table's order, so a dangling `typeRef` (`missing-name`) reports before a
/// multiplicity with `lower > upper` (`unpreserved-model-meaning`), and a
/// malformed member before either, wherever each sits in the node.
///
/// Tracing: TC-048, FR-038-AC-28
#[test]
fn tc_048_one_nodes_failures_report_in_table_order() {
    let missing = ModelRefusal::new(Code::MissingDeclaration, Cause::MissingName);
    let unpreserved = ModelRefusal::new(Code::InvalidModelBinding, Cause::UnpreservedModelMeaning);
    let malformed = ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration);
    let mut backwards = field(WIDGET, "a", "ix://quire/native/Integer");
    backwards["multiplicity"] = multiplicity(3, Some(1));
    let dangling = field(WIDGET, "b", "ix://acme/orders/Nope");
    for fields in [
        vec![backwards.clone(), dangling.clone()],
        vec![dangling.clone(), backwards.clone()],
    ] {
        assert_eq!(
            read(&document(vec![object_type(WIDGET, &[], fields)])).map(|_| ()),
            Err(missing)
        );
    }
    // The same field failing both rows: the reference reports first.
    let mut both = field(WIDGET, "a", "ix://acme/orders/Nope");
    both["multiplicity"] = multiplicity(3, Some(1));
    assert_eq!(
        read(&document(vec![object_type(WIDGET, &[], vec![both])])).map(|_| ()),
        Err(missing)
    );
    // The multiplicity alone is the last row.
    assert_eq!(
        read(&document(vec![object_type(WIDGET, &[], vec![backwards.clone()])])).map(|_| ()),
        Err(unpreserved)
    );
    // A malformed member reports before a dangling reference.
    let mut no_presence = field(WIDGET, "c", "ix://quire/native/Integer");
    no_presence["presence"] = json!("sometimes");
    assert_eq!(
        read(&document(vec![object_type(
            WIDGET,
            &[],
            vec![dangling, no_presence]
        )]))
        .map(|_| ()),
        Err(malformed)
    );
}

/// FR-154 row 3: a `typeRef` naming a relationship names a node of the wrong
/// meaning, whether the object type declaring the relationship is read before
/// or after the one naming it.
///
/// Tracing: TC-048, FR-038-AC-28
#[test]
fn tc_048_a_type_ref_to_a_relationship_is_wrong_meaning_in_any_node_order() {
    let owns = "ix://acme/orders/Widget/owns";
    let mut widget = object_type(WIDGET, &[], vec![]);
    widget["relationships"] = json!([{"identity": owns}]);
    let malformed = ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration);
    for referencing in [GADGET, ZETA] {
        let holder = object_type(referencing, &[], vec![field(referencing, "owned", owns)]);
        assert_eq!(
            read(&document(vec![widget.clone(), holder])).map(|_| ()),
            Err(malformed),
            "{referencing} is read {} the relationship's owner",
            if referencing < WIDGET { "before" } else { "after" }
        );
    }
}

/// FR-154: two nodes that carry no identity share none; each is malformed,
/// not a `conflicting-binding`.
///
/// Tracing: TC-048, FR-038-AC-28
#[test]
fn tc_048_nodes_with_no_identity_are_malformed_not_conflicting() {
    let mut nameless = object_type(WIDGET, &[], vec![]);
    nameless
        .as_object_mut()
        .expect("an object")
        .remove("identity");
    assert_eq!(
        read(&document(vec![nameless.clone(), nameless])).map(|_| ()),
        Err(ModelRefusal::new(
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration
        ))
    );
}

/// FR-322 step 1: reading a document and resolving a member are charged to
/// the reader's `work` limit at the document's `model_selections` row; a
/// limit one below what a read used is `incomplete` there.
///
/// Tracing: TC-048, FR-038-AC-30
#[test]
fn tc_048_reading_and_resolving_are_charged_to_the_work_limit() {
    let types = || {
        vec![
            object_type(
                WIDGET,
                &[],
                vec![field(WIDGET, "code", "ix://quire/native/Integer")],
            ),
            object_type(GADGET, &[WIDGET], vec![]),
        ]
    };
    let incomplete_at = |failure: ValidationFailure| match failure {
        ValidationFailure::Incomplete(record) => record.path.map(|path| path.to_string()),
        ValidationFailure::Refused(refusal) => panic!("a limit, not {refusal:?}"),
    };
    // The read: exact work admits, one less is incomplete at the row.
    let document = document(types());
    let mut unlimited = WorkMeter::new(u64::MAX);
    read_semantic_ir(&document, &mut Budget::new(&mut unlimited, 2)).expect("reads");
    let used = unlimited.consumed();
    assert!(used > 0, "a read is charged");
    let mut exact = WorkMeter::new(used);
    read_semantic_ir(&document, &mut Budget::new(&mut exact, 2)).expect("exact work admits");
    let mut tight = WorkMeter::new(used - 1);
    let Err(ModelFailure::Limit(failure)) =
        read_semantic_ir(&document, &mut Budget::new(&mut tight, 2))
    else {
        panic!("one unit less than the read used is a limit");
    };
    assert_eq!(
        incomplete_at(failure).as_deref(),
        Some("/lock/model_selections/2")
    );
    // Resolution: the same.
    let model = read(&document).expect("reads");
    let mut meter = WorkMeter::new(u64::MAX);
    model
        .resolve(GADGET, MemberKind::Field, "code", &mut Budget::new(&mut meter, 1))
        .expect("resolves");
    let used = meter.consumed();
    assert!(used > 0, "a resolution is charged");
    let mut tight = WorkMeter::new(used - 1);
    let Err(ModelFailure::Limit(failure)) =
        model.resolve(GADGET, MemberKind::Field, "code", &mut Budget::new(&mut tight, 1))
    else {
        panic!("one unit less than the resolution used is a limit");
    };
    assert_eq!(
        incomplete_at(failure).as_deref(),
        Some("/lock/model_selections/1")
    );
}
