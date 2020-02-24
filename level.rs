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


fn random_rect(maxsize : f32, world_size : &Size) -> (f32, f32, Size) {
    let mut rng = rand::thread_rng();
    let x    = rng.gen_range(0.0, world_size.x) as f32;
    let y    = rng.gen_range(0.0, world_size.y) as f32;
    let size = rng.gen_range(0.0, maxsize) as f32;            
    (x, y, Size{x:size, y:size})
}

pub struct WorldChange{
    pub score : u32,
    pub level : Option::<Id>
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
    name          : String,
    size          : Size

}

impl World{
    fn new(name : String) -> World {
        World{
            start_effects : KeyedEffects::new(),
            effects       : KeyedEffects::new(),
            actors        : Vec::<super::actors::Actor>::new(),    
            player_atr_id : get_id(),
            camera_atr_id : get_id(),
            active        : false,
            name          : name,
            size          :Size{x:0.0, y:0.0}
            
        }
    }

    // pub fn start(&mut self, ctx: &Context, input : &super::InputState){
    pub fn start(&mut self){
        self.active = true;
        //
        for a in &mut self.actors{
            a.visible = true;
        }            
        return;
        //..
        // for a in &mut self.actors{
        //     for effs in self.start_effects.get_mut(&a.id){
        //         for e in effs{
        //             e.on_actor(a, ctx, input );
        //         }
        //     }
        // }
        // self.start_effects.clear();
    }

    fn stop(&mut self){
        self.active = false;
        self.actors.clear();
        self.effects.clear();
    }

    fn get_player_actor(&self) -> &super::actors::Actor {
        self.get_actor(&self.player_atr_id).unwrap()     
    }

    fn get_actor(&self, id : &Id) -> Option::<&super::actors::Actor> {
        for a in &self.actors{
            if a.id == *id {
                return Some(a);
            }
        }
        None
    }

    fn get_mut_actor(&mut self, id : &Id) -> Option::<&mut super::actors::Actor> {
        for a in &mut self.actors{
            if a.id == *id {
                return Some(a);
            }
        }
        None
    }

    fn process_collisions(&mut self){

        
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

    fn update(&mut self, ctx: &Context, input : &super::InputState ) -> WorldChange {
        self.process_collisions();

        let mut default_wc = WorldChange {
            score: 0,
            level: None
        };

        for a in &mut self.actors{
            for effs in self.effects.get_mut(&a.id){
                for e in effs{
                    if let Some(wc) = e.on_actor(a, ctx, input ){
                        if let Some(_) = wc.level{
                            return wc;
                        } else{
                            default_wc.score += wc.score;
                        }
                    }
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

    fn get_mut_actor(&mut self, id : &Id) -> Option::<&mut super::actors::Actor>{
        self.w.get_mut_actor(id)
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

    fn add_rect_type(&mut self, mut a :  super::actors::Actor, max_size : f32, color : Color) -> Id {
        let (x, y, size) = random_rect(max_size, &self.w.size);

        a.transform = Position{ x:x, y:y};
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

    fn add_antagonist(&mut self, max_size : f32) -> Id {
        let mut a = super::actors::Actor::new(super::actors::ActorType::Foreground, get_id());   
        return self.add_rect_type(a, max_size, color::random_foreground_color());
    }

    fn add_background(&mut self, max_size : f32) -> Id {
        let mut a = super::actors::Actor::new(super::actors::ActorType::Background, get_id());     
        return self.add_rect_type(a, max_size, color::GREEN);        
    }

    fn add_terrain(&mut self, renderer: &mut render::Renderer, ctx : &mut Context) -> Id {
        let mut a = super::actors::Actor::new(super::actors::ActorType::Terrain, get_id());     
        let pts = terrain::build_terrain(self.w.size, self.w.size.x / 10.0);        
        a.drawable = renderer.build_mesh(pts, color::RED, ctx);
        let id = a.id.clone();
        self.w.actors.push(a);        
        id
    }
      
    fn add_text(&mut self, text: String, fontstyle: super::text::FontStyle, centered: bool) -> Id{
        let mut a   = super::actors::Actor::new(super::actors::ActorType::UI, get_id());
        a.drawctx   = super::actors::DrawContext::ScreenSpace;
        a.drawable  = super::render::Renderable::DynamicTextDraw{ 
            string  : text,
            fontstyle : fontstyle            
        };
        let atr_id = a.id.clone();
        self.w.actors.push(a);        
        atr_id            
    }

    fn build(self) -> World{
        self.w
    }
}

type LevelLoader = fn(&Level, Position, &mut render::Renderer, &mut Context) -> World;


pub struct Level{
    id         : Id,
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

    pub fn get_transition_effect(&self, transition_name : String) -> effect::Effect{
        let next_id = self.transitions.get(&transition_name).unwrap();
        effect::Effect::AutoNextScene{ duration:3.0, cur_scene_idx : self.id.clone(), next_scene_idx : next_id.clone()} 
    }
    
}

fn emptyload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let wb = WorldBuilder::new(level.name.clone());
    wb.build()
} 

pub fn introload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let id = wb.add_text("Pulsar 3".to_string(), text::title_style(),true);     
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string()), false );
    wb.get_mut_actor(&id).unwrap().transform = center;        

    wb.build()
} 

pub fn tutoload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let tuto_text = "Catch the yellow blocks and\n exit with the green one.".to_string();
    let id = wb.add_text(tuto_text, text::tuto_style(),true);     
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string()), false );
    wb.get_mut_actor(&id).unwrap().transform = center;        

    wb.build()
}

pub fn gameoverload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let id = wb.add_text("Game Over".to_string(), text::title_style(),true);     
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string()), false );
    wb.get_mut_actor(&id).unwrap().transform = center;        

    wb.build()
} 

pub fn victoryload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let id = wb.add_text("Victory".to_string(), text::title_style(),true);     
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string()), false );
    wb.get_mut_actor(&id).unwrap().transform = center;        

    wb.build()
} 

pub fn playload(level : &Level, center : Position, renderer: &mut render::Renderer, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());
    wb.set_size(Size{x:1000.0, y:640.0});

    wb.add_terrain(renderer, ctx);
    let max_size = 50.0;
    for _ in 0..1000{
        wb.add_background(max_size);
    }
    wb.add_terrain(renderer, ctx);
    
    // let sound_idx = wb.add_sound("/Randomize6.wav".to_string(), &mut ctx);
    for _ in 0..1000{
        let id = wb.add_antagonist(max_size);
        wb.add_effect_to_actor(&id, effect::Effect::ResetActor{actor_id : id.clone()}, true);
        let a = wb.get_mut_actor(&id).unwrap();
        a.on_collision.push(effect::Effect::KillActor{actor_id:a.id.clone()});            
        // a.on_collision.push(effect::Effect::PlaySound{sound_index:sound_idx});
    }

    let mut player_start = center.clone();
    player_start.x = 10.0;
    let player_actor_id = wb.add_player();
    let eff = effect::Effect::MoveActor{actor_id:player_actor_id, vector:Position{x :1.0, y:0.0}};
    wb.add_effect_to_actor(&player_actor_id, eff, false);
    wb.add_effect_to_actor(&player_actor_id, effect::Effect::ProcessInput, false);
    let eff = effect::Effect::PlaceActor{actor_id:player_actor_id, position: player_start};
    wb.add_effect_to_actor(&player_actor_id, eff, true);

    let camera_start = Position{ x:0 as f32, y:0 as f32};
    let camera_id = wb.add_camera();
    let eff = effect::Effect::MoveActor{actor_id:camera_id, vector:Position{x :1.0, y:0.0}};
    wb.add_effect_to_actor(&camera_id, eff, false);
    let eff = effect::Effect::PlaceActor{actor_id:camera_id, position: camera_start};
    wb.add_effect_to_actor(&camera_id, eff, true);

    let text_id  = wb.add_text("Pulsar 3".to_string(), text::ui_style(), true);
    wb.get_mut_actor(&text_id).unwrap().transform = Position{x: 10.0, y: 10.0};    

    let margin = 10.0;
    // let gtext = renderer.render(wb.get_mut_actor(text_id).drawable);
    // let p = Position{x:a.transform.x+gtext.width(&mut ctx) as f32 +margin, y: 10.0};        
    let text_id = wb.add_text("Score: 0".to_string(), text::ui_style(), false);    
    // wb.get_mut_actor(&text_id).unwrap().transform= p;
    wb.add_effect_to_actor(&text_id, effect::Effect::SetScore{new_value : 0}, true);
    wb.add_effect_to_actor(&text_id, effect::Effect::UpdateScore{actor_id:text_id}, false);

    wb.build()
}