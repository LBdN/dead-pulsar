use std::collections::HashMap;
use ggez::{Context};
use ggez::graphics::{Color};
use rand::Rng;
use crate::unit::*;
use crate::effect;
use crate::text;
use crate::color;
use crate::terrain;
use crate::render;
use crate::GameState;
use crate::actors;

fn random_rect(maxsize : f32, world_size : &Size) -> (Position, Size) {
    let mut rng = rand::thread_rng();
    let x    = rng.gen_range(0.0, world_size.x) as f32;
    let y    = rng.gen_range(0.0, world_size.y) as f32;
    let size = rng.gen_range(0.0, maxsize) as f32;            
    (Position{x:x, y:y}, Size{x:size, y:size})
}

pub struct WorldBounds{
    pub min: Size,
    pub max: Size
}

pub struct WorldChange{
    pub score : u32,
    pub level : Option::<Id>,
    pub dead_effect : bool
}

impl WorldChange{
    pub fn default() -> WorldChange{
        WorldChange{
            score:0,
            level: None,
            dead_effect: false
        }
    }
}

type KeyedEffects = KeyedGroup::<effect::Effect>;

pub struct World{
    start_effects : KeyedEffects,
    effects       : KeyedEffects,
    pub actors        : Vec::<super::actors::Actor>,        
    player_atr_id : Id,
    camera_atr_id : Id,
    //
    active        : bool,
    pub name          : String,
    pub size          : Size

}

impl World{
    pub fn empty() -> World{
        World::new("".to_string())
    }

    fn new(name : String) -> World {
        World{
            start_effects : KeyedEffects::new(),
            effects       : KeyedEffects::new(),
            actors        : Vec::<super::actors::Actor>::new(),    
            player_atr_id : no_id(),
            camera_atr_id : no_id(),
            active        : false,
            name          : name,
            size          :Size{x:0.0, y:0.0}
            
        }
    }

    // pub fn start(&mut self, ctx: &Context, input : &super::InputState){
    pub fn start(&mut self, ctx: &Context, state : &GameState){
        self.active = true;
        //
        let wb = WorldBounds{min: self.get_camera_actor().transform, max:self.size};
        for a in &mut self.actors{
            a.start();
        }                    
        //..
        for a in &mut self.actors{
            for effs in self.start_effects.get_mut(&a.id){
                for e in effs{
                    e.on_actor(a, ctx, state, &wb );
                }
            }
        }
        self.start_effects.clear();
    }

    pub fn stop(&mut self){
        self.active = false;
        self.actors.clear();
        self.effects.clear();
    }

    pub fn get_camera_actor(&self) -> &actors::Actor {
        self.get_actor(&self.camera_atr_id).unwrap()     
    }

    fn get_player_actor(&self) -> &actors::Actor {
        self.get_actor(&self.player_atr_id).unwrap()     
    }

    pub fn get_actor(&self, id : &Id) -> Option::<&actors::Actor> {
        for a in &self.actors{
            if a.id == *id {
                return Some(a);
            }
        }
        None
    }

    fn get_mut_actor(&mut self, id : &Id) -> Option::<&mut actors::Actor> {
        for a in &mut self.actors{
            if a.id == *id {
                return Some(a);
            }
        }
        None
    }

    fn process_collisions(&mut self){
        if self.player_atr_id == no_id(){
            return;
        }
        
        let player_actor = self.get_player_actor(); 
        let size1      = player_actor.collision.get_size();
        let pos1       = Position{
            x : player_actor.transform.x + size1.x/2.0,
            y : player_actor.transform.y + size1.y/2.0
        };
        let collision1 = player_actor.collision;
                                
        for a in &mut self.actors {
            if !a.has_collision() {
                continue;
            }                    

            let size2 = a.collision.get_size();                    
            let pos2 =  Position{
                x : a.transform.x + size2.x/2.0,
                y : a.transform.y + size2.y/2.0
            };
            
            if super::actors::collides(&pos1, &collision1, &pos2, &a.collision){                
                self.effects.insert(a.id, a.on_collision.clone());
            }

        }                            
        
    }

    pub fn update(&mut self, ctx: &Context, state : &GameState ) -> WorldChange {
        self.process_collisions();

        let mut default_wc = WorldChange::default();
        let wb = WorldBounds{min: opposite_pos(&self.get_camera_actor().transform), max:self.size};
        for a in &mut self.actors{
            let mut eff_to_remove = Vec::<usize>::new();
            for effs in self.effects.get_mut(&a.id){
                
                for (i, e) in effs.iter_mut().enumerate(){
                    if let Some(wc) = e.on_actor(a, ctx, state, &wb ){
                        if let Some(_) = wc.level{
                            return wc;
                        } else{
                            default_wc.score += wc.score;
                        }
                        if wc.dead_effect{
                            eff_to_remove.push(i);
                        }
                    }
                }

                for i in eff_to_remove.iter().rev(){
                    effs.remove(*i);
                }
            }                       
        }
        default_wc   
    }
}


struct WorldBuilder{
    w : World
}

impl WorldBuilder{
    fn new(name : String) -> WorldBuilder{
        WorldBuilder { w : World::new(name)}
    }

    fn set_size(&mut self, size : Size){
        self.w.size = size;
    }

    fn get_mut_actor(&mut self, id : &Id) -> Option::<&mut actors::Actor>{
        self.w.get_mut_actor(id)
    }

    fn get_actor(&mut self, id : &Id) -> Option::<&actors::Actor>{
        self.w.get_actor(id)
    }

    fn add_effect_to_actor(&mut self, actor_id : &Id, eff : effect::Effect, start : bool ){
        
        let opt_effs = if start { self.w.start_effects.entry(*actor_id) } 
                       else { self.w.effects.entry(*actor_id) };            
        let effs = opt_effs.or_insert(Vec::<effect::Effect>::new());
        effs.push(eff);                            
    }

    fn add_rect_to_actor(&mut self, a : &mut super::actors::Actor, size: Size, color : Color){
        a.drawable = super::render::Renderable::DynamicRect {
            color   : color,
            size    : size,
        };            
        a.collision = super::actors::Collision::RectCollision { width: size.x, height: size.y };
    }

    fn add_to_world(&mut self, a : actors::Actor) -> Id{
        let atr_id = a.id.clone();
        self.w.actors.push(a);        
        atr_id
    }

    fn add_rect_type(&mut self, mut a :  super::actors::Actor, max_size : f32, color : Color) -> Id {
        let (pos, size) = random_rect(max_size, &self.w.size);

        a.transform = pos;
        self.add_rect_to_actor(&mut a, size, color);
        let atr_id = a.id.clone();
        self.w.actors.push(a);        
        atr_id
    }

    //

    fn add_player(&mut self) -> Id {        
        let size  = Size{x:10.0, y:10.0};        
        let mut a = super::actors::Actor::new(super::actors::ActorType::Player, get_id());
        self.add_rect_to_actor(&mut a, size, super::color::RED);
        self.w.player_atr_id = a.id.clone();
        self.w.actors.push(a);        
        self.w.player_atr_id.clone()
    }

    fn add_camera(&mut self) -> Id {
        let mut a = super::actors::Actor::new(super::actors::ActorType::Camera, get_id());        
        a.drawable  = super::render::Renderable::NoDraw;
        a.collision = super::actors::Collision::NoCollision;
        a.transform = Position{ x:0 as f32, y:0 as f32};
        
        self.w.camera_atr_id = a.id.clone();
        self.w.actors.push(a);        
        self.w.camera_atr_id.clone()        
    }

    fn add_default_camera(&mut self)  {
        if self.w.camera_atr_id == no_id(){
            let camera_start = Position{ x:0 as f32, y:0 as f32};
            let camera_id = self.add_camera();    
            let eff = effect::Effect::PlaceActor{actor_id:camera_id, position: camera_start};
            self.add_effect_to_actor(&camera_id, eff, true);
            self.w.camera_atr_id =camera_id;
        }
    }

    

    fn add_antagonist(&mut self, max_size : f32) -> Id {
        let a = actors::ActorType::Foreground.make();   
        return self.add_rect_type(a, max_size, color::random_foreground_color());
    }


      
    fn add_text(&mut self, text: String, fontstyle: super::text::FontStyle, centered: bool) -> Id{
        let mut a = super::actors::ActorType::UI.make();
        a.drawctx   = super::actors::DrawContext::ScreenSpace;
        a.drawable  = super::render::Renderable::DynamicTextDraw{ 
            string  : text,
            fontstyle : fontstyle,
            text_anchor : if centered { render::TextAnchor::Center} else {render::TextAnchor::TopLeft}        
        };
        self.add_to_world(a)        
    }


    fn add_end_rects(&mut self, exit_size : f32) -> [Id; 3]{
        
        let lose_rect_height = (self.w.size.y- exit_size) / 2.0;

        let mut res : [Id; 3] = [no_id(); 3];

        let mut yy = 0.0;
        for (i, rect_height) in [lose_rect_height, exit_size, lose_rect_height].iter().enumerate() {
            let mut a = actors::ActorType::Foreground.make();
            a.transform = Position{ x:self.w.size.x as f32, y:yy as f32};
            let size = Size{ x:50 as f32, y:*rect_height as f32 };

            self.add_rect_to_actor(&mut a, size, color::RED);
            res[i] = self.add_to_world(a);
            yy += rect_height;
        }
        
        res 
    }

    fn build(self) -> World{


        

        self.w
    }
}

type LevelLoader = fn(&Level, Position, &mut render::Renderer, &mut Context) -> World;


#[derive(Clone)]
pub struct Level{
    pub id         : Id,
    name       : String,
    transitions: HashMap::<String, Id>,
    pub loader     : LevelLoader,    
}

impl Level{
    pub fn new(name : String) -> Level{
        Level{
            id         : get_id(),
            name       : name,
            transitions: HashMap::<String, Id>::new(),
            loader     : emptyload
        }
    }

    pub fn add_transition(&mut self, transition_name : &String, level : &Level){
        self.transitions.insert(transition_name.clone(), level.id.clone());
    }

    pub fn load(&self, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
        let center = Position{x: 0.0, y: 0.0};
        return (self.loader)(self, center, renderer, ctx);
    }

    pub fn get_transition_effect(&self, transition_name : String, duration: f32) -> effect::Effect{
        let next_id = self.transitions.get(&transition_name).unwrap();
        effect::Effect::AutoNextScene{ duration:duration, cur_scene_idx : self.id.clone(), next_scene_idx : next_id.clone()} 
    }
    
}

fn emptyload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let wb = WorldBuilder::new(level.name.clone());
    wb.build()
} 

pub fn introload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let id = wb.add_text("Pulsar 3".to_string(), text::title_style(),false);     
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string(), 3.0 ), false );
    wb.get_mut_actor(&id).map( |a| {
        a.transform = center;      
        a
    });
           
    wb.add_default_camera();
    wb.build()
} 

pub fn tutoload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let tuto_text = "Catch the yellow blocks and\n exit with the green one.".to_string();
    let id = wb.add_text(tuto_text, text::tuto_style(),false);     
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string(), 3.0), false );
    wb.get_mut_actor(&id).unwrap().transform = center;        

    wb.add_default_camera();
    wb.build()
}

pub fn gameoverload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let id = wb.add_text("Game Over".to_string(), text::title_style(),false);     
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string(), 3.0), false );
    wb.get_mut_actor(&id).unwrap().transform = center;        

    wb.add_default_camera();
    wb.build()
} 

pub fn victoryload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let id = wb.add_text("Victory".to_string(), text::title_style(),false);     
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string(), 3.0), false );
    wb.get_mut_actor(&id).unwrap().transform = center;        

    wb.add_default_camera();
    wb.build()
} 

pub fn playload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());
    wb.set_size(Size{x:1000.0, y:600.0});

    // BACKGROUND.
    {
        let mut a = actors::ActorType::Background.make();
        let mut mb = render::MeshBuilderOps::new();    
        let max_size = 50.0;
        for _ in 0..10000{
            let r = random_rect(max_size, &wb.w.size);
            mb = mb.rect(&r.0, &r.1, color::random_grey_color());
        }
        let nbsteps = 10;
        for (i, c)  in color::fade_to(nbsteps, &color::RED, &color::GREEN).iter().enumerate(){
            let b1 = Bounds{min: Size{x:0.0, y:((nbsteps - i as i32) as f32)*10.0}, max: wb.w.size};
            let mut pts = terrain::build_terrain(&b1, wb.w.size.x / 10.0);
            terrain::invert_pos(&wb.w.size, &mut pts);
            mb = mb.polygon(pts, *c);        
        }
        for (i, c)  in color::fade_to(nbsteps, &color::RED, &color::GREEN).iter().enumerate(){
            let b1 = Bounds{min: Size{x:0.0, y:((nbsteps - i as i32) as f32)*10.0}, max: wb.w.size};
            let pts = terrain::build_terrain(&b1, wb.w.size.x / 10.0);            
            mb = mb.polygon(pts, *c);        
        }
        
        let drawable = mb.build(renderer, ctx);
        a.drawable = drawable;
        wb.add_to_world(a);
    }
    
    // ENEMIES
    {
        let max_size = 50.0;
        // let sound_idx = wb.add_sound("/Randomize6.wav".to_string(), &mut ctx);
        for _ in 0..100{
            let id = wb.add_antagonist(max_size);
            wb.add_effect_to_actor(&id, effect::Effect::ResetActor{actor_id : id.clone()}, true);
            let a = wb.get_mut_actor(&id).unwrap();
            a.on_collision.push(effect::Effect::KillActor{actor_id:a.id.clone()});            
            // a.on_collision.push(effect::Effect::PlaySound{sound_index:sound_idx});
        }
    }

    // END TRIGGER
    {
        let ids = wb.add_end_rects(50.0);
        let a = wb.get_mut_actor(&ids[0]).unwrap();
        a.on_collision.push(level.get_transition_effect("lose".to_string(), 0.0));            
        let a = wb.get_mut_actor(&ids[1]).unwrap();
        a.on_collision.push(level.get_transition_effect("win".to_string(), 0.0));            
        if let render::Renderable::DynamicRect{ref mut color, ..} = a.drawable {
            *color = color::GREEN;
        }
        let a = wb.get_mut_actor(&ids[2]).unwrap();
        a.on_collision.push(level.get_transition_effect("lose".to_string(), 0.0));                
    }
    

    // PLAYER
    {
        let mut player_start = center.clone();
        player_start.x = 10.0;
        let player_actor_id = wb.add_player();
        let eff = effect::Effect::MoveActor{actor_id:player_actor_id, vector:Position{x :1.0, y:0.0}};
        wb.add_effect_to_actor(&player_actor_id, eff, false);
        wb.add_effect_to_actor(&player_actor_id, effect::Effect::ProcessInput, false);
        let eff = effect::Effect::PlaceActor{actor_id:player_actor_id, position: player_start};
        wb.add_effect_to_actor(&player_actor_id, eff, true);
    }

    // CAMERA
    let camera_start = Position{ x:0 as f32, y:0 as f32};
    let camera_id = wb.add_camera();
    let eff = effect::Effect::MoveActor{actor_id:camera_id, vector:Position{x :-1.0, y:0.0}};
    wb.add_effect_to_actor(&camera_id, eff, false);
    let eff = effect::Effect::PlaceActor{actor_id:camera_id, position: camera_start};
    wb.add_effect_to_actor(&camera_id, eff, true);
    wb.w.camera_atr_id =camera_id;

    // UI
    {
        let text_id  = wb.add_text("Pulsar 3".to_string(), text::ui_style(), false);

        let ui_pos =Position{x: 10.0, y: 10.0}; 
        wb.get_mut_actor(&text_id).map(|a|{
            a.transform = ui_pos.clone();    
            a.drawable = renderer.convert_to_static_text(&a.drawable);
            a
        });

        let margin = 10.0;
        let mut p = ui_pos.clone();
        if let Some(a) = wb.get_actor(&text_id){
            if let render::Renderable::StaticText(i) = a.drawable{
                let (w, _) = renderer.texts[i].text.dimensions(ctx);
                p.x = a.transform.x+(w as f32) +margin;        
            }
        }
                                
        let text_id = wb.add_text("Score: 0".to_string(), text::ui_style(), false);    
        wb.get_mut_actor(&text_id).map(|a|{ 
            a.transform = p;
            a
        });        
        wb.add_effect_to_actor(&text_id, effect::Effect::SetScore{new_value : 0}, true);
        wb.add_effect_to_actor(&text_id, effect::Effect::UpdateScore{actor_id:text_id}, false);
    }

    wb.build()
}