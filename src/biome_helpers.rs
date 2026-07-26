use delaunator::EMPTY;
use macroquad::color::Color;
use crate::map_details::MapDetails;
use crate::map_helpers::edges_around_point;
use crate::{GRIDSIZE, WAVELENGTH};
use noise::{NoiseFn, Simplex};

pub fn assign_moisture(map: &MapDetails, elevation: &[f64], seed: u32) -> Vec<f64> {
    let noise = Simplex::new(seed);
    let points = map.points;
    let num_regions = map.num_regions;

    let mut moisture: Vec<f64> = Vec::with_capacity(num_regions);

    for r in 0..num_regions {
        let nx = points[r].x / GRIDSIZE as f32 - 1.0 / 2.0;
        let ny = points[r].y / GRIDSIZE as f32 - 1.0 / 2.0;

        let base_moisture = (1.0 + noise.get([(nx / WAVELENGTH) as f64, (ny / WAVELENGTH) as f64])) / 2.0;
        let elev_factor = (elevation[r] - 0.5).max(0.0);
        let final_moisture = (base_moisture + elev_factor * 0.5).min(1.0);

        moisture.push(final_moisture);
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
    regions.sort_by(|&r1, &r2| elevation[r2].total_cmp(&elevation[r1]));

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

pub fn biome_colour(
    elevation: &[f64],
    moisture: &[f64],
    r: usize,
    flow: &[f64],
    downslope: &[Option<usize>],
    threshold: f64,
) -> Color {
    let is_lake = elevation[r] >= 0.5 && downslope[r].is_none() && flow[r] >= threshold;

    if elevation[r] < 0.5 || is_lake {
        if is_lake {
            // Lake water color
            Color::new(0.20, 0.30, 0.55, 1.0)
        } else {
            let mut e = (elevation[r] - 0.5) * 2.0;
            e = e.clamp(-1.0, 0.0);
            let r_val = 48.0 + 48.0 * e;
            let g_val = 64.0 + 64.0 * e;
            let b_val = 127.0 + 127.0 * e;
            Color::new(
                (r_val as f32 / 255.0).clamp(0.0, 1.0),
                (g_val as f32 / 255.0).clamp(0.0, 1.0),
                (b_val as f32 / 255.0).clamp(0.0, 1.0),
                1.0,
            )
        }
    } else {
        // Rescale land elevation 0.5..1.0 to 0.0..1.0
        let e = ((elevation[r] - 0.5) * 2.0).clamp(0.0, 1.0);
        let m = moisture[r].clamp(0.0, 1.0);

        // 1. Lowland Biomes (e ~ 0.0..0.35)
        let (r_low, g_low, b_low) = if m < 0.25 {
            (210.0, 185.0, 139.0) // Desert
        } else if m < 0.55 {
            (136.0, 170.0, 85.0) // Grassland
        } else if m < 0.80 {
            (85.0, 153.0, 68.0) // Deciduous Forest
        } else {
            (45.0, 106.0, 79.0) // Tropical Rainforest
        };

        // 2. Highland / Taiga Biomes (e ~ 0.35..0.65) - Dark coniferous pine forest
        let (r_mid, g_mid, b_mid) = if m < 0.30 {
            (136.0, 153.0, 119.0) // Shrubland
        } else if m < 0.70 {
            (30.0, 77.0, 43.0) // Dark Coniferous Forest / Taiga
        } else {
            (40.0, 90.0, 70.0) // Cold Rainforest
        };

        // 3. Alpine Mountain Biomes (e ~ 0.65..0.85) - Granite Rock & Tundra
        let (r_high, g_high, b_high) = if m < 0.25 {
            (115.0, 113.0, 107.0) // Barren Rock / Scree
        } else if m < 0.60 {
            (156.0, 163.0, 117.0) // Alpine Tundra
        } else {
            (50.0, 80.0, 75.0) // High Alpine Scrub
        };

        // 4. Snow Peak (e >= 0.85)
        let (r_snow, g_snow, b_snow) = (235.0, 240.0, 245.0);

        // Blend smoothly across elevation tiers
        let (r_val, g_val, b_val) = if e < 0.40 {
            let t = e / 0.40;
            (
                r_low * (1.0 - t) + r_mid * t,
                g_low * (1.0 - t) + g_mid * t,
                b_low * (1.0 - t) + b_mid * t,
            )
        } else if e < 0.70 {
            let t = (e - 0.40) / 0.30;
            (
                r_mid * (1.0 - t) + r_high * t,
                g_mid * (1.0 - t) + g_high * t,
                b_mid * (1.0 - t) + b_high * t,
            )
        } else {
            let t = ((e - 0.70) / 0.30).clamp(0.0, 1.0);
            (
                r_high * (1.0 - t) + r_snow * t,
                g_high * (1.0 - t) + g_snow * t,
                b_high * (1.0 - t) + b_snow * t,
            )
        };

        Color::new(
            (r_val as f32 / 255.0).clamp(0.0, 1.0),
            (g_val as f32 / 255.0).clamp(0.0, 1.0),
            (b_val as f32 / 255.0).clamp(0.0, 1.0),
            1.0,
        )
    }
}