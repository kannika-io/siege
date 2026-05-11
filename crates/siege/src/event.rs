pub struct TopicCreatedEvent {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
}

pub struct TopicDeletedEvent {
    pub name: String,
}

pub enum DomainEvent {
    TopicCreated(TopicCreatedEvent),
    TopicDeleted(TopicDeletedEvent),
}

pub trait EventEmitter: Send + Sync + 'static {
    fn emit(&self, event: &DomainEvent);
}
