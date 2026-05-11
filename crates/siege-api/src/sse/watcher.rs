use std::collections::HashSet;
use std::time::Duration;

use siege::kafka::KafkaBackend;
use siege::KafkaProperties;
use siege_api_spec::{SseEvent, TopicResource};

use super::broadcaster::Broadcaster;

pub async fn watch_cluster<K: KafkaBackend>(
    backend: &K,
    broadcaster: &Broadcaster,
    interval: Duration,
) {
    let mut known_names: HashSet<String> = HashSet::new();
    let mut ticker = tokio::time::interval(interval);

    loop {
        ticker.tick().await;

        let Ok(topics) = backend.list_topics().await else {
            continue;
        };

        let current_names: HashSet<String> = topics.iter().map(|t| t.name.clone()).collect();

        for topic in &topics {
            if !known_names.contains(&topic.name) {
                broadcaster.send(SseEvent::TopicCreated {
                    topic: TopicResource {
                        name: topic.name.clone(),
                        partitions: topic.partitions,
                        replication_factor: topic.replication_factor,
                    },
                });
            }
        }

        for name in &known_names {
            if !current_names.contains(name) {
                broadcaster.send(SseEvent::TopicDeleted {
                    name: name.clone(),
                });
            }
        }

        known_names = current_names;
    }
}

#[cfg(test)]
mod tests {
    use siege::TopicDetail;
    use siege::KafkaProperties;

    use super::*;
    use siege::MockKafkaBackend;

    #[tokio::test]
    async fn watcher_detects_new_topic() {
        let backend = MockKafkaBackend::new();
        let broadcaster = Broadcaster::new(16);
        let mut rx = broadcaster.subscribe();

        let bc = broadcaster.clone();
        let b = backend.clone();
        let handle = tokio::spawn(async move {
            watch_cluster(&b, &bc, Duration::from_millis(50)).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        backend.create_topic("new-topic", 1, 1, KafkaProperties::new()).await.unwrap();

        let event = tokio::time::timeout(Duration::from_millis(200), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(event, SseEvent::TopicCreated { topic } if topic.name == "new-topic"));

        handle.abort();
    }

    #[tokio::test]
    async fn watcher_detects_deleted_topic() {
        let backend = MockKafkaBackend::with_topics(vec![TopicDetail {
            name: "doomed".into(),
            partitions: 1,
            replication_factor: 1,
            config: KafkaProperties::new(),
        }]);
        let broadcaster = Broadcaster::new(16);
        let mut rx = broadcaster.subscribe();

        let bc = broadcaster.clone();
        let b = backend.clone();
        let handle = tokio::spawn(async move {
            watch_cluster(&b, &bc, Duration::from_millis(50)).await;
        });

        let event = tokio::time::timeout(Duration::from_millis(200), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(event, SseEvent::TopicCreated { .. }));

        backend.delete_topic("doomed").await.unwrap();

        let event = tokio::time::timeout(Duration::from_millis(200), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(event, SseEvent::TopicDeleted { name } if name == "doomed"));

        handle.abort();
    }
}
