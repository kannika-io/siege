pub mod topics;

use actix_web::web;

use crate::kafka::backend::KafkaBackend;

pub fn configure<K: KafkaBackend>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/topics", web::get().to(topics::list_topics::<K>))
            .route("/topics", web::post().to(topics::create_topic::<K>))
            .route("/topics/{name}", web::get().to(topics::get_topic::<K>))
            .route(
                "/topics/{name}",
                web::delete().to(topics::delete_topic::<K>),
            )
            .route(
                "/topics/{name}/config",
                web::post().to(topics::update_topic_config::<K>),
            )
            .route("/events", web::get().to(topics::events::<K>)),
    );
}
