use crate::{GRIDSIZE, JITTER, MARGIN};
use macroquad::prelude::Vec2;
use rand::random_range;

pub fn generate_jittered_grid_points() -> Vec<Vec2> {
    let mut points: Vec<Vec2> = Vec::new();

    let start = -MARGIN;
    let end = GRIDSIZE as i32 + MARGIN;

    for x in start..end {
        for y in start..end {
            points.push(Vec2::new(
                x as f32 + JITTER * (random_range(0.0..0.99) - random_range(0.0..0.99)),
                y as f32 + JITTER * (random_range(0.0..0.99) - random_range(0.0..0.99)),
            ));
        }
    }

    points
}
