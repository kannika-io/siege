# Crate Architecture

```
siege-kernel          (foundation — no internal deps)
    ↑
siege-api-spec        (depends on kernel)
    ↑
├── siege-api         (depends on kernel + api-spec)
├── siege-api-client  (depends on api-spec)
│       ↑
└── siege-console     (depends on api-client + api-spec)
```

## siege-kernel

Pure domain value objects that can be reused everywhere, including by the kernel itself.
Nothing API-specific.
No utoipa, no request/response types.

Examples: `KafkaProperties`, topic identifiers, configuration primitives.

## siege-api-spec

API contract types and OpenAPI specification.
This is where request/response types, error types, and event types live.
Has utoipa for `ToSchema` derives.

Contains: `Topic`, `TopicDetail`, `CreateTopicRequest`, `TopicConfigUpdate`, `SiegeError`, `SseEvent`, `ApiDoc`.

Re-exports `siege-kernel` types so downstream crates don't need a direct kernel dependency.

## siege-api

HTTP server (actix-web).
Routes, Kafka backend implementations, SSE broadcaster, cluster watcher.

Depends on kernel (for value objects in backend code) and api-spec (for API types).

## siege-api-client

HTTP client for talking to the siege API.
Re-exports api-spec types so the console doesn't need a direct api-spec dependency either.

## siege-console

Dioxus WASM frontend.
Can **only** depend on `siege-api-client` and `siege-api-spec`.
Never on `siege-kernel` directly.

## Rules

- `siege-kernel` has zero internal crate dependencies.
  It is the foundation.
- `siege-api-spec` depends only on `siege-kernel`.
  No other internal crates.
- The console never reaches into kernel — it gets types through api-client/api-spec.
- utoipa/ToSchema lives in `siege-api-spec`, not in kernel.
- Value objects go in kernel.
  API request/response/error/event types go in api-spec.
