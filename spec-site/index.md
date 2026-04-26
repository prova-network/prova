---
layout: home

hero:
  name: Prova
  text: Protocol specification.
  tagline: Formal specs for verifiable storage on Base. Reliable specs are stable; draft specs are explicitly marked.
  image:
    src: /prova-mark.svg
    alt: Prova
  actions:
    - theme: brand
      text: Status overview
      link: /status
    - theme: alt
      text: Read on GitHub
      link: https://github.com/prova-network/prova/tree/main/spec

features:
  - title: 1. Introduction
    details: Spec home, status legend, document conventions. Start here if you're new.
    link: /status
    linkText: Status →
  - title: 2. Storage proofs
    details: How piece-CIDs, PDP, checkpoint anchoring, and data-availability sampling work in Prova.
    link: /pdp-integration
    linkText: PDP integration →
  - title: 3. Deal lifecycle
    details: Marketplace contract, deal state machine, on-chain event schema.
    link: /marketplace
    linkText: Marketplace →
  - title: 4. Network
    details: Prover-to-prover HTTPS conventions and the public REST API gateway.
    link: /network-protocol
    linkText: Network protocol →
  - title: 5. Token economics
    details: PROVA supply, allocation, prover emission, fee burn, governance parameters.
    link: /token-economics
    linkText: Token economics →
  - title: 6. Security
    details: Threat model, mitigations, and the pre-deployment audit checklist.
    link: /security-threat-model
    linkText: Threat model →
---
