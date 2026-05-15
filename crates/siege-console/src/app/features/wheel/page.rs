use dioxus::prelude::*;
use crate::chaos_action::ChaosAction;
use crate::components::ui::modal::Modal;
use crate::components::ui::toast::Toaster;
use crate::state::AppState;

use super::{SpinButton, Wheel, WheelSlice};

#[derive(Clone, PartialEq)]
struct WheelOutcome {
    action: ChaosAction,
    all_topics: bool,
    label: String,
    description: String,
}

fn wheel_slices() -> Vec<WheelSlice<WheelOutcome>> {
    vec![
        WheelSlice {
            label: "Angry coworker".into(),
            color: "#e11d48".into(),
            payload: WheelOutcome {
                action: ChaosAction::Delete,
                all_topics: false,
                label: "Angry coworker".into(),
                description: "A coworker just got fired for replying-all to the CEO with a meme. On his way out he rage-deleted {topic} and walked straight past security. Nobody noticed until the alerts started.".into(),
            },
        },
        WheelSlice {
            label: "Terraform gone wrong".into(),
            color: "#7c3aed".into(),
            payload: WheelOutcome {
                action: ChaosAction::LowRetention,
                all_topics: false,
                label: "Terraform gone wrong".into(),
                description: "Someone copy-pasted a Terraform module from Stack Overflow and set retention to 1ms on {topic} \"because the example did it.\" The plan said 1 resource changed, the reality said otherwise.".into(),
            },
        },
        WheelSlice {
            label: "Oops wrong tab".into(),
            color: "#0891b2".into(),
            payload: WheelOutcome {
                action: ChaosAction::FlipCleanupPolicy,
                all_topics: false,
                label: "Oops wrong tab".into(),
                description: "You had 47 browser tabs open and picked the wrong one. Turns out that wasn't staging. The cleanup policy on {topic} is now backwards and your Slack is already blowing up.".into(),
            },
        },
        WheelSlice {
            label: "Intern's first day".into(),
            color: "#16a34a".into(),
            payload: WheelOutcome {
                action: ChaosAction::IncreasePartitions,
                all_topics: false,
                label: "Intern's first day".into(),
                description: "The intern googled \"how to make Kafka faster\" and the first answer said \"add more partitions.\" They added one to {topic}. Their mentor returns from lunch in 45 minutes.".into(),
            },
        },
        WheelSlice {
            label: "Friday deploy".into(),
            color: "#ca8a04".into(),
            payload: WheelOutcome {
                action: ChaosAction::PoisonPills,
                all_topics: false,
                label: "Friday deploy".into(),
                description: "It's 4:58 PM on a Friday and someone merged straight to main with the message \"fix: final final v2.\" {topic} is now full of garbage. The author's laptop is already in their backpack.".into(),
            },
        },
        WheelSlice {
            label: "YOLO migration".into(),
            color: "#ea580c".into(),
            payload: WheelOutcome {
                action: ChaosAction::SchemaBreak,
                all_topics: false,
                label: "YOLO migration".into(),
                description: "\"We don't need a schema registry\" said the tech lead, mass-deploying a breaking Avro change to {topic}. Every consumer is now throwing deserialization errors. The tech lead is \"working from home\" today.".into(),
            },
        },
        WheelSlice {
            label: "Nuke from orbit".into(),
            color: "#dc2626".into(),
            payload: WheelOutcome {
                action: ChaosAction::Delete,
                all_topics: true,
                label: "Nuke from orbit".into(),
                description: "The CTO said \"let's simplify the architecture\" during an all-hands and someone took it as a direct order. Every single topic has been deleted. The Kafka cluster is now a very expensive empty box.".into(),
            },
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
    let mut result_modal = use_signal(|| None::<(String, String, String)>);

    let slices = wheel_slices();

    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center gap-6 p-4",
            h1 { class: "text-xl font-bold text-foreground", "Wheel of Chaos" }

            Wheel {
                slices: slices.clone(),
                angle: angle,
                angular_velocity: angular_velocity,
                spinning: spinning,
                on_spin_start: move |_| {
                    result_modal.set(None);
                },
                on_result: move |outcome: WheelOutcome| {
                    let client = state.client();
                    let label = outcome.label.clone();
                    let description = outcome.description.clone();
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
                        let affected_topic;
                        let success;
                        if outcome.all_topics {
                            let count = topics.len();
                            let mut errors = 0;
                            for t in &topics {
                                let topic = client.topic(&t.name);
                                if let Err(e) = outcome.action.execute(&topic).await {
                                    web_sys::console::error_1(&format!("chaos: {} on {} failed: {e}", outcome.action.label(), t.name).into());
                                    errors += 1;
                                }
                            }
                            if errors == 0 {
                                toaster.success(format!(
                                    "{} \u{2192} {} topics",
                                    outcome.action.label(),
                                    count
                                ));
                            } else {
                                toaster.error(format!(
                                    "{} \u{2192} {}/{} failed",
                                    outcome.action.label(),
                                    errors,
                                    count
                                ));
                            }
                            affected_topic = format!("{count} topics");
                            success = errors == 0;
                        } else {
                            let idx = (js_sys::Math::random() * topics.len() as f64) as usize;
                            let topic_name = &topics[idx % topics.len()].name;
                            let topic = client.topic(topic_name);
                            match outcome.action.execute(&topic).await {
                                Ok(()) => {
                                    toaster.success(outcome.action.success_message(topic_name));
                                    success = true;
                                }
                                Err(e) => {
                                    toaster.error(format!("Failed: {} - {e}", outcome.action.label()));
                                    success = false;
                                }
                            }
                            affected_topic = topic_name.clone();
                        }
                        if success {
                            let desc = description.replace("{topic}", &affected_topic);
                            result_modal.set(Some((label, desc, affected_topic)));
                        }
                    });
                },
            }

            SpinButton {
                angular_velocity: angular_velocity,
                spinning: spinning,
            }

            {
                let modal_data = result_modal();
                let open = modal_data.is_some();
                let (title, description, topic) = modal_data.unwrap_or_default();
                let parts: Vec<&str> = description.splitn(2, &topic).collect();
                rsx! {
                    Modal {
                        open: open,
                        on_close: move |_| result_modal.set(None),
                        title: title,
                        if !topic.is_empty() && parts.len() == 2 {
                            span { "{parts[0]}" }
                            span { class: "font-bold text-foreground", "{topic}" }
                            span { "{parts[1]}" }
                        } else {
                            "{description}"
                        }
                    }
                }
            }
        }
    }
}
