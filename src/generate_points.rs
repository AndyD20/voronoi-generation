use crate::{GRIDSIZE, JITTER};
use macroquad::prelude::Vec2;
use rand::random_range;

pub fn generate_jittered_grid_points() -> Vec<Vec2> {
    let mut points: Vec<Vec2> = Vec::new();

    for x in 0..GRIDSIZE {
        for y in 0..GRIDSIZE {
            points.push(Vec2::new(
                x as f32 + JITTER * (random_range(0.0..0.99) - random_range(0.0..0.99)),
                y as f32 + JITTER * (random_range(0.0..0.99) - random_range(0.0..0.99))
            ));
        }
    }

    points
}
