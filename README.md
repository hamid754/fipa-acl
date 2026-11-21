# fipa-acl

A clean, standards-compliant **FIPA-ACL** message model for Rust — the 22 standard performatives, the full message envelope, and the standard string encoding.

[![crates.io](https://img.shields.io/crates/v/fipa-acl.svg)](https://crates.io/crates/fipa-acl)
[![docs.rs](https://docs.rs/fipa-acl/badge.svg)](https://docs.rs/fipa-acl)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

## What is this?

FIPA-ACL (Agent Communication Language) is the message language defined by the
Foundation for Intelligent Physical Agents (FIPA) and used by FIPA-compliant
multi-agent systems (systemes multi-agents, SMA). Every message is a
*communicative act* — an `inform`, a `request`, a `cfp`, a `propose`, and so on —
wrapped in a standard envelope carrying the sender, the receivers, the message
content, and the conversation-tracking metadata.

`fipa-acl` provides just that: a faithful, documented **message model** plus the
**standard string encoding** (the S-expression form). It is intentionally
framework-agnostic so it can be reused in any Rust multi-agent project.

> **Scope (level 1).** This crate is purely a message model. There is **no**
> network transport and **no** agent platform. Transports and mailboxes are
> planned (see the roadmap below).

## Features

- **The 22 standard performatives** — exactly the FIPA set, with idiomatic Rust
  names and the exact lower-case hyphenated wire names.
- **The full message envelope** — sender, receivers (multicast), reply-to,
  content, language, encoding, ontology, protocol, `conversation-id`,
  `reply-with`, `in-reply-to`, `reply-by`.
- **Agent identifiers (AIDs)** — with addresses and nested resolvers.
- **Standard string codec** — readable, whitespace-insensitive, round-trip safe,
  with a lenient reader that accepts the representations allowed by the spec.
- **`serde` support** for every type, plus JSON content helpers
  (`set_content_json` / `content_json`).
- **`#![forbid(unsafe_code)]`** and **minimal dependencies** (`serde`,
  `serde_json`).

## Installation

```toml
cargo add fipa-acl
```

## Quick start

```rust
use fipa_acl::{AclMessage, AgentId, Performative};

fn main() {
    // Build an inform message carrying structured JSON content.
    let mut msg = AclMessage::new(Performative::Inform)
        .sender(AgentId::new("alice@example.com"))
        .receiver(AgentId::new("bob@example.com"))
        .language("fipa-sl")
        .ontology("traffic");
    msg.set_content_json(&serde_json::json!({ "car": "car1", "status": "broken" }))
        .unwrap();

    // Encode to the standard FIPA string representation...
    let wire = msg.to_fipa_string();
    println!("{}", wire);

    // ...and decode it back.
    let decoded = AclMessage::from_fipa_string(&wire).unwrap();
    assert_eq!(msg, decoded);
}
```

### Contract Net in a nutshell

```rust
use fipa_acl::{AclMessage, AgentId, Performative};

let manager = AgentId::new("manager@example.com");
let bidder = AgentId::new("alice@example.com");

// Issue a call for proposals...
let cfp = AclMessage::new(Performative::Cfp)
    .sender(manager.clone())
    .receiver(bidder.clone())
    .protocol("fipa-contract-net")
    .conversation_id("cnp-1")
    .reply_with("cfp-1")
    .content("Deliver 10 widgets");

// ...and answer with a proposal.
let propose = AclMessage::new(Performative::Propose)
    .sender(bidder)
    .receiver(manager)
    .protocol("fipa-contract-net")
    .conversation_id("cnp-1")
    .in_reply_to("cfp-1")
    .content("10 widgets for 100 EUR");
```

A fully runnable version lives in
[`examples/contract_net.rs`](examples/contract_net.rs); run it with
`cargo run --example contract_net`.

## API overview

| Type / function | Purpose |
| --- | --- |
| `Performative` | The 22 standard communicative acts. `as_str()`, `FromStr`, `Display`, `ALL`. |
| `AgentId` | A FIPA agent identifier: name, addresses, resolvers. |
| `AclMessage` | The complete message envelope, with a fluent builder and `validate()`. |
| `encode` / `decode` | The FIPA string codec as free functions. |
| `AclMessage::to_fipa_string` / `from_fipa_string` | The same codec as methods. |
| `AclError` / `Result` | The crate error type and its result alias. |

## References

- FIPA Communicative Act Library Specification (performatives) — <https://www.fipa.org/specs/fipa00037/>
- FIPA ACL Message Structure Specification — <https://www.fipa.org/specs/fipa00061/>
- FIPA ACL Message Representation in String Specification — <https://www.fipa.org/specs/fipa00070/>
- FIPA ACL Message Representation in XML Specification — <https://www.fipa.org/specs/fipa00071/>

Reference implementations in other languages:

- **JADE** (Java) — <https://jade.tilab.com/>
- **SPADE** (Python) — <https://spade-mas.readthedocs.io/>

## License

Licensed under either of **Apache-2.0** or **MIT**, at your option.

- [LICENSE-APACHE](LICENSE-APACHE)
- [LICENSE-MIT](LICENSE-MIT)

## Status / Roadmap

- **L1 — done:** the message model (performatives, envelope, AIDs) and the
  standard string codec.
- **L2 — planned:** transports (email, HTTP, ...) and mailboxes.

## Contributing

Issues and pull requests are welcome.
