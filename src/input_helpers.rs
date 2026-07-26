use std::time::{SystemTime, UNIX_EPOCH};
use macroquad::prelude::*;
use crate::{STARTING_RIVER_THRESHOLD, STARTING_SEED};

pub struct InputState {
    pub show_lines: bool,
    pub show_points: bool,
    pub seed_changed: bool,
    pub new_seed: u32,
    pub river_threshold: f64
}

impl InputState {
    pub fn new() -> Self {
        Self {
            show_lines: false,
            show_points: false,
            seed_changed: false,
            new_seed: STARTING_SEED,
            river_threshold: STARTING_RIVER_THRESHOLD
        }
    }

    pub fn end_seed_changed(&mut self) {
        self.seed_changed = false;
    }

    pub fn update(&mut self) {
        if is_key_pressed(KeyCode::L) {
            self.show_lines = !self.show_lines;
        }
        if is_key_pressed(KeyCode::P) {
            self.show_points = !self.show_points;
        }
        if is_key_pressed(KeyCode::R) {
            self.seed_changed = true;
            self.new_seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32;
        }
        if is_key_down(KeyCode::Up) {
            self.river_threshold += 0.01;
        }
        if is_key_down(KeyCode::Down) {
            self.river_threshold -= 0.01;
        }
    }
}
