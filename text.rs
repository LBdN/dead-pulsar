use ggez::graphics::{Color, WHITE};

#[derive(Copy, Clone)]
pub enum FontWeight{
    Light,
    LightItalic,
    Normal,
    NormalItalic,
    Bold,
    BoldItalic,
}

#[derive(Clone)]
pub struct FontStyle{
    pub size  : f32,
    pub name  : String,
    pub weight: FontWeight,
    pub color : Color
}


impl FontStyle{
    pub fn new() -> FontStyle{
        FontStyle{
            size: 10.0,
            name: "".to_string(),
            weight: FontWeight::Normal,
            color : WHITE
        }
    }
}


