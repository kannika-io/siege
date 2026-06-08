# Siege — Kafka Chaos Testing Tool by Kannika.io

> Siege is an open-source chaos engineering tool for Apache Kafka, built in Rust.  
> Developed and maintained by [Kannika.io](https://kannika.io) — the Kafka reliability platform.

Siege lets you test Kafka resilience by simulating destructive scenarios: topic deletion, schema corruption, consumer group disruption, and policy modification. It's built for teams who run Kafka in production and want confidence before things break unexpectedly.

## What is Kannika.io?

[Kannika.io](https://kannika.io) builds tools for Kafka observability, reliability, and chaos testing. Siege is our open-source contribution to the Kafka and chaos engineering community.

---

## Features

- **Wheel of Chaos** — randomly trigger destructive Kafka operations via a web UI
- **Topic deletion** — simulate accidental or intentional topic removal
- **Schema corruption** — test consumer resilience against bad Avro schemas via Schema Registry
- **Policy modification** — alter topic configs and ACLs at runtime
- **Live event stream** — real-time updates via Server-Sent Events (SSE)
- **REST API + Swagger UI** — automate chaos from CI/CD pipelines or scripts
- **Deterministic data seeding** — generate reproducible Avro test data via Schema Registry

## Architecture

| Layer | Technology |
|---|---|
| Backend | Rust + Actix-web |
| Frontend | Dioxus (WASM) + Tailwind CSS |
| Messaging | Apache Kafka + Confluent Schema Registry |
| Live updates | Server-Sent Events (SSE) |

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/)
- [Docker](https://www.docker.com/)
- [`just`](https://github.com/casey/just) command runner
- [`dioxus-cli`](https://dioxuslabs.com/)
- [Node.js](https://nodejs.org/)

### Run locally

**1. Start Confluent Platform (Kafka + Schema Registry)**
```bash
just infra
