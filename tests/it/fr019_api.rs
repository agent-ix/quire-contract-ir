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

fn fields_are_private(item: &syn::ItemStruct) -> bool {
    matches!(&item.fields, Fields::Named(fields)
        if fields.named.iter().all(|field| matches!(field.vis, Visibility::Inherited)))
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

fn semantic_expression_new(signature: &syn::Signature) -> bool {
    let names: Vec<_> = signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            FnArg::Typed(typed) => match typed.ty.as_ref() {
                Type::Path(path) => path.path.segments.last().map(|part| part.ident.to_string()),
                _ => None,
            },
            FnArg::Receiver(_) => None,
        })
        .collect();
    names == ["ExpressionKind", "SourceSpan"] && signature.inputs.len() == 2
}

fn unchecked_public_constructors(file: &syn::File, owner: &str) -> Vec<String> {
    let mut unchecked = Vec::new();
    for item in &file.items {
        let Item::Impl(implementation) = item else {
            continue;
        };
        if !self_name(implementation).is_some_and(|name| name == owner) {
            continue;
        }
        for member in &implementation.items {
            let ImplItem::Fn(function) = member else {
                continue;
            };
            if !matches!(function.vis, Visibility::Public(_))
                || function
                    .sig
                    .inputs
                    .iter()
                    .any(|arg| matches!(arg, FnArg::Receiver(_)))
            {
                continue;
            }
            let returns_validated = matches!(&function.sig.output,
                ReturnType::Type(_, output) if matches!(output.as_ref(), Type::Path(path)
                    if path.path.segments.last().is_some_and(|part|
                        part.ident == "Self" || part.ident == owner)));
            let semantic_constructor = owner == "Expression"
                && function.sig.ident == "new"
                && semantic_expression_new(&function.sig);
            if returns_validated && !semantic_constructor {
                unchecked.push(format!("{owner}::{}", function.sig.ident));
            }
        }
    }
    unchecked
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
        assert!(
            fields_are_private(item),
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
    for (file, owner) in [
        (&identity, "ContractPackage"),
        (&expression, "Expression"),
        (&expression, "DeclarationEnvironment"),
    ] {
        assert!(
            unchecked_public_constructors(file, owner).is_empty(),
            "{owner} has an unchecked public constructor: {:?}",
            unchecked_public_constructors(file, owner)
        );
    }
    let injected = IDENTITY.replacen(
        "impl<B: DependencySource> ContractPackage<B> {",
        "impl<B: DependencySource> ContractPackage<B> { pub fn unchecked(_: &str) -> Self { panic!(\"mutant\") }",
        1,
    );
    assert_ne!(injected, IDENTITY, "constructor mutation anchor must exist");
    assert!(
        unchecked_public_constructors(&parse(&injected), "ContractPackage")
            .iter()
            .any(|name| name == "ContractPackage::unchecked"),
        "an unchecked constructor mutant must be detected"
    );
    let mut exposed = named_struct(&identity, "ContractPackage").clone();
    let Fields::Named(fields) = &mut exposed.fields else {
        unreachable!("the actual ContractPackage has named fields")
    };
    fields.named.first_mut().expect("package has fields").vis = syn::parse_quote!(pub);
    assert!(
        !fields_are_private(&exposed),
        "a public-field mutant must be detected"
    );
}

#[trace("TC-018", "FR-019-AC-10")]
#[test]
fn tc_018_public_semantic_boundary_signatures_are_fallible() {
    let identity = parse(IDENTITY);
    let expression = parse(EXPRESSION);
    let canonical = parse(CANONICAL);
    let coverage = parse(COVERAGE);
    let binding = parse(BINDING);
    for (file, owner, name) in [
        (&identity, "ContractPackage", "from_json_str"),
        (&identity, "ContractPackage", "from_json_bytes"),
        (&identity, "ContractPackage", "new"),
        (&expression, "DeclarationEnvironment", "new"),
        (&expression, "DeclarationEnvironment", "check_expression"),
        (&binding, "BoundPackage", "from_json_bytes"),
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
    let mut nonfallible = method(&identity, "ContractPackage", "from_json_str")
        .sig
        .clone();
    nonfallible.output = syn::parse_quote!(-> Option<Self>);
    assert!(
        !returns_result(&nonfallible),
        "a nonfallible signature mutant must be detected"
    );
    let mut bound_nonfallible = method(&binding, "BoundPackage", "from_json_bytes")
        .sig
        .clone();
    bound_nonfallible.output = syn::parse_quote!(-> Option<Self>);
    assert!(
        !returns_result(&bound_nonfallible),
        "a nonfallible BoundPackage decoder mutant must be detected"
    );
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
    let mut omitted = method(&canonical, "ContractPackage", "canonical_package")
        .sig
        .clone();
    omitted.inputs = omitted
        .inputs
        .into_iter()
        .filter(|argument| {
            !matches!(argument, FnArg::Typed(typed)
                if matches!(typed.ty.as_ref(), Type::Path(path)
                    if path.path.segments.last().is_some_and(|segment| segment.ident == "CanonicalProfile")))
        })
        .collect();
    assert!(
        !has_profile_parameter(&omitted),
        "an omitted-profile signature mutant must be detected"
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

fn forbidden_public_surface(source: &str) -> Vec<String> {
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
            Item::Struct(item) if matches!(item.vis, Visibility::Public(_)) => {
                for field in &item.fields {
                    if matches!(field.vis, Visibility::Public(_)) {
                        found.visit_type(&field.ty);
                    }
                }
            }
            Item::Enum(item) if matches!(item.vis, Visibility::Public(_)) => {
                for variant in &item.variants {
                    for field in &variant.fields {
                        found.visit_type(&field.ty);
                    }
                }
            }
            Item::Type(item) if matches!(item.vis, Visibility::Public(_)) => {
                found.visit_type(&item.ty);
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
            forbidden_public_surface(source).is_empty(),
            "{name} exposes forbidden types: {:?}",
            forbidden_public_surface(source)
        );
    }
    let mutated = IDENTITY.replacen(
        "impl<B: DependencySource> ContractPackage<B> {",
        "impl<B: DependencySource> ContractPackage<B> { pub fn host_path(&self, _: &std::path::Path) {}",
        1,
    );
    assert_ne!(mutated, IDENTITY, "mutation anchor must exist");
    assert!(
        forbidden_public_surface(&mutated)
            .iter()
            .any(|path| path == "std::path::Path"),
        "a public Path parameter mutant must be detected"
    );
    let field_mutant = COVERAGE.replacen(
        "pub struct CoverageResult {",
        "pub struct CoverageResult { pub host: std::path::PathBuf,",
        1,
    );
    assert_ne!(
        field_mutant, COVERAGE,
        "result-field mutation anchor must exist"
    );
    assert!(
        forbidden_public_surface(&field_mutant)
            .iter()
            .any(|path| path == "std::path::PathBuf"),
        "a forbidden public result field mutant must be detected"
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
    let schema = jsonschema::JSONSchema::compile(
        &serde_json::from_str(include_str!(
            "../../schemas/contract-package-reference-v1.schema.json"
        ))
        .expect("published package schema is JSON"),
    )
    .expect("published package schema compiles");
    let mut package: Value = serde_json::from_str(include_str!(
        "../../corpus/contract-v0.1/inputs/package-constructs.json"
    ))
    .expect("published package fixture is JSON");
    assert!(
        schema.is_valid(&package),
        "fixture must satisfy the published schema"
    );
    let original = package.to_string();
    assert!(ContractPackage::from_json_str(&original, ValidationOptions::strict()).is_ok());
    package["unexpected"] = json!(1);
    assert!(
        !schema.is_valid(&package),
        "schema must reject the unknown member"
    );
    let diagnostics =
        ContractPackage::from_json_str(&package.to_string(), ValidationOptions::strict())
            .expect_err("unknown package member must refuse");
    assert_eq!(diagnostics[0].code, DiagnosticCode::InvalidWireFormat);
}
