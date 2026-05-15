# Seed Data & Schema Registry Design

## Goal

Extend seeding to register Avro schemas and produce realistic fake data for newly created topics. Add Redpanda as an alternative infrastructure with built-in schema registry support.

## Infrastructure

### `compose.redpanda.yaml` (standalone)

A separate compose file — does not modify `compose.yaml`.

- **redpanda** — single-node, `--mode dev-container`, Kafka on 19092, Schema Registry on 18081, Pandaproxy on 18082. Healthcheck via `rpk cluster info`.
- **console** — Redpanda Console, pointed at redpanda's internal addresses, exposed on port 9080.

### Justfile

New `rp` task (aliased `redpanda`): runs `docker compose -f compose.redpanda.yaml up -d` and waits for healthcheck. Mirrors the existing `kafka` task pattern.

## New Crates

### `siege-schema-registry`

Confluent-compatible Schema Registry HTTP client using `reqwest`.

**Domain trait** in `siege/src/schema_registry.rs`:

```rust
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

Uses `BoxFuture` and `SiegeError` (matching `KafkaBackend` pattern) for object safety — the `Seeder` boxes this backend.

**Implementation** in `siege-schema-registry`:

- `SchemaRegistryClient::new(url: &str)`
- `register_schema` → `POST /subjects/{subject}/versions` with `{"schemaType": "AVRO", "schema": "..."}`
- `delete_subject` → `DELETE /subjects/{subject}?permanent=true`
- Subject naming follows Confluent convention: `{topic}-value`

### `siege-seed-avsc`

Proc macro crate providing the `avsc!` macro. Re-exported from `siege-seed`.

```rust
avsc!("schemas/kings-landing.avsc")
```

At compile time:
1. Reads the `.avsc` file (relative to crate root, like `include_str!`)
2. Parses as Avro schema JSON via `apache-avro::Schema::parse_str`
3. Fails compilation with a clear error if the schema is invalid
4. Expands to `&'static str` of the validated schema content

## Modified Crates

### `siege`

- New module `src/schema_registry.rs` with `SchemaRegistryBackend` trait and `SchemaId` type
- `SiegeContext` gains a new associated type:

```rust
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

### `siege-seed`

**`TopicSeed` extensions:**

```rust
pub struct TopicSeed {
    name: String,
    partitions: i32,
    replication_factor: i32,
    config: KafkaProperties,
    schema: Option<&'static str>,   // validated .avsc content via avsc! macro
    record_count: Option<u32>,       // number of records to produce
}
```

New builder methods:
- `.schema(avsc!("..."))` — set the Avro schema (compile-time validated)
- `.records(count)` — set number of records to produce

**`Seeder` changes:**

```rust
pub struct Seeder {
    kafka: Box<dyn KafkaBackend>,
    schema_registry: Option<Box<dyn SchemaRegistryBackend>>,
    seeds: Vec<TopicSeed>,
    rng_seed: u64,  // default: 42
}
```

- `Seeder::new(backend, schema_registry)` — default RNG seed of `42`
- `.rng_seed(seed: u64)` — override the RNG seed for deterministic data

**`AvroSerializer`** (internal helper in `siege-seed`):
- Takes a parsed `apache_avro::Schema` + `SchemaId`
- `serialize(value) -> Vec<u8>` — Avro binary encoding with Confluent wire format header: `[0x00][4-byte schema ID BE][Avro binary payload]`

**Fake data generation:**
- Walk the Avro schema fields recursively
- Map field names to `fake` crate generators:
  - `name` → `fake::faker::name::en::Name`
  - `email` → `fake::faker::internet::en::SafeEmail`
  - `address` → `fake::faker::address::en::StreetAddress`
  - Other `String` fields → `fake::faker::lorem::en::Sentence`
  - `Int`/`Long` → random range
  - `Float`/`Double` → random float
  - `Boolean` → random bool
  - `Enum` → random pick from symbols
  - `Array` → random length, recurse
  - `Record` → recurse into fields
  - `Union` → random variant pick
- Uses `StdRng::seed_from_u64(rng_seed)` for deterministic output

### `siege-api`

- New CLI flag: `--schema-registry-url <URL>` (e.g. `http://localhost:18081`)
- Composition root wires `SchemaRegistryClient` into `Siege` context and `Seeder`

## Seed Flow

For each `TopicSeed`:

1. **Create topic** — skip if already exists (as today)
2. **If created** and `.schema()` is set:
   - Register schema via `SchemaRegistryBackend::register_schema("{topic}-value", schema)` → get `SchemaId`
3. **If created** and `.records()` is set:
   - Create `StdRng` from `rng_seed`
   - Generate `record_count` fake records using schema field types + `fake` crate
   - Serialize each with `AvroSerializer` (Confluent wire format)
   - Fire all sends via `KafkaProducer::send()` concurrently
   - `try_join_all` to await all delivery reports

Skipped topics get no schema registration and no data.

## Composition Root Example

```rust
let schema_registry = SchemaRegistryClient::new(&cli.schema_registry_url);

let seeder = Seeder::new(backend.clone(), schema_registry.clone())
    .topic(TopicSeed::new("kings-landing", 6)
        .schema(avsc!("schemas/kings-landing.avsc"))
        .records(100))
    .topic(TopicSeed::new("winterfell", 3)
        .schema(avsc!("schemas/winterfell.avsc"))
        .records(100))
    .topic(TopicSeed::new("the-wall", 1)
        .schema(avsc!("schemas/the-wall.avsc"))
        .records(50))
    .topic(TopicSeed::new("iron-islands", 3)
        .schema(avsc!("schemas/iron-islands.avsc"))
        .records(100))
    .topic(TopicSeed::new("dragonstone", 3)
        .schema(avsc!("schemas/dragonstone.avsc"))
        .records(100))
    .topic(TopicSeed::new("the-citadel", 1)
        .config("cleanup.policy", "compact")
        .schema(avsc!("schemas/the-citadel.avsc"))
        .records(50));
```

## New Files

- `compose.redpanda.yaml` — Redpanda + Redpanda Console
- `schemas/*.avsc` — 6 Avro schema files (one per seed topic)

## New Dependencies

- `apache-avro` — schema parsing + Avro binary encoding
- `fake` — realistic deterministic fake data generation
- `reqwest` (already in workspace) — schema registry HTTP client
