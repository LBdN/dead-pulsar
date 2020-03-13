use rand::Rng;
use rand::thread::ThreadRng;
use rand::seq::SliceRandom;
// use rand::seq::IteratorRandom;

use crate::unit::*;


enum SizeChange{
    Same,
    Smaller,
    Bigger
}

impl SizeChange{
    fn get(rng :&mut ThreadRng) -> SizeChange {
        *SIZES.choose(&mut rng).unwrap()
    }

    fn get_slope(&self, rng :&mut ThreadRng) -> TerrainSlope{
        match self {
            SizeChange::Same => {
                let possibilities = [TerrainSlope::Down, TerrainSlope::Up, TerrainSlope::Flat];
                *possibilities.choose(&mut rng).unwrap()
            },
            SizeChange::Bigger => {
                let possibilities = [TerrainSlope::Down, TerrainSlope::Flat];
                *possibilities.choose(&mut rng).unwrap()
            },
            SizeChange::Smaller => {
                let possibilities = [TerrainSlope::Down, TerrainSlope::Up];
                *possibilities.choose(&mut rng).unwrap()
            }
        }


    }
}

#[derive(Debug, Copy, Clone)]
enum TerrainSlope{
    Flat, 
    Up, 
    Down
}

const SIZES   : [SizeChange;3]   = [SizeChange::Same, SizeChange::Bigger, SizeChange::Smaller];
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

pub fn build_sky(bounds : &Bounds) -> Vec::<Position>{
    let mut points = Vec::<Position>::new();
    points.push( bounds.min.clone() );
    points.push( Position{x: bounds.min.x, y: bounds.max.y});
    points.push( bounds.max.clone());
    points.push( Position{x: bounds.max.x, y: bounds.min.y});    
    points
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
    points.insert(0, Position{ x:0.0, y:0.0});
    points.push( Position{ x:world_size.x, y:0.0}  );

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
        points.push(Position{x:x, y:y});
        length += segment_lenth;
        last_direction = direction;
    }
    
    points.push( Position{ x:world_size.x, y:0.0}  );
    points
}

fn _build_tunnel(world_size : Size, max_length : f32) -> Vec::<Position>{    
    let tan45 : f32 = (45.0 as f32).to_radians().tan();
    let mut length = 0.0;
    let mut last_direction = TerrainSlope::Flat;
    let mut last_sizechange = &SizeChange::Same;
    let mut points = Vec::<Position>::new();
    points.push( Position{ x:0.0, y:0.0}  );
    let mut rng = rand::thread_rng();  

    while length < world_size.x{
        let sizechange = SizeChange::get(rng);
        let direction= sizechange.get_slope(rng);
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
        points.push(Position{x:x, y:y});
        length += segment_lenth;
        last_direction = direction;
    }
    
    points.push( Position{ x:world_size.x, y:0.0}  );
    points
}