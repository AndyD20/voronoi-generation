use macroquad::prelude::Vec2;

pub struct MapDetails<'a> {
    pub points: &'a [Vec2],
    pub num_regions: usize,
    pub num_edges: usize,
    pub half_edges: &'a [usize],
    pub triangles: &'a [usize],
    pub centers: &'a [Vec2],
}
