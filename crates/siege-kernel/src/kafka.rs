use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(transparent)]
pub struct KafkaProperties(HashMap<String, String>);

impl KafkaProperties {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn is_compacted(&self) -> bool {
        self.0
            .get("cleanup.policy")
            .is_some_and(|v| v.contains("compact"))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }

    pub fn extend(&mut self, other: KafkaProperties) {
        self.0.extend(other.0);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.0.iter()
    }
}

impl Default for KafkaProperties {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, String>> for KafkaProperties {
    fn from(map: HashMap<String, String>) -> Self {
        Self(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_properties() {
        let props = KafkaProperties::new();
        assert!(props.is_empty());
        assert!(!props.is_compacted());
    }

    #[test]
    fn compacted_detection() {
        let props: KafkaProperties =
            HashMap::from([("cleanup.policy".into(), "compact".into())]).into();
        assert!(props.is_compacted());
    }

    #[test]
    fn serde_roundtrip() {
        let props: KafkaProperties =
            HashMap::from([("retention.ms".into(), "86400000".into())]).into();
        let json = serde_json::to_string(&props).unwrap();
        let deserialized: KafkaProperties = serde_json::from_str(&json).unwrap();
        assert_eq!(props, deserialized);
    }
}
