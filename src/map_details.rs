use macroquad::prelude::Vec2;

pub struct MapDetails<'a> {
    pub points: &'a Vec<Vec2>,
    pub num_regions: usize,
    pub num_edges: usize,
    pub half_edges: &'a Vec<usize>,
    pub triangles: &'a Vec<usize>,
    pub centers: &'a Vec<Vec2>,
}
