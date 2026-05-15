use std::fmt;
use std::future::Future;

use crate::kafka::TopicDetail;

pub trait ChaosBackend: Send + Sync + 'static {
    type Error: fmt::Display + fmt::Debug + Send + Sync + 'static;

    fn get_topic(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<TopicDetail, Self::Error>> + Send;
    fn delete_topic(
        &self,
        topic: &str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn low_retention(
        &self,
        topic: &str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn flip_cleanup_policy(
        &self,
        topic: &str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn increase_partitions(
        &self,
        topic: &str,
        extra: i32,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn poison_pills(
        &self,
        topic: &str,
        count: u32,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn schema_break(
        &self,
        topic: &str,
        count: u32,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
