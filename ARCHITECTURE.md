# Crate Architecture

```
siege-kernel          (foundation — no internal deps)
    ↑
siege                 (core domain)
    ↑
siege-api-spec        (API contract)
    ↑
├── siege-api         (HTTP server)
├── siege-api-client  (HTTP client)
│       ↑
└── siege-console     (frontend)
```

## siege-kernel

Pure domain value objects that can be reused everywhere, including by the kernel itself.
Nothing API-specific.
No utoipa, no request/response types.

Examples: `KafkaProperties`, topic identifiers, configuration primitives.

## siege

Core domain library. Backend traits, domain events, context trait, and the high-level `Client` SDK.

Contains: `KafkaBackend` trait, `EventEmitter` trait, `SiegeContext` trait, `DomainEvent` enum, `Topic`/`TopicDetail` domain models, `Client` with sub-clients (e.g. `Topics`).

## siege-api-spec

API contract types and OpenAPI specification.
This is where request/response types, error types, and event types live.
Has utoipa for `ToSchema` derives.

Contains: `Topic`, `TopicDetail`, `CreateTopicRequest`, `TopicConfigUpdate`, `SiegeError`, `SseEvent`, `ApiDoc`.

Re-exports kernel types so downstream crates don't need a direct kernel dependency.

## siege-api

HTTP server (actix-web).
Routes, Kafka backend implementations, SSE broadcaster, cluster watcher.

## siege-api-client

HTTP client for talking to the siege API.
Re-exports api-spec types so the console doesn't need a direct api-spec dependency either.

## siege-console

Dioxus WASM frontend.
Can **only** depend on `siege-api-client` and `siege-api-spec`.
Never on `siege-kernel` directly.

## Rules

- `siege-kernel` has zero internal crate dependencies.
- `siege` depends only on kernel.
  Backend traits, domain events, and the client SDK live here.
- `siege-api-spec` depends only on kernel.
- The console never reaches into kernel — it gets types through api-client/api-spec.
- utoipa/ToSchema lives in `siege-api-spec`, not in kernel.
- Value objects go in kernel.
  Domain traits and events go in `siege`.
  API request/response/error/event types go in api-spec.
