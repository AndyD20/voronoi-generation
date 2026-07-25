mod generate_points;
mod triangulate_helpers;
mod map_details;
mod map_helpers;
mod drawing_helpers;
mod biome_helpers;

use crate::biome_helpers::{assign_moisture, biome_colour};
use crate::drawing_helpers::{draw_cell_boundaries, draw_cell_colours, draw_points};
use crate::generate_points::generate_jittered_grid_points;
use crate::map_details::MapDetails;
use crate::map_helpers::assign_elevation;
use crate::triangulate_helpers::{calculate_centroids, get_delaunay_from_points};
use macroquad::prelude::*;

static GRIDSIZE: usize = 50;
static JITTER: f32 = 0.5;
static WAVELENGTH: f32 = 0.5;

#[macroquad::main("voronoi_map_generation")]
async fn main() {
    let points: Vec<Vec2> = generate_jittered_grid_points();

    let delaunay = get_delaunay_from_points(&points);

    let map: MapDetails = MapDetails {
        points: &points,
        num_regions: points.len(),
        num_edges: delaunay.halfedges.len(),
        half_edges: &delaunay.halfedges,
        triangles: &delaunay.triangles,
        centers: &calculate_centroids(&points, &delaunay),
    };

    let elevations: Vec<f64> = assign_elevation(&map);
    let moisture = assign_moisture(&map);

    loop {
        clear_background(WHITE);

        draw_cell_colours(&map, &elevations, &moisture, biome_colour);
        // draw_cell_boundaries(&map);
        // draw_points(&points);

        next_frame().await;
    }
}

