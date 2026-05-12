pub mod chaos;
pub mod seed;
pub mod topics;

use actix_web::web;

use siege::SiegeContext;

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
            .route("/events", web::get().to(topics::events::<C>))
            .route(
                "/chaos/delete-topic",
                web::post().to(chaos::delete_topic::<C>),
            )
            .route(
                "/chaos/zero-retention",
                web::post().to(chaos::zero_retention::<C>),
            )
            .route(
                "/chaos/flip-cleanup-policy",
                web::post().to(chaos::flip_cleanup_policy::<C>),
            )
            .route(
                "/chaos/increase-partitions",
                web::post().to(chaos::increase_partitions::<C>),
            )
            .route(
                "/chaos/poison-pills",
                web::post().to(chaos::poison_pills::<C>),
            )
            .route(
                "/chaos/schema-break",
                web::post().to(chaos::schema_break::<C>),
            )
            .route("/seed", web::post().to(seed::run_seed::<C>)),
    );
}
