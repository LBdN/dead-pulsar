use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
// use rand::seq::IteratorRandom;
use std::ops::RangeInclusive;
use crate::unit::*;
use std::f32::consts::{PI, FRAC_PI_2};


#[derive(Debug, Copy, Clone)]
pub enum SlopeDirection{
    Flat, 
    Up, 
    Down
}


const DIRS      : [SlopeDirection;3] = [SlopeDirection::Flat, SlopeDirection::Up, SlopeDirection::Down];
const UP_DIRS   : [SlopeDirection;2] = [SlopeDirection::Flat, SlopeDirection::Up];
const DOWN_DIRS : [SlopeDirection;2] = [SlopeDirection::Flat, SlopeDirection::Down];
const UP      : Position         = Position{x:0.0, y:1.0};
const FORWARD : Position         = Position{x:1.0, y:0.0};


fn random_direction(last_direction : &SlopeDirection) -> &SlopeDirection{    
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

pub fn build_sky(bounds : &Bounds2D) -> Vec::<Position>{
    let mut points = Vec::<Position>::new();
    points.push( bounds.min.clone() );
    points.push( Position{x: bounds.min.x, y: bounds.max.y});
    points.push( bounds.max.clone());
    points.push( Position{x: bounds.max.x, y: bounds.min.y});    
    points
}


pub fn build_terrain(bounds : &Bounds2D, max_length : f32)-> Vec::<Position>{
    let mut points = _build_terrain(bounds.get_size(), max_length);
    for p in &mut points{
        p.y += bounds.min.y;
        p.x += bounds.min.x;
    }
    points
}

pub fn invert_pos( world_size : &Size, points : &mut Vec::<Position>, insert: bool) {
    if insert {
        points.insert(0, Position{ x:0.0, y:0.0});
        points.push( Position{ x:world_size.x, y:0.0}  );
    }
    

    for p in  &mut points.iter_mut(){
        p.y = world_size.y - p.y
    }
        
}

fn _build_terrain(world_size : Size, max_length : f32) -> Vec::<Position>{    
    let tan45 : f32 = (45.0 as f32).to_radians().tan();
    let mut length = 0.0;
    let mut last_direction = &SlopeDirection::Flat;
    let mut points = Vec::<Position>::new();
    points.push( Position{ x:0.0, y:0.0}  );

    while length < world_size.x{
        let direction     = random_direction(last_direction);
        let segment_lenth = random_length(( world_size.x - length).min(max_length));
        let last_point    = points.last().unwrap();
        let x = last_point.x + segment_lenth*FORWARD.x;
        let mut y = last_point.y;
        match direction {
            SlopeDirection::Flat => (y += 0.0),
            SlopeDirection::Up   => (y += segment_lenth * tan45 * UP.y),
            SlopeDirection::Down => (y -= segment_lenth * tan45 * UP.y)
        }        
        y  = y.max(0.0).min(world_size.y / 2.0);        
        points.push(Position{x:x, y:y});
        length += segment_lenth;
        last_direction = direction;
    }
    
    points.push( Position{ x:world_size.x, y:0.0}  );
    points
}


#[derive(Copy, Clone)]
enum SizeChange{
    Same,
    Bigger,
    Smaller
}

const SIZES : [SizeChange;3]              = [SizeChange::Same, SizeChange::Bigger, SizeChange::Smaller];
const CONSERVATIVE_SIZES : [SizeChange;2] = [SizeChange::Same, SizeChange::Smaller];

#[derive(Copy, Clone)]
enum ChangeAlloc{
    Top,
    Bottom,
    Both,
    None
}

const CHANGE_ALLOCS : [ChangeAlloc;3] = [ChangeAlloc::Top, ChangeAlloc::Bottom, ChangeAlloc::Both];


#[derive(Copy, Clone)]
struct HeightRange{
    bottom: f32,
    top: f32
}

impl HeightRange{
    fn from_nothing() -> HeightRange{
        HeightRange{
            bottom: 0.0f32,
            top: 0.0f32
        }
    }
    fn from_center(center:f32, halfsize:f32) -> HeightRange{
        HeightRange{
            bottom: center - halfsize,
            top: center - halfsize
        }
    }

    fn size(&self) -> f32{
        self.top - self.bottom
    }

    fn contains(&self, other : &Self) -> bool{
        self.bottom < other.bottom && self.top > other.top 
    }

    fn diff(&self, other : &Self) -> (HeightRange, HeightRange){
        let result     = Vec::<HeightRange>::new();
        let mut space_above = HeightRange::from_nothing();
        let mut space_below = HeightRange::from_nothing();
        if self.top > other.top{
            space_above = HeightRange{bottom: other.top, top:self.top};
        }
        if self.bottom < other.bottom {
            space_below = HeightRange{bottom: self.bottom, top:other.bottom};
        }
        ( space_above, space_below )
    }

    fn empty(&self) -> bool{
        self.top <= self.bottom
    }

    fn place_within(&self, height: f32, rng: &mut ThreadRng) -> Option<f32>{
        if self.size() < height {
            return  None;
        }
        let delta     = self.size() - height;        
        let bot_alloc = rng.gen_range(0.0f32, 1.0f32);
        let result = Some( self.bottom + bot_alloc * delta + height );
        return result;
    }
}


fn get_tunnel_height(world_range : HeightRange, segment_range: HeightRange, min_height: f32, max_vert_move: f32, rng :&mut ThreadRng) -> HeightRange{
    let (space_above, space_below) = world_range.diff(&segment_range);
    // size change
    let mut size_change = SizeChange::Same;
    if !space_above.empty() || !space_below.empty(){
        size_change = *SIZES.choose(rng).unwrap();
    } else {
        size_change = *CONSERVATIVE_SIZES.choose(rng).unwrap();
    }
    // change repartition
    let mut change_alloc = ChangeAlloc::None;
    if let SizeChange::Bigger | SizeChange::Smaller = size_change {
        change_alloc = *CHANGE_ALLOCS.choose(rng).unwrap();
    }
    // direction
    let mut direction = SlopeDirection::Flat;
    let mut direction_mask  = 0;
    if !space_above.empty() { direction_mask |= 1; }
    if !space_below.empty() { direction_mask |= 2; }
    if direction_mask == 1 { direction = *UP_DIRS.choose(rng).unwrap(); }
    if direction_mask == 2 { direction = *DOWN_DIRS.choose(rng).unwrap(); }
    if direction_mask == 3 { direction = *DIRS.choose(rng).unwrap(); }
    // compute effect
    let mut effect_on_top    = 0.0f32;
    let mut effect_on_bottom = 0.0f32;
    if let ChangeAlloc::Top | ChangeAlloc::Both = change_alloc {
        if let SizeChange::Bigger =size_change {
            effect_on_top = rng.gen_range(0.0, space_above.size());
        }
        if let SizeChange::Smaller = size_change {
            let max_reduction = segment_range.size() - min_height;
            if max_reduction > 0.0 {
                effect_on_top = -rng.gen_range(0.0, max_reduction);
            }
        }        
    }
    if let ChangeAlloc::Bottom | ChangeAlloc::Both = change_alloc {
        if let SizeChange::Bigger =size_change {
            effect_on_bottom = -rng.gen_range(0.0, space_below.size());
        }
        if let SizeChange::Smaller = size_change {            
            let mut max_reduction = segment_range.size() - min_height;
            if let ChangeAlloc::Both = change_alloc { 
                max_reduction += effect_on_top;
            }
            if max_reduction > 0.0 {
                effect_on_bottom = rng.gen_range(0.0, max_reduction);
            }            
        }        
    }
    let mut result = segment_range.clone();
    result.top    += effect_on_top;
    result.bottom += effect_on_bottom;
    let (space_above, space_below) = world_range.diff(&result);
    if let SlopeDirection::Up = direction {
        let move_up = rng.gen_range(0.0, space_above.size().min(max_vert_move));
        result.top += move_up;
        result.bottom += move_up;
    }
    if let SlopeDirection::Down = direction {
        let move_down = rng.gen_range(0.0, space_below.size().min(max_vert_move));
        result.top -= move_down;
        result.bottom -= move_down;
    }
    
    result
}

pub fn build_tunnel2(world_size : &Size, length_bounds : &Bounds1D, min_height: f32) -> (Vec::<Position>, Vec::<Position>){    
        
    let mut top_pts = Vec::<Position>::new();    
    let mut bot_pts = Vec::<Position>::new();
    top_pts.push(Position{ x:0.0, y: world_size.y});
    bot_pts.push(Position{ x:0.0, y: 0.0});

    let tunnel_height = min_height*2.0;
    let tunnel_bottom = world_size.y/3.0;
    let start_segment = HeightRange{top: tunnel_bottom+tunnel_height, bottom:tunnel_bottom};
    top_pts.push(Position{ x:0.0, y: start_segment.top});
    bot_pts.push(Position{ x:0.0, y: start_segment.bottom});  

    

    let world_top = world_size.y;
    let world_bottom = 0.0f32;
    let world_range = HeightRange{top: world_top, bottom: world_bottom};
    
    let mut rng = rand::thread_rng();  
    let mut length = 0.0;

    let mut first = true;
    let mut current_range = start_segment;
    while length < world_size.x{

        let mut segment_length = 0.0;
        if world_size.x - length < length_bounds.min{
            segment_length = world_size.x - length;
        } else{
            segment_length = rng.gen_range(length_bounds.min, ( world_size.x - length).min(length_bounds.max));
        }        
        if segment_length == 0.0f32{
            break;
        }

        length += segment_length;
        if first {
            first = !first;
        } else {        
            let cos_a = min_height / current_range.size() ;
            let max_vert_move = cos_a.acos().tan() * segment_length;
            current_range = get_tunnel_height2(world_range, current_range, min_height, segment_length, &mut rng);
        } 
        top_pts.push(Position{ x:length, y: current_range.top});
        bot_pts.push(Position{ x:length, y: current_range.bottom});
    }
        
    top_pts.push(Position{ x:world_size.x, y: world_size.y});
    bot_pts.push(Position{ x:world_size.x, y: 0.0});

    (top_pts, bot_pts)
}


struct VectorRange{
    upward : Vector2,
    downward: Vector2
}

impl VectorRange{
    pub fn empty() -> VectorRange {
        VectorRange{
            upward: Vector2::new(0.0, 0.0),
            downward: Vector2::new(0.0, 0.0)
        }
    }

    pub fn gen_random_dir(&self, rng: &mut ThreadRng) -> Vector2 {
        let t = rng.gen_range(0.0f32, 1.0f32);
        self.upward * t + self.downward * (1.0 -t)
    }
}

#[derive(Copy, Clone, Debug)]
struct Line(Point2, Point2);
 
impl Line {
    pub fn intersect(self, other: Self) -> Option<Point2> {
        let a1 = self.1.y - self.0.y;
        let b1 = self.0.x - self.1.x;
        let c1 = a1 * self.0.x + b1 * self.0.y;
 
        let a2 = other.1.y - other.0.y;
        let b2 = other.0.x - other.1.x;
        let c2 = a2 * other.0.x + b2 * other.0.y;
 
        let delta = a1 * b2 - a2 * b1;
 
        if delta == 0.0 {
            return None;
        }
 
        let x = (b2 * c1 - b1 * c2) / delta;
        let y = (a1 * c2 - a2 * c1) / delta;
        Some(Point2::new( x,y))                    
    }
}
 

fn get_tunnel_height2(world_range : HeightRange, segment_range: HeightRange, min_height: f32, distance: f32, rng :&mut ThreadRng) -> HeightRange{
    
    let radius = min_height;

    if let Some(center_height) = segment_range.place_within(min_height, rng){
        let O = Point2::new(0.0f32, center_height);

        let mut top_vector_range = VectorRange::empty();
        let mut bot_vector_range = VectorRange::empty();

        {
            let P1 = Point2::new(distance, world_range.bottom);
            let OP1 = P1 - O;
            let OP1_norm = OP1.norm();
            let cos_alpha = min_height / OP1_norm;
            let alpha = cos_alpha.acos();
            let rot_alpha = na::Rotation2::new(-alpha);
            let OP2  = (rot_alpha * OP1).normalize() * min_height;
            let P2   = O + OP2;
            let P2P1 = P1 - P2; // max downward  vector for bottom            

            bot_vector_range.downward = P2P1;
            top_vector_range.downward = P2P1;
        }

        {
            let P1 = Point2::new(distance, world_range.top);
            let OP1 = P1 - O;
            let OP1_norm = OP1.norm();
            let cos_alpha = min_height / OP1_norm;
            let alpha = cos_alpha.acos();
            let rot_alpha = na::Rotation2::new(alpha);
            let OP2  = (rot_alpha * OP1).normalize() * min_height;
            let P2   = O + OP2;
            let P2P1 = P1 - P2; // max upward  vector for top

            bot_vector_range.upward = P2P1;
            top_vector_range.upward = P2P1;
        }

        let bot = Point2::new(distance, world_range.bottom);
        let top = Point2::new(distance, world_range.top);
        let line0 = Line(bot, top);

        let v1  = top_vector_range.gen_random_dir(rng);
        let v1t = (na::Rotation2::new( FRAC_PI_2) * v1).normalize();
        let p1 = O + v1t * radius;
        let line1 = Line(p1, p1+v1);

        bot_vector_range.upward = v1;
        let v2 = bot_vector_range.gen_random_dir(rng);
        let v2t = (na::Rotation2::new(- FRAC_PI_2) * v2).normalize();
        let p2 = O + v2t * radius;
        let line2 = Line(p2, p2+v2);

        if let None = line1.intersect(line0){
            return HeightRange::from_nothing();
        }
  
        if let None = line2.intersect(line0){
            return HeightRange::from_nothing();
        }

        let opt_bot_pt = line2.intersect(line0).unwrap();

        let top_pt = line1.intersect(line0).unwrap();
        let bot_pt = line2.intersect(line0).unwrap();
        
        return  HeightRange{bottom: bot_pt.y, top: top_pt.y};
        
    }
    
    HeightRange::from_nothing()
}