use std::collections::HashSet;
use macroquad::color::{Color, BLACK, RED};
use macroquad::math::Vec2;
use macroquad::prelude::{draw_circle, draw_line, draw_triangle};
use crate::map_details::MapDetails;
use crate::map_helpers::edges_around_point;
use crate::triangulate_helpers::{next_half_edge, triangle_of_edge};

pub fn draw_points(points: &[Vec2]) {
    for point in points {
        draw_circle(point.x, point.y, 0.1, RED);
    }
}

pub fn draw_cell_boundaries(map: &MapDetails) {
    for e in 0..map.num_edges {
        let opposite = map.half_edges[e];

        if opposite < map.num_edges && e < opposite {
            let centers = map.centers;

            let p = centers[triangle_of_edge(e)];
            let q = centers[triangle_of_edge(opposite)];

            draw_line(p.x, p.y, q.x, q.y, 0.03, BLACK);
        }
    }
}

pub fn draw_cell_colours(
    map: &MapDetails,
    elevations: &[f64],
    moisture: &[f64],
    color_fn: fn(&[f64], &[f64], usize) -> Color,
) {
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

            let v0 = vertices[0];
            for i in 1..(vertices.len() - 1) {
                let v1 = vertices[i];
                let v2 = vertices[i + 1];
                draw_triangle(v0, v1, v2, color);
            }
        }
    }
}

pub fn draw_rivers(
    map: &MapDetails,
    elevations: &[f64],
    flow: &[f64],
    downslope: &[Option<usize>],
    threshold: f64,
) {
    let points = map.points;
    let triangles = map.triangles;

    for r1 in 0..points.len() {
        if elevations[r1] < 0.5 || downslope[r1].is_none() || flow[r1] < threshold {
            continue;
        }

        let r2 = triangles[downslope[r1].unwrap()];

        let p = points[r1];
        let q = if elevations[r2] < 0.5 {
            (points[r1] + points[r2]) * 0.5
        } else {
            points[r2]
        };

        let width = 0.05 * (flow[r1] as f32).sqrt().max(1.0);
        draw_line(
            p.x,
            p.y,
            q.x,
            q.y,
            width,
            Color::new(0.188, 0.251, 0.498, 1.0),
        );
    }
}
