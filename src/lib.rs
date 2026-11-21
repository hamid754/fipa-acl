//! # fipa-acl
//!
//! A clean, standards-compliant **FIPA-ACL** (Agent Communication Language)
//! message model for Rust.
//!
//! FIPA-ACL is the message language used by FIPA-compliant multi-agent systems
//! (systemes multi-agents, SMA). A message is a *communicative act* (an
//! `inform`, a `request`, a `cfp`, ...) wrapped in a standard envelope carrying
//! the sender, the receivers, the content, and conversation-tracking metadata.
//!
//! This crate provides:
//!
//! * the **22 standard performatives** ([`Performative`]);
//! * the **full message envelope** ([`AclMessage`]) and agent identifiers
//!   ([`AgentId`]);
//! * the **standard string encoding** (S-expression form) via [`encode`] /
//!   [`decode`] and the [`AclMessage::to_fipa_string`] /
//!   [`AclMessage::from_fipa_string`] convenience methods;
//! * `serde` support and JSON content helpers.
//!
//! This is "level 1" scope: it is purely a **message model**. There is no
//! network transport and no agent platform.
//!
//! ## Example
//!
//! ```
//! use fipa_acl::{AclMessage, AgentId, Performative};
//!
//! // Build an inform message carrying structured JSON content.
//! let mut msg = AclMessage::new(Performative::Inform)
//!     .sender(AgentId::new("alice@example.com"))
//!     .receiver(AgentId::new("bob@example.com"))
//!     .language("fipa-sl")
//!     .ontology("traffic");
//! msg.set_content_json(&serde_json::json!({ "car": "car1", "status": "broken" }))
//!     .unwrap();
//!
//! // Encode it to the standard FIPA string representation...
//! let wire = msg.to_fipa_string();
//! assert!(wire.starts_with("(inform"));
//!
//! // ...and decode it back.
//! let decoded = AclMessage::from_fipa_string(&wire).unwrap();
//! assert_eq!(msg, decoded);
//! ```
//!
//! ## References
//!
//! * FIPA Communicative Act Library (performative semantics):
//!   <https://www.fipa.org/specs/fipa00037/>
//! * FIPA ACL Message Structure: <https://www.fipa.org/specs/fipa00061/>
//! * FIPA ACL Message Representation in String:
//!   <https://www.fipa.org/specs/fipa00070/>
//! * FIPA ACL Message Representation in XML: <https://www.fipa.org/specs/fipa00071/>

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod agent_id;
pub mod codec;
pub mod error;
pub mod message;
pub mod performative;

pub use agent_id::AgentId;
pub use codec::{decode, encode};
pub use error::{AclError, Result};
pub use message::AclMessage;
pub use performative::Performative;
