use crate::SiegeError;
use crate::kafka::BoxFuture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaId(pub i32);

pub trait SchemaRegistryBackend: Send + Sync + 'static {
    fn register_schema(
        &self,
        subject: &str,
        schema: &str,
    ) -> BoxFuture<'_, Result<SchemaId, SiegeError>>;

    fn delete_subject(
        &self,
        subject: &str,
    ) -> BoxFuture<'_, Result<(), SiegeError>>;
}
