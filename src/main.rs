mod generate_points;
mod triangulate_helpers;
mod map_details;
mod map_helpers;
mod drawing_helpers;
mod biome_helpers;
mod input_helpers;

use crate::biome_helpers::{assign_moisture, biome_colour, assign_downslope, assign_river_flow};
use crate::input_helpers::InputState;
use crate::drawing_helpers::{draw_cell_boundaries, draw_cell_colours, draw_coastlines, draw_points, draw_rivers};
use crate::generate_points::generate_jittered_grid_points;
use crate::map_details::MapDetails;
use crate::map_helpers::assign_elevation;
use crate::triangulate_helpers::{calculate_centroids, get_delaunay_from_points};
use macroquad::prelude::*;

static GRIDSIZE: usize = 50;
static MARGIN: i32 = 5;
static JITTER: f32 = 0.5;
static WAVELENGTH: f32 = 0.5;
static STARTING_SEED: u32 = 1;
static STARTING_RIVER_THRESHOLD: f64 = 1.85;

fn compute_elevations(raw_elevations: &[f64], offset: f64) -> Vec<f64> {
    raw_elevations
        .iter()
        .map(|&e| (e + offset).clamp(0.0, 1.0))
        .collect()
}

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

    let mut input_state = InputState::new();

    let mut raw_elevations: Vec<f64> = assign_elevation(&map, STARTING_SEED);
    let mut elevations: Vec<f64> = compute_elevations(&raw_elevations, input_state.elevation_offset);
    let mut moisture = assign_moisture(&map, &elevations, STARTING_SEED);
    let mut downslope = assign_downslope(&map, &elevations);
    let mut flow = assign_river_flow(&map, &elevations, &moisture, &downslope);

    let camera = Camera2D {
        target: vec2(GRIDSIZE as f32 / 2.0, GRIDSIZE as f32 / 2.0),
        zoom: vec2(2.0 / GRIDSIZE as f32, -2.0 / GRIDSIZE as f32),
        ..Default::default()
    };

    loop {
        clear_background(WHITE);
        input_state.update();

        if input_state.seed_changed {
            raw_elevations = assign_elevation(&map, input_state.new_seed);
            elevations = compute_elevations(&raw_elevations, input_state.elevation_offset);
            moisture = assign_moisture(&map, &elevations, input_state.new_seed);
            downslope = assign_downslope(&map, &elevations);
            flow = assign_river_flow(&map, &elevations, &moisture, &downslope);
            input_state.end_seed_changed();
        }

        if input_state.elevation_offset_changed {
            elevations = compute_elevations(&raw_elevations, input_state.elevation_offset);
            moisture = assign_moisture(&map, &elevations, input_state.new_seed);
            downslope = assign_downslope(&map, &elevations);
            flow = assign_river_flow(&map, &elevations, &moisture, &downslope);
            input_state.end_elevation_offset_changed();
        }

        set_camera(&camera);
        draw_cell_colours(
            &map,
            &elevations,
            &moisture,
            &flow,
            &downslope,
            input_state.river_threshold,
            input_state.new_seed,
            input_state.show_weathering,
            biome_colour,
        );
        draw_coastlines(
            &map,
            &elevations,
            &flow,
            &downslope,
            input_state.river_threshold,
            input_state.new_seed,
            input_state.show_weathering,
        );
        draw_rivers(&map, &elevations, &flow, &downslope, input_state.river_threshold);
        if input_state.show_lines {
            draw_cell_boundaries(
                &map,
                &elevations,
                &flow,
                &downslope,
                input_state.river_threshold,
                input_state.new_seed,
                input_state.show_weathering,
            );
        }
        if input_state.show_points {
            draw_points(&points);
        }
        set_default_camera();

        if input_state.show_ui {
            draw_text(&format!("Seed: {}", input_state.new_seed), 20.0, 30.0, 24.0, DARKGRAY);
            draw_text(&format!("River Threshold: {:.2}", input_state.river_threshold), 20.0, 60.0, 24.0, DARKGRAY);
            draw_text(&format!("Elevation Offset: {:+.2}", input_state.elevation_offset), 20.0, 90.0, 24.0, DARKGRAY);
            draw_text(&format!("Weathering: {}", if input_state.show_weathering { "ON (W)" } else { "OFF (W)" }), 20.0, 120.0, 24.0, DARKGRAY);
        }

        next_frame().await;
    }
}

