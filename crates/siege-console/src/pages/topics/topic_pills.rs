use dioxus::prelude::*;
use siege_api_client::KafkaProperties;

use crate::components::ui::icon::IconName;
use crate::components::ui::pill::{Pill, PillVariant};

#[component]
pub fn TopicPills(
    partitions: i32,
    replication_factor: i32,
    #[props(default)] config: KafkaProperties,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-2",
            Pill { variant: PillVariant::Accent, icon: IconName::Layers, "{partitions} partitions" }
            Pill { variant: PillVariant::Destructive, icon: IconName::Copy, "RF {replication_factor}" }
            if config.is_compacted() {
                Pill { icon: IconName::Compress, "Compacted" }
            }
        }
    }
}
