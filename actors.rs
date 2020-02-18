use cgmath::{Point2};
use crate::effect::{Effect};
use crate::render;
use crate::unit;
use crate::actors;

#[derive(Copy, Clone, PartialEq)]
pub enum DrawContext{
    WorldSpace,
    ScreenSpace
}

#[derive(Debug, Copy, Clone)]
pub enum Collision{
    NoCollision,
    RectCollision{ width : f32, height : f32},
    DiscCollision( f32)
}

impl Collision {
    pub fn get_size(&self) -> unit::Size {
        match self {
            Collision::RectCollision{width, height} => unit::Size{x:*width, y:*height},
            Collision::DiscCollision(radius) => unit::Size{x:*radius, y:*radius},
            Collision::NoCollision => unit::Size{x:0.0, y:0.0}
        }
    }
}

pub fn collides( pos1 : &unit::Position, col1 : &Collision, pos2 : &unit::Position, col2 : &Collision) -> bool {
    let v1 = Point2::<f32>{x : pos1.x ,y : pos1.y };
    let v2 = Point2::<f32>{x : pos2.x ,y : pos2.y };
    let delta = v2-v1;

    match (col1, col2) {
        ( Collision::RectCollision{width : width1, height:height1}, Collision::RectCollision{width : width2, height:height2}) => {            
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
    EndGame,
    UI,
    Camera
}

pub struct Actor {
    pub atype      : ActorType,
    pub id         : unit::Id,
    //==
    pub transform  : unit::Position,    
    //==
    pub drawable   : render::Renderable,
    pub drawctx    : DrawContext,
    pub visible    : bool,
    //==
    pub collision    : Collision,
    pub on_collision : Vec::<Effect>,
    //==
    pub on_start : Vec::<Effect>,
    pub effects  : Vec::<Effect>,
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
            drawable : render::Renderable::StaticRect(0),
            drawctx  : DrawContext::WorldSpace,
            visible  : false,
            //==
            collision    : Collision::DiscCollision(0.0),
            on_collision : Vec::<Effect>::new(),
            //==
            on_start  : Vec::<Effect>::new(),
            effects   : Vec::<Effect>::new(),
            ticking   : false
        }
    }

    pub fn start(&mut self){        
        self.ticking = true;        
    }

    pub fn stop(&mut self){
        self.ticking = false;        
    }

    pub fn has_collision(&self) -> bool{
        if !self.ticking{
            false
        }
        else if let actors::ActorType::Background = self.atype {
            false
        }
        else if let actors::ActorType::Player = self.atype {
            false
        }
        else if let actors::Collision::NoCollision = self.collision {
            false
        }  
        else {
            true
        }        
    }
}