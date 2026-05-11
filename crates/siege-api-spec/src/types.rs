use serde::{Deserialize, Serialize};
use siege_kernel::KafkaProperties;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct TopicResource {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
    #[serde(default)]
    pub config: KafkaProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct TopicDetailResource {
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
    #[serde(default)]
    pub config: KafkaProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TopicConfigUpdateRequest {
    pub config: KafkaProperties,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn topic_resource_roundtrip() {
        let topic = TopicResource {
            name: "test-topic".into(),
            partitions: 6,
            replication_factor: 3,
            config: KafkaProperties::new(),
        };
        let json = serde_json::to_string(&topic).unwrap();
        let deserialized: TopicResource = serde_json::from_str(&json).unwrap();
        assert_eq!(topic, deserialized);
    }

    #[test]
    fn topic_detail_resource_roundtrip() {
        let detail = TopicDetailResource {
            name: "test-topic".into(),
            partitions: 6,
            replication_factor: 3,
            config: HashMap::from([("retention.ms".into(), "86400000".into())]).into(),
        };
        let json = serde_json::to_string(&detail).unwrap();
        let deserialized: TopicDetailResource = serde_json::from_str(&json).unwrap();
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
        let update: TopicConfigUpdateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(update.config.get("retention.ms").unwrap(), "1000");
        assert_eq!(update.config.get("cleanup.policy").unwrap(), "compact");
    }
}
