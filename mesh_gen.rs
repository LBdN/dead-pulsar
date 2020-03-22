// mesh gen
use crate::unit::*;
use nalgebra as nal;
use nal::{Vector3, Rotation3};
use std::f64::consts::{PI};

pub fn regular_polygon(dist : f32, nb_side: i32) -> Vec::<Position> {
    let mut result = Vec::<Position>::new();
    let v  = Vector3::new(dist, 0.0, 0.0);
    for i in 0..nb_side{
        let angle = 2.0 * PI * ( i as f64 / nb_side as f64);
        let rot     = Rotation3::new(Vector3::new(0.0f64, 0.0, angle));
        let tv = rot.transform_vector(&Vector3::new(dist as f64, 0.0, 0.0));
        result.push(Position{x: tv.x as f32, y:tv.y as f32});
    }
    result
}