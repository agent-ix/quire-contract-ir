// SPDX-License-Identifier: AGPL-3.0-or-later
//! Atomic package-local validation of every retained identity channel.

use super::*;

impl SelectedExactIdentities {
    fn check_binding(
        &self,
        package: &CheckedPackageV2,
        budget: &mut NativeJoinBudget,
        at: &CheckedNodeId,
    ) -> Result<(), ExactTypeProjectionError> {
        charge_native_work(budget, at, at, "identity_package_binding", 1)?;
        if self
            .binding
            .as_ref()
            .is_some_and(|binding| binding != &package.admission_binding)
        {
            return Err(witness_error(at, ExactTypeProjectionErrorCode::ForeignNode));
        }
        Ok(())
    }

    fn join_locus<'a>(&'a self, package: &'a CheckedPackageV2) -> Option<&'a CheckedNodeId> {
        self.declarations
            .keys()
            .next()
            .or_else(|| self.fields.first().map(|row| &row.record_node))
            .or_else(|| self.admitted_tuples.keys().next())
            .or_else(|| self.admitted_source_units.keys().next())
            .or_else(|| self.source_unions.keys().next())
            .or_else(|| self.source_records.keys().next())
            .or_else(|| self.source_tuples.keys().next())
            .or_else(|| self.source_enums.keys().next())
            .or_else(|| self.source_collections.keys().next())
            .or_else(|| package.node_index.keys().next())
    }

    /// Join model-tuple and source-unit producer rows to their admitted facts.
    ///
    /// The slices and receiver are borrowed. Every retained copy is precharged;
    /// a failed operation returns no extension. Opaque identities remain claims
    /// of the selecting producer, not identities authenticated by this API.
    pub fn with_admitted_type_rows(
        &self,
        package: &CheckedPackageV2,
        tuple_rows: &[SelectedTupleDeclarationRow],
        source_unit_rows: &[SelectedSourceUnitRow],
        budget: &mut NativeJoinBudget,
    ) -> Result<Self, ExactTypeProjectionError> {
        let Some(at) = tuple_rows
            .first()
            .map(|row| &row.structural_node)
            .or_else(|| source_unit_rows.first().map(|row| &row.structural_node))
            .or_else(|| self.join_locus(package))
        else {
            return self.empty_source_join(package, budget);
        };
        let before = budget.begin();
        let result = (|| {
            self.check_binding(package, budget, at)?;
            charge_native_storage(
                budget,
                at,
                at,
                "admitted_identity_handles",
                storage::selected_clone(),
            )?;
            let mut selected = self.clone();
            if selected.binding.is_none() {
                selected.binding = Some(package.admission_binding.clone());
            }
            for row in tuple_rows {
                validate_admitted_tuple(package, row, budget)?;
                let id = &row.structural_node;
                selected.check_handle(&row.node_key, id, None, budget)?;
                if selected.admitted_tuples.contains_key(id) {
                    return Err(witness_error(
                        id,
                        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    ));
                }
                charge_native_storage(
                    budget,
                    id,
                    id,
                    "admitted_tuple_row",
                    tuple_storage(row)
                        .saturating_add(storage::node(id))
                        .saturating_add(2),
                )?;
                if Arc::strong_count(&selected.admitted_tuples) > 1 {
                    charge_native_work(
                        budget,
                        id,
                        id,
                        "admitted_tuple_copy",
                        selected.admitted_tuples.len() as u128,
                    )?;
                    charge_native_storage(
                        budget,
                        id,
                        id,
                        "admitted_tuple_copy",
                        storage::prior_arc_map(&selected.admitted_tuples),
                    )?;
                }
                Arc::make_mut(&mut selected.admitted_tuples)
                    .insert(id.clone(), Arc::new(row.clone()));
            }
            for row in source_unit_rows {
                validate_admitted_source_unit(package, row, budget)?;
                let id = &row.structural_node;
                let UnitId::Declared(key) = &row.unit_id else {
                    return Err(witness_error(
                        id,
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    ));
                };
                selected.check_handle(key, id, None, budget)?;
                if selected.admitted_source_units.contains_key(id) {
                    return Err(witness_error(
                        id,
                        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    ));
                }
                charge_native_storage(
                    budget,
                    id,
                    id,
                    "admitted_source_unit_row",
                    unit_storage(row)
                        .saturating_add(storage::node(id))
                        .saturating_add(2),
                )?;
                if Arc::strong_count(&selected.admitted_source_units) > 1 {
                    charge_native_work(
                        budget,
                        id,
                        id,
                        "admitted_source_unit_copy",
                        selected.admitted_source_units.len() as u128,
                    )?;
                    charge_native_storage(
                        budget,
                        id,
                        id,
                        "admitted_source_unit_copy",
                        storage::prior_arc_map(&selected.admitted_source_units),
                    )?;
                }
                Arc::make_mut(&mut selected.admitted_source_units)
                    .insert(id.clone(), Arc::new(row.clone()));
            }
            Ok(selected)
        })();
        Self::finish_source_join(result, budget, before)
    }

    /// Validate every retained row and the package-derived completeness domain.
    ///
    /// Checks binding, supplied contents and then completeness, in that order.
    /// Success preserves the full witness; a refusal never mutates the input.
    /// The same caller-owned budget covers all native validation and copies.
    pub fn validate_and_rebind(
        &self,
        package: &CheckedPackageV2,
        budget: &mut NativeJoinBudget,
    ) -> Result<Self, ExactTypeProjectionError> {
        let Some(at) = self.join_locus(package) else {
            return self.empty_source_join(package, budget);
        };
        let before = budget.begin();
        let result = (|| {
            self.check_binding(package, budget, at)?;
            self.validate_contents(package, budget, at)?;
            self.validate_completeness(package, budget)?;
            charge_native_storage(
                budget,
                at,
                at,
                "rebound_identity_handles",
                storage::selected_clone(),
            )?;
            let mut rebound = self.clone();
            if rebound.binding.is_none() {
                rebound.binding = Some(package.admission_binding.clone());
            }
            Ok(rebound)
        })();
        Self::finish_source_join(result, budget, before)
    }

    fn check_handle(
        &self,
        key: &NodeKey,
        at: &CheckedNodeId,
        own: Option<&CheckedNodeId>,
        budget: &mut NativeJoinBudget,
    ) -> Result<(), ExactTypeProjectionError> {
        let mut compare = |node: &CheckedNodeId, candidate: &NodeKey| {
            charge_native_work(budget, at, node, "identity_handle_owner", 1)?;
            if own != Some(node) && candidate == key {
                Err(witness_error(
                    at,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                ))
            } else {
                Ok(())
            }
        };
        for row in self.declarations.values() {
            match &row.exact_identity {
                SelectedExactIdentity::Record(candidate)
                | SelectedExactIdentity::Union(candidate)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Declared(candidate)) => {
                    compare(&row.structural_node, candidate)?
                }
                SelectedExactIdentity::Object(_)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Compound(_)) => {}
            }
        }
        for row in self.admitted_tuples.values() {
            compare(&row.structural_node, &row.node_key)?;
        }
        for row in self.admitted_source_units.values() {
            if let UnitId::Declared(candidate) = &row.unit_id {
                compare(&row.structural_node, candidate)?;
            }
        }
        for row in self.source_unions.values() {
            compare(&row.structural_node, &row.type_handle)?;
        }
        for row in self.source_records.values() {
            compare(&row.structural_node, &row.record_key)?;
        }
        for row in self.source_tuples.values() {
            compare(&row.structural_node, &row.type_handle)?;
        }
        for row in self.source_enums.values() {
            compare(&row.declaration.structural_node, &row.declaration.node_key)?;
        }
        Ok(())
    }

    fn validation_scratch(
        &self,
        at: &CheckedNodeId,
        budget: &mut NativeJoinBudget,
    ) -> Result<Self, ExactTypeProjectionError> {
        charge_native_storage(
            budget,
            at,
            at,
            "identity_validation_handles",
            storage::selected_clone(),
        )?;
        Ok(self.clone())
    }

    fn validate_contents(
        &self,
        package: &CheckedPackageV2,
        budget: &mut NativeJoinBudget,
        at: &CheckedNodeId,
    ) -> Result<(), ExactTypeProjectionError> {
        // Existing acquisition paths own the complete native row checks. Only
        // temporary, fully precharged copies are passed to these paths; no
        // reconstructed subset is ever returned as a complete witness.
        for row in self.declarations.values() {
            charge_native_work(
                budget,
                &row.structural_node,
                &row.structural_node,
                "recheck_declaration",
                1,
            )?;
            if let SelectedExactIdentity::Object(id) = &row.exact_identity {
                storage::preflight_effective(id, budget, &row.structural_node)?;
            }
            charge_native_storage(
                budget,
                &row.structural_node,
                &row.structural_node,
                "recheck_declaration_copy",
                storage::stage_declaration(row),
            )?;
        }
        for row in self.fields.iter() {
            charge_native_work(
                budget,
                &row.record_node,
                &row.record_node,
                "recheck_field",
                1,
            )?;
            charge_native_storage(
                budget,
                &row.record_node,
                &row.record_node,
                "recheck_field_copy",
                storage::stage_field(row),
            )?;
        }
        Self::from_stage_rows(
            package,
            self.declarations.values().cloned(),
            self.fields.iter().cloned(),
            budget,
        )?;
        for row in self.declarations.values() {
            match &row.exact_identity {
                SelectedExactIdentity::Record(key)
                | SelectedExactIdentity::Union(key)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key)) => self.check_handle(
                    key,
                    &row.structural_node,
                    Some(&row.structural_node),
                    budget,
                )?,
                SelectedExactIdentity::Object(_)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Compound(_)) => {}
            }
        }
        for row in self.admitted_tuples.values() {
            validate_admitted_tuple(package, row, budget)?;
            self.check_handle(
                &row.node_key,
                &row.structural_node,
                Some(&row.structural_node),
                budget,
            )?;
        }
        for row in self.admitted_source_units.values() {
            validate_admitted_source_unit(package, row, budget)?;
            let UnitId::Declared(key) = &row.unit_id else {
                unreachable!("validated declared unit")
            };
            self.check_handle(
                key,
                &row.structural_node,
                Some(&row.structural_node),
                budget,
            )?;
        }
        if !self.source_unions.is_empty() {
            let mut scratch = self.validation_scratch(at, budget)?;
            charge_native_storage(budget, at, at, "union_validation_map", 1)?;
            scratch.source_unions = Arc::new(BTreeMap::new());
            for row in self.source_unions.values() {
                charge_source_union_text(package, row, budget)?;
                for member in &row.members {
                    for value in &member.payload {
                        storage::preflight_value_type(value, budget, &row.structural_node)?;
                    }
                }
                charge_native_storage(
                    budget,
                    &row.structural_node,
                    &row.structural_node,
                    "union_validation_copy",
                    storage::source_union(row),
                )?;
                self.check_handle(
                    &row.type_handle,
                    &row.structural_node,
                    Some(&row.structural_node),
                    budget,
                )?;
            }
            scratch.with_source_union_rows(
                package,
                self.source_unions.values().map(|row| row.as_ref().clone()),
                budget,
            )?;
        }
        if !self.source_records.is_empty() {
            let mut scratch = self.validation_scratch(at, budget)?;
            charge_native_storage(budget, at, at, "record_validation_map", 1)?;
            scratch.source_records = Arc::new(BTreeMap::new());
            for row in self.source_records.values() {
                for member in &row.members {
                    storage::preflight_value_type(
                        &member.value_type,
                        budget,
                        &row.structural_node,
                    )?;
                }
                charge_native_storage(
                    budget,
                    &row.structural_node,
                    &row.structural_node,
                    "record_validation_copy",
                    storage::source_record(row),
                )?;
                self.check_handle(
                    &row.record_key,
                    &row.structural_node,
                    Some(&row.structural_node),
                    budget,
                )?;
            }
            scratch.with_source_record_rows(
                package,
                self.source_records.values().map(|row| row.as_ref().clone()),
                budget,
            )?;
        }
        if !self.source_tuples.is_empty() {
            let mut scratch = self.validation_scratch(at, budget)?;
            charge_native_storage(budget, at, at, "tuple_validation_map", 1)?;
            scratch.source_tuples = Arc::new(BTreeMap::new());
            for row in self.source_tuples.values() {
                for position in &row.positions {
                    storage::preflight_value_type(
                        &position.value_type,
                        budget,
                        &row.structural_node,
                    )?;
                }
                charge_native_storage(
                    budget,
                    &row.structural_node,
                    &row.structural_node,
                    "tuple_validation_copy",
                    storage::source_tuple(row),
                )?;
                self.check_handle(
                    &row.type_handle,
                    &row.structural_node,
                    Some(&row.structural_node),
                    budget,
                )?;
            }
            scratch.with_source_tuple_rows(
                package,
                self.source_tuples.values().map(|row| row.as_ref().clone()),
                budget,
            )?;
        }
        if !self.source_enums.is_empty() {
            let mut scratch = self.validation_scratch(at, budget)?;
            charge_native_storage(budget, at, at, "enum_validation_map", 1)?;
            scratch.source_enums = Arc::new(BTreeMap::new());
            for row in self.source_enums.values() {
                for variant in row.declaration.shape.variants() {
                    charge_native_work(
                        budget,
                        &row.declaration.structural_node,
                        &row.declaration.structural_node,
                        "enum_validation_shape",
                        (variant.case().len() as u128).saturating_add(2),
                    )?;
                }
                charge_native_storage(
                    budget,
                    &row.declaration.structural_node,
                    &row.declaration.structural_node,
                    "enum_validation_copy",
                    storage::source_enum(row),
                )?;
                self.check_handle(
                    &row.declaration.node_key,
                    &row.declaration.structural_node,
                    Some(&row.declaration.structural_node),
                    budget,
                )?;
            }
            scratch.with_source_enum_rows(
                package,
                self.source_enums.values().map(|row| row.as_ref().clone()),
                budget,
            )?;
        }
        if !self.source_collections.is_empty() {
            let mut scratch = self.validation_scratch(at, budget)?;
            charge_native_storage(budget, at, at, "collection_validation_map", 1)?;
            scratch.source_collections = Arc::new(BTreeMap::new());
            for row in self.source_collections.values() {
                storage::preflight_value_type(&row.value_type, budget, &row.structural_node)?;
                charge_native_storage(
                    budget,
                    &row.structural_node,
                    &row.structural_node,
                    "collection_validation_copy",
                    storage::source_collection(row),
                )?;
            }
            scratch.with_source_collection_rows(
                package,
                self.source_collections
                    .values()
                    .map(|row| row.as_ref().clone()),
                budget,
            )?;
        }
        Ok(())
    }

    fn validate_completeness(
        &self,
        package: &CheckedPackageV2,
        budget: &mut NativeJoinBudget,
    ) -> Result<(), ExactTypeProjectionError> {
        // Required rows are derived below from admitted tables, never from the
        // number of rows retained by this witness.
        for (id, declaration) in package.models.selected_declarations() {
            charge_native_work(budget, id, id, "complete_declaration", 1)?;
            if matches!(
                declaration.form,
                DeclarationForm::RecordValueType | DeclarationForm::ObjectType
            ) {
                require(
                    self.declarations.contains_key(id),
                    id,
                    "selected_declaration",
                )?;
            }
        }
        for (id, declaration) in package.models.selected_declarations() {
            if declaration.form != DeclarationForm::RecordValueType {
                continue;
            }
            charge_native_work(budget, id, id, "complete_record_fields", 1)?;
            let (ambiguous, fields) =
                package.models.selected_record_fields(id).ok_or_else(|| {
                    witness_error(
                        id,
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                    )
                })?;
            if ambiguous.is_some() {
                return Err(witness_error(
                    id,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                ));
            }
            for field in fields {
                charge_native_work(budget, id, id, "complete_effective_field", 1)?;
                let mut found = false;
                for row in self.fields.iter() {
                    charge_native_text_pair(
                        budget,
                        id,
                        "complete_field_identity",
                        &row.field_original_identity,
                        field.identity,
                    )?;
                    found |= &row.record_node == id
                        && row.field_original_identity.as_ref() == field.identity;
                }
                require(found, id, "selected_effective_field")?;
            }
        }
        for (id, declaration) in package.models.selected_declarations() {
            if declaration.form == DeclarationForm::TupleValueType {
                charge_native_work(budget, id, id, "complete_admitted_tuple", 1)?;
                require(
                    self.admitted_tuples.contains_key(id),
                    id,
                    "selected_model_tuple",
                )?;
            }
        }
        for (id, _) in &package.node_index {
            if package.node_kind(id) == Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)) {
                charge_native_work(budget, id, id, "complete_declared_unit", 1)?;
                match package.node(id).and_then(|node| node.owner.as_ref()) {
                    Some(CheckedNodeOwner::Source { .. }) => require(
                        self.admitted_source_units.contains_key(id),
                        id,
                        "selected_source_unit",
                    )?,
                    Some(CheckedNodeOwner::Model { .. }) => require(
                        self.declarations.contains_key(id),
                        id,
                        "selected_model_unit",
                    )?,
                    None => {
                        return Err(witness_error(
                            id,
                            ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                        ))
                    }
                }
            }
        }
        for family in 0..5 {
            for id in package.node_index.keys() {
                charge_native_work(budget, id, id, "complete_source_type", 1)?;
                let node = package
                    .node(id)
                    .ok_or_else(|| selected_declaration_error(package, id))?;
                let source = matches!(node.owner, Some(CheckedNodeOwner::Source { .. }));
                let kind = package.node_kind(id);
                let (required, found, form) = match family {
                    0 => (
                        source
                            && kind
                                == Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Union)),
                        self.source_unions.contains_key(id),
                        "selected_source_union",
                    ),
                    1 => (
                        source
                            && kind
                                == Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Record)),
                        self.source_records.contains_key(id),
                        "selected_source_record",
                    ),
                    2 => (
                        source
                            && kind
                                == Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Tuple)),
                        self.source_tuples.contains_key(id),
                        "selected_source_tuple",
                    ),
                    3 => (
                        source && kind == Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Enum)),
                        self.source_enums.contains_key(id),
                        "selected_source_enum",
                    ),
                    _ => (
                        is_collection(package, id),
                        self.source_collections.contains_key(id),
                        "selected_collection",
                    ),
                };
                if required && !(family == 4 && helper_collection_base(package, id, budget)?) {
                    require(found, id, form)?;
                }
            }
        }
        self.complete_selected_documents(package, budget)
    }

    fn complete_selected_documents(
        &self,
        package: &CheckedPackageV2,
        budget: &mut NativeJoinBudget,
    ) -> Result<(), ExactTypeProjectionError> {
        for (selection, model) in package.models.selected_models().iter().enumerate() {
            let root = &package.graph().nodes[model.root].node_id;
            for original in model
                .record_types
                .keys()
                .chain(model.object_types.keys())
                .chain(model.tuple_types.keys())
                .chain(model.units.keys())
            {
                charge_native_work(budget, root, root, "complete_selected_document", 1)?;
                let mut emitted = false;
                for (id, declaration) in package.models.selected_declarations() {
                    charge_native_work(budget, root, id, "complete_selected_original", 1)?;
                    charge_native_text_pair(
                        budget,
                        root,
                        "complete_selected_original",
                        &declaration.node,
                        original,
                    )?;
                    emitted |= declaration.selection == selection && &declaration.node == original;
                }
                if model.units.contains_key(original) {
                    for id in package.node_index.keys() {
                        charge_native_work(budget, root, id, "complete_selected_unit", 1)?;
                        emitted |= matches!(package.node(id).and_then(|node| node.owner.as_ref()), Some(CheckedNodeOwner::Model { identity, node }) if identity == &model.identity && node == original)
                            && package.node_kind(id)
                                == Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit));
                    }
                }
                if !emitted {
                    return Err(witness_error(
                        root,
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                    ));
                }
            }
            if !model.variant_types.is_empty() {
                charge_native_work(budget, root, root, "complete_selected_variant", 1)?;
                return Err(witness_error(
                    root,
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                ));
            }
        }
        Ok(())
    }
}

fn require(
    found: bool,
    at: &CheckedNodeId,
    form: &'static str,
) -> Result<(), ExactTypeProjectionError> {
    if found {
        return Ok(());
    }
    let mut error = witness_error(at, ExactTypeProjectionErrorCode::IncompleteIdentityWitness);
    error.form = form.into();
    Err(error)
}

fn validate_admitted_tuple(
    package: &CheckedPackageV2,
    row: &SelectedTupleDeclarationRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), ExactTypeProjectionError> {
    let id = &row.structural_node;
    charge_native_work(budget, id, id, "admitted_tuple_row", 1)?;
    let Some((document, original, form)) = package.models.selected_declaration(id) else {
        return Err(selected_declaration_error(package, id));
    };
    charge_native_text_pair(
        budget,
        id,
        "admitted_tuple_document",
        document,
        &row.selected_document,
    )?;
    charge_native_text_pair(
        budget,
        id,
        "admitted_tuple_original",
        original,
        &row.original_identity,
    )?;
    if form != DeclarationForm::TupleValueType
        || document != row.selected_document.as_ref()
        || original != row.original_identity.as_ref()
    {
        return Err(witness_error(
            id,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
        ));
    }
    Ok(())
}

fn validate_admitted_source_unit(
    package: &CheckedPackageV2,
    row: &SelectedSourceUnitRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), ExactTypeProjectionError> {
    let id = &row.structural_node;
    charge_native_work(budget, id, id, "admitted_source_unit_row", 1)?;
    let Some(node) = package.node(id) else {
        return Err(selected_declaration_error(package, id));
    };
    let mut selected_source = false;
    for source in &package.lock().sources {
        charge_native_work(budget, id, id, "admitted_source_unit_source", 1)?;
        charge_native_text_pair(
            budget,
            id,
            "admitted_source_unit_source",
            &source.authority,
            &row.source_authority,
        )?;
        charge_native_text_pair(
            budget,
            id,
            "admitted_source_unit_source",
            &source.identity,
            &row.source_identity,
        )?;
        selected_source |=
            source.authority == row.source_authority && source.identity == row.source_identity;
    }
    let mut matching_declaration = false;
    if let Some(declaration) = &node.declaration {
        if declaration.qualified_name.len() == row.qualified_declaration.len() {
            matching_declaration = true;
            for (expected, supplied) in declaration
                .qualified_name
                .iter()
                .zip(&row.qualified_declaration)
            {
                charge_native_text_pair(
                    budget,
                    id,
                    "admitted_source_unit_declaration",
                    expected,
                    supplied,
                )?;
                matching_declaration &= expected == supplied;
            }
        }
    }
    let matching_owner = matches!(&node.owner, Some(CheckedNodeOwner::Source { authority, identity }) if authority == &row.source_authority && identity == &row.source_identity);
    if package.node_kind(id) != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
        || !selected_source
        || !matching_owner
        || !matching_declaration
        || !matches!(row.unit_id, UnitId::Declared(_))
    {
        return Err(witness_error(
            id,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
        ));
    }
    Ok(())
}

fn tuple_storage(row: &SelectedTupleDeclarationRow) -> u128 {
    storage::node(&row.structural_node)
        .saturating_add(row.selected_document.len() as u128)
        .saturating_add(row.original_identity.len() as u128)
        .saturating_add(5)
}

fn unit_storage(row: &SelectedSourceUnitRow) -> u128 {
    row.qualified_declaration.iter().fold(
        storage::node(&row.structural_node)
            .saturating_add(row.source_authority.len() as u128)
            .saturating_add(row.source_identity.len() as u128)
            .saturating_add(6),
        |sum, part| sum.saturating_add(part.len() as u128).saturating_add(1),
    )
}

fn is_collection(package: &CheckedPackageV2, id: &CheckedNodeId) -> bool {
    match package.node_kind(id) {
        Some(CheckedNodeKind::CompositeType(form)) => exact_collection_kind(form).is_some(),
        Some(CheckedNodeKind::BoundedDomain(BoundedDomainForm::CollectionBounds)) => true,
        _ => false,
    }
}

fn helper_collection_base(
    package: &CheckedPackageV2,
    id: &CheckedNodeId,
    budget: &mut NativeJoinBudget,
) -> Result<bool, ExactTypeProjectionError> {
    let mut helper = false;
    for other in package.node_index.keys() {
        charge_native_work(budget, id, other, "collection_base_use", 1)?;
        let node = package
            .node(other)
            .ok_or_else(|| selected_declaration_error(package, other))?;
        if other == id {
            continue;
        }
        let bounded = package.node_kind(other)
            == Some(CheckedNodeKind::BoundedDomain(
                BoundedDomainForm::CollectionBounds,
            ));
        if node.semantic_type.as_ref() == Some(id) {
            if bounded {
                helper = true;
            } else {
                return Ok(false);
            }
        }
        if let Some(term) = package.term(other) {
            if term_uses_type(term, id, budget, other)? {
                return Ok(false);
            }
        }
    }
    Ok(helper)
}

fn term_uses_type(
    term: &CheckedTerm,
    target: &CheckedNodeId,
    budget: &mut NativeJoinBudget,
    at: &CheckedNodeId,
) -> Result<bool, ExactTypeProjectionError> {
    charge_native_storage(budget, at, at, "collection_use_stack", 2)?;
    let mut pending = vec![term];
    while let Some(term) = pending.pop() {
        charge_native_work(budget, at, at, "collection_type_use", 1)?;
        match term {
            CheckedTerm::Reference { target: id } => {
                if id == target {
                    return Ok(true);
                }
            }
            CheckedTerm::Literal { type_id, .. } => {
                if type_id == target {
                    return Ok(true);
                }
            }
            CheckedTerm::Binding { value, .. } => {
                charge_native_storage(budget, at, at, "collection_use_stack", 1)?;
                pending.push(value);
            }
            CheckedTerm::Aggregate { members }
            | CheckedTerm::Application {
                arguments: members, ..
            } => {
                if let CheckedTerm::Application { result_type, .. } = term {
                    if result_type == target {
                        return Ok(true);
                    }
                }
                for member in members.iter().rev() {
                    charge_native_work(budget, at, at, "collection_use_child", 1)?;
                    charge_native_storage(budget, at, at, "collection_use_stack", 1)?;
                    pending.push(member);
                }
            }
            CheckedTerm::OperationAnchorBody(body) => {
                for member in body.members.iter().rev() {
                    charge_native_work(budget, at, at, "collection_use_child", 1)?;
                    charge_native_storage(budget, at, at, "collection_use_stack", 1)?;
                    pending.push(member);
                }
            }
            CheckedTerm::ExtensionForm { clauses, .. } => {
                for clause in clauses.iter().rev() {
                    charge_native_work(budget, at, at, "collection_use_child", 1)?;
                    charge_native_storage(budget, at, at, "collection_use_stack", 1)?;
                    pending.push(clause);
                }
            }
            CheckedTerm::DependencyReference { .. }
            | CheckedTerm::Frame { .. }
            | CheckedTerm::AbstractionRelation { .. }
            | CheckedTerm::Refinement { .. }
            | CheckedTerm::NominalBody(_) => {}
        }
    }
    Ok(false)
}
