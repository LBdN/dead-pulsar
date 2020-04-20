use std::collections::HashMap;
use std::ops::Sub;
use std::marker::Copy;
pub use nalgebra as na;

pub type Position = mint::Point2::<f32>;
pub type Size     = mint::Point2::<f32>;
pub type Id       = uuid::Uuid;

pub type Vector2 = na::Vector2::<f32>;
pub type Point2  = na::Point2::<f32>;

pub fn get_id() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

pub fn  no_id() -> uuid::Uuid{
    uuid::Uuid::nil()
}

pub fn opposite_pos(p : &Position)-> Position{
    Position{
        x: -p.x,
        y: -p.y
    }
}

pub type KeyedResource<T> = HashMap<Id, T>;
pub type KeyedGroup<T>    = HashMap<Id, Vec::<T>>;


pub struct Bounds1D<T>{
    pub min: T,
    pub max: T
}

impl<T: Sub<Output = T> + Copy> Bounds1D<T>{
    pub fn new(min:T, max:T) -> Bounds1D<T>{
        Bounds1D{
            min: min,
            max: max
        }
    }

    pub fn get_size(&self) -> T {
        self.max - self.min
    } 
}

pub struct Bounds2D{
    pub min: Position,
    pub max: Position
}

impl Bounds2D{
    pub fn get_size(&self) -> Size {
        Size{
            x: self.max.x - self.min.x,
            y: self.max.y - self.min.y,
        }
    }

    pub fn get_radius(&self) -> f32{
        let s = self.get_size();
        Vector2::new(s.x, s.y).norm() / 2.0
    }

    pub fn from_positions(pts : &Vec::<Position>) -> Bounds2D{
        let mut min_x = pts[0].x;
        let mut min_y = pts[0].y;
        let mut max_x = pts[0].x;
        let mut max_y = pts[0].y;
        for p in pts{
            max_x = p.x.max(max_x);
            min_x = p.x.min(min_x);
            max_y = p.y.max(max_y);
            min_y = p.y.min(min_y);
        }
        Bounds2D {
            min : Position { x : min_x, y : min_y},
            max : Position { x : max_x, y : max_y}
        }
    }
}