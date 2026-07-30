use macroquad::prelude::*;
use ::rand::random_range;
use crate::{STARTING_RIVER_THRESHOLD, STARTING_SEED};

pub struct InputState {
    pub show_lines: bool,
    pub show_points: bool,
    pub show_ui: bool,
    pub show_weathering: bool,
    pub seed_changed: bool,
    pub elevation_offset_changed: bool,
    pub new_seed: u32,
    pub river_threshold: f64,
    pub elevation_offset: f64,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            show_lines: false,
            show_points: false,
            show_ui: false,
            show_weathering: true,
            seed_changed: false,
            elevation_offset_changed: false,
            new_seed: STARTING_SEED,
            river_threshold: STARTING_RIVER_THRESHOLD,
            elevation_offset: 0.2,
        }
    }

    pub fn end_seed_changed(&mut self) {
        self.seed_changed = false;
    }

    pub fn end_elevation_offset_changed(&mut self) {
        self.elevation_offset_changed = false;
    }

    pub fn update(&mut self) {
        if is_key_pressed(KeyCode::L) {
            self.show_lines = !self.show_lines;
        }
        if is_key_pressed(KeyCode::P) {
            self.show_points = !self.show_points;
        }
        if is_key_pressed(KeyCode::I) {
            self.show_ui = !self.show_ui;
        }
        if is_key_pressed(KeyCode::W) {
            self.show_weathering = !self.show_weathering;
        }
        if is_key_pressed(KeyCode::R) {
            self.seed_changed = true;
            self.new_seed = random_range(1..=u32::MAX);
        }
        if is_key_down(KeyCode::Up) {
            self.river_threshold += 0.01;
        }
        if is_key_down(KeyCode::Down) {
            self.river_threshold = (self.river_threshold - 0.01).max(0.0);
        }
        if is_key_down(KeyCode::Right) {
            self.elevation_offset = (self.elevation_offset + 0.005).min(0.45);
            self.elevation_offset_changed = true;
        }
        if is_key_down(KeyCode::Left) {
            self.elevation_offset = (self.elevation_offset - 0.005).max(-0.45);
            self.elevation_offset_changed = true;
        }
    }
}
