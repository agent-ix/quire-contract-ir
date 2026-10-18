//! Focused public API evidence for FR-019's split interface criteria.

use ix_trace_rs::trace;
use quire_contract_model::{
    ContractPackage, DiagnosticCode, ValidationOptions, MAX_SEMANTIC_COLLECTION_ITEMS,
    MAX_SEMANTIC_DEPTH, MAX_SEMANTIC_NODES, MAX_WIRE_JSON_DEPTH,
};
use serde_json::{json, Value};
use syn::{visit::Visit, Fields, FnArg, ImplItem, Item, ReturnType, Type, Visibility};

const IDENTITY: &str = include_str!("../../crates/quire-contract-model/src/identity.rs");
const EXPRESSION: &str = include_str!("../../crates/quire-contract-model/src/expression.rs");
const CANONICAL: &str = include_str!("../../crates/quire-contract-model/src/canonical.rs");
const COVERAGE: &str = include_str!("../../crates/quire-contract-model/src/coverage.rs");
const BINDING: &str = include_str!("../../crates/quire-contract-model/src/binding.rs");
const WIRE: &str = include_str!("../../crates/quire-contract-model/src/wire.rs");

fn parse(source: &str) -> syn::File {
    syn::parse_file(source).expect("model source must parse")
}

fn named_struct<'a>(file: &'a syn::File, name: &str) -> &'a syn::ItemStruct {
    file.items
        .iter()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == name => Some(item),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing struct {name}"))
}

fn self_name(item: &syn::ItemImpl) -> Option<&syn::Ident> {
    match item.self_ty.as_ref() {
        Type::Path(path) => path.path.segments.last().map(|segment| &segment.ident),
        _ => None,
    }
}

fn method<'a>(file: &'a syn::File, owner: &str, name: &str) -> &'a syn::ImplItemFn {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(item) if self_name(item).is_some_and(|ident| ident == owner) => Some(item),
            _ => None,
        })
        .flat_map(|item| &item.items)
        .find_map(|item| match item {
            ImplItem::Fn(function) if function.sig.ident == name => Some(function),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing {owner}::{name}"))
}

fn returns_result(signature: &syn::Signature) -> bool {
    matches!(
        &signature.output,
        ReturnType::Type(_, output)
            if matches!(output.as_ref(), Type::Path(path)
                if path.path.segments.last().is_some_and(|segment| segment.ident == "Result"))
    )
}

fn has_profile_parameter(signature: &syn::Signature) -> bool {
    signature.inputs.iter().any(|argument| {
        matches!(argument, FnArg::Typed(typed)
            if matches!(typed.ty.as_ref(), Type::Path(path)
                if path.path.segments.last().is_some_and(|segment| segment.ident == "CanonicalProfile")))
    })
}

#[trace("TC-018", "FR-019-AC-8")]
#[test]
fn tc_018_wire_requests_do_not_construct_validated_values_unchecked() {
    let identity = parse(IDENTITY);
    let expression = parse(EXPRESSION);
    let wire = parse(WIRE);
    for (file, name) in [
        (&identity, "ContractPackage"),
        (&expression, "Expression"),
        (&expression, "DeclarationEnvironment"),
    ] {
        let item = named_struct(file, name);
        assert!(
            matches!(item.vis, Visibility::Public(_)),
            "{name} is public"
        );
        let Fields::Named(fields) = &item.fields else {
            panic!("{name} must have named fields");
        };
        assert!(
            fields
                .named
                .iter()
                .all(|field| matches!(field.vis, Visibility::Inherited)),
            "{name} exposes an unchecked field"
        );
    }
    for name in ["ExpressionInput", "WireExpression"] {
        assert!(
            matches!(named_struct(&wire, name).vis, Visibility::Inherited),
            "{name} must remain a separate private wire type"
        );
    }
    for file in [&identity, &expression, &wire] {
        for item in &file.items {
            let Item::Impl(implementation) = item else {
                continue;
            };
            let is_from = implementation
                .trait_
                .as_ref()
                .and_then(|(_, path, _)| path.segments.last())
                .is_some_and(|segment| segment.ident == "From");
            let target_is_validated = self_name(implementation).is_some_and(|name| {
                ["ContractPackage", "Expression", "DeclarationEnvironment"]
                    .iter()
                    .any(|target| name == target)
            });
            assert!(
                !(is_from && target_is_validated),
                "unchecked From into validated value"
            );
        }
    }
}

#[trace("TC-018", "FR-019-AC-10")]
#[test]
fn tc_018_public_semantic_boundary_signatures_are_fallible() {
    let identity = parse(IDENTITY);
    let expression = parse(EXPRESSION);
    let canonical = parse(CANONICAL);
    let coverage = parse(COVERAGE);
    for (file, owner, name) in [
        (&identity, "ContractPackage", "from_json_str"),
        (&identity, "ContractPackage", "from_json_bytes"),
        (&identity, "ContractPackage", "new"),
        (&expression, "DeclarationEnvironment", "new"),
        (&expression, "DeclarationEnvironment", "check_expression"),
    ] {
        assert!(
            returns_result(&method(file, owner, name).sig),
            "{owner}::{name} must return Result"
        );
    }
    let mut canonical_count = 0;
    for item in &canonical.items {
        if let Item::Impl(implementation) = item {
            for member in &implementation.items {
                if let ImplItem::Fn(function) = member {
                    if matches!(function.vis, Visibility::Public(_))
                        && function.sig.ident.to_string().starts_with("canonical_")
                    {
                        canonical_count += 1;
                        assert!(
                            returns_result(&function.sig),
                            "{} must return Result",
                            function.sig.ident
                        );
                    }
                }
            }
        }
    }
    assert_eq!(
        canonical_count, 10,
        "all ten public canonical entrypoints examined"
    );
    let classify = coverage
        .items
        .iter()
        .find_map(|item| match item {
            Item::Fn(function) if function.sig.ident == "classify_coverage" => Some(function),
            _ => None,
        })
        .expect("public classify_coverage exists");
    assert!(returns_result(&classify.sig));
}

#[trace("TC-018", "FR-019-AC-11")]
#[test]
fn tc_018_every_canonical_entrypoint_requires_the_closed_profile() {
    let canonical = parse(CANONICAL);
    let mut canonical_count = 0;
    for item in &canonical.items {
        if let Item::Impl(implementation) = item {
            for member in &implementation.items {
                if let ImplItem::Fn(function) = member {
                    if matches!(function.vis, Visibility::Public(_))
                        && function.sig.ident.to_string().starts_with("canonical_")
                    {
                        canonical_count += 1;
                        assert!(
                            has_profile_parameter(&function.sig),
                            "{} omits CanonicalProfile",
                            function.sig.ident
                        );
                    }
                }
            }
        }
    }
    assert_eq!(
        canonical_count, 10,
        "all ten public canonical entrypoints examined"
    );
    let profile = canonical
        .items
        .iter()
        .find_map(|item| match item {
            Item::Enum(item) if item.ident == "CanonicalProfile" => Some(item),
            _ => None,
        })
        .expect("closed CanonicalProfile enum exists");
    assert_eq!(profile.variants.len(), 1);
    assert_eq!(profile.variants[0].ident, "V1");
    assert_eq!(
        quire_contract_model::CanonicalProfile::V1.as_str(),
        "quire.contract.canonical-json/v1"
    );
}

#[derive(Default)]
struct ForbiddenTypes(Vec<String>);

impl<'ast> Visit<'ast> for ForbiddenTypes {
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        let names: Vec<_> = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let full = names.join("::");
        if names.iter().any(|name| {
            [
                "Path", "PathBuf", "File", "Mutex", "RwLock", "RefCell", "usize", "isize",
            ]
            .contains(&name.as_str())
                || name.starts_with("Qsl")
                || name.starts_with("ObservationWire")
                || name.starts_with("ProtocolWire")
        }) || full.contains("serde_json::Value")
        {
            self.0.push(full);
        }
        syn::visit::visit_type_path(self, path);
    }
}

fn forbidden_public_signatures(source: &str) -> Vec<String> {
    let file = parse(source);
    let mut found = ForbiddenTypes::default();
    for item in &file.items {
        match item {
            Item::Fn(function) if matches!(function.vis, Visibility::Public(_)) => {
                found.visit_signature(&function.sig);
            }
            Item::Impl(implementation) => {
                for member in &implementation.items {
                    if let ImplItem::Fn(function) = member {
                        if matches!(function.vis, Visibility::Public(_)) {
                            found.visit_signature(&function.sig);
                        }
                    }
                }
            }
            Item::Trait(item) if matches!(item.vis, Visibility::Public(_)) => {
                for member in &item.items {
                    if let syn::TraitItem::Fn(function) = member {
                        found.visit_signature(&function.sig);
                    }
                }
            }
            _ => {}
        }
    }
    found.0
}

#[trace("TC-018", "FR-019-AC-13")]
#[test]
fn tc_018_semantic_signatures_exclude_host_and_owner_types() {
    for (name, source) in [
        ("identity", IDENTITY),
        ("expression", EXPRESSION),
        ("canonical", CANONICAL),
        ("coverage", COVERAGE),
        ("binding", BINDING),
    ] {
        assert!(
            forbidden_public_signatures(source).is_empty(),
            "{name} exposes forbidden types: {:?}",
            forbidden_public_signatures(source)
        );
    }
    assert_eq!(
        forbidden_public_signatures("pub fn injected(path: &std::path::Path) {}"),
        ["std::path::Path"]
    );
}

#[trace("TC-018", "FR-019-AC-12")]
#[test]
fn tc_018_semantic_and_wire_limits_are_exact_public_u32_constants() {
    let limits: [u32; 4] = [
        MAX_SEMANTIC_NODES,
        MAX_SEMANTIC_DEPTH,
        MAX_SEMANTIC_COLLECTION_ITEMS,
        MAX_WIRE_JSON_DEPTH,
    ];
    assert_eq!(limits, [25_000, 256, 10_000, 576]);
}

#[trace("TC-018", "FR-019-AC-9")]
#[test]
fn tc_018_an_unknown_package_member_refuses_before_validation() {
    let mut package: Value = serde_json::from_str(include_str!(
        "../../corpus/contract-v0.1/inputs/package-constructs.json"
    ))
    .expect("published package fixture is JSON");
    let original = package.to_string();
    assert!(ContractPackage::from_json_str(&original, ValidationOptions::strict()).is_ok());
    package["unexpected"] = json!(1);
    let diagnostics =
        ContractPackage::from_json_str(&package.to_string(), ValidationOptions::strict())
            .expect_err("unknown package member must refuse");
    assert_eq!(diagnostics[0].code, DiagnosticCode::InvalidWireFormat);
}
