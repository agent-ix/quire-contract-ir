use super::*;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};

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
#[test]
fn member_type_keys_are_qsl_fr_092_structural_keys() {
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
/// subtype and a subtype conforms to its supertype, never the reverse.
#[test]
fn a_semantic_ir_document_reads_inherited_members_and_conformance() {
    let model = read_semantic_ir(&document(vec![
        object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "code", "ix://quire/native/Integer")],
        ),
        object_type(GADGET, &[WIDGET], vec![]),
    ]))
    .expect("the document reads");
    let Resolved::Field(code) = model
        .resolve(GADGET, MemberKind::Field, "code")
        .expect("code resolves on the subtype")
    else {
        panic!("a field");
    };
    assert_eq!(code.identity.as_ref(), "ix://acme/orders/Widget/code");
    assert_eq!(model.field_type(code), Some(MemberType::Integer));
    assert!(model.conforms(GADGET, WIDGET));
    assert!(
        model.conforms(WIDGET, GADGET),
        "conformance is decided in either order"
    );
    assert_eq!(
        model
            .resolve(GADGET, MemberKind::Operation, "code")
            .map(|_| ()),
        Err(ModelRefusal::ineligible())
    );
}

/// An FCD value type `scalar: integer` with `min` and `max` constraints is
/// the element type `Int[lo, hi]`.
#[test]
fn a_semantic_ir_integer_value_type_is_an_integer_range() {
    let digit = json!({
        "identity": "ix://acme/orders/Digit", "displayName": "Digit",
        "kind": {"module": "acme/orders", "name": "digit"}, "roles": [], "extensions": [],
        "unknownPolicy": "reject", "scalar": "integer",
        "constraints": [
            {"keyword": "min", "operands": {"value": 0}},
            {"keyword": "max", "operands": {"value": "9"}},
        ],
    });
    let model = read_semantic_ir(&document(vec![
        digit,
        object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "digit", "ix://acme/orders/Digit")],
        ),
    ]))
    .expect("the document reads");
    let Resolved::Field(digit) = model
        .resolve(WIDGET, MemberKind::Field, "digit")
        .expect("digit")
    else {
        panic!("a field");
    };
    assert_eq!(model.field_type(digit), Some(MemberType::IntRange(DIGIT)));
}

/// FR-154's declaration refusals the reader draws, each for one defect.
#[test]
fn semantic_ir_declaration_defects_refuse_with_their_fr_154_cause() {
    let refused = |types| {
        read_semantic_ir(&document(types))
            .map(|_| ())
            .expect_err("refused")
    };
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
#[test]
fn a_selection_admits_only_the_document_it_names() {
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
    let admitted = admit_selection(&selection("1.0.0", &digest), &evidence).expect("admitted");
    assert!(admitted.object_types.contains_key(WIDGET));
    let refusal = |selection: &CheckedDomainPackageRef, evidence: &CheckedPackageEvidence| {
        admit_selection(selection, evidence)
            .map(|_| ())
            .expect_err("refused")
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
