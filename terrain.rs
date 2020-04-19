use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
// use rand::seq::IteratorRandom;
use std::ops::RangeInclusive;
use crate::unit::*;
use crate::cell::Cell;
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
pub struct HeightRange{
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


fn get_section_length(posx :f32, maxx: f32, length_bounds : &Bounds1D, rng :&mut ThreadRng) -> f32{    
    let available_space    =  maxx - posx;
    if available_space < length_bounds.min{
        available_space
    } else{
        rng.gen_range(length_bounds.min, (available_space).min(length_bounds.max))
    }            
}

pub fn build_tunnel2(world_size : &Size, length_bounds : &Bounds1D, height_bounds: &Bounds1D, first_length: f32) -> (Vec::<HeightRange>, Vec::<f32>){    
        
    
    let mut height_ranges = Vec::<HeightRange>::new();
    let mut xpositions = Vec::<f32>::new();
    let mut pos_x = 0.0;
    let mut rng = rand::thread_rng();  

    let world_top    = world_size.y;
    let world_bottom = 0.0f32;
    let world_range = HeightRange{top: world_top, bottom: world_bottom};

    let tunnel_height = height_bounds.min*2.0;
    let tunnel_bottom = world_size.y/3.0;
    let section_length = 0.0f32;
    let start_segment = HeightRange{top: tunnel_bottom+tunnel_height, bottom:tunnel_bottom};
    pos_x += section_length;
    height_ranges.push(start_segment);
    xpositions.push(pos_x);

    let real_first_length =first_length.min( world_size.x - pos_x );    
    pos_x += real_first_length;
    height_ranges.push(start_segment);
    xpositions.push(pos_x);
    
    let mut current_range = start_segment;    
    while pos_x < world_size.x{
        
        let segment_length = get_section_length(pos_x, world_size.x, &length_bounds, &mut rng );
        if segment_length == 0.0f32{
            break;
        }
        pos_x += segment_length;                   
        current_range = get_tunnel_height2(&world_range, &current_range, height_bounds, segment_length, &mut rng);
        height_ranges.push(current_range);
        xpositions.push(pos_x);
         
        assert!(!pos_x.is_nan());
        assert!(!current_range.top.is_nan());        
    }

    // convert_to_polygons(&height_ranges, &xpositions, world_size)
    (height_ranges, xpositions)
}


pub fn convert_to_polygons(height_ranges: &Vec::<HeightRange>, xpositions: &Vec<f32>, world_size : &Size) -> (Vec::<Position>, Vec::<Position>){
    let mut top_pts = Vec::<Position>::new();    
    let mut bot_pts = Vec::<Position>::new();
    let mut first = true;
    for (height_range, length) in height_ranges.iter().zip(xpositions.iter()){
        if first {
            // needed to have a closed correct poly.
            top_pts.push(Position{ x:0.0, y: world_size.y});
            bot_pts.push(Position{ x:0.0, y: 0.0});
            first = false;            
        }
        top_pts.push(Position{ x:*length, y: height_range.top});
        bot_pts.push(Position{ x:*length, y: height_range.bottom});
    }        
    // needed to have a closed correct poly.
    top_pts.push(Position{ x:world_size.x, y: world_size.y});
    bot_pts.push(Position{ x:world_size.x, y: 0.0});

    (top_pts, bot_pts)
}

pub fn convert_to_cells(height_ranges: &Vec::<HeightRange>, xpositions: &Vec<f32>) -> Vec::<Cell> {
    let mut result = Vec::<Cell>::new();
    let height_ranges_pairs = height_ranges.windows(2);
    let xpositions_pairs = xpositions.windows(2);
    for item in height_ranges_pairs.zip(xpositions_pairs){
        if let &[last_hr, cur_hr] = item.0 {
            if let &[last_x, cur_x] = item.1 {                
                let c = Cell{
                    x00 : Position{x: last_x, y: last_hr.bottom},
                    x01 : Position{x: last_x, y: last_hr.top},
                    x10 : Position{x: cur_x, y: cur_hr.bottom},
                    x11 : Position{x: cur_x, y: cur_hr.top}
                };
                // panic!(c.is_valid());
                result.push(c);
            }
        }
    }
    result
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


fn get_tangent_vector(o : Point2, p : Point2, radius : f32, up: bool) -> Vector2 {
    
    let o_p1 = p - o;
    let o_p1_norm = o_p1.norm();
    let cos_alpha = radius / o_p1_norm;
    let alpha = if up { cos_alpha.acos() } else { - cos_alpha.acos() };
    let rot_alpha = na::Rotation2::new(alpha);
    let o_p2 = (rot_alpha * o_p1).normalize() * radius;
    let p2   = o + o_p2;
    let p2_p1 = p - p2; // max downward  vector for bottom            

    p2_p1
} 

fn get_tunnel_height2(world_range : &HeightRange, segment_range: &HeightRange, height_bounds: &Bounds1D, distance: f32, rng :&mut ThreadRng) -> HeightRange{
    
    let radius = height_bounds.min;

    if distance <= radius {
        return HeightRange{bottom:segment_range.bottom, top:segment_range.top};
    }

    if let Some(center_height) = segment_range.place_within(radius, rng){
        let O = Point2::new(0.0f32, center_height);

        let mut top_vector_range = VectorRange::empty();
        let mut bot_vector_range = VectorRange::empty();

        {
            let P1 = Point2::new(distance, world_range.bottom);
            let OP1 = P1 - O;
            let OP1_norm = OP1.norm();
            let cos_alpha = radius / OP1_norm;
            let alpha = cos_alpha.acos();
            let rot_alpha = na::Rotation2::new(-alpha);
            let OP2  = (rot_alpha * OP1).normalize() * radius;
            let P2   = O + OP2;
            let P2P1 = P1 - P2; // max downward  vector for bottom            

            if P2P1.x.is_nan() || P2P1.y.is_nan() {
                assert!(false);
            }

            bot_vector_range.downward = P2P1;
            top_vector_range.downward = P2P1;
        }

        {
            let P1 = Point2::new(distance, world_range.top);
            let OP1 = P1 - O;
            let OP1_norm = OP1.norm();
            let cos_alpha = radius / OP1_norm;
            let alpha = cos_alpha.acos();
            let rot_alpha = na::Rotation2::new(alpha);
            let OP2  = (rot_alpha * OP1).normalize() * radius;
            let P2   = O + OP2;
            let P2P1 = P1 - P2; // max upward  vector for top

            if P2P1.x.is_nan() || P2P1.y.is_nan() {
                assert!(false);
            }
            
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

        let top_pt = line1.intersect(line0).unwrap();

        bot_vector_range.upward = v1;

        let mut max_bottom = top_pt - Vector2::new(0.0, height_bounds.max);
        max_bottom.y = max_bottom.y.max(world_range.bottom);
        bot_vector_range.downward = get_tangent_vector( O, max_bottom, radius, false );

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

        
        let bot_pt = line2.intersect(line0).unwrap();
        
        if top_pt.y.is_nan() || bot_pt.y.is_nan() {
            assert!(false);
        }        
        return  HeightRange{bottom: bot_pt.y, top: top_pt.y};
        
    }
    
    HeightRange::from_nothing()
}