use serde::{Deserialize, Serialize};
use siege_kernel::KafkaProperties;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Topic {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct TopicDetail {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
    pub config: KafkaProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTopicRequest {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TopicConfigUpdate {
    pub config: KafkaProperties,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn topic_roundtrip() {
        let topic = Topic {
            name: "test-topic".into(),
            partitions: 6,
            replication_factor: 3,
        };
        let json = serde_json::to_string(&topic).unwrap();
        let deserialized: Topic = serde_json::from_str(&json).unwrap();
        assert_eq!(topic, deserialized);
    }

    #[test]
    fn topic_detail_roundtrip() {
        let detail = TopicDetail {
            name: "test-topic".into(),
            partitions: 6,
            replication_factor: 3,
            config: HashMap::from([("retention.ms".into(), "86400000".into())]).into(),
        };
        let json = serde_json::to_string(&detail).unwrap();
        let deserialized: TopicDetail = serde_json::from_str(&json).unwrap();
        assert_eq!(detail, deserialized);
    }

    #[test]
    fn create_topic_request_from_json() {
        let json = r#"{"name":"new-topic","partitions":3,"replication_factor":1}"#;
        let req: CreateTopicRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "new-topic");
        assert_eq!(req.partitions, 3);
        assert_eq!(req.replication_factor, 1);
    }

    #[test]
    fn topic_config_update_from_json() {
        let json = r#"{"config":{"retention.ms":"1000","cleanup.policy":"compact"}}"#;
        let update: TopicConfigUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.config.get("retention.ms").unwrap(), "1000");
        assert_eq!(update.config.get("cleanup.policy").unwrap(), "compact");
    }
}
