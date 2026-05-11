use std::collections::HashMap;
use std::time::Duration;

use siege::kafka::KafkaBackend;
use siege_api_spec::{SseEvent, TopicResource};

use super::broadcaster::Broadcaster;

fn to_resource(t: &siege::kafka::TopicMeta) -> TopicResource {
    TopicResource {
        name: t.name.clone(),
        partitions: t.partitions,
        replication_factor: t.replication_factor,
        config: t.config.clone(),
    }
}

pub async fn watch_cluster<K: KafkaBackend>(
    backend: &K,
    broadcaster: &Broadcaster,
    interval: Duration,
) {
    let mut known: HashMap<String, TopicResource> = HashMap::new();
    let mut ticker = tokio::time::interval(interval);

    loop {
        ticker.tick().await;

        let Ok(topics) = backend.list_topics().await else {
            continue;
        };

        let current: HashMap<String, TopicResource> = topics
            .iter()
            .map(|t| (t.name.clone(), to_resource(t)))
            .collect();

        for (name, resource) in &current {
            match known.get(name) {
                None => {
                    broadcaster.send(SseEvent::TopicCreated {
                        topic: resource.clone(),
                    });
                }
                Some(prev) if prev != resource => {
                    broadcaster.send(SseEvent::TopicUpdated {
                        topic: resource.clone(),
                    });
                }
                _ => {}
            }
        }

        for name in known.keys() {
            if !current.contains_key(name) {
                broadcaster.send(SseEvent::TopicDeleted {
                    name: name.clone(),
                });
            }
        }

        known = current;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap as StdHashMap;

    use siege::kafka::TopicDetail;
    use siege::KafkaProperties;
    use siege::MockKafkaBackend;

    use super::*;

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

    #[tokio::test]
    async fn watcher_detects_config_change() {
        let backend = MockKafkaBackend::with_topics(vec![TopicDetail {
            name: "t".into(),
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

        let config: KafkaProperties =
            StdHashMap::from([("cleanup.policy".into(), "compact".into())]).into();
        backend.update_topic_config("t", config).await.unwrap();

        let event = tokio::time::timeout(Duration::from_millis(200), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(event, SseEvent::TopicUpdated { topic } if topic.config.is_compacted()));

        handle.abort();
    }
}
