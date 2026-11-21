//! The 22 standard FIPA-ACL performatives.
//!
//! A *communicative act* (or performative) describes the intention behind a
//! message: an `inform` states a fact, a `request` asks the receiver to do
//! something, a `cfp` opens a call for proposals, and so on. The full set is
//! defined in the FIPA Communicative Act Library specification (fipa00037) and
//! carried by the message structure specification (fipa00061).
//!
//! The Rust variant names are idiomatic camel-case; [`Performative::as_str`]
//! (and the `serde` representation, thanks to `rename_all = "kebab-case"`)
//! yields the exact lower-case hyphenated names used on the wire.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::AclError;

/// One of the 22 standard FIPA-ACL performatives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Performative {
    /// `accept-proposal`: agrees to a previously submitted proposal.
    AcceptProposal,
    /// `agree`: promises to perform some action in the future.
    Agree,
    /// `cancel`: cancels a previously requested action.
    Cancel,
    /// `cfp`: issues a call for proposals to one or more agents.
    Cfp,
    /// `confirm`: informs that a proposition is true, believed to be queried before.
    Confirm,
    /// `disconfirm`: informs that a proposition is false, believed to be queried before.
    Disconfirm,
    /// `failure`: reports that an attempted action failed.
    Failure,
    /// `inform`: informs the receiver that a proposition is true.
    Inform,
    /// `inform-if`: informs whether a given proposition is true or false.
    InformIf,
    /// `inform-ref`: informs the receiver of the value of a referring expression.
    InformRef,
    /// `not-understood`: reports that the received message was not understood.
    NotUnderstood,
    /// `propagate`: asks the receiver to forward a message to other agents.
    Propagate,
    /// `propose`: proposes an action to be performed in exchange for something.
    Propose,
    /// `proxy`: asks the receiver to select a target and send it a given message.
    Proxy,
    /// `query-if`: asks whether a given proposition is true.
    QueryIf,
    /// `query-ref`: asks for the value of a referring expression.
    QueryRef,
    /// `refuse`: refuses to perform an action previously requested.
    Refuse,
    /// `reject-proposal`: rejects a previously submitted proposal.
    RejectProposal,
    /// `request`: asks the receiver to perform some action.
    Request,
    /// `request-when`: asks the receiver to perform an action when a condition holds.
    RequestWhen,
    /// `request-whenever`: asks the receiver to perform an action whenever a condition holds.
    RequestWhenever,
    /// `subscribe`: asks to be notified whenever a given reference changes.
    Subscribe,
}

impl Performative {
    /// All 22 standard performatives, in alphabetical order of their wire names.
    pub const ALL: [Performative; 22] = [
        Performative::AcceptProposal,
        Performative::Agree,
        Performative::Cancel,
        Performative::Cfp,
        Performative::Confirm,
        Performative::Disconfirm,
        Performative::Failure,
        Performative::Inform,
        Performative::InformIf,
        Performative::InformRef,
        Performative::NotUnderstood,
        Performative::Propagate,
        Performative::Propose,
        Performative::Proxy,
        Performative::QueryIf,
        Performative::QueryRef,
        Performative::Refuse,
        Performative::RejectProposal,
        Performative::Request,
        Performative::RequestWhen,
        Performative::RequestWhenever,
        Performative::Subscribe,
    ];

    /// Returns the exact FIPA wire name of this performative.
    ///
    /// For example [`Performative::AcceptProposal`] maps to `"accept-proposal"`
    /// and [`Performative::Inform`] maps to `"inform"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Performative::AcceptProposal => "accept-proposal",
            Performative::Agree => "agree",
            Performative::Cancel => "cancel",
            Performative::Cfp => "cfp",
            Performative::Confirm => "confirm",
            Performative::Disconfirm => "disconfirm",
            Performative::Failure => "failure",
            Performative::Inform => "inform",
            Performative::InformIf => "inform-if",
            Performative::InformRef => "inform-ref",
            Performative::NotUnderstood => "not-understood",
            Performative::Propagate => "propagate",
            Performative::Propose => "propose",
            Performative::Proxy => "proxy",
            Performative::QueryIf => "query-if",
            Performative::QueryRef => "query-ref",
            Performative::Refuse => "refuse",
            Performative::RejectProposal => "reject-proposal",
            Performative::Request => "request",
            Performative::RequestWhen => "request-when",
            Performative::RequestWhenever => "request-whenever",
            Performative::Subscribe => "subscribe",
        }
    }
}

impl fmt::Display for Performative {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Performative {
    type Err = AclError;

    /// Parses a performative from its wire name, case-insensitively.
    ///
    /// Returns [`AclError::UnknownPerformative`] if the name does not match any
    /// of the 22 standard performatives.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for p in Performative::ALL {
            if p.as_str().eq_ignore_ascii_case(s) {
                return Ok(p);
            }
        }
        Err(AclError::UnknownPerformative(s.to_string()))
    }
}
