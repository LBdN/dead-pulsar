pub type Position = mint::Point2::<f32>;
pub type Size     = mint::Point2::<f32>;
pub type Id       = uuid::Uuid;

pub fn get_id() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}