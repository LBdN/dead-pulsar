use ggez::graphics::{Color, WHITE};

#[derive(Copy, Clone)]
pub enum FontWeight{
    // Light,
    // LightItalic,
    Normal,
    // NormalItalic,
    // Bold,
    // BoldItalic,
}

#[derive(Clone)]
pub struct FontStyle{
    pub size  : f32,
    pub name  : String,
    pub weight: FontWeight,
    pub color : Color
}


impl FontStyle{
    pub fn default() -> FontStyle{
        FontStyle{
            size: 10.0,
            name: "".to_string(),
            weight: FontWeight::Normal,
            color : WHITE
        }
    }
}

pub fn title_style() -> FontStyle{
    FontStyle{
        size: 56.0,
        name: "edundot".to_string(),
        weight: FontWeight::Normal,
        color: ggez::graphics::WHITE,
    }
}

pub fn tuto_style() -> FontStyle{
    FontStyle{
        size: 30.0,
        name: "V5PRD___".to_string(),
        weight: FontWeight::Normal,
        color: ggez::graphics::WHITE,
    }
}

pub fn ui_style() -> FontStyle{
    FontStyle{
        size: 28.0,
        name: "edundot".to_string(),
        weight: FontWeight::Normal,
        color: ggez::graphics::WHITE,
    }
}


