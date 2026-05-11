use dioxus::prelude::*;

use crate::app::features::wheel::WheelOfChaosPage;
use crate::layouts::default::Layout;
use crate::pages::topics::TopicsPage;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    TopicsPage {},
    #[route("/wheel")]
    WheelOfChaosPage {},
}
