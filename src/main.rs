// The dioxus prelude contains a ton of common items used in dioxus apps. It's a good idea to import wherever you
// need dioxus
use dioxus::prelude::*;
use pokemon_filter::App;
use std::fs;
use pokemon_filter::pokemon::Pokemon;

use components::{Hero, Echo};

/// Define a components module that contains all shared components for our app.
mod components;

// Add this line for CSS linking
const MAIN_CSS: &str = include_str!("../assets/styling/main.css");

fn main() {
    LaunchBuilder::desktop()
        .with_cfg(dioxus::desktop::Config::new()
            .with_window(dioxus::desktop::WindowBuilder::new()
                .with_title("Pokémon Filter")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1200, 800))
                .with_min_inner_size(dioxus::desktop::LogicalSize::new(800, 600)))
            .with_custom_head(format!(
                r#"<style>{}</style>"#,
                MAIN_CSS
            )))
        .launch(App);
}
