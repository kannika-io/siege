use std::any::Any;

pub trait DomainEvent: Any + Send + Sync {
    fn event_name(&self) -> &'static str;
}

pub trait EventEmitter: Send + Sync + 'static {
    fn emit(&self, event: &dyn DomainEvent);
}
