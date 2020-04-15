// mesh gen

use std::f64::consts::{PI};

use nalgebra as nal;
use nal::{Vector3, Rotation3};
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use crate::unit::*;

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

pub fn irregular_polygon(dist_range: Bounds1D, nb_side: i32, rng : &mut ThreadRng) -> Vec::<Position> {
    let mut result = Vec::<Position>::new();    
    for i in 0..nb_side{
        let dist = rng.gen_range(dist_range.min, dist_range.max);
        let angle = 2.0 * PI * ( i as f64 / nb_side as f64);
        let rot     = Rotation3::new(Vector3::new(0.0f64, 0.0, angle));
        let tv = rot.transform_vector(&Vector3::new(dist as f64, 0.0, 0.0));
        result.push(Position{x: tv.x as f32, y:tv.y as f32});
    }
    result
}



pub fn crystal_polygon(dist_range: Bounds1D, nb_side: i32, rng : &mut ThreadRng) -> Vec::<Position> {
    let mut result = Vec::<Position>::new();    

    let start_angle = rng.gen_range(PI * 0.05, PI * 0.45);
    let delta_angle = PI * 0.43;
    let angle_per_step = delta_angle / nb_side as f64;

    for i in 0..nb_side{        
        if i%2==0 {
            // simple case, the inset.
            let dist = dist_range.min;
            let angle = start_angle +  i as f64 * angle_per_step;
            let rot = Rotation3::new(Vector3::new(0.0f64, 0.0, angle));
            let tv  = rot.transform_vector(&Vector3::new(dist as f64, 0.0, 0.0));
            result.push(Position{x: tv.x as f32, y:tv.y as f32});

        } else {
            let delta = dist_range.get_size();            
            let dist =  rng.gen_range(dist_range.min + delta *0.25, dist_range.max);
            let angle = start_angle +  i as f64 * angle_per_step;

            let angle1 = angle - rng.gen_range(angle_per_step / 4.0, angle_per_step / 2.0);
            let dist1 = rng.gen_range(dist*0.8, dist*0.95);
            let rot = Rotation3::new(Vector3::new(0.0f64, 0.0, angle1));
            let tv  = rot.transform_vector(&Vector3::new(dist1 as f64, 0.0, 0.0));
            result.push(Position{x: tv.x as f32, y:tv.y as f32});

            let rot = Rotation3::new(Vector3::new(0.0f64, 0.0, angle));
            let tv  = rot.transform_vector(&Vector3::new(dist as f64, 0.0, 0.0));
            result.push(Position{x: tv.x as f32, y:tv.y as f32});

            let angle1 = angle + rng.gen_range(angle_per_step / 4.0, angle_per_step / 2.0);
            let dist1 = rng.gen_range(dist*0.8, dist*0.95);
            let rot = Rotation3::new(Vector3::new(0.0f64, 0.0, angle1));
            let tv  = rot.transform_vector(&Vector3::new(dist1 as f64, 0.0, 0.0));
            result.push(Position{x: tv.x as f32, y:tv.y as f32});

        };
        
    }


    result.push(Position{x: 0.0, y:0.0});

    // let start_angle2 = start_angle + delta_angle;
    // let delta_angle2  = 4.0 * PI * 0.33;
    // for i in 0..nb_side{
    //     let dist = rng.gen_range(dist_range.min, dist_range.min*1.20);
    //     let angle = start_angle2 + delta_angle2 * ( i as f64 / nb_side as f64) ;
    //     let rot = Rotation3::new(Vector3::new(0.0f64, 0.0, angle));
    //     let tv  = rot.transform_vector(&Vector3::new(-dist as f64, 0.0, 0.0));
    //     result.push(Position{x: tv.x as f32, y:tv.y as f32});
    // }
            
    result
}


pub fn base_ship(dist: f32) -> Vec::<Position> {
    let mut result = Vec::<Position>::new();
    result.push( Position{x:  3.0f32 * dist, y: 0.0f32 * dist});
    result.push( Position{x: -2.0f32 * dist, y: 2.0f32 * dist});
    result.push( Position{x: -1.0f32 * dist, y: 0.0f32 * dist});
    result.push( Position{x: -1.0f32 * dist, y:-1.0f32 * dist});        
    result
}

pub fn star(min_dist: f32, max_dist: f32, nb_side: i32 ) -> Vec::<Position>{
    let pts = regular_polygon(max_dist, nb_side);
    pts
}

