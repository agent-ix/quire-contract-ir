// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-474 through AC-477: bounded borrowed selected-model populations.

use super::model_members::DomainModel;
use super::{CheckedNodeId, CheckedPackageV2};
use quire_exact::{EffectiveId, OriginalDeclarationKey};
use std::fmt;

/// The caller's finite allowance for one selected-model population catalog read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedModelPopulationCatalogLimits {
    /// Maximum root, population, member and alias-lookup visits plus copied
    /// structural-identity component bytes and storage.
    pub work: u64,
    /// Maximum root, population and member descriptors returned.
    pub entries: u64,
}

/// The selected, admitted identities at the first unpaid or unresolved item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedModelPopulationCatalogLocus<'a> {
    /// Selected domain package identity.
    pub model_identity: &'a str,
    /// Its genuine checked graph root.
    pub root: &'a CheckedNodeId,
    /// Original population identity, if the root itself was not the locus.
    pub population: Option<&'a str>,
    /// Original member identity, if present.
    pub member: Option<&'a str>,
    /// Alias lookup step being visited, if normalization stopped there.
    pub alias_step: Option<usize>,
}

/// The finite allowance that stopped a catalog read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedModelPopulationCatalogLimitKind {
    /// Insufficient visit allowance.
    Work,
    /// Insufficient returned-descriptor allowance.
    Entries,
}

/// Why the catalog could not be returned as a complete value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedModelPopulationCatalogError<'a> {
    /// The first unpaid charge; no partial catalog escapes.
    Limit {
        /// Which allowance was exceeded.
        kind: CheckedModelPopulationCatalogLimitKind,
        /// Caller allowance.
        limit: u64,
        /// Total including the first unpaid charge.
        consumed: u64,
        /// Item that incurred that charge.
        locus: CheckedModelPopulationCatalogLocus<'a>,
    },
    /// An admitted root's retained node is unavailable.
    MissingRoot { model_identity: &'a str },
    /// An admitted population has no retained member set.
    MissingMembers {
        /// Selected model and original population whose member set is missing.
        locus: CheckedModelPopulationCatalogLocus<'a>,
    },
    /// A population member cannot name an admitted object type. No partial
    /// catalog is returned.
    UnresolvedMember {
        /// The offending member's position in the selected catalog.
        locus: CheckedModelPopulationCatalogLocus<'a>,
        /// Why its type could not be selected.
        reason: CheckedModelPopulationMemberUnresolved,
    },
}

impl fmt::Display for CheckedModelPopulationCatalogError<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit { kind, locus, .. } => write!(
                formatter,
                "selected-model population catalog {kind:?} limit at {}",
                locus.model_identity
            ),
            Self::MissingRoot { model_identity } => {
                write!(
                    formatter,
                    "selected model root is missing for {model_identity}"
                )
            }
            Self::MissingMembers { locus } => write!(
                formatter,
                "selected population member set is missing for {}",
                locus.population.unwrap_or(locus.model_identity)
            ),
            Self::UnresolvedMember { locus, reason } => write!(
                formatter,
                "selected population member {} has {reason:?} type",
                locus.member.unwrap_or(locus.model_identity)
            ),
        }
    }
}

impl std::error::Error for CheckedModelPopulationCatalogError<'_> {}

/// Why a population member has no normalized selected type identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedModelPopulationMemberUnresolved {
    /// An alias cycle was retained; no type can be selected from it.
    AliasCycle,
    /// The normalized name is not a selected type declaration.
    MissingType,
    /// The normalized identity names a selected declaration of another kind.
    WrongKind,
}

/// One original member identity and its normalized selected type identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelPopulationMember<'a> {
    original: &'a str,
    normalized: EffectiveId,
}

impl<'a> CheckedModelPopulationMember<'a> {
    /// Identity as declared in the admitted selected document.
    pub fn original_identity(&self) -> &'a str {
        self.original
    }

    /// Complete FR-150 effective type identity. An effective type has no
    /// owning effective type, so its structural chain contains this selected
    /// model's alias-normalized original key as its sole component.
    pub fn normalized_type_identity(&self) -> &EffectiveId {
        &self.normalized
    }
}

/// One admitted population, including those unused by the checked graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelPopulation<'a> {
    original: &'a str,
    members: Vec<CheckedModelPopulationMember<'a>>,
}

impl<'a> CheckedModelPopulation<'a> {
    /// Original population identity in the selected document.
    pub fn original_identity(&self) -> &'a str {
        self.original
    }

    /// Every original member, in ascending identity order.
    pub fn members(&self) -> &[CheckedModelPopulationMember<'a>] {
        &self.members
    }
}

/// One selected model root and all of its declared populations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedSelectedModelPopulations<'a> {
    model_identity: &'a str,
    root: &'a CheckedNodeId,
    populations: Vec<CheckedModelPopulation<'a>>,
}

impl<'a> CheckedSelectedModelPopulations<'a> {
    /// Identity of the selected domain model document.
    pub fn model_identity(&self) -> &'a str {
        self.model_identity
    }

    /// Genuine checked graph node that selected this document.
    pub fn root(&self) -> &'a CheckedNodeId {
        self.root
    }

    /// Every population declaration, in ascending original identity order.
    pub fn populations(&self) -> &[CheckedModelPopulation<'a>] {
        &self.populations
    }
}

/// Complete borrowed population catalog of every selected model root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedModelPopulationCatalog<'a> {
    roots: Vec<CheckedSelectedModelPopulations<'a>>,
    work: u64,
    entries: u64,
}

impl<'a> CheckedModelPopulationCatalog<'a> {
    /// Selected model roots in ascending selected identity order.
    pub fn roots(&self) -> &[CheckedSelectedModelPopulations<'a>] {
        &self.roots
    }

    /// Charged work, including alias edges.
    pub fn work(&self) -> u64 {
        self.work
    }

    /// Charged root, population and member output descriptors.
    pub fn entries(&self) -> u64 {
        self.entries
    }
}

struct CatalogBudget {
    limits: CheckedModelPopulationCatalogLimits,
    work: u64,
    entries: u64,
}

impl CatalogBudget {
    fn new(limits: CheckedModelPopulationCatalogLimits) -> Self {
        Self {
            limits,
            work: 0,
            entries: 0,
        }
    }

    fn visit<'a>(
        &mut self,
        locus: CheckedModelPopulationCatalogLocus<'a>,
    ) -> Result<(), CheckedModelPopulationCatalogError<'a>> {
        self.charge_work(1, locus)
    }

    fn charge_work<'a>(
        &mut self,
        amount: u64,
        locus: CheckedModelPopulationCatalogLocus<'a>,
    ) -> Result<(), CheckedModelPopulationCatalogError<'a>> {
        self.work = self.work.saturating_add(amount);
        if self.work > self.limits.work {
            return Err(CheckedModelPopulationCatalogError::Limit {
                kind: CheckedModelPopulationCatalogLimitKind::Work,
                limit: self.limits.work,
                consumed: self.work,
                locus,
            });
        }
        Ok(())
    }

    fn output<'a>(
        &mut self,
        locus: CheckedModelPopulationCatalogLocus<'a>,
    ) -> Result<(), CheckedModelPopulationCatalogError<'a>> {
        self.entries = self.entries.saturating_add(1);
        if self.entries > self.limits.entries {
            return Err(CheckedModelPopulationCatalogError::Limit {
                kind: CheckedModelPopulationCatalogLimitKind::Entries,
                limit: self.limits.entries,
                consumed: self.entries,
                locus,
            });
        }
        Ok(())
    }
}

fn normalized_member<'a>(
    model: &'a DomainModel,
    original: &'a str,
    budget: &mut CatalogBudget,
    locus: CheckedModelPopulationCatalogLocus<'a>,
) -> Result<EffectiveId, CheckedModelPopulationCatalogError<'a>> {
    let mut current = original;
    // More edges than retained aliases implies a cycle. The extra edge is
    // charged before it is inspected, without building a visited-name set.
    for step in 0..=model.population_alias_count() {
        budget.visit(CheckedModelPopulationCatalogLocus {
            alias_step: Some(step),
            ..locus
        })?;
        let Some(target) = model.population_alias_target(current) else {
            let reason = if model.object_types.contains_key(current) {
                None
            } else if model.population_member_type_exists(current) {
                Some(CheckedModelPopulationMemberUnresolved::WrongKind)
            } else {
                Some(CheckedModelPopulationMemberUnresolved::MissingType)
            };
            if let Some(reason) = reason {
                return Err(CheckedModelPopulationCatalogError::UnresolvedMember { locus, reason });
            }
            // Charge the exact borrowed component bytes before constructing
            // the owned structural identity returned to the consumer.
            let bytes = model
                .identity
                .len()
                .saturating_add(current.len())
                .saturating_add(std::mem::size_of::<OriginalDeclarationKey>());
            budget.charge_work(u64::try_from(bytes).unwrap_or(u64::MAX), locus)?;
            return Ok(EffectiveId::new(
                None,
                OriginalDeclarationKey::new(model.identity.as_ref(), current),
            ));
        };
        current = target;
    }
    Err(CheckedModelPopulationCatalogError::UnresolvedMember {
        locus,
        reason: CheckedModelPopulationMemberUnresolved::AliasCycle,
    })
}

impl CheckedPackageV2 {
    /// Returns every admitted selected model's populations, including unused
    /// declarations, without rereading a document or accepting caller model data.
    /// Each visited root, population, member and alias lookup, plus the copied
    /// bytes of each structural effective type identity, charges work; each
    /// returned descriptor charges an entry before allocation.
    ///
    /// # Errors
    ///
    /// Returns the first unpaid limit charge, or a typed retained-data error
    /// if an admitted package's selected root or member set is unavailable.
    pub fn selected_model_population_catalog(
        &self,
        limits: CheckedModelPopulationCatalogLimits,
    ) -> Result<CheckedModelPopulationCatalog<'_>, CheckedModelPopulationCatalogError<'_>> {
        let mut budget = CatalogBudget::new(limits);
        let mut roots = Vec::new();
        for model in self.models.selected_models() {
            let root = self.wire.semantic_graph.nodes.get(model.root).ok_or(
                CheckedModelPopulationCatalogError::MissingRoot {
                    model_identity: &model.identity,
                },
            )?;
            let root_locus = CheckedModelPopulationCatalogLocus {
                model_identity: &model.identity,
                root: &root.node_id,
                population: None,
                member: None,
                alias_step: None,
            };
            budget.visit(root_locus)?;
            budget.output(root_locus)?;
            let mut populations = Vec::new();
            for original in &model.populations {
                let population_locus = CheckedModelPopulationCatalogLocus {
                    population: Some(original),
                    ..root_locus
                };
                budget.visit(population_locus)?;
                budget.output(population_locus)?;
                let retained = model.population_members.get(original).ok_or(
                    CheckedModelPopulationCatalogError::MissingMembers {
                        locus: population_locus,
                    },
                )?;
                let mut members = Vec::new();
                for original_member in retained {
                    let member_locus = CheckedModelPopulationCatalogLocus {
                        member: Some(original_member),
                        ..population_locus
                    };
                    budget.visit(member_locus)?;
                    budget.output(member_locus)?;
                    let normalized =
                        normalized_member(model, original_member, &mut budget, member_locus)?;
                    members.push(CheckedModelPopulationMember {
                        original: original_member,
                        normalized,
                    });
                }
                populations.push(CheckedModelPopulation { original, members });
            }
            roots.push(CheckedSelectedModelPopulations {
                model_identity: &model.identity,
                root: &root.node_id,
                populations,
            });
        }
        Ok(CheckedModelPopulationCatalog {
            roots,
            work: budget.work,
            entries: budget.entries,
        })
    }
}
