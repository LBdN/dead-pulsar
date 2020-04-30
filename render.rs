use ggez::graphics;
use ggez::graphics::{DrawParam, Color, Rect, Drawable, DrawMode, Mesh, StrokeOptions};
use ggez::{Context, GameResult};

use crate::unit::*;
use crate::text;
use std::collections::HashMap;

pub struct MeshModel{
    pub polygons : Vec::<MeshModelPoly>,
    mesh_oidx: Option<usize>,
    pub  dirty    : bool,
}

impl MeshModel{
    pub fn new() -> Self{
        MeshModel{
            polygons : Vec::<MeshModelPoly>::new(),
            mesh_oidx: None,
            dirty : false
        }
    }

    pub fn draw(&mut self, transform : super::unit::Position, renderer : &mut Renderer, ctx : &mut Context){        
        if self.polygons.len() == 0 {
            return;
        }
        if self.dirty {
            let mut mb = MeshBuilderOps::new();
            for poly in &self.polygons{
                if let PolyMode::Filled = poly.mode {
                    mb.polygon_ref(&poly.positions, poly.color);
                }
                if let PolyMode::Stroked(w) = poly.mode {
                    mb.polyline_ref(&poly.positions, w, poly.color);
                }
            }                                    
            if let Some(ref mesh_idx) = self.mesh_oidx{
                mb.build_at(renderer, ctx, *mesh_idx);
            } else {
                self.mesh_oidx = Some(mb.build_(renderer, ctx));                        
            }                    
            self.dirty = false;  
        } 
        if let Some(mesh_idx) = self.mesh_oidx{
            let _ = renderer.meshes[mesh_idx].draw(ctx, DrawParam::default().dest(transform));
        }        
    }

    pub fn add_poly(&mut self, positions : &Vec::<Position>, color : &Color){
        let mmp = MeshModelPoly{ 
                        positions : positions.clone(), 
                        color : color.clone(), 
                        mode : PolyMode::Filled};
        self.polygons.push(mmp);
        self.dirty = true;
    }

    pub fn add_polyline(&mut self, positions : &Vec::<Position>, color : &Color, width : f32){
        let mmp = MeshModelPoly{ 
                        positions : positions.clone(), 
                        color : color.clone(), 
                        mode : PolyMode::Stroked(width)};
        self.polygons.push(mmp);
        self.dirty = true;
    }
}

pub struct MeshModelPoly{    
    pub positions: Vec::<Position>,
    pub color: Color,  
    pub mode : PolyMode  
}

pub enum PolyMode{
    Filled,
    Stroked(f32)
}


pub struct TextModel{
    pub string: String,
    fontstyle : text::FontStyle, 
    text_anchor : TextAnchor,
    text_oidx:Option<usize>, 
    pub dirty:bool,    
}


impl TextModel{

    pub fn new(string: String, fontstyle : text::FontStyle, text_anchor : TextAnchor) -> Self {
        TextModel{
            string: string,
            fontstyle : fontstyle, 
            text_anchor : text_anchor,
            text_oidx:None, 
            dirty:true,    
        }
    }

    pub fn draw(&mut self, transform : super::unit::Position, renderer : &mut Renderer, ctx : &mut Context){        
        let mut t =  transform.clone();
        if self.dirty {
            let font = renderer.fonts[&self.fontstyle.name];
            let text = graphics::Text::new( (self.string.clone() , font, self.fontstyle.size) );
            
            if let TextAnchor::Center = self.text_anchor {
                t.x -= text.width(ctx) as f32 / 2.0;
                t.y -= text.height(ctx) as f32;
            }


            if let Some(text_idx) = self.text_oidx{
                renderer.texts[text_idx] = text;
            } else {
                renderer.texts.push(text);
                self.text_oidx = Some( renderer.texts.len() - 1 );
            }
            self.dirty = false;  
        } 
        if let Some(text_idx) = self.text_oidx{            
            let _ = &renderer.texts[text_idx].draw(ctx, DrawParam::default().dest(t).color(self.fontstyle.color));
        }        
    }

    pub fn get_screen_size(&self, renderer : &Renderer, ctx : &mut Context) -> (u32, u32) {
        if let Some(text_idx) = self.text_oidx{
            return renderer.texts[text_idx].dimensions(ctx);
        } else {
            let font = renderer.fonts[&self.fontstyle.name];
            let text = graphics::Text::new( (self.string.clone() , font, self.fontstyle.size) );                                
            text.dimensions(ctx)
        }
    }

    pub fn update_string(&mut self, newstr : String) {
        self.string = newstr;
        self.dirty = true;
    }
}

pub struct RendererSource{
    pub meshmodels  : KeyedResource::<MeshModel>,    
    pub textmodels  : KeyedResource::<TextModel>, 
}

impl RendererSource{
    pub fn new() -> Self{
        RendererSource{            
            meshmodels : KeyedResource::<MeshModel>::new(),                        
            textmodels  : KeyedResource::<TextModel>::new(),            
        }
    }

    pub fn add_mesh_model(&mut self, mm : MeshModel) -> Id{
        let id = get_id();
        self.meshmodels.insert(id.clone(), mm);
        id
    }

    pub fn add_text_model(&mut self, tm : TextModel) -> Id{
        let id = get_id();
        self.textmodels.insert(id.clone(), tm);
        id
    }

    pub fn get_text_model(&self, id : &Id) -> Option<&TextModel>{
        self.textmodels.get(&id)
    }

    pub fn draw(&mut self, id : Id, transform : Position, ctx : &mut Context, renderer : &mut Renderer) {
        if let Some(mm)= self.meshmodels.get_mut(&id){
            mm.draw(transform, renderer, ctx);
        }
        if let Some(mm)= self.textmodels.get_mut(&id){
            mm.draw(transform, renderer, ctx);
        }
    }  
}

// pub struct RenderedText{
//     pub text: graphics::Text,
//     text_anchor: TextAnchor
// }

pub struct Renderer{
    pub fonts  : HashMap::<String, graphics::Font>,
    pub mb     : graphics::MeshBuilder,
    pub meshes      : Vec::<Mesh>,    
    pub texts       : Vec::<graphics::Text>,
    cam_tr          : Position
}

impl Renderer{
    pub fn new() -> Renderer{
        Renderer{
            fonts      : HashMap::<String, graphics::Font>::new(),
            mb         : graphics::MeshBuilder::new(),                  
            meshes     : Vec::<Mesh>::new(),            
            texts      : Vec::<graphics::Text>::new(),
            cam_tr     : super::unit::Position{x: 0.0, y:0.0}
        }
    }



    pub fn clear(&mut self){        
        self.meshes.clear();
        self.texts.clear();
    }

    pub fn start_frame(&mut self, ctx: &mut Context, t : super::unit::Position){
        graphics::clear(ctx, graphics::BLACK);
        self.cam_tr = t;
    }

    pub fn push_cam_transform(&mut self, ctx: &mut Context){
        let wh = graphics::screen_coordinates(ctx).h;
        let cam_transform = DrawParam::default()
                                .dest(self.cam_tr)
                                .scale(Vector2::new(1.0, -1.0))
                                .offset(Point2::new(0.0, 360.0))
                                .to_matrix();                
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

}

pub struct MeshBuilderOps{
    pub empty : bool,
    mb : graphics::MeshBuilder,
}

impl MeshBuilderOps{
    pub fn new() -> MeshBuilderOps{
        MeshBuilderOps{
            empty : true,
            mb: graphics::MeshBuilder::new()
        }
    }

    pub fn polygon(mut self, pts : &Vec::<Position>, color: Color) -> MeshBuilderOps{
        // DrawMode::Stroke(StrokeOptions::default())
        let _ = self.mb.polygon(DrawMode::fill(), &pts, color);
        self.empty = false;
        self
    }

    pub fn polygon_ref(&mut self, pts : &Vec::<Position>, color: Color) -> &mut MeshBuilderOps{
        // DrawMode::Stroke(StrokeOptions::default())
        let _ = self.mb.polygon(DrawMode::fill(), &pts, color);
        self.empty = false;
        self
    }        

    pub fn polyline(mut self, pts : &Vec::<Position>, width: f32, color: Color) -> MeshBuilderOps{
        let _ = self.mb.polygon(DrawMode::stroke(width), &pts, color);
        self
    }

    pub fn polyline_ref(&mut self, pts : &Vec::<Position>, width: f32, color: Color) -> &mut MeshBuilderOps{
        let _ = self.mb.polygon(DrawMode::stroke(width), &pts, color);
        self.empty = false;
        self
    }

    pub fn rect(mut self, pos : &Position, size: &Size, color1: Color) -> MeshBuilderOps {
        let _ = self.mb.rectangle(
            DrawMode::fill(),
            Rect {x:pos.x, y:pos.y, w:size.x, h:size.y},
            color1            
        );
        self.empty = false;
        self
    }

    // pub fn build(self, renderer  : &mut Renderer, ctx : &mut Context) -> Renderable {
    //     let mesh = self.mb.build(ctx).unwrap();
    //     renderer.meshes.push(mesh);
    //     Renderable::StaticMesh( renderer.meshes.len() - 1)
    // }

    pub fn build_at(self, renderer  : &mut Renderer, ctx : &mut Context, idx : usize) {
        let mesh = self.mb.build(ctx).unwrap();
        renderer.meshes[idx] = mesh;        
    }

    pub fn build_(self, renderer  : &mut Renderer, ctx : &mut Context) -> usize {
        let mesh = self.mb.build(ctx).unwrap();
        renderer.meshes.push(mesh);
        renderer.meshes.len() - 1
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

// pub enum Renderable{
//     NoDraw,
//     StaticMesh(usize),
//     StaticText(usize),
//     DynamicPoly{poly_idx:usize, mesh_oidx:Option<usize>, dirty:bool},
//     // DynamicRect{ color : Color, size : Size},
//     DynamicTextDraw { string: String, fontstyle : text::FontStyle, text_anchor : TextAnchor},

// }

// impl Renderable {
//     pub fn get_screen_size(&self, renderer : &Renderer, ctx : &mut Context) -> (u32, u32) {
//         match self {            
//             Renderable::StaticText(idx) => {
//                 renderer.texts[*idx].text.dimensions(ctx)                                                
//             },
//             Renderable::DynamicTextDraw{string, fontstyle, text_anchor } => {
//                 let font = renderer.fonts[&fontstyle.name];
//                 let text = graphics::Text::new( (string.clone() , font, fontstyle.size) );                                
//                 text.dimensions(ctx)
//             },
//             _ => (0, 0)
//         }
//     }

//     pub fn draw(&mut self, transform : super::unit::Position, renderer : &mut Renderer, ctx : &mut Context){
//         match self {
//             // Renderable::NoDraw => (),
//             // Renderable::StaticMesh(idx) => {
//             //     let _ = renderer.meshes[*idx].draw(ctx, DrawParam::default().dest(transform));
//             // },
//             // Renderable::StaticText(idx) => {
//             //     let rtext = &renderer.texts[*idx];
//             //     let mut t =  transform.clone();
//             //     if let TextAnchor::Center = rtext.text_anchor {
//             //         t.x -= rtext.text.width(ctx) as f32 / 2.0;
//             //         t.y -= rtext.text.height(ctx) as f32;
//             //     }
//             //     let _ = rtext.text.draw(ctx, DrawParam::default().dest(t));
//             // },
//             // Renderable::DynamicPoly{poly_idx, ref mut mesh_oidx, dirty} => {
//             //     if *dirty {
//             //         let mut mb = MeshBuilderOps::new();
//             //         let poly = &renderer.polygons[*poly_idx];
//             //         mb = mb.polygon(&poly.positions, poly.color);
//             //         if let Some(ref mesh_idx) = mesh_oidx{
//             //             mb.build_at(renderer, ctx, *mesh_idx);
//             //         } else {
//             //             *mesh_oidx = Some(mb.build_(renderer, ctx));                        
//             //         }                    
//             //         *dirty = false;  
//             //     } 
//             //     if let Some(mesh_idx) = mesh_oidx{
//             //         let _ = renderer.meshes[*mesh_idx].draw(ctx, DrawParam::default().dest(transform));
//             //     }
//             // },
//             Renderable::DynamicTextDraw{string, fontstyle, text_anchor } => {
//                 let font = renderer.fonts[&fontstyle.name];
//                 let text = graphics::Text::new( (string.clone() , font, fontstyle.size) );
//                 let mut t =  transform.clone();
//                 if let TextAnchor::Center = text_anchor {
//                     t.x -= text.width(ctx) as f32 / 2.0;
//                     t.y -= text.height(ctx) as f32;
//                 }
//                 let _ = text.draw(ctx, DrawParam::default().dest(t).color(fontstyle.color));
//             }
//         }
//     }
// }