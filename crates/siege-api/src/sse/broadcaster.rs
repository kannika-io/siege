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

fn detail_to_resource(d: &siege::kafka::TopicDetail) -> TopicResource {
    TopicResource {
        name: d.name.clone(),
        partitions: d.partitions,
        replication_factor: d.replication_factor,
        config: d.config.clone(),
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
            DomainEvent::TopicsSeeded(e) => {
                self.send(SseEvent::TopicsSeeded {
                    topics: e.topics.iter().map(detail_to_resource).collect(),
                });
            }
            DomainEvent::ChaosTopicDeleted(e) => {
                self.send(SseEvent::ChaosTopicDeleted { topic: e.topic.clone() });
            }
            DomainEvent::ChaosRetentionLowered(e) => {
                self.send(SseEvent::ChaosRetentionLowered { topic: detail_to_resource(&e.detail) });
            }
            DomainEvent::ChaosCleanupPolicyFlipped(e) => {
                self.send(SseEvent::ChaosCleanupPolicyFlipped { topic: detail_to_resource(&e.detail) });
            }
            DomainEvent::ChaosPartitionsIncreased(e) => {
                self.send(SseEvent::ChaosPartitionsIncreased { topic: detail_to_resource(&e.detail) });
            }
            DomainEvent::ChaosPoisonPillsSent(e) => {
                self.send(SseEvent::ChaosPoisonPillsSent { topic: e.topic.clone(), count: e.count });
            }
            DomainEvent::ChaosSchemaBreakSent(e) => {
                self.send(SseEvent::ChaosSchemaBreakSent { topic: e.topic.clone(), count: e.count });
            }
            DomainEvent::SeedProgress(e) => {
                self.send(SseEvent::SeedProgress {
                    topic: e.topic.clone(),
                    topic_index: e.topic_index,
                    total_topics: e.total_topics,
                    records_generated: e.records_generated,
                    total_records: e.total_records,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use siege::event::{SeedProgressEvent, TopicCreatedEvent, TopicDeletedEvent};

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
    fn event_emitter_converts_seed_progress() {
        let bc = Broadcaster::new(16);
        let mut rx = bc.subscribe();

        bc.emit(&DomainEvent::SeedProgress(SeedProgressEvent {
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
