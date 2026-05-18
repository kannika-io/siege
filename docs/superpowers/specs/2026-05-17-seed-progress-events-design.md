# Seed Progress Events & Toast Notifications

## Problem

Seeding 6 topics with up to 100k records each takes noticeable time. The user clicks the seed button, waits with no feedback, then gets a single success/error toast. There's no indication of what's happening or how far along seeding is.

## Solution

Emit intermediate `SeedProgress` events through the existing SSE infrastructure during seeding, and display them in a single updating toast notification in the console.

## Event Design

### Domain Event (`siege/src/event.rs`)

```rust
pub struct SeedProgressEvent {
    pub topic: String,
    pub topic_index: u32,
    pub total_topics: u32,
    pub records_generated: u32,
    pub total_records: u32,
}
```

Added to `DomainEvent` as `SeedProgress(SeedProgressEvent)`.

- `topic_index` is 0-based, `total_topics` is the count of all seeds
- When a topic has no records (topic-only creation), both record fields are 0
- Structured data enables future progress bar rendering

### SSE Event (`siege-api-spec/src/events.rs`)

```rust
SeedProgress {
    topic: String,
    topic_index: u32,
    total_topics: u32,
    records_generated: u32,
    total_records: u32,
}
```

Broadcaster maps `DomainEvent::SeedProgress` → `SseEvent::SeedProgress`.

## Seeder Changes (`siege-seed/src/lib.rs`)

### EventEmitter on Seeder

The `Seeder` gains an optional `Arc<dyn EventEmitter>`:

```rust
pub struct Seeder {
    // ...existing fields...
    events: Option<Arc<dyn EventEmitter>>,
}

pub fn events(mut self, emitter: Arc<dyn EventEmitter>) -> Self {
    self.events = Some(emitter);
    self
}
```

### Emission Points

1. **Topic start** — emitted when entering each topic in the `seed_topics()` loop, with `records_generated: 0`
2. **During record generation** — time-based throttle in `seed_data()`: check `Instant::elapsed()` on each loop iteration, emit if >= 500ms since last emit
3. **Topic complete** — emitted after records are sent, with `records_generated == total_records`

### seed_data Signature Change

`seed_data` receives `topic_index: u32` and `total_topics: u32` to construct the full event.

### Time-Based Throttle

```rust
let mut last_progress = Instant::now();
for i in 0..count {
    let record = generate_record(&schema, rng)?;
    // ...build record...
    if last_progress.elapsed() >= Duration::from_millis(500) {
        self.emit_progress(topic, topic_index, total_topics, i + 1, count);
        last_progress = Instant::now();
    }
}
```

## Toast System Changes (`siege-console/src/components/ui/toast.rs`)

### New ToastKind Variant

```rust
pub enum ToastKind {
    Success,
    Error,
    Progress,
}
```

`Progress` renders with distinct styling (indigo/blue background).

### Named Toast Support

`Toast` gains an optional `name` field:

```rust
pub struct Toast {
    pub id: u64,
    pub name: Option<String>,
    pub message: String,
    pub kind: ToastKind,
}
```

### New Toaster Methods

- `upsert(name, message, kind)` — finds existing toast by `name` and updates message + kind in place. If not found, creates new. No auto-dismiss timer (persistent).
- `resolve(name, message, kind)` — same as upsert but starts the 3-second auto-dismiss timer. Used for the final success/error transition.

Existing `success()` and `error()` methods remain unchanged.

## Console Integration

### SSE Listener (`siege-console/src/sse.rs`)

The `use_sse_subscription` hook captures a `Toaster` from context. On `SeedProgress`:

Numbers are formatted in short form (e.g. `13k`, `100k`, `1.2M`) via a `ShortFormat` extension trait on numeric types in the console crate (e.g. `records_generated.short()`).

```rust
SseEvent::SeedProgress { topic, topic_index, total_topics, records_generated, total_records } => {
    let msg = if total_records > 0 {
        format!("Seeding {topic} ({}/{total_topics}) — {}/{} records",
                topic_index + 1,
                records_generated.short(),
                total_records.short())
    } else {
        format!("Creating {topic} ({}/{total_topics})", topic_index + 1)
    };
    toaster.upsert("seed", msg, ToastKind::Progress);
}
```

### Seed Button (`siege-console/src/components/ui/seed_button.rs`)

```rust
onclick: move |_| {
    let client = state.client();
    async move {
        toaster.upsert("seed", "Starting seed...", ToastKind::Progress);
        match client.seed().await {
            Ok(()) => toaster.resolve("seed", "Topics seeded", ToastKind::Success),
            Err(e) => toaster.resolve("seed", format!("Seed failed: {e}"), ToastKind::Error),
        }
    }
}
```

### Event Flow

1. User clicks seed → progress toast appears ("Starting seed...")
2. SSE `SeedProgress` events → toast updates in place ("Seeding winterfell (2/6) — 50k/100k records")
3. POST `/api/seed` returns OK → toast transitions to success with 3s auto-dismiss
4. POST `/api/seed` returns error → toast transitions to error with 3s auto-dismiss

## Crates Modified

| Crate | Change |
|-------|--------|
| `siege` | Add `SeedProgressEvent` + `DomainEvent::SeedProgress` variant |
| `siege-seed` | Add `EventEmitter` field, emit progress in `seed_topics()`/`seed_data()` |
| `siege-api-spec` | Add `SseEvent::SeedProgress` variant |
| `siege-api` | Map new domain event in `Broadcaster`, pass emitter to `Seeder` construction |
| `siege-api-client` | Re-export new `SseEvent` variant (automatic via existing re-export) |
| `siege-console` | Named toast support, SSE listener handles progress, seed button uses upsert/resolve |
