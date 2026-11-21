//! The FIPA ACL string encoding (S-expression form).
//!
//! This module implements the standard textual representation of an ACL
//! message as defined in the FIPA ACL Message Representation in String
//! specification (fipa00070). A message is written as an S-expression: a
//! parenthesised form whose first element is the performative name, followed by
//! alternating `:keyword` / value pairs.
//!
//! Example (an `inform` message):
//!
//! ```text
//! (inform
//!     :sender (agent-identifier :name jane@foo.com :addresses (sequence http://foo.com/acc))
//!     :receiver (set (agent-identifier :name jack@bar.com))
//!     :language fipa-sl
//!     :ontology traffic
//!     :content "car1 is broken"
//! )
//! ```
//!
//! * An agent identifier is written `(agent-identifier :name <name>
//!   :addresses (sequence <url> ...) :resolvers (sequence <aid> ...))`.
//! * Multiple receivers are written as `(set <aid> ...)`.
//! * String values (such as `:content`) are double-quoted and use backslash
//!   escapes for `\`, `"`, newline, tab and carriage return.
//! * The encoding is whitespace-insensitive: this crate always *writes* a
//!   readable, indented form, but the *reader* accepts any whitespace.
//!
//! Both free functions ([`encode`], [`decode`]) and convenience methods
//! ([`AclMessage::to_fipa_string`], [`AclMessage::from_fipa_string`]) are
//! provided.

use std::str::FromStr;

use crate::agent_id::AgentId;
use crate::error::{AclError, Result};
use crate::message::AclMessage;
use crate::performative::Performative;

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

/// Encodes `msg` into the FIPA ACL string representation.
///
/// The output is indented with four spaces per parameter for readability; it is
/// otherwise a faithful, whitespace-insensitive rendering that [`decode`] can
/// read back.
pub fn encode(msg: &AclMessage) -> String {
    let mut out = String::new();
    out.push('(');
    out.push_str(msg.performative.as_str());
    out.push('\n');

    if let Some(sender) = &msg.sender {
        out.push_str("    :sender ");
        encode_aid(&mut out, sender);
        out.push('\n');
    }

    if !msg.receiver.is_empty() {
        out.push_str("    :receiver (set");
        for receiver in &msg.receiver {
            out.push(' ');
            encode_aid(&mut out, receiver);
        }
        out.push_str(")\n");
    }

    if let Some(reply_to) = &msg.reply_to {
        out.push_str("    :reply-to ");
        encode_aid(&mut out, reply_to);
        out.push('\n');
    }

    push_scalar_param(&mut out, ":content", &msg.content);
    push_scalar_param(&mut out, ":language", &msg.language);
    push_scalar_param(&mut out, ":encoding", &msg.encoding);
    push_scalar_param(&mut out, ":ontology", &msg.ontology);
    push_scalar_param(&mut out, ":protocol", &msg.protocol);
    push_scalar_param(&mut out, ":conversation-id", &msg.conversation_id);
    push_scalar_param(&mut out, ":reply-with", &msg.reply_with);
    push_scalar_param(&mut out, ":in-reply-to", &msg.in_reply_to);
    push_scalar_param(&mut out, ":reply-by", &msg.reply_by);

    out.push(')');
    out
}

/// Writes a `:keyword value` line for an optional scalar parameter.
fn push_scalar_param(out: &mut String, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        out.push_str("    ");
        out.push_str(key);
        out.push(' ');
        write_scalar(out, value);
        out.push('\n');
    }
}

/// Encodes a single agent identifier into `out`.
fn encode_aid(out: &mut String, aid: &AgentId) {
    out.push_str("(agent-identifier :name ");
    write_scalar(out, &aid.name);

    if !aid.addresses.is_empty() {
        out.push_str(" :addresses (sequence");
        for address in &aid.addresses {
            out.push(' ');
            write_scalar(out, address);
        }
        out.push(')');
    }

    if !aid.resolvers.is_empty() {
        out.push_str(" :resolvers (sequence");
        for resolver in &aid.resolvers {
            out.push(' ');
            encode_aid(out, resolver);
        }
        out.push(')');
    }

    out.push(')');
}

/// Writes a scalar value: as a bare atom when it is "safe", otherwise as a
/// quoted, escaped string.
fn write_scalar(out: &mut String, value: &str) {
    if is_safe_atom(value) {
        out.push_str(value);
    } else {
        write_quoted(out, value);
    }
}

/// Returns `true` if `value` can be written as a bare atom without quoting.
fn is_safe_atom(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '(' | ')' | '"' | '\\'))
}

/// Writes `value` as a double-quoted string with backslash escapes.
fn write_quoted(out: &mut String, value: &str) {
    out.push('"');
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out.push('"');
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

/// Decodes a FIPA ACL string representation into an [`AclMessage`].
///
/// The input is parsed leniently wherever the spec allows it (for example a
/// `:receiver` may be a `(set ...)`, a `(sequence ...)` or a single
/// `(agent-identifier ...)`), but structural errors such as truncated input or
/// an unknown performative are reported as [`AclError`] values.
pub fn decode(input: &str) -> Result<AclMessage> {
    let mut parser = Parser::new(input);
    let tree = parser.parse_sexp()?;
    parser.skip_whitespace();
    if let Some(c) = parser.peek() {
        return Err(AclError::UnexpectedToken {
            position: parser.position(),
            found: c.to_string(),
            expected: "end of input".to_string(),
        });
    }
    sexp_to_message(tree)
}

/// The internal S-expression tree.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Sexp {
    /// A bare atom, e.g. `inform` or `jane@foo.com`.
    Atom(String),
    /// A double-quoted string literal, already unescaped.
    Str(String),
    /// A parenthesised list.
    List(Vec<Sexp>),
}

/// A minimal, whitespace-insensitive S-expression parser.
struct Parser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            chars: input.chars().peekable(),
            position: 0,
        }
    }

    /// Current 0-based character offset.
    fn position(&self) -> usize {
        self.position
    }

    /// Peeks at the next character without consuming it.
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Consumes and returns the next character, advancing the offset.
    fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    /// Skips any run of whitespace.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    /// Parses a single S-expression.
    fn parse_sexp(&mut self) -> Result<Sexp> {
        self.skip_whitespace();
        match self.peek() {
            None => Err(AclError::UnexpectedEnd),
            Some('(') => self.parse_list(),
            Some(')') => Err(AclError::UnexpectedToken {
                position: self.position(),
                found: ")".to_string(),
                expected: "a value".to_string(),
            }),
            Some('"') => self.parse_string(),
            Some(_) => self.parse_atom(),
        }
    }

    /// Parses a parenthesised list.
    fn parse_list(&mut self) -> Result<Sexp> {
        self.next(); // consume '('
        let mut items = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => return Err(AclError::UnexpectedEnd),
                Some(')') => {
                    self.next();
                    return Ok(Sexp::List(items));
                }
                _ => items.push(self.parse_sexp()?),
            }
        }
    }

    /// Parses a double-quoted string, interpreting backslash escapes.
    fn parse_string(&mut self) -> Result<Sexp> {
        self.next(); // consume opening quote
        let mut out = String::new();
        loop {
            match self.next() {
                None => return Err(AclError::UnexpectedEnd),
                Some('"') => return Ok(Sexp::Str(out)),
                Some('\\') => match self.next() {
                    None => return Err(AclError::UnexpectedEnd),
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    // Lenient: an unknown escape keeps the escaped character.
                    Some(other) => out.push(other),
                },
                Some(c) => out.push(c),
            }
        }
    }

    /// Parses a bare atom, terminated by whitespace, `(`, `)`, or `"`.
    fn parse_atom(&mut self) -> Result<Sexp> {
        let mut out = String::new();
        while let Some(c) = self.peek() {
            if c.is_whitespace() || matches!(c, '(' | ')' | '"') {
                break;
            }
            out.push(c);
            self.next();
        }
        if out.is_empty() {
            return Err(AclError::UnexpectedEnd);
        }
        Ok(Sexp::Atom(out))
    }
}

/// Converts a parsed S-expression tree into an [`AclMessage`].
fn sexp_to_message(sexp: Sexp) -> Result<AclMessage> {
    let items = match sexp {
        Sexp::List(items) => items,
        other => {
            return Err(AclError::UnexpectedToken {
                position: 0,
                found: render(&other),
                expected: "a message form starting with a performative".to_string(),
            })
        }
    };

    let mut iter = items.into_iter();
    let head = iter.next().ok_or(AclError::UnexpectedEnd)?;
    let name = match head {
        Sexp::Atom(s) => s,
        Sexp::Str(s) => s,
        Sexp::List(_) => {
            return Err(AclError::UnexpectedToken {
                position: 0,
                found: "(a list)".to_string(),
                expected: "a performative name".to_string(),
            })
        }
    };
    let performative = Performative::from_str(&name)?;

    let mut message = AclMessage::new(performative);
    let rest: Vec<Sexp> = iter.collect();
    let mut i = 0;
    while i < rest.len() {
        let key = match &rest[i] {
            Sexp::Atom(s) => s.clone(),
            other => {
                return Err(AclError::UnexpectedToken {
                    position: 0,
                    found: render(other),
                    expected: "a parameter keyword starting with ':'".to_string(),
                })
            }
        };
        if !key.starts_with(':') {
            return Err(AclError::UnexpectedToken {
                position: 0,
                found: key,
                expected: "a parameter keyword starting with ':'".to_string(),
            });
        }
        i += 1;
        let value = rest.get(i).ok_or(AclError::UnexpectedEnd)?;

        match key.as_str() {
            ":sender" => message.sender = Some(sexp_to_aid(value)?),
            ":receiver" => message.receiver = sexp_to_receivers(value)?,
            ":reply-to" => message.reply_to = Some(sexp_to_aid(value)?),
            ":content" => message.content = Some(sexp_to_scalar(value)?),
            ":language" => message.language = Some(sexp_to_scalar(value)?),
            ":encoding" => message.encoding = Some(sexp_to_scalar(value)?),
            ":ontology" => message.ontology = Some(sexp_to_scalar(value)?),
            ":protocol" => message.protocol = Some(sexp_to_scalar(value)?),
            ":conversation-id" => message.conversation_id = Some(sexp_to_scalar(value)?),
            ":reply-with" => message.reply_with = Some(sexp_to_scalar(value)?),
            ":in-reply-to" => message.in_reply_to = Some(sexp_to_scalar(value)?),
            ":reply-by" => message.reply_by = Some(sexp_to_scalar(value)?),
            // Unknown parameters are ignored for forward compatibility.
            _ => {}
        }
        i += 1;
    }

    Ok(message)
}

/// Extracts a scalar (string or atom) value.
fn sexp_to_scalar(sexp: &Sexp) -> Result<String> {
    match sexp {
        Sexp::Atom(s) | Sexp::Str(s) => Ok(s.clone()),
        Sexp::List(_) => Err(AclError::UnexpectedToken {
            position: 0,
            found: "(a list)".to_string(),
            expected: "a scalar value".to_string(),
        }),
    }
}

/// Converts an S-expression into a single [`AgentId`].
fn sexp_to_aid(sexp: &Sexp) -> Result<AgentId> {
    let items = match sexp {
        Sexp::List(items) => items,
        other => {
            return Err(AclError::UnexpectedToken {
                position: 0,
                found: render(other),
                expected: "(agent-identifier ...)".to_string(),
            })
        }
    };

    let head = items.first().ok_or(AclError::UnexpectedEnd)?;
    match head {
        Sexp::Atom(s) if s == "agent-identifier" => {}
        other => {
            return Err(AclError::UnexpectedToken {
                position: 0,
                found: render(other),
                expected: "\"agent-identifier\"".to_string(),
            })
        }
    }

    let mut aid = AgentId::new("");
    let mut has_name = false;
    let rest = &items[1..];
    let mut i = 0;
    while i < rest.len() {
        let key = match &rest[i] {
            Sexp::Atom(s) => s.as_str(),
            other => {
                return Err(AclError::UnexpectedToken {
                    position: 0,
                    found: render(other),
                    expected: "a field keyword starting with ':'".to_string(),
                })
            }
        };
        i += 1;
        let value = rest.get(i).ok_or(AclError::UnexpectedEnd)?;
        match key {
            ":name" => {
                aid.name = sexp_to_scalar(value)?;
                has_name = true;
            }
            ":addresses" => aid.addresses = sexp_to_sequence_scalars(value)?,
            ":resolvers" => aid.resolvers = sexp_to_sequence_aids(value)?,
            _ => {}
        }
        i += 1;
    }

    if !has_name {
        return Err(AclError::MissingParameter("name"));
    }
    Ok(aid)
}

/// Converts an S-expression into a list of receivers.
///
/// Accepts `(set (agent-identifier ...) ...)`, `(sequence ...)`, or a single
/// `(agent-identifier ...)`.
fn sexp_to_receivers(sexp: &Sexp) -> Result<Vec<AgentId>> {
    let items = match sexp {
        Sexp::List(items) => items,
        other => {
            return Err(AclError::UnexpectedToken {
                position: 0,
                found: render(other),
                expected: "(set (agent-identifier ...) ...)".to_string(),
            })
        }
    };

    match items.first() {
        Some(Sexp::Atom(head)) if head == "set" || head == "sequence" => items[1..]
            .iter()
            .map(sexp_to_aid)
            .collect::<Result<Vec<_>>>(),
        Some(Sexp::Atom(head)) if head == "agent-identifier" => {
            Ok(vec![sexp_to_aid(sexp)?])
        }
        _ => Err(AclError::UnexpectedToken {
            position: 0,
            found: render(sexp),
            expected: "(set (agent-identifier ...) ...)".to_string(),
        }),
    }
}

/// Converts a `(sequence <scalar> ...)` form into a list of scalars.
fn sexp_to_sequence_scalars(sexp: &Sexp) -> Result<Vec<String>> {
    let items = match sexp {
        Sexp::List(items) => items,
        Sexp::Atom(s) | Sexp::Str(s) => return Ok(vec![s.clone()]),
    };
    let start = match items.first() {
        Some(Sexp::Atom(head)) if head == "sequence" => 1,
        _ => 0,
    };
    items[start..]
        .iter()
        .map(sexp_to_scalar)
        .collect::<Result<Vec<_>>>()
}

/// Converts a `(sequence (agent-identifier ...) ...)` form into a list of AIDs.
fn sexp_to_sequence_aids(sexp: &Sexp) -> Result<Vec<AgentId>> {
    let items = match sexp {
        Sexp::List(items) => items,
        other => {
            return Err(AclError::UnexpectedToken {
                position: 0,
                found: render(other),
                expected: "(sequence (agent-identifier ...) ...)".to_string(),
            })
        }
    };
    let start = match items.first() {
        Some(Sexp::Atom(head)) if head == "sequence" => 1,
        _ => 0,
    };
    items[start..]
        .iter()
        .map(sexp_to_aid)
        .collect::<Result<Vec<_>>>()
}

/// Renders an S-expression into a short, human-readable string for error
/// messages.
fn render(sexp: &Sexp) -> String {
    match sexp {
        Sexp::Atom(s) => s.clone(),
        Sexp::Str(s) => format!("{:?}", s),
        Sexp::List(_) => "(a list)".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Convenience methods
// ---------------------------------------------------------------------------

impl AclMessage {
    /// Encodes this message into the FIPA ACL string representation.
    ///
    /// Equivalent to [`encode`].
    pub fn to_fipa_string(&self) -> String {
        encode(self)
    }

    /// Decodes a FIPA ACL string representation into a message.
    ///
    /// Equivalent to [`decode`].
    pub fn from_fipa_string(s: &str) -> Result<AclMessage> {
        decode(s)
    }
}
