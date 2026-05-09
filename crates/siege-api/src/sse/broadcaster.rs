use std::any::Any;

use siege::{DomainEvent, EventEmitter, TopicCreated, TopicDeleted};
use siege_api_spec::{SseEvent, TopicResource};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Broadcaster {
    tx: broadcast::Sender<SseEvent>,
}

impl Broadcaster {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SseEvent> {
        self.tx.subscribe()
    }

    pub fn send(&self, event: SseEvent) {
        let _ = self.tx.send(event);
    }
}

impl EventEmitter for Broadcaster {
    fn emit(&self, event: &dyn DomainEvent) {
        let any = event as &dyn Any;
        if let Some(created) = any.downcast_ref::<TopicCreated>() {
            self.send(SseEvent::TopicCreated {
                topic: TopicResource {
                    name: created.topic.name.clone(),
                    partitions: created.topic.partitions,
                    replication_factor: created.topic.replication_factor,
                },
            });
        } else if let Some(deleted) = any.downcast_ref::<TopicDeleted>() {
            self.send(SseEvent::TopicDeleted {
                name: deleted.name.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn broadcaster_delivers_events() {
        let bc = Broadcaster::new(16);
        let mut rx = bc.subscribe();

        let event = SseEvent::TopicDeleted {
            name: "gone".into(),
        };
        bc.send(event.clone());

        let received = rx.recv().await.unwrap();
        assert!(matches!(received, SseEvent::TopicDeleted { name } if name == "gone"));
    }

    #[test]
    fn send_without_subscribers_does_not_panic() {
        let bc = Broadcaster::new(16);
        bc.send(SseEvent::TopicsSnapshot { topics: vec![] });
    }

    #[test]
    fn event_emitter_converts_topic_created() {
        use siege::{DomainEvent, EventEmitter, Topic, TopicCreated};

        let bc = Broadcaster::new(16);
        let mut rx = bc.subscribe();

        let event = TopicCreated {
            topic: Topic {
                name: "new-topic".into(),
                partitions: 3,
                replication_factor: 1,
            },
        };
        bc.emit(&event);

        let received = rx.try_recv().unwrap();
        assert!(matches!(received, SseEvent::TopicCreated { topic } if topic.name == "new-topic"));
    }

    #[test]
    fn event_emitter_converts_topic_deleted() {
        use siege::{EventEmitter, TopicDeleted};

        let bc = Broadcaster::new(16);
        let mut rx = bc.subscribe();

        let event = TopicDeleted {
            name: "gone".into(),
        };
        bc.emit(&event);

        let received = rx.try_recv().unwrap();
        assert!(matches!(received, SseEvent::TopicDeleted { name } if name == "gone"));
    }
}
