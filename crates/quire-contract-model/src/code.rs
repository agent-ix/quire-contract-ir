//! The typed STD-001 code (FR-044).
//!
//! A [`Std001Code`] guarantees the form STD-001 fixes for a machine-readable
//! code and nothing else: it does not say that STD-001 registers the code, and it
//! does not name the registry that issued it. [`Std001Code::is_registered`] is
//! the only membership test.
//!
//! The one runtime constructor is [`Std001Code::new`]. A code known at compile
//! time is built by [`std001_code!`](crate::std001_code), which fails to compile
//! on an invalid literal, or by a registered constant.
//!
//! ```
//! use quire_contract_model::{std001_code, Std001Code};
//!
//! const CODE: Std001Code = std001_code!("kani_corpus_identity_collision");
//! assert_eq!(CODE.as_str(), "kani_corpus_identity_collision");
//! assert!(!CODE.is_registered());
//! assert!(Std001Code::KANI_VACUOUS_PROOF.is_registered());
//! ```
//!
//! An invalid literal is a compile error. Stable rustc does not check the error
//! code a `compile_fail` doctest names, so each probe below is paired with the
//! positive doctest above, which proves the macro path itself builds:
//!
//! ```compile_fail
//! let _ = quire_contract_model::std001_code!("Bad-Code");
//! ```
//!
//! and so is a value that is not a literal:
//!
//! ```compile_fail
//! let text: &'static str = "kani_proved";
//! let _ = quire_contract_model::std001_code!(text);
//! ```
//!
//! The type has no infallible conversion from a string:
//!
//! ```compile_fail
//! let _ = quire_contract_model::Std001Code::from("kani_proved");
//! ```
//!
//! ```compile_fail
//! let _ = quire_contract_model::Std001Code::from(String::from("kani_proved"));
//! ```

use std::{
    cmp::Ordering,
    error::Error,
    fmt,
    hash::{Hash, Hasher},
};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::DiagnosticCode;

/// The longest code, in bytes.
const MAX_CODE_BYTES: usize = 64;

/// Whether `bytes` is a well-formed STD-001 code: one to 64 bytes of `a` to `z`,
/// `0` to `9` and `_`, beginning with `a` to `z`, with no trailing `_` and no two
/// adjacent `_`. The length is checked before any byte is scanned.
const fn well_formed(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.len() > MAX_CODE_BYTES {
        return false;
    }
    let mut rest = bytes;
    let mut at_start = true;
    let mut after_underscore = false;
    while let [byte, tail @ ..] = rest {
        match *byte {
            b'a'..=b'z' => after_underscore = false,
            b'0'..=b'9' if !at_start => after_underscore = false,
            b'_' if !at_start && !after_underscore => after_underscore = true,
            _ => return false,
        }
        at_start = false;
        rest = tail;
    }
    !after_underscore
}

/// A machine-readable code in the STD-001 form (FR-044).
///
/// The representation is private, so a value is built only by
/// [`Std001Code::new`], [`Std001Code::from_static`], [`std001_code!`], a
/// registered constant or `From<DiagnosticCode>`. Equality, hashing and ordering
/// are those of [`Std001Code::as_str`].
///
/// The code is held inline in a fixed buffer, which makes the value `Copy` with
/// no destructor: a `const` item, which the compile-time constructors fill,
/// cannot evaluate the destructor of an owning string. The buffer is exactly the
/// longest code, so a value stays small enough to sit in a `Result` error arm.
#[derive(Clone, Copy, Debug)]
pub struct Std001Code {
    /// The code, then zero padding. A code holds no zero byte, so its length is
    /// the offset of the first zero, or the whole buffer.
    bytes: [u8; MAX_CODE_BYTES],
}

/// A string that is not a well-formed STD-001 code.
///
/// Its [`code`](Self::code) is `invalid_code_form`. It holds no copy of the
/// rejected input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Std001CodeError(());

/// Builds a [`Std001Code`] from one string literal, checked at compile time.
///
/// A literal that is not a well-formed code, and an argument that is not a
/// literal, fail to compile; the expansion has no run-time error path.
#[macro_export]
macro_rules! std001_code {
    ($code:literal) => {{
        const CODE: $crate::Std001Code = {
            let checked = $crate::Std001Code::from_static($code);
            match checked {
                Ok(code) => code,
                Err(_) => panic!("not a well-formed STD-001 code"),
            }
        };
        CODE
    }};
}

impl Std001Code {
    /// Checks `code` against the STD-001 form and builds the value.
    ///
    /// This is the one constructor for a string known only at run time.
    ///
    /// # Errors
    ///
    /// Returns [`Std001CodeError`] when `code` is not well formed.
    pub fn new(code: &str) -> Result<Self, Std001CodeError> {
        Self::pack(code)
    }

    /// The `const` form of [`Std001Code::new`] for a `&'static str`.
    ///
    /// # Errors
    ///
    /// Returns [`Std001CodeError`] when `code` is not well formed.
    pub const fn from_static(code: &'static str) -> Result<Self, Std001CodeError> {
        Self::pack(code)
    }

    /// The one place a value is built: checks the form, then copies the bytes.
    const fn pack(code: &str) -> Result<Self, Std001CodeError> {
        let code = code.as_bytes();
        if !well_formed(code) {
            return Err(Std001CodeError(()));
        }
        let mut bytes = [0_u8; MAX_CODE_BYTES];
        // `well_formed` bounds the length, so the split is always present.
        let Some((head, _)) = bytes.split_at_mut_checked(code.len()) else {
            return Err(Std001CodeError(()));
        };
        head.copy_from_slice(code);
        Ok(Self { bytes })
    }

    /// The code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        // The bytes are ASCII by construction, so neither fallback is reachable.
        let end = self
            .bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(MAX_CODE_BYTES);
        let bytes = self.bytes.get(..end).unwrap_or_default();
        std::str::from_utf8(bytes).unwrap_or_default()
    }

    /// Whether STD-001 registers this code: it is in [`Std001Code::REGISTERED`] or
    /// is spelled by a [`DiagnosticCode`]. A well-formed code that is neither is a
    /// valid `Std001Code` that some other registry owns.
    #[must_use]
    pub fn is_registered(&self) -> bool {
        let code = self.as_str();
        Self::REGISTERED.binary_search(&code).is_ok()
            || DiagnosticCode::ALL
                .iter()
                .any(|diagnostic| diagnostic.as_str() == code)
    }
}

/// The one list of the codes STD-001 registers outside [`DiagnosticCode`]: each
/// row is a constant of [`Std001Code`] and a member of [`Std001Code::REGISTERED`],
/// which is sorted.
macro_rules! registered_codes {
    ($( $name:ident => $wire:literal ),+ $(,)?) => {
        impl Std001Code {
            $(
                #[doc = concat!("The code `", $wire, "` (STD-001).")]
                pub const $name: Self = std001_code!($wire);
            )+

            /// The codes STD-001 registers outside [`DiagnosticCode`], sorted.
            pub const REGISTERED: &'static [&'static str] = &[$( $wire ),+];
        }
    };
}

registered_codes! {
    INVALID_CODE_FORM => "invalid_code_form",
    KANI_BACKEND_ABSENT => "kani_backend_absent",
    KANI_BOUND_EXHAUSTED => "kani_bound_exhausted",
    KANI_BOUND_INVALID => "kani_bound_invalid",
    KANI_CAPABILITY_MISSING => "kani_capability_missing",
    KANI_CAPABILITY_REQUEST_INVALID => "kani_capability_request_invalid",
    KANI_COUNTEREXAMPLE => "kani_counterexample",
    KANI_IDENTITY_INVALID => "kani_identity_invalid",
    KANI_OUTCOME_INVALID => "kani_outcome_invalid",
    KANI_POPULATION_INCOMPLETE => "kani_population_incomplete",
    KANI_POPULATION_INVALID => "kani_population_invalid",
    KANI_PROVED => "kani_proved",
    KANI_REFERENCE_INVALID => "kani_reference_invalid",
    KANI_SOLVER_ABSENT => "kani_solver_absent",
    KANI_VACUOUS_PROOF => "kani_vacuous_proof",
}

impl From<DiagnosticCode> for Std001Code {
    fn from(code: DiagnosticCode) -> Self {
        // Every `DiagnosticCode` spelling is a well-formed code, so the `Err` arm
        // is unreachable; TC-442 converts every `DiagnosticCode::ALL` entry and
        // asserts the text is unchanged, which fails if that ever stops holding.
        match Self::pack(code.as_str()) {
            Ok(code) => code,
            Err(_) => Self::INVALID_CODE_FORM,
        }
    }
}

impl PartialEq for Std001Code {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Std001Code {}

impl Hash for Std001Code {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for Std001Code {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Std001Code {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl fmt::Display for Std001Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Std001CodeError {
    /// The code of this error: `invalid_code_form`.
    #[must_use]
    pub fn code(&self) -> Std001Code {
        Std001Code::INVALID_CODE_FORM
    }
}

impl fmt::Display for Std001CodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: not a lowercase ASCII snake-case code of one to {MAX_CODE_BYTES} bytes",
            self.code()
        )
    }
}

impl Error for Std001CodeError {}

impl Serialize for Std001Code {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Reads a JSON string through [`Std001Code::new`], without allocating for a
/// string that is refused.
struct CodeVisitor;

impl Visitor<'_> for CodeVisitor {
    type Value = Std001Code;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a lowercase ASCII snake-case STD-001 code string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Std001Code::new(value).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for Std001Code {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CodeVisitor)
    }
}
