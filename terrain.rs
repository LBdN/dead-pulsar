use rand::Rng;
use rand::seq::SliceRandom;

use crate::unit::*;


enum TerrainSlope{
    Flat, 
    Up, 
    Down
}

const DIRS    : [TerrainSlope;3] = [TerrainSlope::Flat, TerrainSlope::Up, TerrainSlope::Down];
const UP      : Position         = Position{x:0.0, y:1.0};
const FORWARD : Position         = Position{x:1.0, y:0.0};


fn random_direction(last_direction : &TerrainSlope) -> &TerrainSlope{    
    let mut rng = rand::thread_rng();    
    DIRS.choose(&mut rng).unwrap()
}

fn random_length( max : f32) -> f32{
    let mut rng = rand::thread_rng();    
    let low : f32 = 10.0;
    if max < low{
        return max;
    }
    rng.gen_range(low, max)
}


pub fn build_terrain(bounds : &Bounds, max_length : f32)-> Vec::<Position>{
    let mut points = _build_terrain(bounds.get_size(), max_length);
    for p in &mut points{
        p.y += bounds.min.y;
        p.x += bounds.min.x;
    }
    points
}

pub fn invert_pos( world_size : &Size, points : &mut Vec::<Position>) {
    
    for p in  &mut points.iter_mut(){
        p.y = world_size.y - p.y
    }
        
}

fn _build_terrain(world_size : Size, max_length : f32) -> Vec::<Position>{    
    let tan45 : f32 = (45.0 as f32).to_radians().tan();
    let mut length = 0.0;
    let mut last_direction = &TerrainSlope::Flat;
    let mut points = Vec::<Position>::new();
    points.push( Position{ x:0.0, y:0.0}  );

    while length < world_size.x{
        let direction     = random_direction(last_direction);
        let segment_lenth = random_length(( world_size.x - length).min(max_length));
        let last_point    = points.last().unwrap();
        let x = last_point.x + segment_lenth*FORWARD.x;
        let mut y = last_point.y;
        match direction {
            TerrainSlope::Flat => (y += 0.0),
            TerrainSlope::Up   => (y += segment_lenth * tan45 * UP.y),
            TerrainSlope::Down => (y -= segment_lenth * tan45 * UP.y)
        }        
        y  = y.max(0.0).min(world_size.y / 2.0);
        // reverse because coord frame
        // y = world_size.y -y;
        points.push(Position{x:x, y:y});
        length += segment_lenth;
        last_direction = direction;
    }
    
    points.push( Position{ x:world_size.x, y:0.0}  );
    points
}