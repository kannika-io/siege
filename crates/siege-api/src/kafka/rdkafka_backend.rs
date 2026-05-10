use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use rdkafka::admin::{
    AdminClient, AdminOptions, AlterConfig, NewTopic, ResourceSpecifier, TopicReplication,
};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use siege::kafka::KafkaBackend;
use siege::{SiegeError, Topic, TopicDetail};
use siege::KafkaProperties;

#[derive(Clone)]
pub struct RdKafkaBackend {
    admin: Arc<AdminClient<DefaultClientContext>>,
}

impl RdKafkaBackend {
    pub fn new(bootstrap_servers: &str) -> Self {
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .create()
            .expect("failed to create Kafka admin client");
        Self {
            admin: Arc::new(admin),
        }
    }
}

impl KafkaBackend for RdKafkaBackend {
    fn list_topics(&self) -> impl Future<Output = Result<Vec<Topic>, SiegeError>> + Send {
        let admin = self.admin.clone();
        async move {
            let metadata = admin
                .inner()
                .fetch_metadata(None, Duration::from_secs(10))
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            let topics = metadata
                .topics()
                .iter()
                .filter(|t| !t.name().starts_with("__"))
                .map(|t| Topic {
                    name: t.name().to_owned(),
                    partitions: t.partitions().len() as i32,
                    replication_factor: t
                        .partitions()
                        .first()
                        .map(|p| p.replicas().len() as i32)
                        .unwrap_or(0),
                })
                .collect();

            Ok(topics)
        }
    }

    fn get_topic(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<TopicDetail, SiegeError>> + Send {
        let admin = self.admin.clone();
        let name = name.to_owned();
        async move {
            let metadata = admin
                .inner()
                .fetch_metadata(None, Duration::from_secs(10))
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            let topic_meta = metadata
                .topics()
                .iter()
                .find(|t| t.name() == name)
                .ok_or_else(|| SiegeError::TopicNotFound(name.clone()))?;

            let partitions = topic_meta.partitions().len() as i32;
            let replication_factor = topic_meta
                .partitions()
                .first()
                .map(|p| p.replicas().len() as i32)
                .unwrap_or(0);

            drop(metadata);

            let configs = admin
                .describe_configs(
                    &[ResourceSpecifier::Topic(&name)],
                    &AdminOptions::new(),
                )
                .await
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            let mut config = KafkaProperties::new();
            for result in configs {
                if let Ok(resource) = result {
                    for entry in resource.entries {
                        if let Some(value) = entry.value {
                            config.insert(entry.name, value);
                        }
                    }
                }
            }

            Ok(TopicDetail {
                name: name.clone(),
                partitions,
                replication_factor,
                config,
            })
        }
    }

    fn create_topic(
        &self,
        name: &str,
        partitions: i32,
        replication_factor: i32,
        config: KafkaProperties,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send {
        let admin = self.admin.clone();
        let name = name.to_owned();
        async move {
            let mut new_topic = NewTopic::new(
                &name,
                partitions,
                TopicReplication::Fixed(replication_factor),
            );
            for (key, value) in config.iter() {
                new_topic = new_topic.set(key, value);
            }

            let results = admin
                .create_topics(&[new_topic], &AdminOptions::new())
                .await
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            for result in results {
                result.map_err(|(_, code)| {
                    SiegeError::KafkaError(format!("create topic failed: {:?}", code))
                })?;
            }

            Ok(())
        }
    }

    fn delete_topic(&self, name: &str) -> impl Future<Output = Result<(), SiegeError>> + Send {
        let admin = self.admin.clone();
        let name = name.to_owned();
        async move {
            let results = admin
                .delete_topics(&[&name], &AdminOptions::new())
                .await
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            for result in results {
                result.map_err(|(_, code)| {
                    SiegeError::KafkaError(format!("delete topic failed: {:?}", code))
                })?;
            }

            Ok(())
        }
    }

    fn update_topic_config(
        &self,
        name: &str,
        config: KafkaProperties,
    ) -> impl Future<Output = Result<(), SiegeError>> + Send {
        let admin = self.admin.clone();
        let name = name.to_owned();
        async move {
            let mut alter = AlterConfig::new(ResourceSpecifier::Topic(&name));
            for (key, value) in config.iter() {
                alter = alter.set(key, value);
            }

            let results = admin
                .alter_configs(&[alter], &AdminOptions::new())
                .await
                .map_err(|e| SiegeError::KafkaError(e.to_string()))?;

            for result in results {
                result.map_err(|(_, code)| {
                    SiegeError::KafkaError(format!("alter config failed: {:?}", code))
                })?;
            }

            Ok(())
        }
    }
}
