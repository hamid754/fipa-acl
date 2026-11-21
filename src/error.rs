//! Error types for the `fipa-acl` crate.
//!
//! All fallible operations in this crate return [`Result`], whose error type is
//! [`AclError`]. The error is deliberately small and `Clone`/`PartialEq` so that
//! it is easy to assert on in tests.

use std::fmt;

/// The error type returned by fallible operations in this crate.
///
/// It covers the two broad categories of failure you can hit:
///
/// * **Semantic** problems with a message (an unknown performative, a missing
///   required parameter, content that is not valid JSON); and
/// * **Syntactic** problems while parsing the FIPA string encoding (truncated
///   input, an unexpected token, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AclError {
    /// The input ended before a complete value could be read.
    ///
    /// This is returned when parsing a FIPA string that is truncated, for
    /// example `"(inform :sender (agent-identifier :name a)"` where the closing
    /// parenthesis of the outer form is missing.
    UnexpectedEnd,

    /// A token was found where a different token was expected.
    ///
    /// `position` is the character offset at which the offending token was
    /// found, `found` is what was actually read, and `expected` is a
    /// human-readable description of what the parser was looking for.
    UnexpectedToken {
        /// Character offset (0-based) of the offending token in the input.
        position: usize,
        /// The token that was actually found.
        found: String,
        /// A human-readable description of what was expected instead.
        expected: String,
    },

    /// The message carried a performative name this crate does not know.
    ///
    /// The contained string is the raw name that could not be mapped to one of
    /// the 22 standard [`Performative`](crate::Performative) values.
    UnknownPerformative(String),

    /// A required message parameter was missing.
    ///
    /// The contained string names the missing parameter, e.g. `"sender"`.
    MissingParameter(&'static str),

    /// The message content could not be (de)serialized as JSON.
    ///
    /// The contained string carries the underlying `serde_json` diagnostic.
    InvalidContent(String),
}

impl fmt::Display for AclError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AclError::UnexpectedEnd => write!(f, "unexpected end of input"),
            AclError::UnexpectedToken {
                position,
                found,
                expected,
            } => write!(
                f,
                "unexpected token {:?} at position {}: expected {}",
                found, position, expected
            ),
            AclError::UnknownPerformative(name) => {
                write!(f, "unknown performative: {:?}", name)
            }
            AclError::MissingParameter(name) => {
                write!(f, "missing required parameter: {}", name)
            }
            AclError::InvalidContent(msg) => {
                write!(f, "invalid content: {}", msg)
            }
        }
    }
}

impl std::error::Error for AclError {}

/// A convenient alias for `Result` values produced by this crate.
pub type Result<T> = std::result::Result<T, AclError>;
