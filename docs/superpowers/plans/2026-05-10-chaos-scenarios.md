# Chaos Scenarios Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add chaos scenarios to Siege that simulate user errors against Kafka topics (delete, zero retention, flip cleanup policy, increase partitions, poison pills, schema-breaking messages).

**Architecture:** Two new crates: `siege-kafka` (extracts `RdKafkaBackend` from `siege-api`, adds producer) and `siege-chaos` (chaos logic using `siege-kafka`). Six flat `POST /api/chaos/*` endpoints. `siege-api-client` gets a `Topic` handle with `ChaosExt` extension trait. Console side panel becomes non-modal split-screen with chaos action buttons.

**Tech Stack:** Rust, rdkafka (admin + FutureProducer), actix-web, Dioxus, utoipa

---

### Task 1: Create `siege-kafka` crate and move `RdKafkaBackend`

**Files:**
- Create: `crates/siege-kafka/Cargo.toml`
- Create: `crates/siege-kafka/src/lib.rs`
- Create: `crates/siege-kafka/src/backend.rs`
- Create: `crates/siege-kafka/src/producer.rs`
- Modify: `Cargo.toml` (workspace members + workspace deps)
- Modify: `crates/siege-api/Cargo.toml`
- Modify: `crates/siege-api/src/main.rs`
- Modify: `crates/siege-api/src/kafka/mod.rs`
- Modify: `crates/siege-api/src/kafka/rdkafka_backend.rs` (delete file)
- Modify: `crates/siege-api/src/seed.rs`

- [ ] **Step 1: Create `crates/siege-kafka/Cargo.toml`**

```toml
[package]
name = "siege-kafka"
version.workspace = true
edition.workspace = true

[dependencies]
siege.workspace = true
rdkafka.workspace = true
tokio.workspace = true
```

- [ ] **Step 2: Create `crates/siege-kafka/src/backend.rs`**

Move the entire contents of `crates/siege-api/src/kafka/rdkafka_backend.rs` here. No changes to the code — it already uses `siege::kafka::KafkaBackend`, `siege::SiegeError`, `siege::Topic`, `siege::TopicDetail`, `siege::KafkaProperties`.

```rust
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use rdkafka::admin::{
    AdminClient, AdminOptions, AlterConfig, NewTopic, ResourceSpecifier, TopicReplication,
};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use siege::kafka::KafkaBackend;
use siege::{SiegeError, Topic, TopicDetail};
use siege::KafkaProperties;

#[derive(Clone)]
pub struct RdKafkaBackend {
    admin: Arc<AdminClient<DefaultClientContext>>,
}

impl RdKafkaBackend {
    pub fn new(bootstrap_servers: &str) -> Self {
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .create()
            .expect("failed to create Kafka admin client");
        Self {
            admin: Arc::new(admin),
        }
    }

    pub fn admin(&self) -> &AdminClient<DefaultClientContext> {
        &self.admin
    }
}

impl KafkaBackend for RdKafkaBackend {
    fn list_topics(&self) -> impl Future<Output = Result<Vec<Topic>, SiegeError>> + Send {
        let admin = self.admin.clone();
        async move {
            let metadata = admin
                .inner()
                .fetch_metadata(None, Duration::from_secs(10))
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            let topics = metadata
                .topics()
                .iter()
                .filter(|t| !t.name().starts_with("__"))
                .map(|t| Topic {
                    name: t.name().to_owned(),
                    partitions: t.partitions().len() as i32,
                    replication_factor: t
                        .partitions()
                        .first()
                        .map(|p| p.replicas().len() as i32)
                        .unwrap_or(0),
                })
                .collect();

            Ok(topics)
        }
    }

    fn get_topic(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<TopicDetail, SiegeError>> + Send {
        let admin = self.admin.clone();
        let name = name.to_owned();
        async move {
            let metadata = admin
                .inner()
                .fetch_metadata(None, Duration::from_secs(10))
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            let topic_meta = metadata
                .topics()
                .iter()
                .find(|t| t.name() == name)
                .ok_or_else(|| SiegeError::TopicNotFound(name.clone()))?;

            let partitions = topic_meta.partitions().len() as i32;
            let replication_factor = topic_meta
                .partitions()
                .first()
                .map(|p| p.replicas().len() as i32)
                .unwrap_or(0);

            drop(metadata);

            let configs = admin
                .describe_configs(
                    &[ResourceSpecifier::Topic(&name)],
                    &AdminOptions::new(),
                )
                .await
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            let mut config = KafkaProperties::new();
            for result in configs {
                if let Ok(resource) = result {
                    for entry in resource.entries {
                        if let Some(value) = entry.value {
                            config.insert(entry.name, value);
                        }
                    }
                }
            }

            Ok(TopicDetail {
                name: name.clone(),
                partitions,
                replication_factor,
                config,
            })
        }
    }

    fn create_topic(
        &self,
        name: &str,
        partitions: i32,
        replication_factor: i32,
        config: KafkaProperties,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send {
        let admin = self.admin.clone();
        let name = name.to_owned();
        async move {
            let mut new_topic = NewTopic::new(
                &name,
                partitions,
                TopicReplication::Fixed(replication_factor),
            );
            for (key, value) in config.iter() {
                new_topic = new_topic.set(key, value);
            }

            let results = admin
                .create_topics(&[new_topic], &AdminOptions::new())
                .await
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            for result in results {
                result.map_err(|(_, code)| {
                    SiegeError::KafkaError(format!("create topic failed: {:?}", code))
                })?;
            }

            Ok(())
        }
    }

    fn delete_topic(&self, name: &str) -> impl Future<Output = Result<(), SiegeError>> + Send {
        let admin = self.admin.clone();
        let name = name.to_owned();
        async move {
            let results = admin
                .delete_topics(&[&name], &AdminOptions::new())
                .await
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            for result in results {
                result.map_err(|(_, code)| {
                    SiegeError::KafkaError(format!("delete topic failed: {:?}", code))
                })?;
            }

            Ok(())
        }
    }

    fn update_topic_config(
        &self,
        name: &str,
        config: KafkaProperties,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send {
        let admin = self.admin.clone();
        let name = name.to_owned();
        async move {
            let mut alter = AlterConfig::new(ResourceSpecifier::Topic(&name));
            for (key, value) in config.iter() {
                alter = alter.set(key, value);
            }

            let results = admin
                .alter_configs(&[alter], &AdminOptions::new())
                .await
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            for result in results {
                result.map_err(|(_, code)| {
                    SiegeError::KafkaError(format!("alter config failed: {:?}", code))
                })?;
            }

            Ok(())
        }
    }
}
```

Note: Added `pub fn admin(&self)` accessor — `siege-chaos` will need access to the admin client for `describe_configs` (to read current cleanup policy before flipping).

- [ ] **Step 3: Create `crates/siege-kafka/src/producer.rs`**

```rust
use std::sync::Arc;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use siege::SiegeError;

#[derive(Clone)]
pub struct Producer {
    producer: Arc<FutureProducer>,
}

impl Producer {
    pub fn new(bootstrap_servers: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("failed to create Kafka producer");
        Self {
            producer: Arc::new(producer),
        }
    }

    pub async fn send(&self, topic: &str, payload: &[u8]) -> Result<(), SiegeError> {
        let record = FutureRecord::<(), [u8]>::to(topic).payload(payload);
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| SiegeError::KafkaError(e.to_string()))?;
        Ok(())
    }
}
```

- [ ] **Step 4: Create `crates/siege-kafka/src/lib.rs`**

```rust
mod backend;
mod producer;

pub use backend::RdKafkaBackend;
pub use producer::Producer;
```

- [ ] **Step 5: Update workspace `Cargo.toml`**

Add `"crates/siege-kafka"` to the `members` list and `siege-kafka = { path = "crates/siege-kafka" }` to `[workspace.dependencies]`.

- [ ] **Step 6: Update `siege-api` to use `siege-kafka`**

In `crates/siege-api/Cargo.toml`, add `siege-kafka.workspace = true`.

Delete `crates/siege-api/src/kafka/rdkafka_backend.rs`.

Update `crates/siege-api/src/kafka/mod.rs`:

```rust
pub mod backend;
pub mod mock;
```

(Remove the `pub mod rdkafka_backend;` line.)

Update `crates/siege-api/src/main.rs` — change:
```rust
use kafka::rdkafka_backend::RdKafkaBackend;
```
to:
```rust
use siege_kafka::RdKafkaBackend;
```

Update `crates/siege-api/src/seed.rs` — no changes needed (it uses `siege::kafka::KafkaBackend` trait, not the concrete type).

- [ ] **Step 7: Verify it compiles**

Run: `cargo check -p siege-kafka -p siege-api`
Expected: compiles with no errors.

- [ ] **Step 8: Run existing tests**

Run: `cargo test -p siege-api`
Expected: all existing tests pass (they use `MockKafkaBackend`, not `RdKafkaBackend`).

- [ ] **Step 9: Commit**

```bash
git add crates/siege-kafka/ Cargo.toml crates/siege-api/
git commit -m "refactor: extract siege-kafka crate from siege-api"
```

---

### Task 2: Create `siege-chaos` crate with `ChaosClient`

**Files:**
- Create: `crates/siege-chaos/Cargo.toml`
- Create: `crates/siege-chaos/src/lib.rs`
- Create: `crates/siege-chaos/src/error.rs`
- Create: `crates/siege-chaos/src/payloads.rs`
- Modify: `Cargo.toml` (workspace members + deps)

- [ ] **Step 1: Create `crates/siege-chaos/Cargo.toml`**

```toml
[package]
name = "siege-chaos"
version.workspace = true
edition.workspace = true

[dependencies]
siege.workspace = true
siege-kafka.workspace = true
rdkafka.workspace = true
tokio.workspace = true
thiserror.workspace = true
serde.workspace = true
serde_json.workspace = true
rand = "0.9"
```

- [ ] **Step 2: Add to workspace `Cargo.toml`**

Add `"crates/siege-chaos"` to `members` and `siege-chaos = { path = "crates/siege-chaos" }` to `[workspace.dependencies]`. Add `rand = "0.9"` to `[workspace.dependencies]`.

- [ ] **Step 3: Create `crates/siege-chaos/src/error.rs`**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChaosError {
    #[error("topic not found: {0}")]
    TopicNotFound(String),
    #[error("kafka error: {0}")]
    KafkaError(String),
    #[error("producer error: {0}")]
    ProducerError(String),
}

impl From<siege::SiegeError> for ChaosError {
    fn from(e: siege::SiegeError) -> Self {
        match e {
            siege::SiegeError::TopicNotFound(s) => ChaosError::TopicNotFound(s),
            siege::SiegeError::KafkaError(s) => ChaosError::KafkaError(s),
            _ => ChaosError::KafkaError(e.to_string()),
        }
    }
}
```

- [ ] **Step 4: Create `crates/siege-chaos/src/payloads.rs`**

```rust
use rand::Rng;

pub fn poison_pill() -> Vec<u8> {
    let mut rng = rand::rng();
    let len = rng.random_range(16..256);
    let mut bytes = vec![0u8; len];
    rng.fill(&mut bytes[..]);
    // Ensure at least some invalid UTF-8
    if len > 4 {
        bytes[0] = 0xFF;
        bytes[1] = 0xFE;
    }
    bytes
}

pub fn schema_breaking_json() -> Vec<u8> {
    let mut rng = rand::rng();
    let variant = rng.random_range(0..4);
    let json = match variant {
        0 => r#"{"__corrupted":true,"value":null,"ts":-1}"#,
        1 => r#"{"key":12345,"payload":["not","a","valid","schema"]}"#,
        2 => r#"{"error":"chaos","nested":{"broken":true,"data":"0xDEADBEEF"}}"#,
        _ => r#"{"type":"INVALID","version":-999,"fields":{}}"#,
    };
    json.as_bytes().to_vec()
}
```

- [ ] **Step 5: Create `crates/siege-chaos/src/lib.rs`**

```rust
mod error;
mod payloads;

pub use error::ChaosError;

use rdkafka::admin::{AdminOptions, AlterConfig, NewPartitions, ResourceSpecifier};
use siege::KafkaProperties;
use siege_kafka::{Producer, RdKafkaBackend};

pub struct ChaosClient {
    backend: RdKafkaBackend,
    producer: Producer,
}

impl ChaosClient {
    pub fn new(bootstrap_servers: &str) -> Self {
        Self {
            backend: RdKafkaBackend::new(bootstrap_servers),
            producer: Producer::new(bootstrap_servers),
        }
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<(), ChaosError> {
        use siege::kafka::KafkaBackend;
        self.backend.delete_topic(topic).await?;
        Ok(())
    }

    pub async fn zero_retention(&self, topic: &str) -> Result<(), ChaosError> {
        let config: KafkaProperties =
            std::collections::HashMap::from([("retention.ms".into(), "0".into())]).into();
        use siege::kafka::KafkaBackend;
        self.backend.update_topic_config(topic, config).await?;
        Ok(())
    }

    pub async fn flip_cleanup_policy(&self, topic: &str) -> Result<(), ChaosError> {
        use siege::kafka::KafkaBackend;
        let detail = self.backend.get_topic(topic).await?;
        let current = detail
            .config
            .get("cleanup.policy")
            .map(|s| s.as_str())
            .unwrap_or("delete");
        let new_policy = if current.contains("compact") {
            "delete"
        } else {
            "compact"
        };
        let config: KafkaProperties = std::collections::HashMap::from([(
            "cleanup.policy".into(),
            new_policy.into(),
        )])
        .into();
        self.backend.update_topic_config(topic, config).await?;
        Ok(())
    }

    pub async fn increase_partitions(
        &self,
        topic: &str,
        partitions: i32,
    ) -> Result<(), ChaosError> {
        let new_parts = NewPartitions::new(topic, partitions as usize);
        self.backend
            .admin()
            .create_partitions(&[new_parts], &AdminOptions::new())
            .await
            .map_err(|e| ChaosError::KafkaError(e.to_string()))?;
        Ok(())
    }

    pub async fn poison_pills(&self, topic: &str, count: u32) -> Result<(), ChaosError> {
        for _ in 0..count {
            let payload = payloads::poison_pill();
            self.producer
                .send(topic, &payload)
                .await
                .map_err(|e| ChaosError::ProducerError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn schema_break(&self, topic: &str, count: u32) -> Result<(), ChaosError> {
        for _ in 0..count {
            let payload = payloads::schema_breaking_json();
            self.producer
                .send(topic, &payload)
                .await
                .map_err(|e| ChaosError::ProducerError(e.to_string()))?;
        }
        Ok(())
    }
}
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo check -p siege-chaos`
Expected: compiles with no errors.

- [ ] **Step 7: Commit**

```bash
git add crates/siege-chaos/ Cargo.toml
git commit -m "feat: add siege-chaos crate with ChaosClient"
```

---

### Task 3: Add chaos API spec types

**Files:**
- Create: `crates/siege-api-spec/src/chaos.rs`
- Modify: `crates/siege-api-spec/src/lib.rs`
- Modify: `crates/siege-api-spec/Cargo.toml`

- [ ] **Step 1: Create `crates/siege-api-spec/src/chaos.rs`**

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChaosTopicRequest {
    pub topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChaosPartitionsRequest {
    pub topic: String,
    pub partitions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChaosProduceRequest {
    pub topic: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChaosResult {
    pub topic: String,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "code")]
pub enum ChaosError {
    TopicNotFound { message: String },
    KafkaError { message: String },
    ProducerError { message: String },
}
```

- [ ] **Step 2: Update `crates/siege-api-spec/src/lib.rs`**

Add `pub mod chaos;` and `pub use chaos::*;` alongside the existing modules. Add the chaos types to the `ApiDoc` struct's `components(schemas(...))` list and add the six chaos utoipa path stubs:

```rust
pub mod chaos;
pub mod error;
pub mod events;
pub mod types;

pub use chaos::*;
pub use error::*;
pub use events::SseEvent;
pub use siege_kernel::KafkaProperties;
pub use types::*;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Siege API",
        version = "0.1.0",
        description = "Kafka topic management API"
    ),
    paths(
        list_topics, get_topic, create_topic, delete_topic, update_topic_config,
        chaos_delete_topic, chaos_zero_retention, chaos_flip_cleanup_policy,
        chaos_increase_partitions, chaos_poison_pills, chaos_schema_break,
    ),
    components(schemas(
        TopicResource, TopicDetailResource, CreateTopicRequest, TopicConfigUpdateRequest,
        ListTopicsError, GetTopicError, CreateTopicError, DeleteTopicError,
        UpdateTopicConfigError, SseEvent,
        ChaosTopicRequest, ChaosPartitionsRequest, ChaosProduceRequest,
        ChaosResult, ChaosError,
    ))
)]
pub struct ApiDoc;

// -- existing path stubs --

/// List all topics on the cluster
#[utoipa::path(
    get,
    path = "/api/topics",
    responses(
        (status = 200, description = "List all topics", body = Vec<TopicResource>),
        (status = 502, description = "Kafka error", body = ListTopicsError)
    )
)]
async fn list_topics() {}

/// Get topic detail
#[utoipa::path(
    get,
    path = "/api/topics/{name}",
    params(("name" = String, Path, description = "Topic name")),
    responses(
        (status = 200, description = "Topic detail", body = TopicDetailResource),
        (status = 404, description = "Topic not found", body = GetTopicError)
    )
)]
async fn get_topic() {}

/// Create a topic
#[utoipa::path(
    post,
    path = "/api/topics",
    request_body = CreateTopicRequest,
    responses(
        (status = 201, description = "Topic created"),
        (status = 409, description = "Topic already exists", body = CreateTopicError)
    )
)]
async fn create_topic() {}

/// Delete a topic
#[utoipa::path(
    delete,
    path = "/api/topics/{name}",
    params(("name" = String, Path, description = "Topic name")),
    responses(
        (status = 204, description = "Topic deleted"),
        (status = 404, description = "Topic not found", body = DeleteTopicError)
    )
)]
async fn delete_topic() {}

/// Update topic configuration
#[utoipa::path(
    post,
    path = "/api/topics/{name}/config",
    params(("name" = String, Path, description = "Topic name")),
    request_body = TopicConfigUpdateRequest,
    responses(
        (status = 200, description = "Config updated"),
        (status = 404, description = "Topic not found", body = UpdateTopicConfigError)
    )
)]
async fn update_topic_config() {}

// -- chaos path stubs --

/// Delete a topic (chaos)
#[utoipa::path(
    post,
    path = "/api/chaos/delete-topic",
    request_body = ChaosTopicRequest,
    responses(
        (status = 200, description = "Topic deleted", body = ChaosResult),
        (status = 404, description = "Topic not found", body = ChaosError),
        (status = 502, description = "Kafka error", body = ChaosError)
    )
)]
async fn chaos_delete_topic() {}

/// Set retention to zero (chaos)
#[utoipa::path(
    post,
    path = "/api/chaos/zero-retention",
    request_body = ChaosTopicRequest,
    responses(
        (status = 200, description = "Retention set to zero", body = ChaosResult),
        (status = 404, description = "Topic not found", body = ChaosError),
        (status = 502, description = "Kafka error", body = ChaosError)
    )
)]
async fn chaos_zero_retention() {}

/// Flip cleanup policy (chaos)
#[utoipa::path(
    post,
    path = "/api/chaos/flip-cleanup-policy",
    request_body = ChaosTopicRequest,
    responses(
        (status = 200, description = "Cleanup policy flipped", body = ChaosResult),
        (status = 404, description = "Topic not found", body = ChaosError),
        (status = 502, description = "Kafka error", body = ChaosError)
    )
)]
async fn chaos_flip_cleanup_policy() {}

/// Increase partition count (chaos)
#[utoipa::path(
    post,
    path = "/api/chaos/increase-partitions",
    request_body = ChaosPartitionsRequest,
    responses(
        (status = 200, description = "Partitions increased", body = ChaosResult),
        (status = 502, description = "Kafka error", body = ChaosError)
    )
)]
async fn chaos_increase_partitions() {}

/// Produce poison pill messages (chaos)
#[utoipa::path(
    post,
    path = "/api/chaos/poison-pills",
    request_body = ChaosProduceRequest,
    responses(
        (status = 200, description = "Poison pills produced", body = ChaosResult),
        (status = 502, description = "Producer error", body = ChaosError)
    )
)]
async fn chaos_poison_pills() {}

/// Produce schema-breaking messages (chaos)
#[utoipa::path(
    post,
    path = "/api/chaos/schema-break",
    request_body = ChaosProduceRequest,
    responses(
        (status = 200, description = "Schema-breaking messages produced", body = ChaosResult),
        (status = 502, description = "Producer error", body = ChaosError)
    )
)]
async fn chaos_schema_break() {}

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn spec_generates_valid_json() {
        let doc = ApiDoc::openapi();
        let json = doc.to_pretty_json().unwrap();
        assert!(json.contains("\"title\": \"Siege API\""));
        assert!(json.contains("/api/topics"));
        assert!(json.contains("/api/topics/{name}"));
        assert!(json.contains("/api/topics/{name}/config"));
        assert!(json.contains("/api/chaos/delete-topic"));
        assert!(json.contains("/api/chaos/zero-retention"));
    }
}
```

- [ ] **Step 3: Verify it compiles and test passes**

Run: `cargo test -p siege-api-spec`
Expected: `spec_generates_valid_json` passes with the new chaos paths.

- [ ] **Step 4: Commit**

```bash
git add crates/siege-api-spec/
git commit -m "feat: add chaos API spec types and OpenAPI paths"
```

---

### Task 4: Add chaos route handlers in `siege-api`

**Files:**
- Create: `crates/siege-api/src/routes/chaos.rs`
- Modify: `crates/siege-api/src/routes/mod.rs`
- Modify: `crates/siege-api/src/error.rs`
- Modify: `crates/siege-api/src/main.rs`
- Modify: `crates/siege-api/Cargo.toml`

- [ ] **Step 1: Add `siege-chaos` dependency to `crates/siege-api/Cargo.toml`**

Add `siege-chaos.workspace = true`.

- [ ] **Step 2: Update `crates/siege-api/src/error.rs`**

Add a `From<siege_chaos::ChaosError>` impl for `HttpError`:

```rust
impl From<siege_chaos::ChaosError> for HttpError {
    fn from(e: siege_chaos::ChaosError) -> Self {
        match e {
            siege_chaos::ChaosError::TopicNotFound(s) => Self::not_found(s),
            siege_chaos::ChaosError::KafkaError(s) => Self::bad_gateway(s),
            siege_chaos::ChaosError::ProducerError(s) => Self::bad_gateway(s),
        }
    }
}
```

- [ ] **Step 3: Create `crates/siege-api/src/routes/chaos.rs`**

```rust
use actix_web::{web, HttpResponse};
use siege_api_spec::{ChaosPartitionsRequest, ChaosProduceRequest, ChaosResult, ChaosTopicRequest};
use siege_chaos::ChaosClient;

use crate::error::HttpError;

fn ok(topic: String) -> HttpResponse {
    HttpResponse::Ok().json(ChaosResult {
        topic,
        result: "ok".into(),
    })
}

pub async fn delete_topic(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.delete_topic(&req.topic).await?;
    Ok(ok(req.topic))
}

pub async fn zero_retention(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.zero_retention(&req.topic).await?;
    Ok(ok(req.topic))
}

pub async fn flip_cleanup_policy(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosTopicRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.flip_cleanup_policy(&req.topic).await?;
    Ok(ok(req.topic))
}

pub async fn increase_partitions(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosPartitionsRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.increase_partitions(&req.topic, req.partitions).await?;
    Ok(ok(req.topic))
}

pub async fn poison_pills(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosProduceRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.poison_pills(&req.topic, req.count).await?;
    Ok(ok(req.topic))
}

pub async fn schema_break(
    chaos: web::Data<ChaosClient>,
    body: web::Json<ChaosProduceRequest>,
) -> Result<HttpResponse, HttpError> {
    let req = body.into_inner();
    chaos.schema_break(&req.topic, req.count).await?;
    Ok(ok(req.topic))
}
```

- [ ] **Step 4: Update `crates/siege-api/src/routes/mod.rs`**

```rust
pub mod chaos;
pub mod topics;

use actix_web::web;

use siege::SiegeContext;

pub fn configure<C: SiegeContext>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/topics", web::get().to(topics::list_topics::<C>))
            .route("/topics", web::post().to(topics::create_topic::<C>))
            .route("/topics/{name}", web::get().to(topics::get_topic::<C>))
            .route(
                "/topics/{name}",
                web::delete().to(topics::delete_topic::<C>),
            )
            .route(
                "/topics/{name}/config",
                web::post().to(topics::update_topic_config::<C>),
            )
            .route("/events", web::get().to(topics::events::<C>))
            .route(
                "/chaos/delete-topic",
                web::post().to(chaos::delete_topic),
            )
            .route(
                "/chaos/zero-retention",
                web::post().to(chaos::zero_retention),
            )
            .route(
                "/chaos/flip-cleanup-policy",
                web::post().to(chaos::flip_cleanup_policy),
            )
            .route(
                "/chaos/increase-partitions",
                web::post().to(chaos::increase_partitions),
            )
            .route(
                "/chaos/poison-pills",
                web::post().to(chaos::poison_pills),
            )
            .route(
                "/chaos/schema-break",
                web::post().to(chaos::schema_break),
            ),
    );
}
```

- [ ] **Step 5: Update `crates/siege-api/src/main.rs`**

Add the `ChaosClient` as app data. After creating the `backend`, create the chaos client with the same bootstrap servers and register it:

```rust
use siege_chaos::ChaosClient;
```

In `main()`, after `let backend = RdKafkaBackend::new(&cli.bootstrap_servers);`:

```rust
let chaos_client = web::Data::new(ChaosClient::new(&cli.bootstrap_servers));
```

In the `HttpServer::new` closure, add `.app_data(chaos_client.clone())`.

- [ ] **Step 6: Verify it compiles**

Run: `cargo check -p siege-api`
Expected: compiles with no errors.

- [ ] **Step 7: Run existing tests**

Run: `cargo test -p siege-api`
Expected: all existing tests still pass.

- [ ] **Step 8: Commit**

```bash
git add crates/siege-api/
git commit -m "feat: add chaos API route handlers"
```

---

### Task 5: Refactor `siege-api-client` with `Topic` handle and `ChaosExt`

**Files:**
- Create: `crates/siege-api-client/src/topic.rs`
- Create: `crates/siege-api-client/src/chaos.rs`
- Modify: `crates/siege-api-client/src/lib.rs`

- [ ] **Step 1: Create `crates/siege-api-client/src/topic.rs`**

```rust
use siege_api_spec::{TopicConfigUpdateRequest, TopicDetailResource};

use crate::{ClientError, SiegeClient};

pub struct Topic<'a> {
    pub(crate) client: &'a SiegeClient,
    pub(crate) name: String,
}

impl Topic<'_> {
    pub async fn get(&self) -> Result<TopicDetailResource, ClientError> {
        let resp = self
            .client
            .http()
            .get(format!("{}/api/topics/{}", self.client.base_url(), self.name))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            Err(SiegeClient::api_error(resp).await)
        }
    }

    pub async fn delete(&self) -> Result<(), ClientError> {
        let resp = self
            .client
            .http()
            .delete(format!("{}/api/topics/{}", self.client.base_url(), self.name))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(SiegeClient::api_error(resp).await)
        }
    }

    pub async fn update_config(
        &self,
        config: &TopicConfigUpdateRequest,
    ) -> Result<(), ClientError> {
        let resp = self
            .client
            .http()
            .post(format!(
                "{}/api/topics/{}/config",
                self.client.base_url(),
                self.name
            ))
            .json(config)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(SiegeClient::api_error(resp).await)
        }
    }
}
```

- [ ] **Step 2: Create `crates/siege-api-client/src/chaos.rs`**

```rust
use siege_api_spec::ChaosResult;
use thiserror::Error;

use crate::topic::Topic;
use crate::SiegeClient;

#[derive(Debug, Error)]
pub enum ChaosError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("chaos API error ({status}): {body}")]
    Api { status: u16, body: String },
}

pub trait ChaosExt {
    async fn zero_retention(&self) -> Result<ChaosResult, ChaosError>;
    async fn flip_cleanup_policy(&self) -> Result<ChaosResult, ChaosError>;
    async fn increase_partitions(&self, partitions: i32) -> Result<ChaosResult, ChaosError>;
    async fn poison_pills(&self, count: u32) -> Result<ChaosResult, ChaosError>;
    async fn schema_break(&self, count: u32) -> Result<ChaosResult, ChaosError>;
}

impl ChaosExt for Topic<'_> {
    async fn zero_retention(&self) -> Result<ChaosResult, ChaosError> {
        self.chaos_post("zero-retention", serde_json::json!({ "topic": self.name }))
            .await
    }

    async fn flip_cleanup_policy(&self) -> Result<ChaosResult, ChaosError> {
        self.chaos_post(
            "flip-cleanup-policy",
            serde_json::json!({ "topic": self.name }),
        )
        .await
    }

    async fn increase_partitions(&self, partitions: i32) -> Result<ChaosResult, ChaosError> {
        self.chaos_post(
            "increase-partitions",
            serde_json::json!({ "topic": self.name, "partitions": partitions }),
        )
        .await
    }

    async fn poison_pills(&self, count: u32) -> Result<ChaosResult, ChaosError> {
        self.chaos_post(
            "poison-pills",
            serde_json::json!({ "topic": self.name, "count": count }),
        )
        .await
    }

    async fn schema_break(&self, count: u32) -> Result<ChaosResult, ChaosError> {
        self.chaos_post(
            "schema-break",
            serde_json::json!({ "topic": self.name, "count": count }),
        )
        .await
    }
}

impl Topic<'_> {
    async fn chaos_post(
        &self,
        action: &str,
        body: serde_json::Value,
    ) -> Result<ChaosResult, ChaosError> {
        let resp = self
            .client
            .http()
            .post(format!("{}/api/chaos/{action}", self.client.base_url()))
            .json(&body)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            Err(ChaosError::Api { status, body })
        }
    }
}
```

- [ ] **Step 3: Update `crates/siege-api-client/src/lib.rs`**

Refactor the client to expose `http()` and `base_url()` accessors (used by `Topic` and `ChaosExt`), add `topic()` method, keep existing methods for backwards compatibility with the console:

```rust
pub mod chaos;
pub mod topic;

pub use chaos::{ChaosError, ChaosExt};
pub use siege_api_spec::{
    ChaosResult, CreateTopicRequest, KafkaProperties, SseEvent, TopicConfigUpdateRequest,
    TopicDetailResource, TopicResource,
};
pub use topic::Topic;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error ({status}): {body}")]
    Api { status: u16, body: String },
}

#[derive(Clone)]
pub struct SiegeClient {
    base_url: String,
    client: reqwest::Client,
}

impl SiegeClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            client: reqwest::Client::new(),
        }
    }

    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.client
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) async fn api_error(resp: reqwest::Response) -> ClientError {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        ClientError::Api { status, body }
    }

    pub fn topic(&self, name: &str) -> Topic<'_> {
        Topic {
            client: self,
            name: name.to_owned(),
        }
    }

    pub async fn list_topics(&self) -> Result<Vec<TopicResource>, ClientError> {
        let resp = self
            .client
            .get(format!("{}/api/topics", self.base_url))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            Err(Self::api_error(resp).await)
        }
    }

    pub async fn get_topic(&self, name: &str) -> Result<TopicDetailResource, ClientError> {
        self.topic(name).get().await
    }

    pub async fn create_topic(&self, req: &CreateTopicRequest) -> Result<(), ClientError> {
        let resp = self
            .client
            .post(format!("{}/api/topics", self.base_url))
            .json(req)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Self::api_error(resp).await)
        }
    }

    pub async fn delete_topic(&self, name: &str) -> Result<(), ClientError> {
        self.topic(name).delete().await
    }

    pub async fn update_topic_config(
        &self,
        name: &str,
        config: &TopicConfigUpdateRequest,
    ) -> Result<(), ClientError> {
        self.topic(name).update_config(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let client = SiegeClient::new("http://localhost:8080");
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn client_strips_trailing_slash() {
        let client = SiegeClient::new("http://localhost:8080/");
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn topic_handle_holds_name() {
        let client = SiegeClient::new("http://localhost:8080");
        let topic = client.topic("my-topic");
        assert_eq!(topic.name, "my-topic");
    }
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p siege-api-client`
Expected: compiles with no errors.

- [ ] **Step 5: Run tests**

Run: `cargo test -p siege-api-client`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/siege-api-client/
git commit -m "feat: add Topic handle and ChaosExt to siege-api-client"
```

---

### Task 6: Update console — non-modal split-screen side panel

**Files:**
- Modify: `crates/siege-console/src/main.rs`
- Modify: `crates/siege-console/src/pages/topics/topic_detail.rs`
- Modify: `crates/siege-console/src/pages/topics/topic_list.rs`
- Modify: `crates/siege-console/src/layouts/default.rs`

- [ ] **Step 1: Update `crates/siege-console/src/main.rs`**

Change the layout so `TopicList` and `TopicDetailPanel` sit side by side inside a flex container, not as overlay/modal:

```rust
mod components;
mod layouts;
mod pages;
mod sse;
mod state;

use layouts::default::Layout;
use pages::topics::topic_detail::TopicDetailPanel;
use pages::topics::topic_list::TopicList;
use dioxus::prelude::*;
use siege_api_client::TopicDetailResource;
use state::{AppState, Theme};

const API_URL: &str = "http://localhost:8080";

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let topics = use_signal(Vec::new);
    let selected_topic = use_signal(|| None::<TopicDetailResource>);
    let theme = use_signal(|| Theme::Dark);

    use_context_provider(|| AppState {
        topics,
        selected_topic,
        theme,
        api_url: API_URL,
    });

    sse::use_sse_subscription();

    let selected = (use_context::<AppState>().selected_topic)();

    rsx! {
        Layout {
            div { class: "flex flex-1 overflow-hidden",
                div { class: if selected.is_some() { "w-1/2 flex flex-col overflow-hidden" } else { "flex-1 flex flex-col overflow-hidden" },
                    TopicList {}
                }
                if let Some(detail) = selected {
                    div { class: "w-1/2 border-l border-border overflow-y-auto",
                        TopicDetailPanel { detail }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Update `crates/siege-console/src/pages/topics/topic_detail.rs`**

Remove the modal overlay (backdrop div, fixed positioning). Make it a plain panel. Add chaos action buttons below the config table:

```rust
use dioxus::prelude::*;
use siege_api_client::{ChaosExt, TopicDetailResource};

use super::topic_pills::TopicPills;
use crate::state::AppState;

#[component]
pub fn TopicDetailPanel(detail: TopicDetailResource) -> Element {
    let mut state = use_context::<AppState>();
    let name = detail.name.clone();
    let mut feedback = use_signal(|| None::<String>);

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "flex items-center justify-between px-6 py-4 border-b border-border",
                h2 { class: "text-sm font-semibold truncate", "{detail.name}" }
                button {
                    class: "text-muted-foreground hover:text-foreground text-lg leading-none cursor-pointer",
                    onclick: move |_| state.selected_topic.set(None),
                    "\u{00d7}"
                }
            }

            div { class: "flex-1 overflow-y-auto",
                div { class: "px-6 py-4 border-b border-border",
                    TopicPills { partitions: detail.partitions, replication_factor: detail.replication_factor, config: detail.config.clone() }
                }

                if !detail.config.is_empty() {
                    div { class: "px-6 py-4 border-b border-border",
                        h3 { class: "text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3",
                            "Configuration"
                        }
                        div { class: "border border-border rounded-lg overflow-hidden",
                            table { class: "w-full text-sm",
                                thead {
                                    tr { class: "bg-muted",
                                        th { class: "text-left px-3 py-2 text-xs font-medium text-muted-foreground", "Key" }
                                        th { class: "text-left px-3 py-2 text-xs font-medium text-muted-foreground", "Value" }
                                    }
                                }
                                tbody {
                                    for (key, value) in detail.config.iter() {
                                        tr { class: "border-t border-border",
                                            td { class: "px-3 py-2 text-xs break-all", "{key}" }
                                            td { class: "px-3 py-2 text-xs text-muted-foreground break-all", "{value}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "px-6 py-4",
                    h3 { class: "text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3",
                        "Chaos"
                    }

                    if let Some(msg) = feedback() {
                        div { class: "mb-3 px-3 py-2 rounded-md text-xs bg-muted text-foreground",
                            "{msg}"
                        }
                    }

                    div { class: "flex flex-col gap-2",
                        ChaosButton {
                            label: "Delete topic",
                            variant: "destructive",
                            name: name.clone(),
                            feedback: feedback,
                            action: ChaosAction::DeleteTopic,
                        }
                        ChaosButton {
                            label: "Zero retention",
                            variant: "warning",
                            name: name.clone(),
                            feedback: feedback,
                            action: ChaosAction::ZeroRetention,
                        }
                        ChaosButton {
                            label: "Flip cleanup policy",
                            variant: "warning",
                            name: name.clone(),
                            feedback: feedback,
                            action: ChaosAction::FlipCleanupPolicy,
                        }
                        ChaosNumberButton {
                            label: "Increase partitions",
                            variant: "warning",
                            name: name.clone(),
                            feedback: feedback,
                            action: ChaosAction::IncreasePartitions,
                            default_value: 100,
                        }
                        ChaosNumberButton {
                            label: "Poison pills",
                            variant: "warning",
                            name: name.clone(),
                            feedback: feedback,
                            action: ChaosAction::PoisonPills,
                            default_value: 10,
                        }
                        ChaosNumberButton {
                            label: "Schema break",
                            variant: "warning",
                            name: name.clone(),
                            feedback: feedback,
                            action: ChaosAction::SchemaBreak,
                            default_value: 10,
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum ChaosAction {
    DeleteTopic,
    ZeroRetention,
    FlipCleanupPolicy,
    IncreasePartitions,
    PoisonPills,
    SchemaBreak,
}

#[component]
fn ChaosButton(
    label: &'static str,
    variant: &'static str,
    name: String,
    feedback: Signal<Option<String>>,
    action: ChaosAction,
) -> Element {
    let mut state = use_context::<AppState>();
    let btn_class = if variant == "destructive" {
        "px-3 py-1.5 rounded-md text-xs font-medium bg-destructive text-destructive-foreground hover:bg-destructive-hover cursor-pointer transition-colors"
    } else {
        "px-3 py-1.5 rounded-md text-xs font-medium bg-amber-600 text-white hover:bg-amber-700 cursor-pointer transition-colors"
    };

    rsx! {
        button {
            class: btn_class,
            onclick: {
                let name = name.clone();
                let action = action.clone();
                move |_| {
                    let client = state.client();
                    let name = name.clone();
                    let action = action.clone();
                    let mut feedback = feedback;
                    spawn(async move {
                        let topic = client.topic(&name);
                        let result = match action {
                            ChaosAction::DeleteTopic => {
                                topic.delete().await.map(|_| {
                                    state.selected_topic.set(None);
                                    "Topic deleted".to_string()
                                })
                            }
                            ChaosAction::ZeroRetention => {
                                topic.zero_retention().await
                                    .map(|r| format!("{}: {}", r.topic, r.result))
                                    .map_err(|e| siege_api_client::ClientError::Api {
                                        status: 0,
                                        body: e.to_string(),
                                    })
                            }
                            ChaosAction::FlipCleanupPolicy => {
                                topic.flip_cleanup_policy().await
                                    .map(|r| format!("{}: {}", r.topic, r.result))
                                    .map_err(|e| siege_api_client::ClientError::Api {
                                        status: 0,
                                        body: e.to_string(),
                                    })
                            }
                            _ => unreachable!(),
                        };
                        match result {
                            Ok(msg) => feedback.set(Some(msg)),
                            Err(e) => feedback.set(Some(format!("Error: {e}"))),
                        }
                    });
                }
            },
            "{label}"
        }
    }
}

#[component]
fn ChaosNumberButton(
    label: &'static str,
    variant: &'static str,
    name: String,
    feedback: Signal<Option<String>>,
    action: ChaosAction,
    default_value: i64,
) -> Element {
    let mut state = use_context::<AppState>();
    let mut input_value = use_signal(move || default_value.to_string());

    rsx! {
        div { class: "flex items-center gap-2",
            button {
                class: "px-3 py-1.5 rounded-md text-xs font-medium bg-amber-600 text-white hover:bg-amber-700 cursor-pointer transition-colors",
                onclick: {
                    let name = name.clone();
                    let action = action.clone();
                    move |_| {
                        let client = state.client();
                        let name = name.clone();
                        let action = action.clone();
                        let value = input_value().parse::<i64>().unwrap_or(default_value);
                        let mut feedback = feedback;
                        spawn(async move {
                            let topic = client.topic(&name);
                            let result = match action {
                                ChaosAction::IncreasePartitions => {
                                    topic.increase_partitions(value as i32).await
                                        .map(|r| format!("{}: {}", r.topic, r.result))
                                        .map_err(|e| e.to_string())
                                }
                                ChaosAction::PoisonPills => {
                                    topic.poison_pills(value as u32).await
                                        .map(|r| format!("{}: {}", r.topic, r.result))
                                        .map_err(|e| e.to_string())
                                }
                                ChaosAction::SchemaBreak => {
                                    topic.schema_break(value as u32).await
                                        .map(|r| format!("{}: {}", r.topic, r.result))
                                        .map_err(|e| e.to_string())
                                }
                                _ => unreachable!(),
                            };
                            match result {
                                Ok(msg) => feedback.set(Some(msg)),
                                Err(e) => feedback.set(Some(format!("Error: {e}"))),
                            }
                        });
                    }
                },
                "{label}"
            }
            input {
                r#type: "number",
                class: "w-20 px-2 py-1 rounded-md text-xs border border-border bg-background text-foreground",
                value: "{input_value}",
                oninput: move |e| input_value.set(e.value()),
            }
        }
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p siege-console`
Expected: compiles with no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/siege-console/
git commit -m "feat: add chaos actions to console topic detail panel"
```

---

### Task 7: Verify full workspace compiles and tests pass

**Files:** None (verification only)

- [ ] **Step 1: Check entire workspace compiles**

Run: `cargo check --workspace`
Expected: all crates compile with no errors.

- [ ] **Step 2: Run all tests**

Run: `cargo test --workspace`
Expected: all tests pass.

- [ ] **Step 3: Commit any fixes if needed**

If any compilation or test failures were found and fixed in the previous steps, commit the fixes:

```bash
git add -A
git commit -m "fix: resolve workspace compilation issues"
```
