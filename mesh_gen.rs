// mesh gen
use crate::unit::*;
use nalgebra as nal;
use nal::{Vector3, Rotation3};
use std::f64::consts::{PI};

pub fn regular_polygon(dist : f32, nb_side: i32) -> Vec::<Position> {
    let mut result = Vec::<Position>::new();    
    for i in 0..nb_side{
        let angle = 2.0 * PI * ( i as f64 / nb_side as f64);
        let rot     = Rotation3::new(Vector3::new(0.0f64, 0.0, angle));
        let tv = rot.transform_vector(&Vector3::new(dist as f64, 0.0, 0.0));
        result.push(Position{x: tv.x as f32, y:tv.y as f32});
    }
    result
}

pub fn base_ship(dist: f32) -> Vec::<Position> {
    let mut result = Vec::<Position>::new();
    result.push( Position{x:  3.0f32 * dist, y: 0.0f32 * -dist});
    result.push( Position{x: -2.0f32 * dist, y: 2.0f32 * -dist});
    result.push( Position{x: -1.0f32 * dist, y: 0.0f32 * -dist});
    result.push( Position{x: -1.0f32 * dist, y:-1.0f32 * -dist});        
    result
}