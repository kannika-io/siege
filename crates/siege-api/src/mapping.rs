use siege::SiegeContext;
use siege_api_spec::{TopicDetailResource, TopicResource};
use siege::KafkaProperties;

pub fn topic_to_resource<C: SiegeContext>(t: &siege::Topic<'_, C>) -> TopicResource {
    TopicResource {
        name: t.name.clone(),
        partitions: t.partitions,
        replication_factor: t.replication_factor,
    }
}

pub fn topic_to_detail_resource<C: SiegeContext>(
    t: &siege::Topic<'_, C>,
    config: KafkaProperties,
) -> TopicDetailResource {
    TopicDetailResource {
        name: t.name.clone(),
        partitions: t.partitions,
        replication_factor: t.replication_factor,
        config,
    }
}
