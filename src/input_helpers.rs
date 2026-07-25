use macroquad::prelude::*;

pub struct InputState {
    pub show_lines: bool,
    pub show_points: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            show_lines: false,
            show_points: false,
        }
    }

    pub fn update(&mut self) {
        if is_key_pressed(KeyCode::L) {
            self.show_lines = !self.show_lines;
        }
        if is_key_pressed(KeyCode::P) {
            self.show_points = !self.show_points;
        }
    }
}
