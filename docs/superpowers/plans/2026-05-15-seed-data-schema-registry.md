# Seed Data & Schema Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend seeding to register Avro schemas in a Confluent-compatible schema registry and produce deterministic fake Avro-encoded data for newly created topics.

**Architecture:** Three new crates (`siege-schema-registry`, `siege-seed-avsc`, and schemas infrastructure) plus modifications to `siege`, `siege-seed`, and `siege-api`. The schema registry is a first-class backend trait following the existing `KafkaBackend` pattern. A proc macro validates `.avsc` files at compile time. The `fake` crate generates deterministic data seeded by a configurable RNG.

**Tech Stack:** `apache-avro` (schema parsing + Avro binary encoding), `fake` (deterministic fake data), `reqwest` (schema registry HTTP client), `rand` (seeded RNG), `syn`/`quote`/`proc-macro2` (proc macro)

---

## File Map

### New files

| File | Responsibility |
|------|---------------|
| `compose.redpanda.yaml` | Standalone Redpanda + Redpanda Console docker compose |
| `crates/siege/src/schema_registry.rs` | `SchemaRegistryBackend` trait + `SchemaId` type |
| `crates/siege-schema-registry/Cargo.toml` | Schema registry client crate manifest |
| `crates/siege-schema-registry/src/lib.rs` | `SchemaRegistryClient` impl using reqwest |
| `crates/siege-seed-avsc/Cargo.toml` | Proc macro crate manifest |
| `crates/siege-seed-avsc/src/lib.rs` | `avsc!` proc macro |
| `crates/siege-seed/src/avro.rs` | `AvroSerializer` — Confluent wire format encoding |
| `crates/siege-seed/src/faker.rs` | Schema-driven fake data generation |
| `schemas/kings-landing.avsc` | Avro schema for kings-landing topic |
| `schemas/winterfell.avsc` | Avro schema for winterfell topic |
| `schemas/the-wall.avsc` | Avro schema for the-wall topic |
| `schemas/iron-islands.avsc` | Avro schema for iron-islands topic |
| `schemas/dragonstone.avsc` | Avro schema for dragonstone topic |
| `schemas/the-citadel.avsc` | Avro schema for the-citadel topic |

### Modified files

| File | Change |
|------|--------|
| `Cargo.toml` | Add workspace members + dependencies (`apache-avro`, `fake`, `siege-schema-registry`, `siege-seed-avsc`) |
| `Justfile` | Add `rp`/`redpanda` task |
| `crates/siege/src/lib.rs` | Add `pub mod schema_registry;` and re-exports |
| `crates/siege/src/context.rs` | Add `SchemaRegistry` associated type + accessor |
| `crates/siege/src/error.rs` | Add `SchemaRegistry` variant |
| `crates/siege/src/mock.rs` | Add `NoopSchemaRegistry` |
| `crates/siege/src/client/mod.rs` | Update `TestCtx` to include `NoopSchemaRegistry` |
| `crates/siege/Cargo.toml` | No changes needed (trait is generic) |
| `crates/siege-seed/Cargo.toml` | Add deps: `apache-avro`, `fake`, `rand`, `serde_json`, `futures`, re-export `siege-seed-avsc` |
| `crates/siege-seed/src/lib.rs` | Extend `TopicSeed` + `Seeder` with schema/records/rng_seed, new seed flow |
| `crates/siege-api/Cargo.toml` | Add dep: `siege-schema-registry` |
| `crates/siege-api/src/main.rs` | Add `--schema-registry-url` CLI flag, wire `SchemaRegistryClient` into context + seeder |

---

### Task 1: Infrastructure — compose.redpanda.yaml + Justfile

**Files:**
- Create: `compose.redpanda.yaml`
- Modify: `Justfile`

- [ ] **Step 1: Create compose.redpanda.yaml**

```yaml
services:
  redpanda:
    image: docker.redpanda.com/redpandadata/redpanda:v24.3.1
    command:
      - redpanda
      - start
      - --kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:19092
      - --advertise-kafka-addr internal://redpanda:9092,external://localhost:19092
      - --schema-registry-addr internal://0.0.0.0:8081,external://0.0.0.0:18081
      - --advertise-schema-registry-addr internal://redpanda:8081,external://localhost:18081
      - --pandaproxy-addr internal://0.0.0.0:8082,external://0.0.0.0:18082
      - --advertise-pandaproxy-addr internal://redpanda:8082,external://localhost:18082
      - --advertise-rpc-addr redpanda:33145
      - --smp 1
      - --memory 512M
      - --mode dev-container
    ports:
      - "19092:19092"
      - "18081:18081"
      - "18082:18082"
    healthcheck:
      test: ["CMD", "rpk", "cluster", "info", "--brokers", "localhost:9092"]
      interval: 5s
      timeout: 10s
      retries: 10

  console:
    image: docker.redpanda.com/redpandadata/console:v2.8.0
    ports:
      - "9080:8080"
    environment:
      CONFIG_FILEPATH: ""
      KAFKA_BROKERS: redpanda:9092
      KAFKA_SCHEMAREGISTRY_ENABLED: "true"
      KAFKA_SCHEMAREGISTRY_URLS: http://redpanda:8081
      REDPANDA_ADMINAPI_ENABLED: "true"
      REDPANDA_ADMINAPI_URLS: http://redpanda:9644
    depends_on:
      redpanda:
        condition: service_healthy
```

- [ ] **Step 2: Add rp task to Justfile**

Add after the existing `kafka` task (line 13 of current Justfile):

```just
# Start Redpanda in Docker
[alias('redpanda')]
rp:
    docker compose -f compose.redpanda.yaml up -d
    @echo "Waiting for Redpanda to be healthy..."
    @until docker compose -f compose.redpanda.yaml exec redpanda rpk cluster info --brokers localhost:9092 > /dev/null 2>&1; do sleep 1; done
    @echo "Redpanda ready (Kafka: localhost:19092, Schema Registry: localhost:18081, Console: localhost:9080)"
```

- [ ] **Step 3: Verify Redpanda starts**

Run: `just rp`
Expected: Redpanda and Console containers start, healthcheck passes, "Redpanda ready" printed.

- [ ] **Step 4: Verify Redpanda Console accessible**

Open http://localhost:9080 in browser. Expected: Redpanda Console UI loads.

- [ ] **Step 5: Stop and commit**

```bash
docker compose -f compose.redpanda.yaml down
git add compose.redpanda.yaml Justfile
git commit -m "feat: add Redpanda compose and Justfile task"
```

---

### Task 2: SchemaRegistryBackend trait in siege crate

**Files:**
- Create: `crates/siege/src/schema_registry.rs`
- Modify: `crates/siege/src/lib.rs`
- Modify: `crates/siege/src/error.rs`

- [ ] **Step 1: Add SchemaRegistry error variant**

In `crates/siege/src/error.rs`, add a new variant to the `SiegeError` enum after the `Seed` variant (line 16):

```rust
#[error("schema registry error: {0}")]
SchemaRegistry(String),
```

- [ ] **Step 2: Create the schema_registry module**

Create `crates/siege/src/schema_registry.rs`:

```rust
use crate::SiegeError;
use crate::kafka::BoxFuture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaId(pub i32);

pub trait SchemaRegistryBackend: Send + Sync + 'static {
    fn register_schema(
        &self,
        subject: &str,
        schema: &str,
    ) -> BoxFuture<'_, Result<SchemaId, SiegeError>>;

    fn delete_subject(
        &self,
        subject: &str,
    ) -> BoxFuture<'_, Result<(), SiegeError>>;
}
```

- [ ] **Step 3: Wire into lib.rs**

In `crates/siege/src/lib.rs`, add the module declaration and re-exports. Add after `pub mod seed;` (line 6):

```rust
pub mod schema_registry;
```

Add to the re-export section (after `pub use seed::{SeedBackend, SeedResult};` on line 17):

```rust
pub use schema_registry::{SchemaId, SchemaRegistryBackend};
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p siege`
Expected: compiles clean

- [ ] **Step 5: Commit**

```bash
git add crates/siege/src/schema_registry.rs crates/siege/src/lib.rs crates/siege/src/error.rs
git commit -m "feat: add SchemaRegistryBackend trait and SchemaId type"
```

---

### Task 3: Add NoopSchemaRegistry mock + update SiegeContext

**Files:**
- Modify: `crates/siege/src/context.rs`
- Modify: `crates/siege/src/mock.rs`
- Modify: `crates/siege/src/lib.rs`
- Modify: `crates/siege/src/client/mod.rs`

- [ ] **Step 1: Update SiegeContext trait**

Replace the entire contents of `crates/siege/src/context.rs` with:

```rust
use crate::EventEmitter;
use crate::chaos::ChaosBackend;
use crate::kafka::KafkaBackend;
use crate::schema_registry::SchemaRegistryBackend;
use crate::seed::SeedBackend;

pub trait SiegeContext: Send + Sync + 'static {
    type Kafka: KafkaBackend;
    type Events: EventEmitter;
    type Chaos: ChaosBackend;
    type Seeder: SeedBackend;
    type SchemaRegistry: SchemaRegistryBackend;

    fn kafka(&self) -> &Self::Kafka;
    fn events(&self) -> &Self::Events;
    fn chaos(&self) -> &Self::Chaos;
    fn seeder(&self) -> &Self::Seeder;
    fn schema_registry(&self) -> &Self::SchemaRegistry;
}
```

- [ ] **Step 2: Add NoopSchemaRegistry to mock.rs**

Add the following after the `NoopSeeder` impl (after line 33) in `crates/siege/src/mock.rs`:

```rust
use crate::kafka::BoxFuture;
use crate::schema_registry::{SchemaId, SchemaRegistryBackend};
```

Add these imports to the existing import block at the top of mock.rs. Then add after `NoopSeeder`:

```rust
pub struct NoopSchemaRegistry;

impl SchemaRegistryBackend for NoopSchemaRegistry {
    fn register_schema(
        &self,
        _subject: &str,
        _schema: &str,
    ) -> BoxFuture<'_, Result<SchemaId, SiegeError>> {
        Box::pin(async { Ok(SchemaId(1)) })
    }

    fn delete_subject(
        &self,
        _subject: &str,
    ) -> BoxFuture<'_, Result<(), SiegeError>> {
        Box::pin(async { Ok(()) })
    }
}
```

- [ ] **Step 3: Re-export NoopSchemaRegistry from lib.rs**

In `crates/siege/src/lib.rs`, update the mock re-export line (line 16) from:

```rust
pub use mock::{MockKafkaBackend, NoopChaos, NoopSeeder};
```

to:

```rust
pub use mock::{MockKafkaBackend, NoopChaos, NoopSchemaRegistry, NoopSeeder};
```

- [ ] **Step 4: Update TestCtx in client/mod.rs**

In `crates/siege/src/client/mod.rs`, update the test module. Add `NoopSchemaRegistry` to the import on line 38:

```rust
use crate::{
    EventEmitter, KafkaProperties, MockKafkaBackend, NoopChaos, NoopSchemaRegistry, NoopSeeder,
    SiegeContext, SiegeError,
};
```

Update the `TestCtx` struct (around line 68) to add the field:

```rust
struct TestCtx {
    kafka: MockKafkaBackend,
    events: RecordingEmitter,
    chaos: NoopChaos,
    seeder: NoopSeeder,
    schema_registry: NoopSchemaRegistry,
}
```

Update the `SiegeContext` impl for `TestCtx` (around line 75) to add the associated type and accessor:

```rust
impl SiegeContext for TestCtx {
    type Kafka = MockKafkaBackend;
    type Events = RecordingEmitter;
    type Chaos = NoopChaos;
    type Seeder = NoopSeeder;
    type SchemaRegistry = NoopSchemaRegistry;

    fn kafka(&self) -> &MockKafkaBackend { &self.kafka }
    fn events(&self) -> &RecordingEmitter { &self.events }
    fn chaos(&self) -> &NoopChaos { &self.chaos }
    fn seeder(&self) -> &NoopSeeder { &self.seeder }
    fn schema_registry(&self) -> &NoopSchemaRegistry { &self.schema_registry }
}
```

Update `test_client` (around line 87) to include the new field:

```rust
fn test_client(topics: Vec<TopicDetail>) -> Client<TestCtx> {
    Client::new(TestCtx {
        kafka: MockKafkaBackend::with_topics(topics),
        events: RecordingEmitter::default(),
        chaos: NoopChaos,
        seeder: NoopSeeder,
        schema_registry: NoopSchemaRegistry,
    })
}
```

- [ ] **Step 5: Verify tests pass**

Run: `cargo test -p siege`
Expected: all existing tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/siege/src/context.rs crates/siege/src/mock.rs crates/siege/src/lib.rs crates/siege/src/client/mod.rs
git commit -m "feat: add SchemaRegistry to SiegeContext with NoopSchemaRegistry mock"
```

---

### Task 4: siege-schema-registry crate

**Files:**
- Create: `crates/siege-schema-registry/Cargo.toml`
- Create: `crates/siege-schema-registry/src/lib.rs`
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Write the failing test**

Create `crates/siege-schema-registry/src/lib.rs`:

```rust
use siege::kafka::BoxFuture;
use siege::schema_registry::{SchemaId, SchemaRegistryBackend};
use siege::SiegeError;

pub struct SchemaRegistryClient {
    base_url: String,
    client: reqwest::Client,
}

impl SchemaRegistryClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            client: reqwest::Client::new(),
        }
    }
}

impl SchemaRegistryBackend for SchemaRegistryClient {
    fn register_schema(
        &self,
        _subject: &str,
        _schema: &str,
    ) -> BoxFuture<'_, Result<SchemaId, SiegeError>> {
        todo!()
    }

    fn delete_subject(
        &self,
        _subject: &str,
    ) -> BoxFuture<'_, Result<(), SiegeError>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_schema_sends_post() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/subjects/test-topic-value/versions")
            .match_header("content-type", "application/vnd.schemaregistry.v1+json")
            .match_body(mockito::Matcher::JsonString(
                r#"{"schemaType":"AVRO","schema":"{\"type\":\"record\",\"name\":\"Test\",\"fields\":[]}"}"#.into(),
            ))
            .with_status(200)
            .with_body(r#"{"id":42}"#)
            .create_async()
            .await;

        let client = SchemaRegistryClient::new(&server.url());
        let schema = r#"{"type":"record","name":"Test","fields":[]}"#;
        let result = client.register_schema("test-topic-value", schema).await;

        let id = result.expect("register_schema should succeed");
        assert_eq!(id, SchemaId(42));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn delete_subject_sends_delete() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/subjects/test-topic-value?permanent=true")
            .with_status(200)
            .with_body("[1]")
            .create_async()
            .await;

        let client = SchemaRegistryClient::new(&server.url());
        let result = client.delete_subject("test-topic-value").await;

        result.expect("delete_subject should succeed");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn register_schema_maps_http_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/subjects/bad-value/versions")
            .with_status(500)
            .with_body(r#"{"error_code":50001,"message":"Error in the backend"}"#)
            .create_async()
            .await;

        let client = SchemaRegistryClient::new(&server.url());
        let result = client.register_schema("bad-value", "{}").await;

        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Create Cargo.toml**

Create `crates/siege-schema-registry/Cargo.toml`:

```toml
[package]
name = "siege-schema-registry"
version.workspace = true
edition.workspace = true

[dependencies]
siege.workspace = true
reqwest.workspace = true
serde.workspace = true
serde_json.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["test-util", "macros"] }
mockito = "1"
```

- [ ] **Step 3: Update workspace Cargo.toml**

In the root `Cargo.toml`, add `"crates/siege-schema-registry"` to the `members` array (after `"crates/siege-seed"`).

Add to `[workspace.dependencies]`:

```toml
siege-schema-registry = { path = "crates/siege-schema-registry" }
```

Also add new workspace dependencies:

```toml
apache-avro = "0.17"
fake = { version = "5", features = ["derive"] }
```

- [ ] **Step 4: Run tests to verify they fail**

Run: `cargo test -p siege-schema-registry`
Expected: FAIL — the `todo!()` stubs panic

- [ ] **Step 5: Implement register_schema**

Replace the `register_schema` method in the `SchemaRegistryBackend` impl:

```rust
fn register_schema(
    &self,
    subject: &str,
    schema: &str,
) -> BoxFuture<'_, Result<SchemaId, SiegeError>> {
    let url = format!("{}/subjects/{}/versions", self.base_url, subject);
    let body = serde_json::json!({
        "schemaType": "AVRO",
        "schema": schema,
    });
    Box::pin(async move {
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/vnd.schemaregistry.v1+json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SiegeError::SchemaRegistry(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SiegeError::SchemaRegistry(
                format!("HTTP {status}: {body}"),
            ));
        }

        let parsed: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SiegeError::SchemaRegistry(e.to_string()))?;

        let id = parsed["id"]
            .as_i64()
            .ok_or_else(|| SiegeError::SchemaRegistry("missing 'id' in response".into()))?;

        Ok(SchemaId(id as i32))
    })
}
```

- [ ] **Step 6: Implement delete_subject**

Replace the `delete_subject` method:

```rust
fn delete_subject(
    &self,
    subject: &str,
) -> BoxFuture<'_, Result<(), SiegeError>> {
    let url = format!(
        "{}/subjects/{}?permanent=true",
        self.base_url, subject
    );
    Box::pin(async move {
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| SiegeError::SchemaRegistry(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SiegeError::SchemaRegistry(
                format!("HTTP {status}: {body}"),
            ));
        }

        Ok(())
    })
}
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p siege-schema-registry`
Expected: all 3 tests pass

- [ ] **Step 8: Commit**

```bash
git add crates/siege-schema-registry/ Cargo.toml
git commit -m "feat: add siege-schema-registry crate with Confluent REST API client"
```

---

### Task 5: siege-seed-avsc proc macro crate

**Files:**
- Create: `crates/siege-seed-avsc/Cargo.toml`
- Create: `crates/siege-seed-avsc/src/lib.rs`
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Create Cargo.toml**

Create `crates/siege-seed-avsc/Cargo.toml`:

```toml
[package]
name = "siege-seed-avsc"
version.workspace = true
edition.workspace = true

[lib]
proc-macro = true

[dependencies]
apache-avro.workspace = true
proc-macro2 = "1"
quote = "1"
syn = "2"
```

- [ ] **Step 2: Implement the proc macro**

Create `crates/siege-seed-avsc/src/lib.rs`:

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

/// Validates an Avro schema file at compile time and embeds it as a `&'static str`.
///
/// Usage: `avsc!("schemas/my-topic.avsc")`
#[proc_macro]
pub fn avsc(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let relative_path = lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let full_path = std::path::Path::new(&manifest_dir).join(&relative_path);

    let content = match std::fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            return syn::Error::new(lit.span(), format!("failed to read {}: {e}", full_path.display()))
                .to_compile_error()
                .into();
        }
    };

    if let Err(e) = apache_avro::Schema::parse_str(&content) {
        return syn::Error::new(lit.span(), format!("invalid Avro schema in {}: {e}", full_path.display()))
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        {
            const _VALIDATED: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #relative_path));
            _VALIDATED
        }
    };

    expanded.into()
}
```

- [ ] **Step 3: Update workspace Cargo.toml**

Add `"crates/siege-seed-avsc"` to `members` (after `"crates/siege-schema-registry"`).

Add to `[workspace.dependencies]`:

```toml
siege-seed-avsc = { path = "crates/siege-seed-avsc" }
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p siege-seed-avsc`
Expected: compiles clean

- [ ] **Step 5: Commit**

```bash
git add crates/siege-seed-avsc/ Cargo.toml
git commit -m "feat: add siege-seed-avsc proc macro for compile-time Avro schema validation"
```

---

### Task 6: Avro schema files

**Files:**
- Create: `schemas/kings-landing.avsc`
- Create: `schemas/winterfell.avsc`
- Create: `schemas/the-wall.avsc`
- Create: `schemas/iron-islands.avsc`
- Create: `schemas/dragonstone.avsc`
- Create: `schemas/the-citadel.avsc`

- [ ] **Step 1: Create kings-landing.avsc**

Create `schemas/kings-landing.avsc`:

```json
{
  "type": "record",
  "name": "KingsLandingEvent",
  "namespace": "io.siege.schemas",
  "fields": [
    {"name": "name", "type": "string"},
    {"name": "title", "type": "string"},
    {"name": "house", "type": "string"},
    {"name": "alive", "type": "boolean"},
    {"name": "age", "type": "int"}
  ]
}
```

- [ ] **Step 2: Create winterfell.avsc**

Create `schemas/winterfell.avsc`:

```json
{
  "type": "record",
  "name": "WinterfellEvent",
  "namespace": "io.siege.schemas",
  "fields": [
    {"name": "name", "type": "string"},
    {"name": "role", "type": "string"},
    {"name": "location", "type": "string"},
    {"name": "loyalty_score", "type": "double"},
    {"name": "is_stark", "type": "boolean"}
  ]
}
```

- [ ] **Step 3: Create the-wall.avsc**

Create `schemas/the-wall.avsc`:

```json
{
  "type": "record",
  "name": "TheWallEvent",
  "namespace": "io.siege.schemas",
  "fields": [
    {"name": "ranger_name", "type": "string"},
    {"name": "section", "type": "int"},
    {"name": "temperature", "type": "double"},
    {"name": "threat_detected", "type": "boolean"}
  ]
}
```

- [ ] **Step 4: Create iron-islands.avsc**

Create `schemas/iron-islands.avsc`:

```json
{
  "type": "record",
  "name": "IronIslandsEvent",
  "namespace": "io.siege.schemas",
  "fields": [
    {"name": "captain", "type": "string"},
    {"name": "ship_name", "type": "string"},
    {"name": "crew_size", "type": "int"},
    {"name": "destination", "type": "string"},
    {"name": "is_raiding", "type": "boolean"}
  ]
}
```

- [ ] **Step 5: Create dragonstone.avsc**

Create `schemas/dragonstone.avsc`:

```json
{
  "type": "record",
  "name": "DragonstoneEvent",
  "namespace": "io.siege.schemas",
  "fields": [
    {"name": "advisor", "type": "string"},
    {"name": "counsel", "type": "string"},
    {"name": "urgency", "type": "int"},
    {"name": "approved", "type": "boolean"}
  ]
}
```

- [ ] **Step 6: Create the-citadel.avsc**

Create `schemas/the-citadel.avsc`:

```json
{
  "type": "record",
  "name": "TheCitadelEvent",
  "namespace": "io.siege.schemas",
  "fields": [
    {"name": "maester", "type": "string"},
    {"name": "subject", "type": "string"},
    {"name": "scroll_id", "type": "long"},
    {"name": "is_restricted", "type": "boolean"}
  ]
}
```

- [ ] **Step 7: Commit**

```bash
git add schemas/
git commit -m "feat: add Avro schema files for seed topics"
```

---

### Task 7: AvroSerializer in siege-seed

**Files:**
- Create: `crates/siege-seed/src/avro.rs`
- Modify: `crates/siege-seed/Cargo.toml`

- [ ] **Step 1: Write failing test**

Create `crates/siege-seed/src/avro.rs`:

```rust
use apache_avro::types::Value;
use apache_avro::Schema;
use siege::schema_registry::SchemaId;
use siege::SiegeError;

pub struct AvroSerializer {
    schema: Schema,
    header: [u8; 5],
}

impl AvroSerializer {
    pub fn new(schema: Schema, schema_id: SchemaId) -> Self {
        let mut header = [0u8; 5];
        header[0] = 0x00; // magic byte
        header[1..5].copy_from_slice(&schema_id.0.to_be_bytes());
        Self { schema, header }
    }

    pub fn serialize(&self, value: Value) -> Result<Vec<u8>, SiegeError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema() -> Schema {
        Schema::parse_str(r#"{
            "type": "record",
            "name": "Test",
            "fields": [
                {"name": "name", "type": "string"},
                {"name": "age", "type": "int"}
            ]
        }"#).unwrap()
    }

    #[test]
    fn serialize_prepends_confluent_header() {
        let schema = test_schema();
        let serializer = AvroSerializer::new(schema, SchemaId(42));

        let value = Value::Record(vec![
            ("name".into(), Value::String("test".into())),
            ("age".into(), Value::Int(25)),
        ]);

        let bytes = serializer.serialize(value).unwrap();

        assert_eq!(bytes[0], 0x00, "magic byte");
        assert_eq!(&bytes[1..5], &42_i32.to_be_bytes(), "schema id");
        assert!(bytes.len() > 5, "should have avro payload after header");
    }

    #[test]
    fn serialize_produces_valid_avro() {
        let schema = test_schema();
        let serializer = AvroSerializer::new(schema.clone(), SchemaId(1));

        let value = Value::Record(vec![
            ("name".into(), Value::String("alice".into())),
            ("age".into(), Value::Int(30)),
        ]);

        let bytes = serializer.serialize(value).unwrap();
        let avro_payload = &bytes[5..];

        let reader = apache_avro::Reader::with_schema(&schema, avro_payload).unwrap();
        let records: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        match &records[0] {
            Value::Record(fields) => {
                assert_eq!(fields[0].1, Value::String("alice".into()));
                assert_eq!(fields[1].1, Value::Int(30));
            }
            _ => panic!("expected record"),
        }
    }
}
```

- [ ] **Step 2: Update Cargo.toml**

Replace `crates/siege-seed/Cargo.toml` with:

```toml
[package]
name = "siege-seed"
version.workspace = true
edition.workspace = true

[dependencies]
siege.workspace = true
siege-seed-avsc.workspace = true
apache-avro.workspace = true
fake.workspace = true
rand.workspace = true
serde_json.workspace = true
futures.workspace = true
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p siege-seed`
Expected: FAIL — `todo!()` panics

- [ ] **Step 4: Implement serialize**

Replace the `serialize` method in `crates/siege-seed/src/avro.rs`:

```rust
pub fn serialize(&self, value: Value) -> Result<Vec<u8>, SiegeError> {
    let mut buf = Vec::with_capacity(128);
    buf.extend_from_slice(&self.header);

    let resolved = value
        .resolve(&self.schema)
        .map_err(|e| SiegeError::Seed(format!("avro resolve: {e}")))?;

    apache_avro::to_avro_datum(&self.schema, resolved)
        .map(|datum| {
            buf.extend_from_slice(&datum);
            buf
        })
        .map_err(|e| SiegeError::Seed(format!("avro encode: {e}")))
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p siege-seed`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/siege-seed/src/avro.rs crates/siege-seed/Cargo.toml
git commit -m "feat: add AvroSerializer with Confluent wire format encoding"
```

---

### Task 8: Fake data generator in siege-seed

**Files:**
- Create: `crates/siege-seed/src/faker.rs`

- [ ] **Step 1: Write failing test**

Create `crates/siege-seed/src/faker.rs`:

```rust
use apache_avro::Schema;
use apache_avro::types::Value;
use fake::Fake;
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::Name;
use rand::SeedableRng;
use rand::rngs::StdRng;

use siege::SiegeError;

pub fn generate_record(schema: &Schema, rng: &mut StdRng) -> Result<Value, SiegeError> {
    generate_value(schema, None, rng)
}

fn generate_value(
    schema: &Schema,
    field_name: Option<&str>,
    rng: &mut StdRng,
) -> Result<Value, SiegeError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema() -> Schema {
        Schema::parse_str(r#"{
            "type": "record",
            "name": "Test",
            "fields": [
                {"name": "name", "type": "string"},
                {"name": "age", "type": "int"},
                {"name": "active", "type": "boolean"},
                {"name": "score", "type": "double"}
            ]
        }"#).unwrap()
    }

    #[test]
    fn generates_record_with_all_fields() {
        let schema = test_schema();
        let mut rng = StdRng::seed_from_u64(42);
        let value = generate_record(&schema, &mut rng).unwrap();

        match value {
            Value::Record(fields) => {
                assert_eq!(fields.len(), 4);
                assert!(matches!(fields[0].1, Value::String(_)));
                assert!(matches!(fields[1].1, Value::Int(_)));
                assert!(matches!(fields[2].1, Value::Boolean(_)));
                assert!(matches!(fields[3].1, Value::Double(_)));
            }
            _ => panic!("expected record"),
        }
    }

    #[test]
    fn deterministic_with_same_seed() {
        let schema = test_schema();
        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);

        let v1 = generate_record(&schema, &mut rng1).unwrap();
        let v2 = generate_record(&schema, &mut rng2).unwrap();

        assert_eq!(format!("{v1:?}"), format!("{v2:?}"));
    }

    #[test]
    fn different_seeds_produce_different_data() {
        let schema = test_schema();
        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(99);

        let v1 = generate_record(&schema, &mut rng1).unwrap();
        let v2 = generate_record(&schema, &mut rng2).unwrap();

        assert_ne!(format!("{v1:?}"), format!("{v2:?}"));
    }

    #[test]
    fn name_field_gets_realistic_name() {
        let schema = Schema::parse_str(r#"{
            "type": "record",
            "name": "Test",
            "fields": [{"name": "name", "type": "string"}]
        }"#).unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        let value = generate_record(&schema, &mut rng).unwrap();

        match value {
            Value::Record(fields) => {
                match &fields[0].1 {
                    Value::String(s) => {
                        assert!(s.contains(' '), "name field should contain a space (first + last name), got: {s}");
                    }
                    _ => panic!("expected string"),
                }
            }
            _ => panic!("expected record"),
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p siege-seed -- faker`
Expected: FAIL — `todo!()` panics

- [ ] **Step 3: Implement generate_value**

Replace the `generate_value` function:

```rust
fn generate_value(
    schema: &Schema,
    field_name: Option<&str>,
    rng: &mut StdRng,
) -> Result<Value, SiegeError> {
    use fake::faker::address::en::StreetAddress;
    use fake::faker::internet::en::SafeEmail;
    use rand::Rng;

    match schema {
        Schema::String => {
            let s: String = match field_name {
                Some("name" | "ranger_name" | "captain" | "advisor" | "maester") => {
                    Name().fake_with_rng(rng)
                }
                Some("email") => SafeEmail().fake_with_rng(rng),
                Some("address" | "location" | "destination") => {
                    StreetAddress().fake_with_rng(rng)
                }
                _ => Sentence(3..6).fake_with_rng(rng),
            };
            Ok(Value::String(s))
        }
        Schema::Int => Ok(Value::Int(rng.random_range(1..1000))),
        Schema::Long => Ok(Value::Long(rng.random_range(1..100_000))),
        Schema::Float => Ok(Value::Float(rng.random_range(0.0..100.0))),
        Schema::Double => Ok(Value::Double(rng.random_range(0.0..100.0))),
        Schema::Boolean => Ok(Value::Boolean(rng.random_bool(0.5))),
        Schema::Record(record_schema) => {
            let fields: Result<Vec<(String, Value)>, SiegeError> = record_schema
                .fields
                .iter()
                .map(|field| {
                    let val = generate_value(&field.schema, Some(&field.name), rng)?;
                    Ok((field.name.clone(), val))
                })
                .collect();
            Ok(Value::Record(fields?))
        }
        Schema::Enum(enum_schema) => {
            let idx = rng.random_range(0..enum_schema.symbols.len() as u32);
            Ok(Value::Enum(idx, enum_schema.symbols[idx as usize].clone()))
        }
        Schema::Array(inner) => {
            let len = rng.random_range(1..5);
            let items: Result<Vec<Value>, SiegeError> = (0..len)
                .map(|_| generate_value(&inner.items, None, rng))
                .collect();
            Ok(Value::Array(items?))
        }
        Schema::Union(union_schema) => {
            let variants: Vec<_> = union_schema
                .variants()
                .iter()
                .filter(|v| !matches!(v, Schema::Null))
                .collect();
            if variants.is_empty() {
                Ok(Value::Null)
            } else {
                let idx = rng.random_range(0..variants.len());
                generate_value(variants[idx], field_name, rng)
            }
        }
        Schema::Null => Ok(Value::Null),
        Schema::Bytes => Ok(Value::Bytes(vec![rng.random(), rng.random(), rng.random()])),
        _ => Err(SiegeError::Seed(format!("unsupported schema type: {schema:?}"))),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p siege-seed -- faker`
Expected: all 4 tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/siege-seed/src/faker.rs
git commit -m "feat: add schema-driven fake data generator with deterministic RNG"
```

---

### Task 9: Extend TopicSeed and Seeder with schema + records + data production

**Files:**
- Modify: `crates/siege-seed/src/lib.rs`

- [ ] **Step 1: Write failing test**

Add tests to the end of `crates/siege-seed/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use siege::{MockKafkaBackend, NoopSchemaRegistry};

    #[tokio::test]
    async fn seed_creates_topics_and_reports_result() {
        let kafka = MockKafkaBackend::new();
        let seeder = Seeder::new(kafka)
            .topic(TopicSeed::new("topic-a", 3))
            .topic(TopicSeed::new("topic-b", 1));

        let result = seeder.seed_topics().await.unwrap();
        assert_eq!(result.created, vec!["topic-a", "topic-b"]);
        assert!(result.skipped.is_empty());
    }

    #[tokio::test]
    async fn seed_skips_existing_topics() {
        let kafka = MockKafkaBackend::with_topics(vec![
            siege::kafka::TopicDetail {
                name: "existing".into(),
                partitions: 3,
                replication_factor: 1,
                config: KafkaProperties::new(),
            },
        ]);
        let seeder = Seeder::new(kafka)
            .topic(TopicSeed::new("existing", 3))
            .topic(TopicSeed::new("new-one", 1));

        let result = seeder.seed_topics().await.unwrap();
        assert_eq!(result.created, vec!["new-one"]);
        assert_eq!(result.skipped, vec!["existing"]);
    }

    #[tokio::test]
    async fn seed_with_schema_and_records_produces_data() {
        let kafka = MockKafkaBackend::new();
        let schema_registry = NoopSchemaRegistry;
        let schema_str = r#"{"type":"record","name":"Test","namespace":"io.siege.schemas","fields":[{"name":"name","type":"string"}]}"#;

        let seeder = Seeder::new(kafka)
            .schema_registry(schema_registry)
            .topic(
                TopicSeed::new("test-topic", 1)
                    .schema(schema_str)
                    .records(10),
            );

        let result = seeder.seed_topics().await.unwrap();
        assert_eq!(result.created, vec!["test-topic"]);
    }

    #[tokio::test]
    async fn seed_deterministic_with_same_rng_seed() {
        let schema_str = r#"{"type":"record","name":"Test","namespace":"io.siege.schemas","fields":[{"name":"name","type":"string"},{"name":"age","type":"int"}]}"#;

        let kafka1 = MockKafkaBackend::new();
        let sr1 = NoopSchemaRegistry;
        let seeder1 = Seeder::new(kafka1)
            .schema_registry(sr1)
            .rng_seed(123)
            .topic(TopicSeed::new("t", 1).schema(schema_str).records(5));

        let kafka2 = MockKafkaBackend::new();
        let sr2 = NoopSchemaRegistry;
        let seeder2 = Seeder::new(kafka2)
            .schema_registry(sr2)
            .rng_seed(123)
            .topic(TopicSeed::new("t", 1).schema(schema_str).records(5));

        let r1 = seeder1.seed_topics().await.unwrap();
        let r2 = seeder2.seed_topics().await.unwrap();
        assert_eq!(r1.created, r2.created);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p siege-seed`
Expected: FAIL — `Seeder::new` signature doesn't accept schema_registry yet, no `.schema()` or `.records()` methods.

- [ ] **Step 3: Rewrite lib.rs with extended TopicSeed and Seeder**

Replace the entire contents of `crates/siege-seed/src/lib.rs`:

```rust
mod avro;
mod faker;

pub use siege_seed_avsc::avsc;

use apache_avro::Schema;
use rand::SeedableRng;
use rand::rngs::StdRng;
use siege::kafka::KafkaBackend;
use siege::schema_registry::{SchemaId, SchemaRegistryBackend};
use siege::{KafkaProperties, SeedBackend, SeedResult, SiegeError};

use avro::AvroSerializer;
use faker::generate_record;

pub struct TopicSeed {
    name: String,
    partitions: i32,
    replication_factor: i32,
    config: KafkaProperties,
    schema: Option<&'static str>,
    record_count: Option<u32>,
}

impl TopicSeed {
    pub fn new(name: &str, partitions: i32) -> Self {
        Self {
            name: name.to_owned(),
            partitions,
            replication_factor: 1,
            config: KafkaProperties::new(),
            schema: None,
            record_count: None,
        }
    }

    pub fn config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }

    pub fn schema(mut self, schema: &'static str) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn records(mut self, count: u32) -> Self {
        self.record_count = Some(count);
        self
    }
}

pub struct Seeder {
    kafka: Box<dyn KafkaBackend>,
    schema_registry: Option<Box<dyn SchemaRegistryBackend>>,
    seeds: Vec<TopicSeed>,
    rng_seed: u64,
}

impl Seeder {
    pub fn new(kafka: impl KafkaBackend) -> Self {
        Self {
            kafka: Box::new(kafka),
            schema_registry: None,
            seeds: Vec::new(),
            rng_seed: 42,
        }
    }

    pub fn schema_registry(mut self, sr: impl SchemaRegistryBackend) -> Self {
        self.schema_registry = Some(Box::new(sr));
        self
    }

    pub fn topic(mut self, seed: TopicSeed) -> Self {
        self.seeds.push(seed);
        self
    }

    pub fn rng_seed(mut self, seed: u64) -> Self {
        self.rng_seed = seed;
        self
    }

    async fn seed_data(
        &self,
        topic: &str,
        schema_str: &str,
        count: u32,
        rng: &mut StdRng,
    ) -> Result<(), SiegeError> {
        let sr = self
            .schema_registry
            .as_ref()
            .ok_or_else(|| SiegeError::Seed("schema_registry is required to seed data".into()))?;

        let subject = format!("{topic}-value");
        let schema_id = sr.register_schema(&subject, schema_str).await?;

        let schema = Schema::parse_str(schema_str)
            .map_err(|e| SiegeError::Seed(format!("failed to parse schema: {e}")))?;

        let serializer = AvroSerializer::new(schema.clone(), schema_id);
        let producer = self.kafka.producer();

        let mut futures = Vec::with_capacity(count as usize);
        let mut payloads = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let record = generate_record(&schema, rng)?;
            let payload = serializer.serialize(record)?;
            payloads.push(payload);
        }

        for payload in &payloads {
            futures.push(producer.send(topic, payload));
        }

        futures::future::try_join_all(futures).await?;

        Ok(())
    }
}

impl SeedBackend for Seeder {
    type Error = SiegeError;

    async fn seed_topics(&self) -> Result<SeedResult, SiegeError> {
        let mut created = Vec::new();
        let mut skipped = Vec::new();
        let mut rng = StdRng::seed_from_u64(self.rng_seed);

        for seed in &self.seeds {
            match self
                .kafka
                .create_topic(
                    &seed.name,
                    seed.partitions,
                    seed.replication_factor,
                    seed.config.clone(),
                )
                .await
            {
                Ok(()) => {
                    created.push(seed.name.clone());

                    if let (Some(schema_str), Some(count)) = (seed.schema, seed.record_count) {
                        self.seed_data(&seed.name, schema_str, count, &mut rng)
                            .await?;
                    }
                }
                Err(_) => skipped.push(seed.name.clone()),
            }
        }

        Ok(SeedResult { created, skipped })
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p siege-seed`
Expected: all tests pass (avro, faker, and lib tests)

- [ ] **Step 5: Commit**

```bash
git add crates/siege-seed/src/lib.rs
git commit -m "feat: extend Seeder with schema registration and fake data production"
```

---

### Task 10: Wire into siege-api composition root

**Files:**
- Modify: `crates/siege-api/Cargo.toml`
- Modify: `crates/siege-api/src/main.rs`

- [ ] **Step 1: Add dependency to Cargo.toml**

In `crates/siege-api/Cargo.toml`, add to `[dependencies]`:

```toml
siege-schema-registry.workspace = true
```

- [ ] **Step 2: Update main.rs — add CLI flag, imports, and context wiring**

In `crates/siege-api/src/main.rs`, add to the imports (after line 18):

```rust
use siege::schema_registry::SchemaRegistryBackend;
use siege_schema_registry::SchemaRegistryClient;
use siege_seed::avsc;
```

Add the new field to the `Cli` struct (after line 60):

```rust
#[arg(long)]
schema_registry_url: Option<String>,
```

Add the new field to the `Siege` struct (after line 24):

```rust
schema_registry: Option<SchemaRegistryClient>,
```

Wait — `SiegeContext` requires `type SchemaRegistry: SchemaRegistryBackend`. Since schema_registry is optional, we need a concrete type that's always present. We'll use an enum wrapper that either delegates to the real client or returns noop responses.

Instead, update the `Siege` struct and impl to use a `SchemaRegistryOption` enum. Actually, the simplest approach: use `NoopSchemaRegistry` when no URL is given, and `SchemaRegistryClient` when one is given. Since `SiegeContext` needs a single associated type, we need a wrapper.

Replace the `Siege` struct and `SiegeContext` impl (lines 20–48) with:

```rust
use siege::NoopSchemaRegistry;

pub(crate) enum SchemaRegistryChoice {
    Real(SchemaRegistryClient),
    Noop(NoopSchemaRegistry),
}

impl SchemaRegistryBackend for SchemaRegistryChoice {
    fn register_schema(
        &self,
        subject: &str,
        schema: &str,
    ) -> siege::BoxFuture<'_, Result<siege::SchemaId, siege::SiegeError>> {
        match self {
            Self::Real(c) => c.register_schema(subject, schema),
            Self::Noop(c) => c.register_schema(subject, schema),
        }
    }

    fn delete_subject(
        &self,
        subject: &str,
    ) -> siege::BoxFuture<'_, Result<(), siege::SiegeError>> {
        match self {
            Self::Real(c) => c.delete_subject(subject),
            Self::Noop(c) => c.delete_subject(subject),
        }
    }
}

pub(crate) struct Siege {
    kafka: RdKafkaBackend,
    events: Broadcaster,
    chaos: ChaosClient,
    seeder: Seeder,
    schema_registry: SchemaRegistryChoice,
}

impl SiegeContext for Siege {
    type Kafka = RdKafkaBackend;
    type Events = Broadcaster;
    type Chaos = ChaosClient;
    type Seeder = Seeder;
    type SchemaRegistry = SchemaRegistryChoice;

    fn kafka(&self) -> &RdKafkaBackend {
        &self.kafka
    }

    fn events(&self) -> &Broadcaster {
        &self.events
    }

    fn chaos(&self) -> &ChaosClient {
        &self.chaos
    }

    fn seeder(&self) -> &Seeder {
        &self.seeder
    }

    fn schema_registry(&self) -> &SchemaRegistryChoice {
        &self.schema_registry
    }
}
```

- [ ] **Step 3: Update seeder construction in main()**

Replace the seeder construction block (lines 70–76) and surrounding code with:

```rust
let schema_registry_choice = match &cli.schema_registry_url {
    Some(url) => SchemaRegistryChoice::Real(SchemaRegistryClient::new(url)),
    None => SchemaRegistryChoice::Noop(NoopSchemaRegistry),
};

let mut seeder_builder = Seeder::new(backend.clone());

if let Some(url) = &cli.schema_registry_url {
    seeder_builder = seeder_builder
        .schema_registry(SchemaRegistryClient::new(url));
}

let seeder = seeder_builder
    .topic(TopicSeed::new("kings-landing", 6)
        .schema(avsc!("../../schemas/kings-landing.avsc"))
        .records(100))
    .topic(TopicSeed::new("winterfell", 3)
        .schema(avsc!("../../schemas/winterfell.avsc"))
        .records(100))
    .topic(TopicSeed::new("the-wall", 1)
        .schema(avsc!("../../schemas/the-wall.avsc"))
        .records(50))
    .topic(TopicSeed::new("iron-islands", 3)
        .schema(avsc!("../../schemas/iron-islands.avsc"))
        .records(100))
    .topic(TopicSeed::new("dragonstone", 3)
        .schema(avsc!("../../schemas/dragonstone.avsc"))
        .records(100))
    .topic(TopicSeed::new("the-citadel", 1)
        .config("cleanup.policy", "compact")
        .schema(avsc!("../../schemas/the-citadel.avsc"))
        .records(50));
```

Update the `Siege` struct instantiation (around the `Client::new(Siege { ... })` block) to include:

```rust
let client = web::Data::new(siege::client::Client::new(Siege {
    kafka: backend,
    events: broadcaster,
    chaos,
    seeder,
    schema_registry: schema_registry_choice,
}));
```

- [ ] **Step 4: Update compose.yaml for schema registry URL**

In `compose.yaml`, update the siege-api command to include the schema registry URL when using Redpanda. This step is informational — users who combine compose files manually will add this flag. No change to compose.yaml itself (it stays untouched per the spec).

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p siege-api`
Expected: compiles clean

- [ ] **Step 6: Commit**

```bash
git add crates/siege-api/Cargo.toml crates/siege-api/src/main.rs
git commit -m "feat: wire schema registry and seed data into composition root"
```

---

### Task 11: Integration test with Redpanda

**Files:** None — manual verification

- [ ] **Step 1: Start Redpanda**

Run: `just rp`
Expected: "Redpanda ready" message

- [ ] **Step 2: Run siege-api with schema registry**

Run: `cargo run -p siege-api -- --bootstrap-servers localhost:19092 --schema-registry-url http://localhost:18081 --seed`
Expected: API starts, topics created, schemas registered, data produced, no errors.

- [ ] **Step 3: Verify schemas in Redpanda Console**

Open http://localhost:9080 in browser. Navigate to Schema Registry.
Expected: 6 subjects visible (`kings-landing-value`, `winterfell-value`, etc.) each with 1 version.

- [ ] **Step 4: Verify data in topics**

In Redpanda Console, click on the `kings-landing` topic and view messages.
Expected: 100 messages visible. They should be Avro-encoded and deserializable by the console (since the schema is registered).

- [ ] **Step 5: Verify idempotency**

Stop and re-run siege-api with `--seed` again.
Expected: All topics skipped (already exist). No new schemas registered. No new data produced.

- [ ] **Step 6: Clean up**

```bash
docker compose -f compose.redpanda.yaml down
```

---

### Task 12: Full workspace check

**Files:** None

- [ ] **Step 1: Run workspace check**

Run: `cargo check --workspace --exclude siege-console`
Expected: compiles clean

- [ ] **Step 2: Run workspace tests**

Run: `cargo test --workspace --exclude siege-console`
Expected: all tests pass

- [ ] **Step 3: Run Dioxus check**

Run: `dx check --package siege-console`
Expected: compiles clean (console doesn't depend on new crates directly)
