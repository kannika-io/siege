use siege::event::DomainEvent;
use siege::EventEmitter;
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
    fn emit(&self, event: &DomainEvent) {
        match event {
            DomainEvent::TopicCreated(e) => {
                self.send(SseEvent::TopicCreated {
                    topic: TopicResource {
                        name: e.name.clone(),
                        partitions: e.partitions,
                        replication_factor: e.replication_factor,
                        config: e.config.clone(),
                    },
                });
            }
            DomainEvent::TopicDeleted(e) => {
                self.send(SseEvent::TopicDeleted {
                    name: e.name.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use siege::event::{TopicCreatedEvent, TopicDeletedEvent};

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
        let bc = Broadcaster::new(16);
        let mut rx = bc.subscribe();

        bc.emit(&DomainEvent::TopicCreated(TopicCreatedEvent {
            name: "new-topic".into(),
            partitions: 3,
            replication_factor: 1,
            config: siege::KafkaProperties::new(),
        }));

        let received = rx.try_recv().unwrap();
        assert!(matches!(received, SseEvent::TopicCreated { topic } if topic.name == "new-topic"));
    }

    #[test]
    fn event_emitter_converts_topic_deleted() {
        let bc = Broadcaster::new(16);
        let mut rx = bc.subscribe();

        bc.emit(&DomainEvent::TopicDeleted(TopicDeletedEvent {
            name: "gone".into(),
        }));

        let received = rx.try_recv().unwrap();
        assert!(matches!(received, SseEvent::TopicDeleted { name } if name == "gone"));
    }
}
