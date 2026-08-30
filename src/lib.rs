//! Versioned semantic contract model and canonical representation for assurance tooling.

/// Placeholder entry point.
pub fn hello() -> &'static str {
    "hello from quire_contract_ir"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_returns_greeting() {
        assert!(hello().contains("quire_contract_ir"));
    }
}
