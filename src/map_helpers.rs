use delaunator::EMPTY;
use crate::map_details::MapDetails;
use crate::triangulate_helpers::next_half_edge;
use crate::{GRIDSIZE, WAVELENGTH};
use noise::{NoiseFn, Simplex};

pub fn assign_elevation(map: &MapDetails, seed: u32) -> Vec<f64> {
    let noise = Simplex::new(seed);
    let points = map.points;
    let num_regions = map.num_regions;

    let mut elevation: Vec<f64> = Vec::with_capacity(num_regions);

    for r in 0..num_regions {
        let nx = points[r].x / GRIDSIZE as f32 - 1.0 / 2.0;
        let ny = points[r].y / GRIDSIZE as f32 - 1.0 / 2.0;

        // Base continent landmass shape from low-frequency noise
        let base_noise = (1.0 + noise.get([(nx / WAVELENGTH) as f64, (ny / WAVELENGTH) as f64])) / 2.0;

        // Ridged noise octave: (1.0 - |noise|)^2 creates narrow, sharp mountain ridges
        let raw_ridge = noise.get([(2.0 * nx / WAVELENGTH) as f64, (2.0 * ny / WAVELENGTH) as f64]).abs();
        let ridge_noise = (1.0 - raw_ridge).powi(2);

        // Distance from center for island masking
        let d = 2.0 * nx.abs().max(ny.abs());

        // Balance base landmass, ridged mountain layer, and island falloff
        let combined = base_noise * 0.45 + ridge_noise * 0.35 + 0.35 - (d as f64) * 0.65;
        let final_elevation = combined.clamp(0.0, 1.0);

        elevation.push(final_elevation);
    }

    elevation
}

pub fn edges_around_point(half_edges: &[usize], start: usize) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::new();
    let mut incoming = start;

    loop {
        result.push(incoming);
        let outgoing = next_half_edge(incoming);
        incoming = half_edges[outgoing];

        if incoming == EMPTY || incoming == start {
            break;
        }
    }

    result
}
