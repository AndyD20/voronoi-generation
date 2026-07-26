use std::collections::HashSet;
use macroquad::color::{Color, BLACK, RED};
use macroquad::math::{vec2, Vec2};
use macroquad::prelude::{draw_circle, draw_line, draw_triangle, screen_height, screen_width};
use crate::GRIDSIZE;
use crate::map_details::MapDetails;
use crate::map_helpers::edges_around_point;
use crate::triangulate_helpers::{next_half_edge, triangle_of_edge};

pub fn draw_points(points: &Vec<Vec2>) {
    let scale_x = screen_width() / GRIDSIZE as f32;
    let scale_y = screen_height() / GRIDSIZE as f32;

    for point in points {
        let screen_x = point.x * scale_x;
        let screen_y = point.y * scale_y;

        draw_circle(screen_x, screen_y, 1.5, RED);
    }
}

pub fn draw_cell_boundaries(map: &MapDetails) {
    let scale_x = screen_width() / GRIDSIZE as f32;
    let scale_y = screen_height() / GRIDSIZE as f32;

    for e in 0..map.num_edges {
        let opposite = map.half_edges[e];

        if opposite < map.num_edges && e < opposite {
            let centers = &map.centers;

            let p = centers[triangle_of_edge(e)];

            let screen_x = p.x * scale_x;
            let screen_y = p.y * scale_y;

            let q = centers[triangle_of_edge(opposite)];

            draw_line(screen_x, screen_y, q.x * scale_x, q.y * scale_y, 1.0, BLACK);
        }
    }
}

pub fn draw_cell_colours(map: &MapDetails, elevations: &Vec<f64>, moisture: &Vec<f64>, color_fn: fn(&Vec<f64>, &Vec<f64>, usize) -> Color) {
    let scale_x = screen_width() / GRIDSIZE as f32;
    let scale_y = screen_height() / GRIDSIZE as f32;

    let mut seen: HashSet<usize> = HashSet::new();

    let triangles = map.triangles;
    let num_edges = map.num_edges;
    let centers = map.centers;

    for e in 0..num_edges {
        let r = triangles[next_half_edge(e)];

        if !seen.contains(&r) {
            seen.insert(r);

            let edges = edges_around_point(map.half_edges, e);

            let vertices: Vec<Vec2> = edges
                .into_iter()
                .map(|edge| centers[edge / 3])
                .collect();

            if vertices.len() < 3 {
                continue;
            }
            let color = color_fn(elevations, moisture, r);

            let v0 = vec2(vertices[0].x * scale_x, vertices[0].y * scale_y);
            for i in 1..(vertices.len() - 1) {
                let v1 = vec2(vertices[i].x * scale_x, vertices[i].y * scale_y);
                let v2 = vec2(vertices[i + 1].x * scale_x, vertices[i + 1].y * scale_y);
                draw_triangle(v0, v1, v2, color);
            }
        }
    }
}

pub fn draw_rivers(map: &MapDetails, flow: &[f64], downslope: &[Option<usize>], threshold: f64) {
    let points = map.points;
    let triangles = map.triangles;

    let scale_x = screen_width() / GRIDSIZE as f32;
    let scale_y = screen_height() / GRIDSIZE as f32;

    for r1 in 0..points.len() {
        if downslope[r1].is_none() || flow[r1] < threshold {
            continue;
        }

        let r2 = triangles[downslope[r1].unwrap()];

        let p = points[r1];
        let q = points[r2];

        let screen_x = p.x * scale_x;
        let screen_y = p.y * scale_y;

        draw_line(screen_x, screen_y, q.x * scale_x, q.y * scale_y, 1.0, Color::new(0.188, 0.251, 0.498, 1.0))
    }
}
