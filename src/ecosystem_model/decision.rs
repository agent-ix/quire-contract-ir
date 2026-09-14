//! Closed FR-027 refusal vocabulary.

use core::fmt;

/// Stable cause for refusing a manifest, graph, export, or read.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ModelCauseCode {
    InvalidDocument,
    ResourceExhausted,
    ContractMismatch,
    CampaignMismatch,
    RepositorySetMismatch,
    MovingRevision,
    DuplicateNode,
    DuplicateEdge,
    DuplicateGap,
    PopulationOutOfOrder,
    DanglingEdge,
    IllTypedEdge,
    SelfEdge,
    OwnershipIncomplete,
    MultipleOwners,
    DependencyCycle,
    GraphMismatch,
    IdentityMismatch,
}

impl ModelCauseCode {
    const ALL: [Self; 18] = [
        Self::InvalidDocument,
        Self::ResourceExhausted,
        Self::ContractMismatch,
        Self::CampaignMismatch,
        Self::RepositorySetMismatch,
        Self::MovingRevision,
        Self::DuplicateNode,
        Self::DuplicateEdge,
        Self::DuplicateGap,
        Self::PopulationOutOfOrder,
        Self::DanglingEdge,
        Self::IllTypedEdge,
        Self::SelfEdge,
        Self::OwnershipIncomplete,
        Self::MultipleOwners,
        Self::DependencyCycle,
        Self::GraphMismatch,
        Self::IdentityMismatch,
    ];

    /// Returns every closed FR-027 cause.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &Self::ALL
    }

    /// Returns the stable STD-001 spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidDocument => "ecosystem_invalid_document",
            Self::ResourceExhausted => "ecosystem_resource_exhausted",
            Self::ContractMismatch => "ecosystem_contract_mismatch",
            Self::CampaignMismatch => "ecosystem_campaign_mismatch",
            Self::RepositorySetMismatch => "ecosystem_repository_set_mismatch",
            Self::MovingRevision => "ecosystem_moving_revision",
            Self::DuplicateNode => "ecosystem_duplicate_node",
            Self::DuplicateEdge => "ecosystem_duplicate_edge",
            Self::DuplicateGap => "ecosystem_duplicate_gap",
            Self::PopulationOutOfOrder => "ecosystem_population_out_of_order",
            Self::DanglingEdge => "ecosystem_dangling_edge",
            Self::IllTypedEdge => "ecosystem_ill_typed_edge",
            Self::SelfEdge => "ecosystem_self_edge",
            Self::OwnershipIncomplete => "ecosystem_ownership_incomplete",
            Self::MultipleOwners => "ecosystem_multiple_owners",
            Self::DependencyCycle => "ecosystem_dependency_cycle",
            Self::GraphMismatch => "ecosystem_graph_mismatch",
            Self::IdentityMismatch => "ecosystem_identity_mismatch",
        }
    }
}

/// One bounded typed refusal; it contains no partial manifest or graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelDecision {
    code: ModelCauseCode,
    path: Box<str>,
    detail: Box<str>,
}

impl ModelDecision {
    pub(crate) fn new(
        code: ModelCauseCode,
        path: impl Into<Box<str>>,
        detail: impl Into<Box<str>>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            detail: detail.into(),
        }
    }

    /// Returns the stable cause code.
    #[must_use]
    pub const fn code(&self) -> ModelCauseCode {
        self.code
    }

    /// Returns the narrowest known semantic path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns bounded diagnostic detail that is not used for dispatch.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for ModelDecision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl std::error::Error for ModelDecision {}
