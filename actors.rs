use cgmath::{Point2 as CPoint};
use crate::effect::{Effect};
use crate::render;
use crate::unit;
use crate::unit::*;


// use num_traits as num;
use nalgebra as nal;
use ncollide2d;
use ncollide2d::shape::{Shape};
use ncollide2d::query::{Contact};
use ggez::nalgebra as na;
use nal::{Point2, Isometry2, Vector2};


#[derive(Copy, Clone, PartialEq)]
pub enum DrawContext{
    WorldSpace,
    ScreenSpace
}

pub type ColPolygon = ncollide2d::shape::ConvexPolygon::<f32>;
pub type ColBall    = ncollide2d::shape::Ball::<f32>;
pub type Polyline   = ncollide2d::shape::Polyline::<f32>;

pub fn rect_col_polygon(width : f32, height : f32) -> ColPolygon{

    let points = [
        Point2::new(0.0f32, 0.0f32),
        Point2::new(0.0f32, height),
        Point2::new(width, height),
        Point2::new(width, 0.0f32)
    ];
    let convex = ColPolygon::try_from_points(&points[..]).expect("Convex hull computation failed.");
    return convex;
}


#[derive(Clone)]
pub enum Collision{
    NoCollision{ ncol : ColBall },
    RectCollision{ width : f32, height : f32, ncol : ColPolygon},
    DiscCollision{ radius: f32,  ncol : ColBall },
    PolyCollision{ ncol : Polyline}
}

impl Collision {
    pub fn get_size(&self) -> unit::Size {
        match self {
            Collision::RectCollision{width, height, ..} => unit::Size{x:*width, y:*height},
            Collision::DiscCollision{radius, ..} => unit::Size{x:*radius, y:*radius},
            Collision::NoCollision{..} => unit::Size{x:0.0, y:0.0},
            _ => unit::Size{x: 0.0, y:0.0}
        }
    }    

    pub fn get_ncol(&self) -> Box<&dyn Shape<f32>> {
        match self {
            Collision::RectCollision{width, height , ncol} => Box::new(ncol),
            Collision::DiscCollision{radius, ncol} => Box::new(ncol),
            Collision::NoCollision{ncol} => Box::new(ncol),
            Collision::PolyCollision{ncol} => Box::new(ncol)            
        }
    }
}

pub fn mk_nocol() -> Collision{
    Collision::NoCollision{ ncol : ColBall::new(0.0001f32)}
}

pub fn mk_polycol(pts : &Vec::<unit::Position>) -> Collision{
    let mut points = Vec::<Point2<f32>>::new();
    for p in pts{
        points.push( Point2::new(p.x, p.y));
    }    
    Collision::PolyCollision{ncol : Polyline::new(points, None) }
}


pub fn collides2(pos1 : &unit::Position, col1 : &Collision, pos2 : &unit::Position, col2 : &Collision) -> Option<Contact<f32>>{
    let prediction = 1.0f32;
    let iso1 = Isometry2::new(Vector2::new(pos1.x, pos1.y), na::zero());
    let iso2 = Isometry2::new(Vector2::new(pos2.x, pos2.y), na::zero());    

    let shp1 = col1.get_ncol();
    let shp2 = col2.get_ncol();

    ncollide2d::query::contact(
        &iso1,
        *shp1,
        &iso2,
        *shp2,
        prediction,
    )
}

pub fn collides( pos1 : &unit::Position, col1 : &Collision, pos2 : &unit::Position, col2 : &Collision) -> bool {
    let v1 = CPoint{x : pos1.x ,y : pos1.y };
    let v2 = CPoint{x : pos2.x ,y : pos2.y };
    let delta = v2-v1;

    if let Some(_) = collides2(pos1, col1, pos2, col2){
        return  true;
    };

    match (col1, col2) {
        ( Collision::RectCollision{width : width1, height:height1, ..}, Collision::RectCollision{width : width2, height:height2, ..}) => {            
            if delta.x.abs() > ((width1 + width2)/2.0)  {
                return false
            }
            if delta.y.abs() > ((height1 + height2)/2.0) {
                return false
            }    
            return true
        },
        _ => {
            return false
        }
    }        
}


#[derive(PartialEq)]
pub enum ActorType {
    Background,
    Foreground,
    Player,
    Terrain,
    UI,
    Camera
}

impl ActorType{
    pub fn make(self) -> Actor {
        Actor::new(self, unit::get_id())
    }
}

pub struct Actor {
    pub atype      : ActorType,
    pub id         : Id,
    //==
    pub transform  : unit::Position,    
    //==
    drawable   : Id,
    pub drawctx    : DrawContext,
    pub visible    : bool,
    //==
    pub collision    : Collision,
    pub on_collision : Vec::<Effect>,
    //==
    pub on_start : Vec::<Effect>,
    pub on_tick  : Vec::<Effect>,
    pub ticking  : bool,    
    //==
    
}



impl Actor {
    pub fn new(atype : ActorType, id : unit::Id) -> Actor {
        Actor {
            atype    : atype,
            id       : id,
            //==
            transform: unit::Position{ x:0.0, y:0.0},
            //==
            drawable : no_id(),
            drawctx  : DrawContext::WorldSpace,
            visible  : false,
            //==
            collision    : mk_nocol(),
            on_collision : Vec::<Effect>::new(),
            //==
            on_start  : Vec::<Effect>::new(),
            on_tick   : Vec::<Effect>::new(),
            ticking   : false
        }
    }


    pub fn add_drawable(&mut self, drawable : Id){
        self.drawable = drawable
    }

    pub fn get_drawable(&self) -> Id{
        self.drawable.clone()
    }

    pub fn start(&mut self){      
        match self.atype{
            ActorType::Player => {
                self.visible = true;
                self.ticking = true;
            },
            ActorType::Foreground => {
                self.visible = true;
                self.ticking = true;
            },
            ActorType::Background => {
                self.visible = true;
                self.ticking = true;
            },
            _ => {
                self.visible = true;
                self.ticking = false;
            }
        }                
    }

    pub fn stop(&mut self){
        self.ticking = false;        
    }

    pub fn has_collision(&self) -> bool{
        if !self.ticking{
            false
        }
        // else if let ActorType::Background = self.atype {
        //     false
        // }
        else if let ActorType::Player = self.atype {
            false
        }
        else if let Collision::NoCollision{..} = self.collision {
            false
        }  
        else {
            true
        }        
    }
}

