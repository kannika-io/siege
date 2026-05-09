pub mod topics;

use actix_web::web;

use crate::context::SiegeContext;

pub fn configure<C: SiegeContext>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/topics", web::get().to(topics::list_topics::<C>))
            .route("/topics", web::post().to(topics::create_topic::<C>))
            .route("/topics/{name}", web::get().to(topics::get_topic::<C>))
            .route(
                "/topics/{name}",
                web::delete().to(topics::delete_topic::<C>),
            )
            .route(
                "/topics/{name}/config",
                web::post().to(topics::update_topic_config::<C>),
            )
            .route("/events", web::get().to(topics::events::<C>)),
    );
}
