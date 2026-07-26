use macroquad::color::{Color, BLACK, RED};
use macroquad::math::Vec2;
use macroquad::prelude::{draw_circle, draw_line, draw_triangle};
use noise::{NoiseFn, Simplex};
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
    let mut seen = vec![false; map.num_regions];

    let triangles = map.triangles;
    let num_edges = map.num_edges;
    let centers = map.centers;

    for e in 0..num_edges {
        let r = triangles[next_half_edge(e)];

        if !seen[r] {
            seen[r] = true;

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
    let noise = Simplex::new(42);

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

        let dir = q - p;
        let len = dir.length();
        if len < 0.001 {
            continue;
        }

        let norm_dir = dir / len;
        let normal = Vec2::new(-norm_dir.y, norm_dir.x);

        let width = 0.05 * (flow[r1] as f32).sqrt().max(1.0);
        let color = Color::new(0.188, 0.251, 0.498, 1.0);

        let num_subdivisions = 4;
        let mut prev_pt = p;

        for step in 1..=num_subdivisions {
            let t = step as f32 / num_subdivisions as f32;
            let current_pt = if step == num_subdivisions {
                q
            } else {
                let base_pt = p + dir * t;
                // Envelope function (sin(t * PI)) ensures offset is 0 at endpoints (t=0 and t=1)
                let envelope = (t * std::f32::consts::PI).sin();
                let n_val = noise.get([(base_pt.x / 2.0) as f64, (base_pt.y / 2.0) as f64]) as f32;
                let offset_dist = n_val * envelope * 0.25 * len.min(1.5);
                base_pt + normal * offset_dist
            };

            draw_line(prev_pt.x, prev_pt.y, current_pt.x, current_pt.y, width, color);
            prev_pt = current_pt;
        }
    }
}
