use ggez::graphics;
use ggez::graphics::{DrawParam, Color, Rect, Drawable, DrawMode, Mesh, StrokeOptions};
use ggez::{Context, GameResult};

use crate::unit::*;
use crate::text;
use std::collections::HashMap;


pub struct RenderedText{
    pub text: graphics::Text, 
    text_anchor: TextAnchor 
}

pub struct Renderer{
    pub fonts  : HashMap::<String, graphics::Font>,
    pub mb     : graphics::MeshBuilder,
    pub meshes : Vec::<Mesh>,
    pub texts  : Vec::<RenderedText>,
    cam_tr     : Position
}

impl Renderer{
    pub fn new() -> Renderer{
        Renderer{
            fonts  : HashMap::<String, graphics::Font>::new(),
            mb     : graphics::MeshBuilder::new(),
            meshes : Vec::<Mesh>::new(),
            texts  : Vec::<RenderedText>::new(),
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
    

    pub fn convert_to_static_text(&mut self, drawable : &Renderable) -> Renderable {
        if let Renderable::DynamicTextDraw{string, fontstyle, text_anchor} = drawable{
            let font = self.fonts[&fontstyle.name];
            let gtext = graphics::Text::new((string.clone(), font, fontstyle.size));        
            let d = RenderedText{ text: gtext, text_anchor : *text_anchor };
            self.texts.push(d);
            return Renderable::StaticText(self.texts.len() -1 );
        };
        Renderable::NoDraw
        // let font    = self.renderer.fonts[&fontstyle.name];
        // let gtext   = graphics::Text::new((text.clone(), font, fontstyle.size));        
        // if static_{                        
        //     let text_anchor = if centered  {render::TextAnchor::Center} else {render::TextAnchor::TopLeft};
        //     a.drawable  = render::Renderable::StaticText{ text: gtext, text_anchor : text_anchor };            
    }
}

pub struct MeshBuilderOps{
    mb : graphics::MeshBuilder,
}

impl MeshBuilderOps{
    pub fn new() -> MeshBuilderOps{
        MeshBuilderOps{
            mb: graphics::MeshBuilder::new()
        }
    }

    pub fn polygon(mut self, pts : Vec::<Position>, color: Color) -> MeshBuilderOps{
        // DrawMode::Stroke(StrokeOptions::default())
        let _ = self.mb.polygon(DrawMode::fill(), &pts, color);
        self
    }

    pub fn polyline(mut self, pts : Vec::<Position>,width: f32, color: Color) -> MeshBuilderOps{        
        let _ = self.mb.polygon(DrawMode::stroke(width), &pts, color);
        self
    }

    pub fn rect(mut self, pos : &Position, size: &Size, color1: Color) -> MeshBuilderOps {
        let _ = self.mb.rectangle(
            DrawMode::fill(),
            Rect {x:pos.x, y:pos.y, w:size.x, h:size.y},
            color1
        );
        self
    }

    pub fn build(self, renderer  : &mut Renderer, ctx : &mut Context) -> Renderable {
        let mesh = self.mb.build(ctx).unwrap();
        renderer.meshes.push(mesh);
        Renderable::StaticMesh( renderer.meshes.len() - 1)
    }
}





pub enum TextRenderState{
    Dirty,    
    TextState(graphics::Text),
}

#[derive(Copy, Clone)]
pub enum TextAnchor{
    Center,
    TopLeft
}

pub enum Renderable{
    NoDraw,
    StaticMesh(usize),
    StaticText(usize),
    DynamicRect{ color : Color, size : Size},    
    DynamicTextDraw { string: String, fontstyle : text::FontStyle, text_anchor : TextAnchor},
    
}

impl Renderable {
    pub fn draw(&self, transform : super::unit::Position, renderer : &mut Renderer, ctx : &mut Context){
        match self {
            Renderable::NoDraw => (),
            Renderable::StaticMesh(idx) => {
                let _ = renderer.meshes[*idx].draw(ctx, DrawParam::default().dest(transform));    
            },
            Renderable::StaticText(idx) => {
                let rtext = &renderer.texts[*idx];
                let mut t =  transform.clone();
                if let TextAnchor::Center = rtext.text_anchor {
                    t.x -= rtext.text.width(ctx) as f32 / 2.0;
                    t.y -= rtext.text.height(ctx) as f32;
                }                
                let _ = rtext.text.draw(ctx, DrawParam::default().dest(t));
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
            Renderable::DynamicTextDraw{string, fontstyle, text_anchor } => {
                let font = renderer.fonts[&fontstyle.name];
                let text = graphics::Text::new( (string.clone() , font, fontstyle.size) );                
                let mut t =  transform.clone();
                if let TextAnchor::Center = text_anchor {
                    t.x -= text.width(ctx) as f32 / 2.0;
                    t.y -= text.height(ctx) as f32;
                }   
                let _ = text.draw(ctx, DrawParam::default().dest(t).color(fontstyle.color));
            }            
        } 
    }
}