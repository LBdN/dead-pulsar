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


pub type KeyedResource<T> = HashMap<Id, T>;
pub type KeyedGroup<T>    = HashMap<Id, Vec::<T>>;