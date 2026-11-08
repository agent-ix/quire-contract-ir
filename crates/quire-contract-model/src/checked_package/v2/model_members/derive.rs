// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-322 step 4's field type, derived with every allocation requested
//! before it is made: the walk that decides the type allocates nothing, the
//! bytes of the result are computed from it, and the result is then built
//! with no temporary.

use super::charged::{declaration_id_bytes, node_id_bytes};
use super::{
    declaration_id, Budget, CollectionKind, DeclarationForm, DomainModel, FieldDecl, MemberType,
    Multiplicity, NativeTypeRef, RationalBounds, SelectedTypeDefinition, SelectedTypeRef,
    TextBounds, TypedSlot, ValueBounds, DURATION_RESOLVED_PREFIX,
};
use crate::checked_package::shared::CheckedNodeId;
use crate::checked_package::v2::ValidationFailure;

/// What a type reference bottoms out at, borrowed from the model.
enum Leaf<'a> {
    Plain(MemberType),
    Quantity(&'a CheckedNodeId),
    Carried(RationalBounds, &'a CheckedNodeId),
    Text(&'a TextBounds),
    Declared(DeclarationForm, &'a str),
}

/// One wrapper around the leaf, outermost first.
#[derive(Clone, Copy)]
enum Wrap {
    Option,
    Collection(CollectionKind, Option<(u64, u64)>),
    Sequence,
}

impl Leaf<'_> {
    /// The bytes building this leaf requests.
    fn bytes(&self, identity: &str) -> usize {
        match self {
            Self::Plain(_) => 0,
            Self::Quantity(node) | Self::Carried(_, node) => node_id_bytes(node),
            Self::Text(bounds) => bounds.profile.len(),
            Self::Declared(form, node) => declaration_id_bytes(identity, *form, node),
        }
    }
}

impl MemberType {
    /// The member inside a wrapper.
    fn inner_mut(&mut self) -> Option<&mut MemberType> {
        match self {
            Self::Option(inner) => Some(&mut **inner),
            Self::Collection { element, .. } => Some(&mut **element),
            _ => None,
        }
    }
}

impl DomainModel {
    /// What `type_ref` bottoms out at, the outer layers it passes through
    /// counted; `None` where the tables give it no type. The walk allocates
    /// nothing: a chain with no repeat is at most as long as the table.
    fn bottom<'a>(&'a self, type_ref: &'a SelectedTypeRef) -> Option<(&'a SelectedTypeRef, usize)> {
        let mut current = type_ref;
        let mut layers = 0_usize;
        let mut followed = 0_usize;
        while let SelectedTypeRef::Declared(name) = current {
            let Some(definition) = self.collection_types.get(name) else {
                break;
            };
            followed += 1;
            if followed > self.collection_types.len() {
                return None;
            }
            match definition {
                SelectedTypeDefinition::Sequence(items) | SelectedTypeDefinition::Option(items) => {
                    layers += 1;
                    current = items;
                }
                SelectedTypeDefinition::Reference(target) => current = target,
                SelectedTypeDefinition::Map(_) => return None,
            }
        }
        Some((current, layers))
    }

    /// The leaf of a type reference that bottoms out, borrowed. A Duration
    /// leaf assembles its unit key in a buffer requested first.
    fn leaf<'a>(
        &'a self,
        type_ref: &'a SelectedTypeRef,
        budget: &mut Budget<'_, '_>,
        form: &'static str,
    ) -> Result<Option<Leaf<'a>>, ValidationFailure> {
        let name = match type_ref {
            SelectedTypeRef::Map(..) => return Ok(None),
            SelectedTypeRef::Unit(name) => {
                return Ok(self.unit_refs.get(name).map(Leaf::Quantity));
            }
            SelectedTypeRef::Native(native) => {
                let plain = match native {
                    NativeTypeRef::Boolean => MemberType::Boolean,
                    NativeTypeRef::Integer => MemberType::Integer,
                    NativeTypeRef::Float32 => MemberType::Float32,
                    NativeTypeRef::Float64 => MemberType::Float64,
                    NativeTypeRef::String => MemberType::String,
                    NativeTypeRef::Uuid => MemberType::Uuid,
                    NativeTypeRef::Timestamp => MemberType::Timestamp,
                    NativeTypeRef::Bytes => MemberType::Bytes,
                    NativeTypeRef::JsonObject => MemberType::JsonObject,
                    NativeTypeRef::Clock => MemberType::Clock,
                    NativeTypeRef::Duration => {
                        const UNIT_PREFIX: &str = "ix://quire/unit/";
                        let parts = if self.own_seconds_declared {
                            [
                                DURATION_RESOLVED_PREFIX,
                                UNIT_PREFIX,
                                self.identity.as_ref(),
                                "/s",
                            ]
                        } else {
                            [DURATION_RESOLVED_PREFIX, UNIT_PREFIX, "", "s"]
                        };
                        budget.request(parts.iter().map(|part| part.len()).sum(), form)?;
                        let key = parts.concat();
                        return Ok(self.unit_refs.get(key.as_str()).map(Leaf::Quantity));
                    }
                    NativeTypeRef::Rational | NativeTypeRef::Decimal | NativeTypeRef::Text => {
                        return Ok(None);
                    }
                };
                return Ok(Some(Leaf::Plain(plain)));
            }
            SelectedTypeRef::Declared(name) => name.as_ref(),
        };
        if let Some(bounds) = self.value_types.get(name) {
            return Ok(bounds.as_ref().map(|bounds| match bounds {
                ValueBounds::Integer(range) => Leaf::Plain(MemberType::IntRange(*range)),
                ValueBounds::Rational(bounds, None) => {
                    Leaf::Plain(MemberType::RationalRange(*bounds))
                }
                ValueBounds::Rational(bounds, Some(unit)) => Leaf::Carried(*bounds, unit),
                ValueBounds::Text(bounds) => Leaf::Text(bounds),
            }));
        }
        if self.record_types.contains_key(name) {
            return Ok(Some(Leaf::Declared(DeclarationForm::RecordValueType, name)));
        }
        if self.tuple_types.contains_key(name) {
            return Ok(Some(Leaf::Declared(DeclarationForm::TupleValueType, name)));
        }
        if self.variant_types.contains_key(name) {
            return Ok(Some(Leaf::Declared(DeclarationForm::VariantType, name)));
        }
        let Some(object) = self.object_types.get(name) else {
            return Ok(None);
        };
        if object.interface {
            return Ok(None);
        }
        let form = if self.state_machines.contains(name) {
            DeclarationForm::StateMachine
        } else if self.processes.contains(name) {
            DeclarationForm::Process
        } else {
            DeclarationForm::ObjectType
        };
        Ok(Some(Leaf::Declared(form, name)))
    }

    /// A field's type, each box and each
    /// owned id requested before it is made, at `form` of `budget`'s locus.
    pub(in crate::checked_package::v2) fn field_type_charged(
        &self,
        field: &FieldDecl,
        budget: &mut Budget<'_, '_>,
        form: &'static str,
    ) -> Result<Option<MemberType>, ValidationFailure> {
        self.slot_type_charged(&field.slot, field.optional, budget, form)
    }

    /// A slot's type as [`Self::slot_type`] derives it (`optional` false) or,
    /// with `optional`, including the field's optionality wrapper.
    pub(in crate::checked_package::v2) fn slot_type_charged(
        &self,
        slot: &TypedSlot,
        optional: bool,
        budget: &mut Budget<'_, '_>,
        form: &'static str,
    ) -> Result<Option<MemberType>, ValidationFailure> {
        let Some(type_ref) = slot.type_ref.as_ref() else {
            return Ok(None);
        };
        self.derive_type_charged(type_ref, slot.multiplicity, optional, budget, form)
    }

    pub(in crate::checked_package::v2) fn element_type_in(
        &self,
        type_ref: &SelectedTypeRef,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<MemberType>, ValidationFailure> {
        self.derive_type_charged(
            type_ref,
            Multiplicity {
                lower: 1,
                upper: Some(1),
                ordered: false,
                unique: false,
            },
            false,
            budget,
            "member_type",
        )
    }

    fn derive_type_charged(
        &self,
        type_ref: &SelectedTypeRef,
        multiplicity: Multiplicity,
        optional: bool,
        budget: &mut Budget<'_, '_>,
        form: &'static str,
    ) -> Result<Option<MemberType>, ValidationFailure> {
        let Some((bottom, layers)) = self.bottom(type_ref) else {
            return Ok(None);
        };
        let Some(leaf) = self.leaf(bottom, budget, form)? else {
            return Ok(None);
        };
        let collection = if (multiplicity.lower, multiplicity.upper) == (1, Some(1)) {
            None
        } else {
            let bounds = match (multiplicity.lower, multiplicity.upper) {
                (0, None) => None,
                (_, None) => return Ok(None),
                (lower, Some(upper)) => Some((lower, upper)),
            };
            Some(Wrap::Collection(
                CollectionKind::of(multiplicity.ordered, multiplicity.unique),
                bounds,
            ))
        };
        let clock = matches!(type_ref, SelectedTypeRef::Native(NativeTypeRef::Clock));
        let mut wraps = [None; 2];
        match (optional, collection) {
            (false, collection) => wraps[0] = collection,
            (true, None) => wraps[0] = Some(Wrap::Option),
            (true, Some(collection)) if clock => wraps = [Some(collection), Some(Wrap::Option)],
            (true, Some(collection)) => wraps = [Some(Wrap::Option), Some(collection)],
        }
        let boxes = wraps.iter().flatten().count() + layers;
        let leaf_bytes = leaf.bytes(&self.identity);
        budget.request(
            boxes
                .saturating_mul(std::mem::size_of::<MemberType>())
                .saturating_add(leaf_bytes),
            form,
        )?;
        let mut root = MemberType::Boolean;
        let mut cursor = &mut root;
        let put = |cursor: &mut MemberType, wrap: Wrap| {
            *cursor = match wrap {
                Wrap::Option => MemberType::Option(Box::new(MemberType::Boolean)),
                Wrap::Sequence => MemberType::Collection {
                    kind: CollectionKind::Sequence,
                    element: Box::new(MemberType::Boolean),
                    bounds: None,
                },
                Wrap::Collection(kind, bounds) => MemberType::Collection {
                    kind,
                    element: Box::new(MemberType::Boolean),
                    bounds,
                },
            };
        };
        for wrap in wraps.into_iter().flatten() {
            put(cursor, wrap);
            let Some(inner) = cursor.inner_mut() else {
                return Ok(None);
            };
            cursor = inner;
        }
        // The chain's layers, outermost first, as the walk met them.
        let mut current = type_ref;
        while let SelectedTypeRef::Declared(name) = current {
            let Some(definition) = self.collection_types.get(name) else {
                break;
            };
            match definition {
                SelectedTypeDefinition::Sequence(items) => {
                    put(cursor, Wrap::Sequence);
                    current = items;
                }
                SelectedTypeDefinition::Option(inner) => {
                    put(cursor, Wrap::Option);
                    current = inner;
                }
                SelectedTypeDefinition::Reference(target) => {
                    current = target;
                    continue;
                }
                SelectedTypeDefinition::Map(_) => return Ok(None),
            }
            let Some(inner) = cursor.inner_mut() else {
                return Ok(None);
            };
            cursor = inner;
        }
        *cursor = match leaf {
            Leaf::Plain(plain) => plain,
            Leaf::Quantity(node) => MemberType::Quantity(node.clone()),
            Leaf::Carried(bounds, node) => MemberType::CarriedRational(bounds, node.clone()),
            Leaf::Text(bounds) => MemberType::TextBounds(bounds.clone()),
            Leaf::Declared(form, node) => {
                let id = declaration_id(&self.identity, form, node);
                match form {
                    DeclarationForm::RecordValueType => MemberType::Record(id),
                    DeclarationForm::TupleValueType => MemberType::Tuple(id),
                    DeclarationForm::VariantType => MemberType::Variant(id),
                    _ => MemberType::Reference(id),
                }
            }
        };
        Ok(Some(root))
    }
}
