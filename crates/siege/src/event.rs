use std::any::Any;

pub trait DomainEvent: Any + Send + Sync {
    fn event_name(&self) -> &'static str;
}
