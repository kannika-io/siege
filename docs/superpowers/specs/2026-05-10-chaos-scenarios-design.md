# Chaos Scenarios Design

Siege gets chaos scenarios that simulate user errors against Kafka topics — the kind of mistakes that MirrorMaker and Cluster Linking don't protect against. The purpose is to demonstrate the value of a separate backup solution.

## Scenarios

Six one-shot chaos actions, all scoped to a single topic:

| Scenario | What it does |
|---|---|
| **Delete topic** | Deletes the topic entirely |
| **Zero retention** | Sets `retention.ms=0`, causing Kafka to purge all data |
| **Flip cleanup policy** | Toggles `cleanup.policy` between `delete` and `compact` |
| **Increase partitions** | Sets partition count to a given value (breaks key-based ordering) |
| **Poison pills** | Produces N corrupt/malformed messages into the topic |
| **Schema break** | Produces N messages with an intentionally wrong schema/format |

## New crates

### `siege-kafka`

Extracted from `siege-api` — the `RdKafkaBackend` implementation moves here, along with a new `FutureProducer` wrapper for message production. Implements the `KafkaBackend` trait from `siege`.

### `siege-chaos`

Chaos scenario logic. Depends on `siege-kafka` for admin operations (delete topic, alter configs) and message production (poison pills, schema break).

```
siege-kernel
    ↑
siege                (KafkaBackend trait, domain)
    ↑
siege-kafka          (RdKafkaBackend + producer, extracted from siege-api)
    ↑
├── siege-chaos      (chaos logic, uses siege-kafka)
│
siege-api-spec       (depends on siege-kernel)
    ↑
siege-api            (depends on siege-kafka, siege-chaos, siege-api-spec)
```

### `siege-chaos` public API

```rust
pub struct ChaosClient {
    kafka: siege_kafka::RdKafkaBackend,
    producer: siege_kafka::Producer,
}

impl ChaosClient {
    pub fn new(bootstrap_servers: &str) -> Result<Self, SiegeError>;

    pub async fn delete_topic(&self, topic: &str) -> Result<(), SiegeError>;
    pub async fn zero_retention(&self, topic: &str) -> Result<(), SiegeError>;
    pub async fn flip_cleanup_policy(&self, topic: &str) -> Result<(), SiegeError>;
    pub async fn increase_partitions(&self, topic: &str, partitions: i32) -> Result<(), SiegeError>;
    pub async fn poison_pills(&self, topic: &str, count: u32) -> Result<(), SiegeError>;
    pub async fn schema_break(&self, topic: &str, count: u32) -> Result<(), SiegeError>;
}
```

### Payload generation

- **Poison pills**: random bytes, invalid UTF-8, null keys with non-null values or vice versa.
- **Schema break**: valid JSON with unexpected structure — wrong field names, wrong types, missing required fields. Not tied to any specific schema registry; the point is to inject obviously wrong data.

## API endpoints

Six `POST` endpoints under `/api/chaos`, each targeting a single topic:

| Endpoint | Request body |
|---|---|
| `POST /api/chaos/delete-topic` | `{ "topic": "name" }` |
| `POST /api/chaos/zero-retention` | `{ "topic": "name" }` |
| `POST /api/chaos/flip-cleanup-policy` | `{ "topic": "name" }` |
| `POST /api/chaos/increase-partitions` | `{ "topic": "name", "partitions": 100 }` |
| `POST /api/chaos/poison-pills` | `{ "topic": "name", "count": 10 }` |
| `POST /api/chaos/schema-break` | `{ "topic": "name", "count": 10 }` |

All return:

```json
{ "topic": "name", "result": "ok" }
```

On error, standard Siege error response (maps to 404 if topic not found, 502 on Kafka errors).

### API spec types (`siege-api-spec`)

```rust
pub struct ChaosTopicRequest {
    pub topic: String,
}

pub struct ChaosPartitionsRequest {
    pub topic: String,
    pub partitions: i32,
}

pub struct ChaosProduceRequest {
    pub topic: String,
    pub count: u32,
}

pub struct ChaosResult {
    pub topic: String,
    pub result: String,
}
```

OpenAPI annotations via `utoipa::ToSchema` on all types. Endpoints added to `ApiDoc`.

## API client (`siege-api-client`)

New `Topic` handle returned by `client.topic("name")`. Holds client ref + topic name, no fetch. All per-topic operations (existing and chaos) live on it:

```rust
impl SiegeClient {
    pub fn topic(&self, name: &str) -> Topic<'_>;
    pub fn topics(&self) -> Topics<'_>;  // existing, for list()
}

pub struct Topic<'a> {
    client: &'a SiegeClient,
    name: String,
}

impl Topic<'_> {
    // existing operations
    pub async fn get(&self) -> Result<TopicDetailResource, ClientError>;
    pub async fn delete(&self) -> Result<ChaosResult, ClientError>;
    pub async fn update_config(&self, config: &TopicConfigUpdateRequest) -> Result<(), ClientError>;

    // chaos operations
    pub async fn zero_retention(&self) -> Result<ChaosResult, ClientError>;
    pub async fn flip_cleanup_policy(&self) -> Result<ChaosResult, ClientError>;
    pub async fn increase_partitions(&self, partitions: i32) -> Result<ChaosResult, ClientError>;
    pub async fn poison_pills(&self, count: u32) -> Result<ChaosResult, ClientError>;
    pub async fn schema_break(&self, count: u32) -> Result<ChaosResult, ClientError>;
}
```

## Console UI

The existing topic detail side panel changes from a modal overlay to a **non-modal split-screen sidenav** on the right side of the topic list.

### Layout change

- Topic list takes the left portion of the screen
- When a topic is selected, the detail panel slides in on the right as a persistent sidenav (no backdrop, no click-outside-to-close)
- Explicit close button to dismiss

### Chaos actions section

Below the existing topic config table, a new "Chaos" section with:

- **Delete topic** button
- **Zero retention** button
- **Flip cleanup policy** button
- **Increase partitions** — button with a numeric input for target partition count
- **Poison pills** — button with a numeric input for message count
- **Schema break** — button with a numeric input for message count

Each button fires the corresponding API call. Results shown inline — success confirmation or error message.

### SSE events

Existing SSE events (`TopicCreated`, `TopicDeleted`, `TopicsSnapshot`) already cover topic deletion and will keep the topic list in sync. No new SSE event types needed — config changes and produced messages don't need real-time push since the user who triggered them sees the inline result.
