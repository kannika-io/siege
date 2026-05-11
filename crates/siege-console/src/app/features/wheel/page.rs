use dioxus::prelude::*;
use crate::chaos_action::ChaosAction;
use crate::components::ui::toast::Toaster;
use crate::state::AppState;

use super::{SpinButton, Wheel, WheelSlice};

#[derive(Clone, PartialEq)]
enum WheelOutcome {
    SingleTopic(ChaosAction),
    AllTopics(ChaosAction),
}

fn wheel_slices() -> Vec<WheelSlice<WheelOutcome>> {
    vec![
        WheelSlice {
            label: "Delete topic".into(),
            color: "#dc2626".into(),
            payload: WheelOutcome::SingleTopic(ChaosAction::Delete),
        },
        WheelSlice {
            label: "Zero retention".into(),
            color: "#b45309".into(),
            payload: WheelOutcome::SingleTopic(ChaosAction::ZeroRetention),
        },
        WheelSlice {
            label: "Flip cleanup".into(),
            color: "#d97706".into(),
            payload: WheelOutcome::SingleTopic(ChaosAction::FlipCleanupPolicy),
        },
        WheelSlice {
            label: "Add partition".into(),
            color: "#f59e0b".into(),
            payload: WheelOutcome::SingleTopic(ChaosAction::IncreasePartitions),
        },
        WheelSlice {
            label: "Poison pills".into(),
            color: "#c2410c".into(),
            payload: WheelOutcome::SingleTopic(ChaosAction::PoisonPills),
        },
        WheelSlice {
            label: "Schema break".into(),
            color: "#ea580c".into(),
            payload: WheelOutcome::SingleTopic(ChaosAction::SchemaBreak),
        },
        WheelSlice {
            label: "Delete ALL".into(),
            color: "#991b1b".into(),
            payload: WheelOutcome::AllTopics(ChaosAction::Delete),
        },
        WheelSlice {
            label: "Zero ret. ALL".into(),
            color: "#b91c1c".into(),
            payload: WheelOutcome::AllTopics(ChaosAction::ZeroRetention),
        },
        WheelSlice {
            label: "Poison ALL".into(),
            color: "#ef4444".into(),
            payload: WheelOutcome::AllTopics(ChaosAction::PoisonPills),
        },
    ]
}

#[component]
pub fn WheelOfChaosPage() -> Element {
    let state = use_context::<AppState>();
    let mut toaster = use_context::<Toaster>();

    let angle = use_signal(|| 0.0_f64);
    let angular_velocity = use_signal(|| 0.0_f64);
    let spinning = use_signal(|| false);

    let slices = wheel_slices();

    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center gap-6 p-8",
            h1 { class: "text-xl font-bold text-foreground", "Wheel of Chaos" }

            Wheel {
                slices: slices.clone(),
                angle: angle,
                angular_velocity: angular_velocity,
                spinning: spinning,
                on_result: move |outcome: WheelOutcome| {
                    let client = state.client();
                    spawn(async move {
                        let topics = match client.list_topics().await {
                            Ok(t) => t,
                            Err(e) => {
                                toaster.error(format!("Failed to fetch topics: {e}"));
                                return;
                            }
                        };
                        if topics.is_empty() {
                            toaster.error("No topics available");
                            return;
                        }
                        match outcome {
                            WheelOutcome::SingleTopic(action) => {
                                let idx = (js_sys::Math::random() * topics.len() as f64) as usize;
                                let topic_name = &topics[idx % topics.len()].name;
                                let topic = client.topic(topic_name);
                                if action.execute(&topic).await.is_ok() {
                                    toaster.success(action.success_message(topic_name));
                                } else {
                                    toaster.error(format!("Failed: {}", action.label()));
                                }
                            }
                            WheelOutcome::AllTopics(action) => {
                                let count = topics.len();
                                let mut errors = 0;
                                for t in &topics {
                                    let topic = client.topic(&t.name);
                                    if action.execute(&topic).await.is_err() {
                                        errors += 1;
                                    }
                                }
                                if errors == 0 {
                                    toaster.success(format!(
                                        "{} \u{2192} {} topics",
                                        action.label(),
                                        count
                                    ));
                                } else {
                                    toaster.error(format!(
                                        "{} \u{2192} {}/{} failed",
                                        action.label(),
                                        errors,
                                        count
                                    ));
                                }
                            }
                        }
                    });
                },
            }

            SpinButton {
                angular_velocity: angular_velocity,
                spinning: spinning,
            }
        }
    }
}
