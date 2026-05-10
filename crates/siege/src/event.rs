use crate::{Topic, TopicDeleted};

pub enum DomainEvent {
    TopicCreated { topic: Topic },
    TopicDeleted { name: String },
}

impl From<TopicDeleted> for DomainEvent {
    fn from(e: TopicDeleted) -> Self {
        DomainEvent::TopicDeleted { name: e.name }
    }
}

pub trait EventEmitter: Send + Sync + 'static {
    fn emit(&self, event: &DomainEvent);
}
