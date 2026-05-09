use siege_api_spec::{TopicDetailResource, TopicResource};
use siege_core::{Topic, TopicDetail};

pub fn topic_to_resource(t: Topic) -> TopicResource {
    TopicResource {
        name: t.name,
        partitions: t.partitions,
        replication_factor: t.replication_factor,
    }
}

pub fn detail_to_resource(d: TopicDetail) -> TopicDetailResource {
    TopicDetailResource {
        name: d.name,
        partitions: d.partitions,
        replication_factor: d.replication_factor,
        config: d.config,
    }
}
