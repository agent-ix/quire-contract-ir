// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-052-AC-16: IR's external consumer of the native Quire owner portion.
//! This does not construct a neutral owner adapter or a four-owner census.

use std::{fs, path::Path};

use ix_trace_rs::trace;
use quire_contract_model::source_obligation::{
    SourceAuthoredCriterionId, SourceContractAuthorId, SourceCriterionSourceBinding,
};
use quire_contract_model::{SourceDocumentId, SourceIdentity, SourceRevision};
use quire_rs::obligation::inventory::{
    inventory, CriterionEntry, CriterionOrigin, InventoryCauseCode, InventoryCauseDetail,
    InventoryInterruption, InventoryLookupError, InventoryOptions, InventoryTerminal,
    SelectedInventory,
};
use quire_rs::traceability::TraceabilityModel;
use serde_json::json;

use crate::support::corpus::Scratch;

const HEADER: &str = "---\nid: FR-701\ntype: FR\ntitle: Independent criterion source\n---\n\n";
const TABLE: &str =
    "## Acceptance Criteria\n\n| Case | Claim | Check |\n|------|-------|-------|\n";
const FIRST: &str =
    "| FR-701-AC-1 | The service shall retain each accepted item. | Test (TC-701) |\n";
const UNMAPPED: &str =
    "| FR-701-AC-2 | The service shall report an empty selection. | Analysis |\n";

fn model() -> TraceabilityModel {
    serde_json::from_value(json!({
        "trace_targets": [{"name":"authored-cases","archetype":"FR",
            "section":"Acceptance Criteria","id_column":"Case"}],
        "obligations": [{"name":"review-criteria","target":"authored-cases",
            "statement_column":"Claim","method_column":"Check"}]
    }))
    .expect("owner-declared traceability model")
}

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn source_tree(label: &str) -> Scratch {
    let scratch = Scratch::empty(label);
    write(
        scratch.root(),
        "spec/FR-701.md",
        &(HEADER.to_owned() + TABLE + FIRST + UNMAPPED),
    );
    scratch
}

fn read(root: &Path) -> SelectedInventory {
    inventory(
        root,
        &model(),
        &["review-criteria"],
        InventoryOptions::default(),
    )
}

/// Borrow the immutable owner's entries; a caller count cannot replace full output.
fn agrees_with_full_owner(
    owner: &SelectedInventory,
    observed: &[CriterionEntry<'_>],
    observed_count: usize,
) -> bool {
    owner.terminal() == InventoryTerminal::Complete
        && owner.len() == observed_count
        && observed_count == observed.len()
        && owner.entries().zip(observed).all(|(expected, observed)| {
            expected.identity() == observed.identity()
                && expected.obligation() == observed.obligation()
                && expected.row() == observed.row()
                && expected.line() == observed.line()
        })
}

/// Only the owner's authored-origin branch has an authored criterion endpoint.
fn authored_identity(
    entry: CriterionEntry<'_>,
) -> Result<SourceAuthoredCriterionId, CriterionOrigin<'_>> {
    let identity = entry.identity();
    match identity.origin() {
        CriterionOrigin::AuthoredRow { authored_id } => Ok(SourceAuthoredCriterionId::new(
            identity.source_name(),
            identity.document(),
            authored_id,
        )),
        origin @ (CriterionOrigin::RenderedRow { .. }
        | CriterionOrigin::CombinatorialTable { .. }) => Err(origin),
    }
}

#[trace("TC-452", "FR-052-AC-16")]
#[test]
fn quire_complete_owner_output_retains_unmapped_criteria_and_authored_provenance() {
    let scratch = source_tree("ir812-complete");
    let root = scratch.root().join("spec");
    let owner = read(&root);
    assert_eq!(owner.terminal(), InventoryTerminal::Complete);
    assert_eq!(owner.len(), 2);
    assert!(owner.causes().is_empty());
    let entries: Vec<_> = owner.entries().collect();
    assert!(agrees_with_full_owner(&owner, &entries, owner.len()));
    let unmapped = entries[1];
    assert_eq!(
        unmapped.obligation().statement,
        "The service shall report an empty selection."
    );
    assert_eq!(unmapped.obligation().method.as_deref(), Some("Analysis"));
    assert!(unmapped.obligation().target_ids.is_empty());
    assert_eq!(entries[0].obligation().target_ids, ["TC-701"]);
    let native = unmapped.identity();
    assert!(std::ptr::eq(native.selection(), owner.selection().unwrap()));
    assert_eq!(native.selection().root(), root.canonicalize().unwrap());
    assert_eq!(
        native.selection().sources()[0].declaration().name,
        "review-criteria"
    );
    assert_eq!(native.document(), "FR-701.md");
    assert_eq!(unmapped.row(), Some(2));
    assert!(unmapped.line().is_some());
    let criterion = authored_identity(unmapped).unwrap();
    assert_eq!(criterion.obligation_declaration(), "review-criteria");
    assert_eq!(criterion.scoped_document(), "FR-701.md");
    assert_eq!(criterion.criterion_id(), "FR-701-AC-2");
    assert!(std::ptr::eq(
        owner.lookup(&native).unwrap().obligation(),
        unmapped.obligation()
    ));
    let source = SourceIdentity::new(
        SourceDocumentId::new("author-selected-service").unwrap(),
        SourceRevision::new(1).unwrap(),
    );
    let binding = SourceCriterionSourceBinding::new(
        "service-contract",
        "service-criterion-model",
        "review-criteria",
        source.clone(),
        SourceContractAuthorId::new("service-author"),
    );
    assert_eq!(binding.source(), &source);
    assert_eq!(binding.obligation_scope(), native.source_name());
    assert_eq!(
        binding.author(),
        &SourceContractAuthorId::new("service-author")
    );
    // The explicit provenance labels do not manufacture a native selection identity.
    let other = source_tree("ir812-foreign");
    let foreign = read(&other.root().join("spec"));
    assert_eq!(
        foreign.lookup(&native).unwrap_err(),
        InventoryLookupError::SelectionMismatch
    );
    let mut shortened = entries;
    shortened.remove(1);
    assert!(
        !agrees_with_full_owner(&owner, &shortened, shortened.len()),
        "a matching shortened count must not hide an unmapped owner row"
    );
    assert!(!agrees_with_full_owner(&owner, &shortened, owner.len()));
}

#[trace("TC-452", "FR-052-AC-16")]
#[test]
fn quire_statement_changes_and_each_row_deletion_change_owner_output_before_proof() {
    let scratch = source_tree("ir812-source-edits");
    let root = scratch.root().join("spec");
    let original = read(&root);
    let key = original.entries().nth(1).unwrap().identity();
    write(
        &root,
        "FR-701.md",
        &(HEADER.to_owned()
            + TABLE
            + FIRST
            + &UNMAPPED.replace("an empty selection", "all retained selections")),
    );
    let edited = read(&root);
    assert_eq!(edited.len(), 2);
    assert_eq!(edited.terminal(), InventoryTerminal::Complete);
    assert_eq!(
        edited.entries().nth(1).unwrap().obligation().statement,
        "The service shall report all retained selections."
    );
    assert_eq!(
        edited.lookup(&key).unwrap_err(),
        InventoryLookupError::SelectionMismatch
    );
    for (retained, id) in [(FIRST, "FR-701-AC-1"), (UNMAPPED, "FR-701-AC-2")] {
        write(&root, "FR-701.md", &(HEADER.to_owned() + TABLE + retained));
        let deleted = read(&root);
        assert_eq!(deleted.terminal(), InventoryTerminal::Complete);
        assert_eq!(deleted.len(), 1);
        assert_eq!(
            authored_identity(deleted.entries().next().unwrap())
                .unwrap()
                .criterion_id(),
            id
        );
        assert_eq!(
            deleted.lookup(&key).unwrap_err(),
            InventoryLookupError::SelectionMismatch
        );
    }
    write(&root, "FR-701.md", &(HEADER.to_owned() + TABLE));
    let empty = read(&root);
    assert_eq!(empty.terminal(), InventoryTerminal::Complete);
    assert_eq!(empty.len(), 0);
    assert!(agrees_with_full_owner(&empty, &[], 0));
    fs::remove_file(root.join("FR-701.md")).unwrap();
    assert_eq!(read(&root).terminal(), InventoryTerminal::Complete);
    assert!(read(&root).is_empty());
    let missing = read(&root.join("absent"));
    assert_eq!(missing.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(missing.causes()[0].code(), InventoryCauseCode::RootMissing);
    assert!(!agrees_with_full_owner(&missing, &[], 0));
}

#[trace("TC-452", "FR-052-AC-16")]
#[test]
fn quire_partial_table_and_load_failures_remain_native_typed_causes() {
    let scratch = source_tree("ir812-failures");
    let root = scratch.root().join("spec");
    for (bad, expected) in [
        (
            HEADER.to_owned() + "## Context\n\nNo selected section.\n",
            InventoryCauseCode::MissingSection,
        ),
        (
            HEADER.to_owned() + "## Acceptance Criteria\n\nNo table.\n",
            InventoryCauseCode::MissingTable,
        ),
        (
            HEADER.to_owned() + &TABLE.replace("| Case |", "| Wrong |") + FIRST,
            InventoryCauseCode::MissingColumn,
        ),
        (
            HEADER.to_owned() + TABLE + "| FR-702-AC-1 | | Analysis |\n",
            InventoryCauseCode::UnusableRow,
        ),
    ] {
        write(&root, "FR-702.md", &bad);
        let owner = read(&root);
        assert_eq!(owner.terminal(), InventoryTerminal::Incomplete);
        assert_eq!(owner.len(), 2);
        let cause = owner.causes().first().unwrap();
        assert_eq!(cause.code(), expected);
        assert_eq!(cause.document(), Some(root.join("FR-702.md").as_path()));
        assert_eq!(cause.source_name(), Some("review-criteria"));
        assert!(matches!(cause.detail(), InventoryCauseDetail::Scan { .. }));
        assert!(!agrees_with_full_owner(
            &owner,
            &owner.entries().collect::<Vec<_>>(),
            owner.len()
        ));
        fs::remove_file(root.join("FR-702.md")).unwrap();
        assert_eq!(read(&root).terminal(), InventoryTerminal::Complete);
    }
    fs::write(root.join("unreadable.md"), [0xff]).unwrap();
    let owner = read(&root);
    assert_eq!(owner.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(owner.len(), 2);
    let cause = &owner.causes()[0];
    assert_eq!(cause.code(), InventoryCauseCode::DocumentUnreadable);
    assert!(
        matches!(cause.detail(), InventoryCauseDetail::Diagnostic(quire_rs::Diagnostic::DocumentUnreadable { path, reason }) if path == &root.join("unreadable.md") && reason == &fs::read_to_string(path).unwrap_err().to_string())
    );
    fs::remove_file(root.join("unreadable.md")).unwrap();
    assert_eq!(read(&root).terminal(), InventoryTerminal::Complete);
}

#[trace("TC-452", "FR-052-AC-16")]
#[test]
fn quire_stopping_and_interruption_preserve_the_owners_exact_terminal_and_cause() {
    use std::error::Error;
    let scratch = source_tree("ir812-stop");
    let root = scratch.root().join("spec");
    for (limit, count, terminal) in [
        (0, 0, InventoryTerminal::Incomplete),
        (1, 1, InventoryTerminal::Incomplete),
        (2, 2, InventoryTerminal::Incomplete),
        (3, 2, InventoryTerminal::Complete),
    ] {
        let owner = inventory(
            &root,
            &model(),
            &["review-criteria"],
            InventoryOptions {
                stop_limit: Some(limit),
                ..Default::default()
            },
        );
        assert_eq!(owner.len(), count);
        assert_eq!(owner.terminal(), terminal);
        if terminal == InventoryTerminal::Incomplete {
            assert_eq!(owner.causes()[0].code(), InventoryCauseCode::LimitReached);
            assert!(
                matches!(owner.causes()[0].detail(), InventoryCauseDetail::Limit {limit: actual} if *actual == limit)
            );
        } else {
            assert!(owner.causes().is_empty());
        }
    }
    let stop = |count| {
        (count == 1).then(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "consumer requested stop",
            )) as InventoryInterruption
        })
    };
    let interrupted = inventory(
        &root,
        &model(),
        &["review-criteria"],
        InventoryOptions {
            interrupt: Some(&stop),
            ..Default::default()
        },
    );
    assert_eq!(interrupted.len(), 1);
    assert_eq!(interrupted.terminal(), InventoryTerminal::Incomplete);
    let cause = &interrupted.causes()[0];
    assert_eq!(cause.code(), InventoryCauseCode::Interrupted);
    let original = cause
        .source()
        .unwrap()
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .unwrap();
    assert_eq!(original.kind(), std::io::ErrorKind::Interrupted);
    assert_eq!(original.to_string(), "consumer requested stop");
}

#[cfg(unix)]
#[trace("TC-452", "FR-052-AC-16")]
#[test]
fn quire_real_walk_failure_preserves_partial_rows_and_original_owner_error() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = source_tree("ir812-walk");
    let root = scratch.root().join("spec");
    let denied = root.join("denied");
    fs::create_dir(&denied).unwrap();
    let permissions = fs::metadata(&denied).unwrap().permissions();
    struct Restore<'a>(&'a Path, fs::Permissions);
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            fs::set_permissions(self.0, self.1.clone()).unwrap();
        }
    }
    let restore = Restore(&denied, permissions);
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o0)).unwrap();
    assert_eq!(
        fs::read_dir(&denied).unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    let owner = read(&root);
    assert_eq!(owner.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(owner.len(), 2);
    let cause = owner
        .causes()
        .iter()
        .find(|cause| cause.code() == InventoryCauseCode::WalkFailed)
        .unwrap();
    assert_eq!(cause.document(), Some(denied.as_path()));
    assert!(
        matches!(cause.detail(), InventoryCauseDetail::Walk(error) if error.io_error().is_some_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied))
    );
    drop(restore);
    assert_eq!(read(&root).terminal(), InventoryTerminal::Complete);
    write(&root, "FR-703.md", "---\nid: FR-703\ntype: FR\n---\n\n## Acceptance Criteria\n\n| Case | Claim | Check |\n|------|-------|-------|\n| FR-703-AC-1 | The service shall retain a healthy partial criterion. | Analysis |\n");
    let document = root.join("FR-701.md");
    let restore = Restore(&document, fs::metadata(&document).unwrap().permissions());
    fs::set_permissions(&document, fs::Permissions::from_mode(0o0)).unwrap();
    assert_eq!(
        fs::read(&document).unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    let unreadable = read(&root);
    assert_eq!(unreadable.terminal(), InventoryTerminal::Incomplete);
    assert_eq!(unreadable.len(), 1);
    assert_eq!(
        unreadable.entries().next().unwrap().obligation().id,
        "FR-703-AC-1"
    );
    assert_eq!(
        unreadable.causes()[0].code(),
        InventoryCauseCode::DocumentUnreadable
    );
    assert_eq!(unreadable.causes()[0].document(), Some(document.as_path()));
    drop(restore);
    assert_eq!(read(&root).terminal(), InventoryTerminal::Complete);
    assert_eq!(read(&root).len(), 3);
}

#[trace("TC-452", "FR-052-AC-16")]
#[test]
fn quire_rendered_and_table_origins_are_never_coerced_into_authored_criteria() {
    let scratch = source_tree("ir812-origin");
    let root = scratch.root().join("spec");
    let mut rendered = model();
    rendered.obligations[0].target = None;
    rendered.obligations[0].archetype = Some("FR".into());
    rendered.obligations[0].section = Some("Acceptance Criteria".into());
    rendered.obligations[0].id_format = Some("{document}-R-{row}".into());
    let owner = inventory(&root, &rendered, &["review-criteria"], Default::default());
    assert_eq!(owner.terminal(), InventoryTerminal::Complete);
    for entry in owner.entries() {
        assert!(matches!(
            authored_identity(entry),
            Err(CriterionOrigin::RenderedRow { .. })
        ));
    }
    rendered.obligations[0].combinatorial = Some(
        serde_json::from_value(
            json!({"dimension_column":"Claim", "values_column":"Check", "strength":2}),
        )
        .unwrap(),
    );
    write(
        &root,
        "FR-701.md",
        &(HEADER.to_owned() + TABLE + "| a | mode | low, high |\n| b | scope | local, shared |\n"),
    );
    let table = inventory(&root, &rendered, &["review-criteria"], Default::default());
    assert_eq!(table.terminal(), InventoryTerminal::Complete);
    assert_eq!(table.len(), 1);
    let entry = table.entries().next().unwrap();
    assert!(matches!(
        authored_identity(entry),
        Err(CriterionOrigin::CombinatorialTable { .. })
    ));
}

#[trace("TC-452", "FR-052-AC-16")]
#[test]
fn quire_ambiguous_owner_id_is_not_deduplicated_into_a_complete_population() {
    let scratch = source_tree("ir812-duplicate");
    let root = scratch.root().join("spec");
    write(
        &root,
        "FR-701.md",
        &(HEADER.to_owned() + TABLE + FIRST + FIRST + UNMAPPED),
    );
    let owner = read(&root);
    assert_eq!(owner.len(), 3);
    assert_eq!(owner.terminal(), InventoryTerminal::Incomplete);
    let entries: Vec<_> = owner.entries().collect();
    assert_eq!(
        owner.lookup(&entries[0].identity()).unwrap_err(),
        InventoryLookupError::AmbiguousIdentity
    );
    assert_eq!(
        owner.lookup(&entries[1].identity()).unwrap_err(),
        InventoryLookupError::AmbiguousIdentity
    );
    assert_eq!(
        owner
            .lookup(&entries[2].identity())
            .unwrap()
            .obligation()
            .id,
        "FR-701-AC-2"
    );
    assert_eq!(
        owner.causes()[0].code(),
        InventoryCauseCode::DuplicateIdentity
    );
    assert!(
        matches!(owner.causes()[0].detail(), InventoryCauseDetail::Duplicate { authored_id, rows } if authored_id == "FR-701-AC-1" && rows == &[1, 2])
    );
    assert!(!agrees_with_full_owner(&owner, &entries, 3));
}
