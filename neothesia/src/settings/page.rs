use macroquad::prelude::*;
use neothesia_core::config::Config;

use super::interaction::SettingsInteraction;

pub trait SettingsPage {
    fn title(&self) -> &str;

    fn description(&self) -> &str;

    fn render(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        config: &Config,
        mx: f32,
        my: f32,
        mouse_pressed: bool,
        mouse_down: bool,
    ) -> SettingsInteraction;

    fn handle_scroll(&mut self, delta: f32) {}
}
