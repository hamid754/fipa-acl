//! A small, runnable demonstration of the Contract Net Protocol vocabulary
//! (`fipa-contract-net`) built on top of the `fipa-acl` message model.
//!
//! Run it with:
//!
//! ```text
//! cargo run --example contract_net
//! ```
//!
//! The script plays out a classic contract-net exchange:
//!
//! 1. the initiator issues a `cfp` to two participants;
//! 2. one participant answers `propose`, the other `refuse`;
//! 3. the initiator sends `accept-proposal` to the winner and
//!    `reject-proposal` to the loser.
//!
//! Every message is printed in its FIPA string form and re-decoded to show the
//! round-trip is faithful.

use fipa_acl::{AclMessage, AgentId, Performative};

/// Prints an ACL message and verifies that it round-trips through the codec.
fn show(title: &str, msg: &AclMessage) {
    println!("=== {} ===", title);
    let wire = msg.to_fipa_string();
    println!("{}", wire);

    match AclMessage::from_fipa_string(&wire) {
        Ok(decoded) if &decoded == msg => println!("[round-trip OK]\n"),
        Ok(_) => println!("[round-trip MISMATCH]\n"),
        Err(e) => println!("[decode error: {}]\n", e),
    }
}

fn main() {
    let initiator = AgentId::new("manager@example.com").with_address("http://example.com/manager");
    let alice = AgentId::new("alice@example.com").with_address("http://example.com/alice");
    let bob = AgentId::new("bob@example.com").with_address("http://example.com/bob");

    let conversation = "cnp-2026-0001";

    // 1. Call for proposals, sent to both participants.
    let cfp = AclMessage::new(Performative::Cfp)
        .sender(initiator.clone())
        .receiver(alice.clone())
        .receiver(bob.clone())
        .protocol("fipa-contract-net")
        .conversation_id(conversation)
        .reply_with("cfp-1")
        .reply_by("2026-11-21T18:00:00Z")
        .content("Deliver 10 widgets");
    show("initiator -> participants : cfp", &cfp);

    // 2a. Alice proposes.
    let propose = AclMessage::new(Performative::Propose)
        .sender(alice.clone())
        .receiver(initiator.clone())
        .protocol("fipa-contract-net")
        .conversation_id(conversation)
        .in_reply_to("cfp-1")
        .reply_with("propose-1")
        .content("10 widgets for 100 EUR, delivered in 3 days");
    show("alice -> initiator : propose", &propose);

    // 2b. Bob refuses.
    let refuse = AclMessage::new(Performative::Refuse)
        .sender(bob.clone())
        .receiver(initiator.clone())
        .protocol("fipa-contract-net")
        .conversation_id(conversation)
        .in_reply_to("cfp-1")
        .reply_with("refuse-1")
        .content("At capacity, cannot take the job");
    show("bob -> initiator : refuse", &refuse);

    // 3a. The initiator accepts Alice's proposal.
    let accept = AclMessage::new(Performative::AcceptProposal)
        .sender(initiator.clone())
        .receiver(alice.clone())
        .protocol("fipa-contract-net")
        .conversation_id(conversation)
        .in_reply_to("propose-1")
        .content("Contract awarded to you");
    show("initiator -> alice : accept-proposal", &accept);

    // 3b. The initiator rejects Bob's (implicit) offer.
    let reject = AclMessage::new(Performative::RejectProposal)
        .sender(initiator)
        .receiver(bob)
        .protocol("fipa-contract-net")
        .conversation_id(conversation)
        .in_reply_to("refuse-1")
        .content("No contract for you this time");
    show("initiator -> bob : reject-proposal", &reject);

    println!("Contract Net exchange complete.");
}
