use ggez::graphics;
use ggez::graphics::{DrawParam, Color, Rect, Drawable, DrawMode, Mesh};
use ggez::{Context, GameResult};

use crate::unit::*;
use std::collections::HashMap;

pub struct Renderer{
    pub fonts  : HashMap::<String, graphics::Font>,
    pub mb     : graphics::MeshBuilder,
    pub meshes : Vec::<Mesh>,
    cam_tr     : super::unit::Position
}

impl Renderer{
    pub fn new() -> Renderer{
        Renderer{
            fonts  : HashMap::<String, graphics::Font>::new(),
            mb     : graphics::MeshBuilder::new(),
            meshes : Vec::<Mesh>::new(),
            cam_tr : super::unit::Position{x: 0.0, y:0.0}
        } 
    }

    pub fn start_frame(&mut self, ctx: &mut Context, t : super::unit::Position){
        graphics::clear(ctx, graphics::BLACK);
        self.cam_tr = t;
    }

    pub fn push_cam_transform(&mut self, ctx: &mut Context){
        let cam_transform = DrawParam::default().dest(self.cam_tr).to_matrix();
        graphics::push_transform(ctx, Some(cam_transform));
        graphics::apply_transformations(ctx).unwrap();
    }

    pub fn pop_cam_transform(&mut self, ctx: &mut Context){
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx).unwrap();       
    }

    pub fn start_batch(&mut self) {
        self.mb = graphics::MeshBuilder::new();
    }
    pub fn end_batch(&self, ctx: &mut Context){
        let mesh = self.mb.build(ctx).unwrap();
        mesh.draw(ctx, DrawParam::default().dest([0.0,0.0])).unwrap();
    }

    pub fn end_frame(&self, ctx: &mut Context) -> GameResult<()>{
        return graphics::present(ctx);
    }

    pub fn build_mesh(&mut self, pts : Vec::<Position>, color: Color, ctx: &mut Context) -> Renderable{
        let mut mb = graphics::MeshBuilder::new();
        let _ = mb.polygon(DrawMode::fill(), &pts, color);
        let mesh = mb.build(ctx).unwrap();
        self.meshes.push(mesh);
        Renderable::StaticRect( self.meshes.len() - 1)
    }
}

pub enum TextRenderState{
    Dirty,    
    TextState(graphics::Text),
}

pub enum TextAnchor{
    Center,
    TopLeft
}

pub enum Renderable{
    NoDraw,
    StaticRect(usize),
    DynamicRect{ color : Color, size : super::unit::Size},
    StaticText{text: graphics::Text, text_anchor: TextAnchor },
    DynamicTextDraw { string: String, fontstyle : super::text::FontStyle},
    
}

impl Renderable {
    pub fn draw(&self, transform : super::unit::Position, renderer : &mut Renderer, ctx : &mut Context){
        match self {
            Renderable::NoDraw => (),
            Renderable::StaticRect(idx) => {
                let _ = renderer.meshes[*idx].draw(ctx, DrawParam::default().dest(transform));    
            },
            Renderable::DynamicRect{color, size} => {
                renderer.mb.rectangle(
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
            Renderable::DynamicTextDraw{string, fontstyle} => {
                let font = renderer.fonts[&fontstyle.name];
                let text = graphics::Text::new( (string.clone() , font, fontstyle.size) );                
                let t =  transform.clone();
                // t.x -= text.width(ctx) as f32 / 2.0;
                // t.y -= text.height(ctx) as f32 / 2.0;
                let _ = text.draw(ctx, DrawParam::default().dest(t).color(fontstyle.color));
            }            
        } 
    }
}