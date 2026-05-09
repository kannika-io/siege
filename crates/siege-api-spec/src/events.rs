use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::types::Topic;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    TopicsSnapshot { topics: Vec<Topic> },
    TopicCreated { topic: Topic },
    TopicDeleted { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_event_serializes_with_tag() {
        let event = SseEvent::TopicsSnapshot {
            topics: vec![Topic {
                name: "test".into(),
                partitions: 3,
                replication_factor: 1,
            }],
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"topics_snapshot\""));
        let deserialized: SseEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, SseEvent::TopicsSnapshot { .. }));
    }

    #[test]
    fn deleted_event_serializes() {
        let event = SseEvent::TopicDeleted {
            name: "dead-topic".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"topic_deleted\""));
        assert!(json.contains("\"name\":\"dead-topic\""));
    }
}
