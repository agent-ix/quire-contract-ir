use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use ix_trace_rs::trace;
use quire_contract_model::{
    derive_structural_candidates, CheckedSemanticNodeV2, StructuralCandidateError,
};
use serde_json::json;

const MODEL_ROOT: &str = include_str!("../../crates/quire-contract-model/src/lib.rs");
const BRIDGE_ROOT: &str = include_str!("../../src/lib.rs");
const FR_019: &str = include_str!("../../spec/model/functional/FR-019-rust-library-interface.md");

fn quoted_names(text: &str) -> impl Iterator<Item = &str> {
    text.split('`').skip(1).step_by(2)
}

fn expected_items() -> BTreeMap<String, BTreeSet<String>> {
    let (_, after_heading) = FR_019
        .split_once("### Public items")
        .expect("FR-019 public items table must exist");
    let (table, _) = after_heading
        .split_once("## Acceptance Criteria")
        .expect("FR-019 acceptance criteria must follow the table");
    let mut items = BTreeMap::<String, BTreeSet<String>>::new();
    for row in table.lines().filter(|line| line.starts_with("| `")) {
        let cells: Vec<_> = row.split('|').collect();
        let module = quoted_names(cells[1])
            .next()
            .expect("public item row must name a module")
            .split_whitespace()
            .next()
            .expect("module name must be present");
        items
            .entry(module.into())
            .or_default()
            .extend(quoted_names(cells[2]).map(str::to_owned));
    }
    items
}

fn exported_items() -> BTreeMap<String, BTreeSet<String>> {
    let mut items = BTreeMap::<String, BTreeSet<String>>::new();
    let mut exports = MODEL_ROOT.split("pub use ").skip(1);
    for export in exports.by_ref() {
        let statement = export
            .split(';')
            .next()
            .expect("export must end with semicolon");
        assert!(
            !statement.contains('*'),
            "model root must not glob-export: {statement}"
        );
        if statement.starts_with("output_mapping::MappingAllocationPoint") {
            continue;
        }
        let (module, names) = statement
            .split_once("::{")
            .expect("default-feature model exports must name items in braces");
        let names = names.strip_suffix('}').expect("item list must close");
        for name in names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            assert!(
                items.entry(module.into()).or_default().insert(name.into()),
                "duplicate export: {module}::{name}"
            );
        }
    }
    let mut sources =
        vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("crates/quire-contract-model/src")];
    let mut exported_macros = BTreeSet::new();
    while let Some(path) = sources.pop() {
        for entry in fs::read_dir(path).expect("model source directory must be readable") {
            let path = entry
                .expect("source directory entry must be readable")
                .path();
            if path.is_dir() {
                sources.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path).expect("model source file must be readable");
                for macro_export in source.split("#[macro_export]").skip(1) {
                    assert_eq!(
                        path.file_name().and_then(|name| name.to_str()),
                        Some("code.rs"),
                        "FR-019 lists exported macros under code"
                    );
                    let declaration = macro_export
                        .trim_start()
                        .strip_prefix("macro_rules! ")
                        .expect("exported macro must have an adjacent macro_rules declaration");
                    let name = declaration
                        .split(|character: char| {
                            !character.is_ascii_alphanumeric() && character != '_'
                        })
                        .next()
                        .expect("exported macro must have a name");
                    assert!(
                        exported_macros.insert(name.to_owned()),
                        "duplicate exported macro: {name}"
                    );
                }
            }
        }
    }
    items
        .entry("code".into())
        .or_default()
        .extend(exported_macros);
    items
}

/// Trace: FR-019-AC-5
#[trace("TC-058", "FR-019-AC-5")]
#[test]
fn tc_058_model_root_exports_exactly_the_documented_public_items() {
    assert_eq!(exported_items(), expected_items());
    assert_eq!(
        MODEL_ROOT
            .lines()
            .filter(|line| line.starts_with("pub ") && !line.starts_with("pub use "))
            .count(),
        0,
        "model root must expose items only through named exports"
    );
}

#[trace("TC-058", "FR-019-AC-5")]
#[test]
fn tc_058_fault_injection_adds_only_its_named_export() {
    let gate =
        "#[cfg(feature = \"fault-injection\")]\npub use output_mapping::MappingAllocationPoint;";
    assert!(MODEL_ROOT.contains(gate));
    assert_eq!(
        MODEL_ROOT
            .lines()
            .filter(|line| *line == "pub use output_mapping::MappingAllocationPoint;")
            .count(),
        1
    );
}

#[trace("TC-058", "FR-019-AC-5")]
#[test]
fn tc_058_root_crate_exposes_only_its_kani_module() {
    let public: Vec<_> = BRIDGE_ROOT
        .lines()
        .filter(|line| line.starts_with("pub "))
        .collect();
    assert_eq!(public, ["pub mod kani;"]);
}

/// The external consumer can call the borrowed graph API and distinguish
/// every refusal by its variant, without reading error messages.
/// Trace: FR-048-AC-1
#[trace("TC-058", "FR-048-AC-1")]
#[test]
fn tc_058_structural_candidate_api_is_typed_and_closed() {
    let draft: CheckedSemanticNodeV2 = serde_json::from_value(json!({
        "node_id": {"domain": "quire.checked-semantic-node/v1", "digest": "draft"},
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "boolean",
        "semantic_type": {"domain": "quire.checked-semantic-node/v1", "digest": "draft"},
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": {"term": "aggregate", "members": []},
    }))
    .expect("typed draft");
    let candidates =
        derive_structural_candidates(std::slice::from_ref(&draft), 4096, 10).expect("candidate");
    assert_eq!(candidates.nodes().len(), 1);
    assert_eq!(candidates.nodes()[0].position(), 0);
    assert_eq!(
        candidates.nodes()[0].candidate_node_id().domain.as_ref(),
        "quire.checked-semantic-node/v1"
    );
    assert!(candidates.groups().is_empty());
    let repeated = [draft.clone(), draft];
    let error = derive_structural_candidates(&repeated, 4096, 10).expect_err("repeat");
    let kind = match error {
        StructuralCandidateError::RepeatedKey {
            first_position,
            repeated_position,
            ..
        } => {
            assert_eq!((first_position, repeated_position), (0, 1));
            "repeated"
        }
        StructuralCandidateError::InvalidTagForm { .. } => "tag-form",
        StructuralCandidateError::InvalidClosedShape { .. } => "shape",
        StructuralCandidateError::UnsupportedGroupMember { .. } => "group-member",
        StructuralCandidateError::CounterOverflow { .. } => "overflow",
        StructuralCandidateError::WorkExhausted { .. } => "work",
        StructuralCandidateError::CanonicalEncoding { .. } => "canonical",
        StructuralCandidateError::ApplicationLocal { .. } => "application-local",
    };
    assert_eq!(kind, "repeated");
}
