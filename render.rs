use ggez::graphics;
use ggez::graphics::{DrawParam, Color, Rect, Drawable, DrawMode, Mesh};
use ggez::{Context};



pub enum TextAnchor{
    Center,
    TopLeft
}

pub enum Renderable{
    NoDraw,
    StaticRect(usize),
    DynamicRect{ color : Color, size : super::unit::Size},
    StaticText{text: graphics::Text, text_anchor: TextAnchor },
    DynamicTextDraw { string: String, font : graphics::Font, fontsize : f32, color: graphics::Color},
    
}

impl Renderable {
    pub fn draw(&self, transform : super::unit::Position, mb : &mut graphics::MeshBuilder, meshes : &mut Vec::<Mesh>, ctx : &mut Context){
        match self {
            Renderable::NoDraw => (),
            Renderable::StaticRect(idx) => {
                let _ = meshes[*idx].draw(ctx, DrawParam::default().dest(transform));    
            },
            Renderable::DynamicRect{color, size} => {
                mb.rectangle(
                    DrawMode::fill(),
                    Rect {
                        x:transform.x,
                        y:transform.y,
                        w:size.x,
                        h:size.y
                    },
                    *color,
                );
            },
            Renderable::StaticText{text, text_anchor} =>{
                let mut t =  transform.clone();
                if let TextAnchor::Center = text_anchor {
                    t.x -= text.width(ctx) as f32 / 2.0;
                    t.y -= text.height(ctx) as f32;
                }                
                let _ = text.draw(ctx, DrawParam::default().dest(t));
            },                        
            Renderable::DynamicTextDraw{string, font, fontsize, color} => {
                let text = graphics::Text::new( (string.clone() , *font, *fontsize) );                
                let t =  transform.clone();
                // t.x -= text.width(ctx) as f32 / 2.0;
                // t.y -= text.height(ctx) as f32 / 2.0;
                let _ = text.draw(ctx, DrawParam::default().dest(t).color(*color));
            }            
        } 
    }
}