//! Agent identifiers (AIDs).
//!
//! An *Agent IDentifier* (AID) names an agent in a multi-agent system. A FIPA
//! AID consists of:
//!
//! * a **name** — a globally unique symbolic name such as `jane@foo.com`;
//! * an optional list of **addresses** — transport-level locations (URLs, IIOP
//!   addresses, ...); and
//! * an optional list of **resolvers** — other AIDs that can be asked to
//!   resolve this one.
//!
//! See the FIPA ACL Message Structure specification (fipa00061) for the full
//! definition.

use serde::{Deserialize, Serialize};

/// A FIPA agent identifier (AID).
///
/// Build one fluently:
///
/// ```
/// use fipa_acl::AgentId;
///
/// let aid = AgentId::new("jane@foo.com")
///     .with_address("http://foo.com/acc")
///     .with_resolver(AgentId::new("ams@foo.com"));
/// assert_eq!(aid.name, "jane@foo.com");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentId {
    /// The globally unique symbolic name of the agent, e.g. `jane@foo.com`.
    pub name: String,
    /// Transport-level addresses at which the agent can be reached.
    pub addresses: Vec<String>,
    /// Other AIDs that can resolve this agent's address.
    pub resolvers: Vec<AgentId>,
}

impl AgentId {
    /// Creates a new AID with the given name and no addresses or resolvers.
    pub fn new(name: impl Into<String>) -> Self {
        AgentId {
            name: name.into(),
            addresses: Vec::new(),
            resolvers: Vec::new(),
        }
    }

    /// Adds a transport-level address, returning the updated AID.
    pub fn with_address(mut self, addr: impl Into<String>) -> Self {
        self.addresses.push(addr.into());
        self
    }

    /// Adds a resolver AID, returning the updated AID.
    pub fn with_resolver(mut self, resolver: AgentId) -> Self {
        self.resolvers.push(resolver);
        self
    }
}

impl From<&str> for AgentId {
    /// Convenience conversion so that `"jane@foo.com"` can be used where an
    /// [`AgentId`] is expected.
    fn from(name: &str) -> Self {
        AgentId::new(name)
    }
}

impl From<String> for AgentId {
    /// Convenience conversion so that an owned `String` can be used where an
    /// [`AgentId`] is expected.
    fn from(name: String) -> Self {
        AgentId::new(name)
    }
}
