# Seed Progress Events Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Emit intermediate seed progress events through SSE during seeding and display them in a single updating toast notification.

**Architecture:** New `SeedProgressEvent` flows from the Seeder → domain event → SSE broadcaster → browser EventSource → named toast. The Seeder gets an optional `Arc<dyn EventEmitter>` and emits time-throttled progress during record generation. The toast system gains named-toast support (upsert/resolve) so a single toast updates in place.

**Tech Stack:** Rust, Dioxus (WASM), SSE, tokio broadcast channel

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `crates/siege/src/event.rs` | Add `SeedProgressEvent` struct + `DomainEvent::SeedProgress` variant |
| Modify | `crates/siege-api-spec/src/events.rs` | Add `SseEvent::SeedProgress` variant |
| Modify | `crates/siege-api/src/sse/broadcaster.rs` | Map `DomainEvent::SeedProgress` → `SseEvent::SeedProgress` |
| Modify | `crates/siege-seed/src/lib.rs` | Add `EventEmitter` field, `emit_progress` helper, throttled emission in `seed_data` and `seed_topics` |
| Modify | `crates/siege-api/src/main.rs` | Pass `Arc<Broadcaster>` to `Seeder` construction |
| Create | `crates/siege-console/src/short_format.rs` | `ShortFormat` extension trait for numeric display |
| Modify | `crates/siege-console/src/main.rs` | Declare `mod short_format` |
| Modify | `crates/siege-console/src/components/ui/toast.rs` | Add `ToastKind::Progress`, `name` field, `upsert`/`resolve` methods |
| Modify | `crates/siege-console/src/sse.rs` | Handle `SseEvent::SeedProgress`, update named toast |
| Modify | `crates/siege-console/src/components/ui/seed_button.rs` | Use `upsert`/`resolve` for progress flow |

---

### Task 1: Add SeedProgressEvent domain event

**Files:**
- Modify: `crates/siege/src/event.rs`

- [ ] **Step 1: Add `SeedProgressEvent` struct and `DomainEvent` variant**

In `crates/siege/src/event.rs`, add the struct before `TopicsSeededEvent` and the variant to the enum:

```rust
pub struct SeedProgressEvent {
    pub topic: String,
    pub topic_index: u32,
    pub total_topics: u32,
    pub records_generated: u32,
    pub total_records: u32,
}
```

Add to the `DomainEvent` enum, before the `TopicsSeeded` variant:

```rust
SeedProgress(SeedProgressEvent),
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p siege`
Expected: compiles with no errors (there will be a warning about unmatched variant in broadcaster — that's expected, we fix it in Task 3)

- [ ] **Step 3: Commit**

```bash
git add crates/siege/src/event.rs
git commit -m "feat(siege): add SeedProgressEvent domain event"
```

---

### Task 2: Add SseEvent::SeedProgress variant

**Files:**
- Modify: `crates/siege-api-spec/src/events.rs`

- [ ] **Step 1: Add the variant to `SseEvent`**

In `crates/siege-api-spec/src/events.rs`, add after the `TopicsSeeded` variant:

```rust
SeedProgress {
    topic: String,
    topic_index: u32,
    total_topics: u32,
    records_generated: u32,
    total_records: u32,
},
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p siege-api-spec`
Expected: compiles clean

- [ ] **Step 3: Commit**

```bash
git add crates/siege-api-spec/src/events.rs
git commit -m "feat(siege-api-spec): add SseEvent::SeedProgress variant"
```

---

### Task 3: Map domain event in Broadcaster

**Files:**
- Modify: `crates/siege-api/src/sse/broadcaster.rs`

- [ ] **Step 1: Write a failing test**

Add this test to the existing `mod tests` block in `crates/siege-api/src/sse/broadcaster.rs`:

```rust
#[test]
fn event_emitter_converts_seed_progress() {
    let bc = Broadcaster::new(16);
    let mut rx = bc.subscribe();

    bc.emit(&DomainEvent::SeedProgress(siege::event::SeedProgressEvent {
        topic: "winterfell".into(),
        topic_index: 1,
        total_topics: 6,
        records_generated: 5000,
        total_records: 100_000,
    }));

    let received = rx.try_recv();
    assert!(received.is_ok());
    let event = received.unwrap_or_else(|_| panic!("expected event"));
    assert!(matches!(
        event,
        SseEvent::SeedProgress {
            topic,
            topic_index: 1,
            total_topics: 6,
            records_generated: 5000,
            total_records: 100_000,
        } if topic == "winterfell"
    ));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p siege-api event_emitter_converts_seed_progress`
Expected: compilation error — the `DomainEvent::SeedProgress` arm is not handled in `emit()`

- [ ] **Step 3: Add the mapping in the `EventEmitter` impl**

In the `emit` method's match block in `crates/siege-api/src/sse/broadcaster.rs`, add a new arm before the closing `}`:

```rust
DomainEvent::SeedProgress(e) => {
    self.send(SseEvent::SeedProgress {
        topic: e.topic.clone(),
        topic_index: e.topic_index,
        total_topics: e.total_topics,
        records_generated: e.records_generated,
        total_records: e.total_records,
    });
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p siege-api event_emitter_converts_seed_progress`
Expected: PASS

- [ ] **Step 5: Run full test suite**

Run: `cargo test --workspace --exclude siege-console`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/siege-api/src/sse/broadcaster.rs
git commit -m "feat(siege-api): map SeedProgress domain event to SSE"
```

---

### Task 4: Emit progress events from Seeder

**Files:**
- Modify: `crates/siege-seed/src/lib.rs`

- [ ] **Step 1: Write failing tests**

Add these two tests to the existing `mod tests` in `crates/siege-seed/src/lib.rs`. First, add the test helper at the top of the test module (after `use super::*;`):

```rust
use siege::event::{DomainEvent, EventEmitter, SeedProgressEvent};

struct ProgressRecorder {
    events: std::sync::Mutex<Vec<(String, u32, u32, u32, u32)>>,
}

impl ProgressRecorder {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            events: std::sync::Mutex::new(Vec::new()),
        })
    }

    fn events(&self) -> Vec<(String, u32, u32, u32, u32)> {
        self.events
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }
}

impl EventEmitter for ProgressRecorder {
    fn emit(&self, event: &DomainEvent) {
        if let DomainEvent::SeedProgress(e) = event {
            if let Ok(mut events) = self.events.lock() {
                events.push((
                    e.topic.clone(),
                    e.topic_index,
                    e.total_topics,
                    e.records_generated,
                    e.total_records,
                ));
            }
        }
    }
}
```

Then add the tests:

```rust
#[tokio::test]
async fn seed_emits_progress_per_topic() -> Result<(), SiegeError> {
    let recorder = ProgressRecorder::new();
    let kafka = MockKafkaBackend::new();
    let seeder = Seeder::new(kafka)
        .events(recorder.clone())
        .topic(TopicSeed::new("topic-a", 3))
        .topic(TopicSeed::new("topic-b", 1));

    seeder.seed_topics().await?;

    let events = recorder.events();
    // Should have at least a start event for each topic
    assert!(events.iter().any(|(t, idx, total, _, _)| t == "topic-a" && *idx == 0 && *total == 2));
    assert!(events.iter().any(|(t, idx, total, _, _)| t == "topic-b" && *idx == 1 && *total == 2));
    Ok(())
}

#[tokio::test]
async fn seed_emits_record_progress() -> Result<(), SiegeError> {
    let recorder = ProgressRecorder::new();
    let kafka = MockKafkaBackend::new();
    let schema_str = r#"{"type":"record","name":"Test","namespace":"io.siege.schemas","fields":[{"name":"name","type":"string"}]}"#;
    let seeder = Seeder::new(kafka)
        .events(recorder.clone())
        .schema_registry(NoopSchemaRegistry)
        .topic(TopicSeed::new("data-topic", 1).schema(schema_str).records(10));

    seeder.seed_topics().await?;

    let events = recorder.events();
    // Should have start (records_generated=0) and completion (records_generated=10)
    assert!(events.iter().any(|(t, _, _, gen, total)| t == "data-topic" && *gen == 0 && *total == 10));
    assert!(events.iter().any(|(t, _, _, gen, total)| t == "data-topic" && *gen == 10 && *total == 10));
    Ok(())
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p siege-seed seed_emits_progress`
Expected: compilation error — `Seeder` has no `events` method

- [ ] **Step 3: Add `events` field and builder method**

In `crates/siege-seed/src/lib.rs`, add the import at the top with the other `use` statements:

```rust
use std::sync::Arc;
use siege::event::{DomainEvent, EventEmitter, SeedProgressEvent};
```

Add the field to `Seeder`:

```rust
pub struct Seeder {
    kafka: Box<dyn KafkaBackend>,
    schema_registry: Option<Box<dyn SchemaRegistryBackend>>,
    seeds: Vec<TopicSeed>,
    rng_seed: u64,
    on_complete: Option<tokio::sync::Mutex<Box<dyn DynHook>>>,
    idempotent: bool,
    running: AtomicBool,
    events: Option<Arc<dyn EventEmitter>>,
}
```

Update `new()` to initialize the field:

```rust
events: None,
```

Add the builder method after `on_complete`:

```rust
pub fn events(mut self, emitter: Arc<dyn EventEmitter>) -> Self {
    self.events = Some(emitter);
    self
}
```

Add the helper method on `Seeder` (before the `SeedBackend` impl):

```rust
fn emit_progress(
    &self,
    topic: &str,
    topic_index: u32,
    total_topics: u32,
    records_generated: u32,
    total_records: u32,
) {
    if let Some(events) = &self.events {
        events.emit(&DomainEvent::SeedProgress(SeedProgressEvent {
            topic: topic.to_owned(),
            topic_index,
            total_topics,
            records_generated,
            total_records,
        }));
    }
}
```

- [ ] **Step 4: Add progress emission to `seed_topics`**

In the `seed_topics` method, replace the `for seed in &self.seeds` loop with:

```rust
let total_topics = self.seeds.len() as u32;

for (topic_index, seed) in self.seeds.iter().enumerate() {
    let topic_index = topic_index as u32;
    let total_records = seed.record_count.unwrap_or(0);

    self.emit_progress(&seed.name, topic_index, total_topics, 0, total_records);

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
                self.seed_data(&seed.name, schema_str, count, &mut rng, topic_index, total_topics)
                    .await?;
            }

            self.emit_progress(&seed.name, topic_index, total_topics, total_records, total_records);
        }
        Err(_) => skipped.push(seed.name.clone()),
    }
}
```

- [ ] **Step 5: Add progress emission to `seed_data` with time-based throttle**

Update the `seed_data` signature to accept progress context:

```rust
async fn seed_data(
    &self,
    topic: &str,
    schema_str: &str,
    count: u32,
    rng: &mut StdRng,
    topic_index: u32,
    total_topics: u32,
) -> Result<(), SiegeError> {
```

In the record generation loop inside `seed_data`, add the time-based throttle. Replace:

```rust
for i in 0..count {
    let record = generate_record(&schema, rng)?;
    let payload = serializer.serialize(record)?;
    let key = uuid::Uuid::new_v5(&namespace, &i.to_be_bytes()).to_string();
    records.push((key, payload));
}
```

With:

```rust
let mut last_progress = std::time::Instant::now();
for i in 0..count {
    let record = generate_record(&schema, rng)?;
    let payload = serializer.serialize(record)?;
    let key = uuid::Uuid::new_v5(&namespace, &i.to_be_bytes()).to_string();
    records.push((key, payload));
    if last_progress.elapsed() >= std::time::Duration::from_millis(500) {
        self.emit_progress(topic, topic_index, total_topics, i + 1, count);
        last_progress = std::time::Instant::now();
    }
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p siege-seed seed_emits_progress`
Expected: both tests pass

- [ ] **Step 7: Run full test suite**

Run: `cargo test --workspace --exclude siege-console`
Expected: all tests pass

- [ ] **Step 8: Commit**

```bash
git add crates/siege-seed/src/lib.rs
git commit -m "feat(siege-seed): emit progress events during seeding"
```

---

### Task 5: Wire Broadcaster to Seeder in main

**Files:**
- Modify: `crates/siege-api/src/main.rs`

- [ ] **Step 1: Store broadcaster in an `Arc` and pass to Seeder**

In `crates/siege-api/src/main.rs`, add `use std::sync::Arc;` to the imports.

Change the broadcaster creation (line 83) from:

```rust
let broadcaster = Broadcaster::new(256);
```

To:

```rust
let broadcaster = Arc::new(Broadcaster::new(256));
```

Then add `.events(broadcaster.clone())` to the seeder builder chain. After line 88 (`let mut seeder = Seeder::new(backend.clone())`), the chain should include:

```rust
let mut seeder = Seeder::new(backend.clone())
    .idempotent()
    .events(broadcaster.clone())
    .topic(
```

Update the watcher clone (line 143) — since `broadcaster` is now `Arc<Broadcaster>`, the clone is `Arc::clone`. The existing `.clone()` call works because `Arc<T>` implements `Clone`. No change needed.

Update `broadcaster_data` (line 155) — since `broadcaster` is already `Arc`, change:

```rust
let broadcaster_data = web::Data::new(broadcaster.clone());
```

To:

```rust
let broadcaster_data = web::Data::from(broadcaster.clone());
```

(`web::Data::from` accepts `Arc<T>` directly, avoiding double-wrapping.)

Update the `Siege` struct construction (lines 157-163). The `events` field is type `Broadcaster`, but we have `Arc<Broadcaster>`. Since `Broadcaster` is `Clone`, we can dereference the Arc. Change:

```rust
events: broadcaster,
```

To:

```rust
events: (*broadcaster).clone(),
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p siege-api`
Expected: compiles clean

- [ ] **Step 3: Run full test suite**

Run: `cargo test --workspace --exclude siege-console`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/siege-api/src/main.rs
git commit -m "feat(siege-api): wire broadcaster to seeder for progress events"
```

---

### Task 6: ShortFormat extension trait

**Files:**
- Create: `crates/siege-console/src/short_format.rs`
- Modify: `crates/siege-console/src/main.rs`

- [ ] **Step 1: Create the `ShortFormat` trait**

Create `crates/siege-console/src/short_format.rs`:

```rust
pub trait ShortFormat {
    fn short(&self) -> String;
}

impl ShortFormat for u32 {
    fn short(&self) -> String {
        let n = *self;
        if n >= 1_000_000 {
            let whole = n / 1_000_000;
            let frac = (n % 1_000_000) / 100_000;
            if frac == 0 {
                format!("{whole}M")
            } else {
                format!("{whole}.{frac}M")
            }
        } else if n >= 1_000 {
            let whole = n / 1_000;
            let frac = (n % 1_000) / 100;
            if frac == 0 {
                format!("{whole}k")
            } else {
                format!("{whole}.{frac}k")
            }
        } else {
            format!("{n}")
        }
    }
}
```

- [ ] **Step 2: Declare the module**

In `crates/siege-console/src/main.rs`, add the module declaration alongside the other `mod` statements:

```rust
mod short_format;
```

- [ ] **Step 3: Verify it compiles**

Run: `dx check --package siege-console`
Expected: compiles clean (with possible warnings about unused import — fine, we use it in Task 8)

- [ ] **Step 4: Commit**

```bash
git add crates/siege-console/src/short_format.rs crates/siege-console/src/main.rs
git commit -m "feat(siege-console): add ShortFormat extension trait for numeric display"
```

---

### Task 7: Named toast support

**Files:**
- Modify: `crates/siege-console/src/components/ui/toast.rs`

- [ ] **Step 1: Add `Progress` variant and `name` field**

In `crates/siege-console/src/components/ui/toast.rs`, add the variant to `ToastKind`:

```rust
#[derive(Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Progress,
}
```

Add `name` to `Toast`:

```rust
#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub name: Option<String>,
    pub message: String,
    pub kind: ToastKind,
}
```

Update the `push` method to set `name: None`:

```rust
self.items.write().push(Toast { id, name: None, message, kind });
```

- [ ] **Step 2: Add `upsert` method**

Add to the `impl Toaster` block, after the `error` method:

```rust
pub fn upsert(&mut self, name: impl Into<String>, message: impl Into<String>, kind: ToastKind) {
    let name = name.into();
    let message = message.into();
    let mut items = self.items.write();
    if let Some(toast) = items.iter_mut().find(|t| t.name.as_deref() == Some(&name)) {
        toast.message = message;
        toast.kind = kind;
    } else {
        let id = (self.next_id)();
        self.next_id.set(id + 1);
        items.push(Toast {
            id,
            name: Some(name),
            message,
            kind,
        });
    }
}
```

- [ ] **Step 3: Add `resolve` method**

Add after `upsert`:

```rust
pub fn resolve(&mut self, name: &str, message: impl Into<String>, kind: ToastKind) {
    let message = message.into();
    let id = {
        let mut items = self.items.write();
        if let Some(toast) = items.iter_mut().find(|t| t.name.as_deref() == Some(name)) {
            toast.message = message;
            toast.kind = kind;
            toast.name = None;
            toast.id
        } else {
            let id = (self.next_id)();
            self.next_id.set(id + 1);
            items.push(Toast {
                id,
                name: None,
                message,
                kind,
            });
            id
        }
    };

    let mut items = self.items;
    let cb = Closure::once(move || {
        items.write().retain(|t| t.id != id);
    });
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            3_000,
        );
    }
    cb.forget();
}
```

- [ ] **Step 4: Add Progress styling to `ToastContainer`**

In the `ToastContainer` component, update the match to handle `Progress`:

```rust
let bg = match toast.kind {
    ToastKind::Success => "bg-emerald-600 text-white",
    ToastKind::Error => "bg-destructive text-destructive-foreground",
    ToastKind::Progress => "bg-indigo-600 text-white",
};
```

- [ ] **Step 5: Verify it compiles**

Run: `dx check --package siege-console`
Expected: compiles clean

- [ ] **Step 6: Commit**

```bash
git add crates/siege-console/src/components/ui/toast.rs
git commit -m "feat(siege-console): add named toast support with upsert/resolve"
```

---

### Task 8: SSE listener and seed button integration

**Files:**
- Modify: `crates/siege-console/src/sse.rs`
- Modify: `crates/siege-console/src/components/ui/seed_button.rs`

- [ ] **Step 1: Handle `SeedProgress` in SSE listener**

In `crates/siege-console/src/sse.rs`, add the toaster import and context:

```rust
use crate::components::ui::toast::{ToastKind, Toaster};
use crate::short_format::ShortFormat;
```

Add `let mut toaster = use_context::<Toaster>();` after the existing `use_context` calls:

```rust
pub fn use_sse_subscription() {
    let app = use_context::<AppState>();
    let mut topics_state = use_context::<TopicsState>();
    let mut toaster = use_context::<Toaster>();
```

In the match block inside the `onmessage` closure, add a new arm before the final catch-all:

```rust
SseEvent::SeedProgress { topic, topic_index, total_topics, records_generated, total_records } => {
    let msg = if total_records > 0 {
        format!(
            "Seeding {topic} ({}/{total_topics}) \u{2014} {}/{} records",
            topic_index + 1,
            records_generated.short(),
            total_records.short(),
        )
    } else {
        format!("Creating {topic} ({}/{total_topics})", topic_index + 1)
    };
    toaster.upsert("seed", msg, ToastKind::Progress);
}
```

(The `\u{2014}` is an em dash.)

- [ ] **Step 2: Update seed button to use upsert/resolve**

Replace the full contents of `crates/siege-console/src/components/ui/seed_button.rs` with:

```rust
use dioxus::prelude::*;

use super::icon::{Icon, IconName};
use super::toast::{ToastKind, Toaster};
use crate::state::AppState;

#[component]
pub fn SeedButton() -> Element {
    let state = use_context::<AppState>();
    let mut toaster = use_context::<Toaster>();

    rsx! {
        button {
            class: "self-start w-8 h-8 flex items-center justify-center rounded-full text-sidebar-foreground hover:bg-subtle hover:text-sidebar-active cursor-pointer transition-colors",
            title: "Seed topics",
            onclick: move |_| {
                let client = state.client();
                async move {
                    toaster.upsert("seed", "Starting seed...", ToastKind::Progress);
                    match client.seed().await {
                        Ok(()) => toaster.resolve("seed", "Topics seeded", ToastKind::Success),
                        Err(e) => toaster.resolve("seed", format!("Seed failed: {e}"), ToastKind::Error),
                    }
                }
            },
            Icon { name: IconName::Sprout }
        }
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `dx check --package siege-console`
Expected: compiles clean

- [ ] **Step 4: Run full backend test suite**

Run: `cargo test --workspace --exclude siege-console`
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/siege-console/src/sse.rs crates/siege-console/src/components/ui/seed_button.rs
git commit -m "feat(siege-console): show seed progress in updating toast notification"
```

---

### Task 9: Manual browser verification

- [ ] **Step 1: Start the full stack**

Run: `just api` in one terminal, `just console` in another.

- [ ] **Step 2: Test the seed flow**

1. Open the console in a browser
2. Click the seed button (sprout icon)
3. Verify: a blue/indigo progress toast appears ("Starting seed...")
4. Verify: the toast updates in place as topics are seeded (e.g., "Seeding winterfell (2/6) — 13k/100k records")
5. Verify: on completion, the toast turns green ("Topics seeded") and auto-dismisses after 3 seconds

- [ ] **Step 3: Test error case**

1. Stop the API server
2. Click the seed button
3. Verify: progress toast appears, then transitions to red error toast with the failure message

- [ ] **Step 4: Test rapid clicking**

1. Click the seed button while a seed is already in progress
2. Verify: only one toast is shown (the `AlreadySeeding` error should resolve the existing progress toast to an error)
