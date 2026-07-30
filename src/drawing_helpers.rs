use macroquad::color::{Color, BLACK, RED};
use macroquad::math::Vec2;
use macroquad::prelude::{draw_circle, draw_line, draw_triangle};
use noise::{NoiseFn, Simplex};
use crate::map_details::MapDetails;
use crate::triangulate_helpers::{next_half_edge, triangle_of_edge};

pub fn draw_points(points: &[Vec2]) {
    for point in points {
        draw_circle(point.x, point.y, 0.1, RED);
    }
}

pub fn is_land_region(
    r: usize,
    elevations: &[f64],
    flow: &[f64],
    downslope: &[Option<usize>],
    threshold: f64,
) -> bool {
    let is_lake = elevations[r] >= 0.5 && downslope[r].is_none() && flow[r] >= threshold;
    elevations[r] >= 0.5 && !is_lake
}

pub fn get_weathered_points(
    p1: Vec2,
    p2: Vec2,
    seed: u32,
    show_weathering: bool,
) -> Vec<Vec2> {
    if !show_weathering {
        return vec![p1, p2];
    }

    let dir = p2 - p1;
    let len = dir.length();
    if len < 0.001 {
        return vec![p1, p2];
    }

    let swap = (p1.x, p1.y) > (p2.x, p2.y);
    let (start_pt, end_pt) = if swap { (p2, p1) } else { (p1, p2) };
    let seg_dir = end_pt - start_pt;
    let seg_len = seg_dir.length();
    let norm_dir = seg_dir / seg_len;
    let normal = Vec2::new(-norm_dir.y, norm_dir.x);

    let noise = Simplex::new(seed);

    let num_subdivisions = (seg_len * 6.0).clamp(6.0, 16.0) as usize;
    let mut canonical_points = Vec::with_capacity(num_subdivisions + 1);
    canonical_points.push(start_pt);

    for step in 1..num_subdivisions {
        let t = step as f32 / num_subdivisions as f32;
        let base_pt = start_pt + seg_dir * t;
        let envelope = (t * std::f32::consts::PI).sin();

        let nx = base_pt.x as f64 * 1.5;
        let ny = base_pt.y as f64 * 1.5;

        let n1 = noise.get([nx, ny]) as f32;
        let n2 = noise.get([nx * 2.5, ny * 2.5]) as f32 * 0.5;
        let combined_noise = (n1 + n2) / 1.5;

        let max_displacement = (seg_len * 0.25).min(0.35);
        let offset_dist = combined_noise * envelope * max_displacement;

        canonical_points.push(base_pt + normal * offset_dist);
    }
    canonical_points.push(end_pt);

    if swap {
        canonical_points.reverse();
    }

    canonical_points
}

pub fn draw_cell_boundaries(
    map: &MapDetails,
    elevations: &[f64],
    flow: &[f64],
    downslope: &[Option<usize>],
    threshold: f64,
    seed: u32,
    show_weathering: bool,
) {
    for e in 0..map.num_edges {
        let opposite = map.half_edges[e];

        if opposite < map.num_edges && e < opposite {
            let centers = map.centers;
            let r1 = map.triangles[e];
            let r2 = map.triangles[next_half_edge(e)];

            let is_r1_land = is_land_region(r1, elevations, flow, downslope, threshold);
            let is_r2_land = is_land_region(r2, elevations, flow, downslope, threshold);
            let is_coastline = is_r1_land != is_r2_land;

            let p = centers[triangle_of_edge(e)];
            let q = centers[triangle_of_edge(opposite)];

            let pts = get_weathered_points(p, q, seed, show_weathering && is_coastline);
            for i in 0..(pts.len() - 1) {
                draw_line(pts[i].x, pts[i].y, pts[i + 1].x, pts[i + 1].y, 0.03, BLACK);
            }
        }
    }
}

pub fn draw_coastlines(
    map: &MapDetails,
    elevations: &[f64],
    flow: &[f64],
    downslope: &[Option<usize>],
    threshold: f64,
    seed: u32,
    show_weathering: bool,
) {
    let stroke_color = Color::new(0.12, 0.18, 0.28, 0.85);

    for e in 0..map.num_edges {
        let opposite = map.half_edges[e];

        if opposite < map.num_edges && e < opposite {
            let centers = map.centers;
            let r1 = map.triangles[e];
            let r2 = map.triangles[next_half_edge(e)];

            let is_r1_land = is_land_region(r1, elevations, flow, downslope, threshold);
            let is_r2_land = is_land_region(r2, elevations, flow, downslope, threshold);

            if is_r1_land != is_r2_land {
                let p = centers[triangle_of_edge(e)];
                let q = centers[triangle_of_edge(opposite)];

                let pts = get_weathered_points(p, q, seed, show_weathering);
                for i in 0..(pts.len() - 1) {
                    draw_line(pts[i].x, pts[i].y, pts[i + 1].x, pts[i + 1].y, 0.05, stroke_color);
                }
            }
        }
    }
}

pub fn draw_cell_colours(
    map: &MapDetails,
    elevations: &[f64],
    moisture: &[f64],
    flow: &[f64],
    downslope: &[Option<usize>],
    threshold: f64,
    seed: u32,
    show_weathering: bool,
    color_fn: fn(&[f64], &[f64], usize, &[f64], &[Option<usize>], f64) -> Color,
) {
    let mut seen = vec![false; map.num_regions];
    let mut boundary_pts: Vec<Vec2> = Vec::with_capacity(32);

    let triangles = map.triangles;
    let num_edges = map.num_edges;
    let centers = map.centers;
    let half_edges = map.half_edges;

    for e in 0..num_edges {
        let r = triangles[next_half_edge(e)];

        if !seen[r] {
            seen[r] = true;

            boundary_pts.clear();
            let mut incoming = e;
            loop {
                let outgoing = next_half_edge(incoming);
                let v_start = centers[incoming / 3];
                let opposite = half_edges[outgoing];

                if opposite == delaunator::EMPTY {
                    boundary_pts.push(v_start);
                    break;
                }

                let v_end = centers[opposite / 3];
                let r_adj = triangles[next_half_edge(outgoing)];

                let is_coastline = is_land_region(r, elevations, flow, downslope, threshold)
                    != is_land_region(r_adj, elevations, flow, downslope, threshold);

                let seg_pts = get_weathered_points(v_start, v_end, seed, show_weathering && is_coastline);

                for &pt in &seg_pts[..seg_pts.len() - 1] {
                    boundary_pts.push(pt);
                }

                incoming = opposite;
                if incoming == e {
                    break;
                }
            }

            if boundary_pts.len() < 3 {
                continue;
            }

            let color = color_fn(elevations, moisture, r, flow, downslope, threshold);
            let center_pt = map.points[r];

            for i in 0..boundary_pts.len() {
                let p1 = boundary_pts[i];
                let p2 = boundary_pts[(i + 1) % boundary_pts.len()];
                draw_triangle(center_pt, p1, p2, color);
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

        let is_r2_water = elevations[r2] < 0.5 || (downslope[r2].is_none() && flow[r2] >= threshold);

        let p = points[r1];
        let q = if is_r2_water {
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
