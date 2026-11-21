//! The ACL message envelope.
//!
//! [`AclMessage`] models a complete FIPA-ACL message: its performative plus all
//! of the standard message parameters (sender, receivers, reply-to, content,
//! language, encoding, ontology, protocol, conversation tracking, ...). See the
//! FIPA ACL Message Structure specification (fipa00061) for the semantics of
//! each parameter.

use serde::{Deserialize, Serialize};

use crate::agent_id::AgentId;
use crate::error::{AclError, Result};
use crate::performative::Performative;

/// A complete FIPA-ACL message.
///
/// Build one with [`AclMessage::new`] and the builder methods, e.g.:
///
/// ```
/// use fipa_acl::{AclMessage, AgentId, Performative};
///
/// let msg = AclMessage::new(Performative::Inform)
///     .sender(AgentId::new("alice@example.com"))
///     .receiver(AgentId::new("bob@example.com"))
///     .language("fipa-sl")
///     .content("the temperature is 21 degrees");
/// assert!(msg.validate().is_ok());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AclMessage {
    /// The communicative act performed by this message.
    pub performative: Performative,
    /// The agent that sent the message.
    pub sender: Option<AgentId>,
    /// The agents that should receive the message. May hold more than one entry
    /// for multicast / `(set ...)` receivers.
    pub receiver: Vec<AgentId>,
    /// The agent to which replies should be sent, if different from the sender.
    pub reply_to: Option<AgentId>,
    /// The content of the message (an expression in the chosen `language` /
    /// `ontology`, or opaque text).
    pub content: Option<String>,
    /// The language in which `content` is expressed, e.g. `fipa-sl`.
    pub language: Option<String>,
    /// The encoding of `content`, e.g. `ISO-8859-1`.
    pub encoding: Option<String>,
    /// The ontology that gives meaning to the symbols in `content`.
    pub ontology: Option<String>,
    /// The interaction protocol being used, e.g. `fipa-contract-net`.
    pub protocol: Option<String>,
    /// An identifier used to group the messages of a single conversation.
    pub conversation_id: Option<String>,
    /// A token that the receiver should echo in `in_reply_to`.
    pub reply_with: Option<String>,
    /// Echoes the `reply_with` token of the message being replied to.
    pub in_reply_to: Option<String>,
    /// A deadline for the message, as an ISO-8601 date-time string,
    /// e.g. `2026-01-01T00:00:00Z`.
    pub reply_by: Option<String>,
}

impl AclMessage {
    /// Creates a new message with the given performative and all optional
    /// parameters left empty.
    pub fn new(performative: Performative) -> Self {
        AclMessage {
            performative,
            sender: None,
            receiver: Vec::new(),
            reply_to: None,
            content: None,
            language: None,
            encoding: None,
            ontology: None,
            protocol: None,
            conversation_id: None,
            reply_with: None,
            in_reply_to: None,
            reply_by: None,
        }
    }

    /// Sets the sender of the message.
    pub fn sender(mut self, sender: impl Into<AgentId>) -> Self {
        self.sender = Some(sender.into());
        self
    }

    /// Appends a receiver to the message's receiver list.
    ///
    /// Call it once per receiver to build a multicast message.
    pub fn receiver(mut self, receiver: impl Into<AgentId>) -> Self {
        self.receiver.push(receiver.into());
        self
    }

    /// Sets the `reply-to` agent.
    pub fn reply_to(mut self, reply_to: impl Into<AgentId>) -> Self {
        self.reply_to = Some(reply_to.into());
        self
    }

    /// Sets the message content.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Sets the content language.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Sets the content encoding.
    pub fn encoding(mut self, encoding: impl Into<String>) -> Self {
        self.encoding = Some(encoding.into());
        self
    }

    /// Sets the content ontology.
    pub fn ontology(mut self, ontology: impl Into<String>) -> Self {
        self.ontology = Some(ontology.into());
        self
    }

    /// Sets the interaction protocol name.
    pub fn protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }

    /// Sets the conversation identifier.
    pub fn conversation_id(mut self, conversation_id: impl Into<String>) -> Self {
        self.conversation_id = Some(conversation_id.into());
        self
    }

    /// Sets the `reply-with` token.
    pub fn reply_with(mut self, reply_with: impl Into<String>) -> Self {
        self.reply_with = Some(reply_with.into());
        self
    }

    /// Sets the `in-reply-to` token, echoing a previous `reply-with`.
    pub fn in_reply_to(mut self, in_reply_to: impl Into<String>) -> Self {
        self.in_reply_to = Some(in_reply_to.into());
        self
    }

    /// Sets the `reply-by` deadline (an ISO-8601 date-time string).
    pub fn reply_by(mut self, reply_by: impl Into<String>) -> Self {
        self.reply_by = Some(reply_by.into());
        self
    }

    /// Checks that the message satisfies the FIPA syntactic requirements that
    /// this crate can validate locally: it must have a sender and at least one
    /// receiver.
    ///
    /// Returns [`AclError::MissingParameter`] otherwise.
    pub fn validate(&self) -> Result<()> {
        if self.sender.is_none() {
            return Err(AclError::MissingParameter("sender"));
        }
        if self.receiver.is_empty() {
            return Err(AclError::MissingParameter("receiver"));
        }
        Ok(())
    }

    /// Serializes `value` as JSON and stores the result in [`content`](Self::content).
    ///
    /// Returns [`AclError::InvalidContent`] if serialization fails.
    pub fn set_content_json<T: serde::Serialize>(&mut self, value: &T) -> Result<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| AclError::InvalidContent(e.to_string()))?;
        self.content = Some(json);
        Ok(())
    }

    /// Deserializes [`content`](Self::content) as JSON of type `T`.
    ///
    /// Returns [`AclError::InvalidContent`] if the content is absent or is not
    /// valid JSON for `T`.
    pub fn content_json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let content = self
            .content
            .as_ref()
            .ok_or_else(|| AclError::InvalidContent("content is empty".to_string()))?;
        serde_json::from_str(content).map_err(|e| AclError::InvalidContent(e.to_string()))
    }
}
