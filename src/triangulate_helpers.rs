use delaunator::{triangulate, Point, Triangulation};
use macroquad::prelude::Vec2;

pub fn get_delaunay_from_points(points: &Vec<Vec2>) -> Triangulation {
    let delaunay_points: Vec<Point> = points
        .iter()
        .map(|loc| Point {
            x: loc.x as f64,
            y: loc.y as f64
        })
        .collect();

    triangulate(&delaunay_points)
}

pub fn calculate_centroids(points: &Vec<Vec2>, delaunay: &Triangulation) -> Vec<Vec2> {
    let num_triangles = delaunay.halfedges.len() / 3;
    let mut centroids: Vec<Vec2> = Vec::new();

    for i in 0..num_triangles {
        let mut sum_of_x: f32 = 0.0;
        let mut sum_of_y: f32 = 0.0;

        for j in 0..3 {
            let s =3 * i + j;
            let p = points[delaunay.triangles[s]];
            sum_of_x += p.x;
            sum_of_y += p.y;
        }

        centroids.push(Vec2::new(sum_of_x / 3.0, sum_of_y / 3.0));
    }

    centroids
}

pub fn triangle_of_edge(e: usize) -> usize {
    e/3
}

pub fn next_half_edge(e: usize) -> usize {
    if e % 3 == 2 {e - 2} else {e + 1}
}
