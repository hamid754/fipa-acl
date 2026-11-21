//! Integration tests for the `fipa-acl` crate.

use std::str::FromStr;

use fipa_acl::*;

/// A minimal helper: a message seen through an encode/decode round-trip.
fn roundtrip(msg: &AclMessage) -> AclMessage {
    let wire = msg.to_fipa_string();
    AclMessage::from_fipa_string(&wire)
        .unwrap_or_else(|e| panic!("decode failed for:\n{}\nerror: {}", wire, e))
}

#[test]
fn inform_with_multiple_receivers_and_json_content() {
    let mut msg = AclMessage::new(Performative::Inform)
        .sender("alice@example.com")
        .receiver("bob@example.com")
        .receiver("carol@example.com")
        .language("fipa-sl")
        .ontology("traffic");
    msg.set_content_json(&serde_json::json!({ "car": "car1", "status": "broken" }))
        .unwrap();

    let decoded = roundtrip(&msg);
    assert_eq!(msg, decoded);
    assert_eq!(decoded.receiver.len(), 2);
}

#[test]
fn every_performative_roundtrips() {
    for p in Performative::ALL {
        // Parsing the wire name yields the same variant.
        assert_eq!(Performative::from_str(p.as_str()), Ok(p));

        // A message using that performative survives a round-trip.
        let msg = AclMessage::new(p)
            .sender("agent-a@example.com")
            .receiver("agent-b@example.com")
            .content("hello");
        let decoded = roundtrip(&msg);
        assert_eq!(msg, decoded);
        assert_eq!(decoded.performative, p);
    }
}

#[test]
fn string_escaping_roundtrips() {
    let tricky = "a \"quoted\" value with a backslash \\ and a\nnewline\tand a tab";
    let msg = AclMessage::new(Performative::Inform)
        .sender("a@example.com")
        .receiver("b@example.com")
        .content(tricky);

    let decoded = roundtrip(&msg);
    assert_eq!(decoded.content.as_deref(), Some(tricky));
    assert_eq!(msg, decoded);
}

#[test]
fn nested_resolvers_survive_roundtrip() {
    let deep = AgentId::new("root-resolver@example.com")
        .with_address("http://example.com/root")
        .with_resolver(AgentId::new("leaf@example.com"));

    let aid = AgentId::new("jane@foo.com")
        .with_address("http://foo.com/acc")
        .with_resolver(deep);

    let msg = AclMessage::new(Performative::Request)
        .sender(aid)
        .receiver("jack@bar.com");

    let decoded = roundtrip(&msg);
    assert_eq!(msg, decoded);

    let sender = decoded.sender.expect("sender");
    assert_eq!(sender.resolvers.len(), 1);
    assert_eq!(sender.resolvers[0].name, "root-resolver@example.com");
    assert_eq!(sender.resolvers[0].resolvers.len(), 1);
    assert_eq!(sender.resolvers[0].resolvers[0].name, "leaf@example.com");
}

#[test]
fn decodes_the_spec_example() {
    let input = r#"(inform
    :sender (agent-identifier :name jane@foo.com :addresses (sequence http://foo.com/acc))
    :receiver (set (agent-identifier :name jack@bar.com))
    :language fipa-sl
    :ontology traffic
    :content "car1 is broken"
)"#;

    let msg = AclMessage::from_fipa_string(input).expect("should decode");
    assert_eq!(msg.performative, Performative::Inform);
    assert_eq!(msg.content.as_deref(), Some("car1 is broken"));
    assert_eq!(msg.language.as_deref(), Some("fipa-sl"));
    assert_eq!(msg.ontology.as_deref(), Some("traffic"));
    assert_eq!(msg.sender.expect("sender").name, "jane@foo.com");
    assert_eq!(msg.receiver.len(), 1);
    assert_eq!(msg.receiver[0].name, "jack@bar.com");
}

#[test]
fn unknown_performative_is_rejected() {
    let err = AclMessage::from_fipa_string("(nonsense :sender (agent-identifier :name a))")
        .unwrap_err();
    assert!(matches!(err, AclError::UnknownPerformative(_)));
}

#[test]
fn truncated_input_is_rejected() {
    let err =
        AclMessage::from_fipa_string("(inform :sender (agent-identifier :name a)").unwrap_err();
    assert!(
        matches!(err, AclError::UnexpectedEnd | AclError::UnexpectedToken { .. }),
        "unexpected error: {:?}",
        err
    );
}

#[test]
fn validate_reports_missing_parameters() {
    let msg = AclMessage::new(Performative::Inform);
    assert_eq!(msg.validate(), Err(AclError::MissingParameter("sender")));

    let msg = AclMessage::new(Performative::Inform).sender("a@example.com");
    assert_eq!(msg.validate(), Err(AclError::MissingParameter("receiver")));

    let msg = AclMessage::new(Performative::Inform)
        .sender("a@example.com")
        .receiver("b@example.com");
    assert!(msg.validate().is_ok());
}
