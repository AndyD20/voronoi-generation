use delaunator::EMPTY;
use macroquad::color::Color;
use crate::map_details::MapDetails;
use crate::map_helpers::edges_around_point;
use crate::{GRIDSIZE, WAVELENGTH};
use noise::{NoiseFn, Simplex};

pub fn assign_moisture(map: &MapDetails, seed: u32) -> Vec<f64> {
    let noise = Simplex::new(seed);
    let points = map.points;
    let num_regions = map.num_regions;

    let mut moisture: Vec<f64> = Vec::new();

    for r in 0..num_regions {
        let nx = points[r].x / GRIDSIZE as f32 - 1.0 / 2.0;
        let ny = points[r].y / GRIDSIZE as f32 - 1.0 / 2.0;

        moisture.push((1.0 + noise.get([(nx / WAVELENGTH) as f64, (ny / WAVELENGTH) as f64])) / 2.0);
    }
    moisture
}

pub fn assign_downslope(map: &MapDetails, elevation: &[f64]) -> Vec<Option<usize>> {
    let half_edges = map.half_edges;
    let triangles = map.triangles;

    let mut downslope: Vec<Option<usize>> = vec![None; elevation.len()];

    for incoming1 in 0..triangles.len() {
        let outgoing1 = half_edges[incoming1];
        if outgoing1 == EMPTY {
            continue; // on convex hull, ignore
        }
        let r1 = triangles[outgoing1];
        let mut best_elevation = elevation[r1];
        let mut best_edge = None;

        for incoming2 in edges_around_point(half_edges, incoming1) {
            let r2 = triangles[incoming2];
            if elevation[r2] < best_elevation {
                best_elevation = elevation[r2];
                best_edge = Some(incoming2);
            }
        }
        downslope[r1] = best_edge;
    }

    downslope
}

pub fn assign_river_flow(
    map: &MapDetails,
    elevation: &[f64],
    moisture: &[f64],
    downslope: &[Option<usize>],
) -> Vec<f64> {
    let triangles = map.triangles;

    let mut regions: Vec<usize> = (0..elevation.len()).collect();
    regions.sort_by(|&r1, &r2| {
        elevation[r2]
            .partial_cmp(&elevation[r1])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut flow = vec![0.0; elevation.len()];

    for &r in &regions {
        if elevation[r] < 0.5 {
            continue; // skip oceans
        }
        flow[r] += moisture[r]; // rainfall

        if let Some(incoming_edge) = downslope[r] {
            let outgoing_region = triangles[incoming_edge];
            flow[outgoing_region] += flow[r];
        }
    }

    flow
}

pub fn biome_colour(elevation: &Vec<f64>, moisture: &Vec<f64>, r: usize) -> Color {
    let mut e = (elevation[r] - 0.5) * 2.0;
    let m = moisture[r];

    let mut r: f64;
    let mut g: f64;
    let mut b: f64;

    if e < 0.0 {
        r = 48.0 + 48.0 * e;
        g = 64.0 + 64.0 * e;
        b = 127.0 + 127.0 * e;
    } else {
        e = e.powi(4); // tweak for better coloring
        r = 210.0 - 100.0 * m;
        g = 185.0 - 45.0 * m;
        b = 139.0 - 45.0 * m;
        r = 255.0 * e + r * (1.0 - e);
        g = 255.0 * e + g * (1.0 - e);
        b = 255.0 * e + b * (1.0 - e);
    }
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}