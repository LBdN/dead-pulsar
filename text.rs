
pub enum FontWeight{
    Light,
    LightItalic,
    Normal,
    NormalItalic,
    Bold,
    BoldItalic,
}
    
pub struct FontStyle{
    size  : usize,
    name  : String,
    weight: FontWeight,
    color : [f32;3]
}

impl FontStyle{
    pub fn new() -> FontStyle{
        FontStyle{
            size: 10,
            name: "".to_string(),
            weight: FontWeight::Normal,
            color : [1.0, 1.0, 1.0]
        }
    }
}


