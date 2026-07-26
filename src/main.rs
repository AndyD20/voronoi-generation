mod generate_points;
mod triangulate_helpers;
mod map_details;
mod map_helpers;
mod drawing_helpers;
mod biome_helpers;
mod input_helpers;

use crate::biome_helpers::{assign_moisture, biome_colour, assign_downslope, assign_river_flow};
use crate::input_helpers::InputState;
use crate::drawing_helpers::{draw_cell_boundaries, draw_cell_colours, draw_points, draw_rivers};
use crate::generate_points::generate_jittered_grid_points;
use crate::map_details::MapDetails;
use crate::map_helpers::assign_elevation;
use crate::triangulate_helpers::{calculate_centroids, get_delaunay_from_points};
use macroquad::prelude::*;

static GRIDSIZE: usize = 50;
static JITTER: f32 = 0.5;
static WAVELENGTH: f32 = 0.5;
static STARTING_SEED: u32 = 1;
static STARTING_RIVER_THRESHOLD: f64 = 1.85;

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

    let mut elevations: Vec<f64> = assign_elevation(&map, STARTING_SEED);
    let mut moisture = assign_moisture(&map, STARTING_SEED);
    let mut downslope = assign_downslope(&map, &elevations);
    let mut flow = assign_river_flow(
        &map,
        &elevations,
        &moisture,
        &downslope
    );

    let mut input_state = InputState::new();

    loop {
        clear_background(WHITE);
        input_state.update();

        if input_state.seed_changed {
            elevations = assign_elevation(&map, input_state.new_seed);
            moisture = assign_moisture(&map, input_state.new_seed);

            downslope = assign_downslope(&map, &elevations);
            flow = assign_river_flow(
                &map,
                &elevations,
                &moisture,
                &downslope
            );

            input_state.end_seed_changed();
        }

        draw_cell_colours(&map, &elevations, &moisture, biome_colour);
        draw_rivers(&map, &flow, &downslope, input_state.river_threshold);
        if input_state.show_lines {
            draw_cell_boundaries(&map);
        }
        if input_state.show_points {
            draw_points(&points);
        }

        draw_text(input_state.new_seed.to_string(), 20.0, 20.0, 30.0, DARKGRAY);
        draw_text(input_state.river_threshold.to_string(), 20.0, 40.0, 30.0, DARKGRAY);

        next_frame().await;
    }
}

