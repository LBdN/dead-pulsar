use std::collections::HashMap;

pub type Position = mint::Point2::<f32>;
pub type Size     = mint::Point2::<f32>;
pub type Id       = uuid::Uuid;

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


pub struct Bounds1D{
    pub min: f32,
    pub max: f32
}

pub struct Bounds2D{
    pub min: Size,
    pub max: Size
}

impl Bounds2D{
    pub fn get_size(&self) -> Size{
        Size{
            x: self.max.x - self.min.x,
            y: self.max.y - self.min.y,
        }
    }
}