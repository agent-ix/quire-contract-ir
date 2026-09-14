//! Stable operation errors for malformed or resource-incomplete bridge work.

/// Stable bridge operation-error code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BridgeErrorCode {
    /// A strict bridge document is malformed, noncanonical, or mismatched.
    InvalidNativePredicateProjection,
    /// Predicate projection could not complete within selected resources.
    PredicateProjectionResourceExhausted,
    /// Predicate valuation could not complete within selected resources.
    PredicateValuationResourceExhausted,
}

impl BridgeErrorCode {
    const ALL: [Self; 3] = [
        Self::InvalidNativePredicateProjection,
        Self::PredicateProjectionResourceExhausted,
        Self::PredicateValuationResourceExhausted,
    ];

    /// Returns every stable operation code.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &Self::ALL
    }

    /// Returns the STD-001 spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidNativePredicateProjection => "invalid_native_predicate_projection",
            Self::PredicateProjectionResourceExhausted => "predicate_projection_resource_exhausted",
            Self::PredicateValuationResourceExhausted => "predicate_valuation_resource_exhausted",
        }
    }
}

/// One bounded bridge operation error; it never carries partial output.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{code}: {message}", code = .code.as_str())]
pub struct BridgeError {
    code: BridgeErrorCode,
    message: Box<str>,
    path: Box<str>,
}

impl BridgeError {
    /// Constructs one stable operation error.
    #[must_use]
    pub fn new(
        code: BridgeErrorCode,
        message: impl Into<Box<str>>,
        path: impl Into<Box<str>>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path: path.into(),
        }
    }

    /// Returns the stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> BridgeErrorCode {
        self.code
    }

    /// Returns the bounded human-readable detail.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the narrowest public path known at refusal.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}
