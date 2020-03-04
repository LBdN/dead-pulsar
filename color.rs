use ggez::graphics::{Color};
use rand::Rng;

pub const GREY  : Color = Color{ r: 0.5, g:0.5, b:0.5, a:1.0};
pub const GREEN : Color = Color{ r: 0.2, g:1.0, b:0.2, a:1.0};
pub const RED   : Color = Color{ r: 1.0, g:0.0, b:0.0, a:1.0};
pub const MARROON : Color = Color{ r: 0.5, g:0.0, b:0.0, a:1.0};

pub fn random_foreground_color() -> Color{
    let mut rng = rand::thread_rng();
    let r    = 1.0;
    let b    = rng.gen_range(0.0, 1.0);        
    Color{r:r, g:r, b:b, a:1.0}
}

pub fn random_grey_color() -> Color{
    let mut rng = rand::thread_rng();
    let r       = rng.gen_range(0.1, 0.5);
    Color{r:r, g:r, b:r, a:1.0}
}

pub fn fade_to_transparent(nbsteps: i32, color : &Color) -> Vec::<Color> {
    let mut colors = Vec::<Color>::new();
    let step_size = color.a / nbsteps as f32;    
    for i in 0..nbsteps{
        let mut c = color.clone();
        c.a -= (i as f32)*step_size;
        colors.push(c);
    }
    colors
}