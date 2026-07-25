use crate::map_details::MapDetails;
use crate::triangulate_helpers::next_half_edge;
use crate::{GRIDSIZE, WAVELENGTH};
use macroquad::prelude::*;
use noise::{NoiseFn, Simplex};

pub fn assign_elevation(map: &MapDetails) -> Vec<f64> {
    let noise = Simplex::new(1);
    let points = map.points;
    let num_regions = map.num_regions;

    let mut elevation: Vec<f64> = Vec::new();

    for r in 0..num_regions {
        let nx = points[r].x / GRIDSIZE as f32 - 1.0 / 2.0;
        let ny = points[r].y / GRIDSIZE as f32 - 1.0 / 2.0;

        elevation.push(
            1.0 / 2.0
            + noise.get([(nx / WAVELENGTH) as f64, (ny / WAVELENGTH) as f64]) / 2.0
            + noise.get([(2.0 * nx / WAVELENGTH) as f64, (2.0 * ny / WAVELENGTH) as f64])
        );

        let d = 2.0 * f32::max(nx.abs(), ny.abs());

        elevation[r] = (1.0 + elevation[r] - d as f64) / 2.0;
    }

    elevation
}

pub fn edges_around_point(half_edges: &Vec<usize>, start: usize) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::new();
    let mut incoming = start;

    loop {
        result.push(incoming);
        let outgoing = next_half_edge(incoming);
        incoming = half_edges[outgoing];

        if incoming as i32 == -1 || incoming == start {
            break;
        }
    }

    result
}
