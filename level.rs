use std::collections::HashMap;
use ggez::{Context};
use ggez::graphics::{Color};
use rand::Rng;
use crate::unit::*;
use crate::effect;


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

struct World{
    start_effects : KeyedEffects,
    effects       : KeyedEffects,
    actors        : Vec::<super::actors::Actor>,        
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

    fn start(&mut self, ctx: &Context, input : &super::InputState){
        self.active = true;
        //..
        for a in &mut self.actors{
            for effs in self.start_effects.get_mut(&a.id){
                for e in effs{
                    e.on_actor(a, ctx, input );
                }
            }
        }
        self.start_effects.clear();
    }

    fn stop(&mut self){
        self.active = false;
        self.actors.clear();
        self.effects.clear();
    }

    fn get_player_actor(&self) -> &super::actors::Actor {
        for a in &self.actors{
            if a.id == self.player_atr_id {
                return a
            }
        }
        return &self.actors[0];
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

    fn add_effect_to_actor(&mut self, a : &super::actors::Actor, eff : effect::Effect, start : bool ){
        let opt_effs = if start { self.w.start_effects.get_mut(&a.id) } 
                       else { self.w.effects.get_mut(&a.id) };            
        if let Some(effs) = opt_effs {
            effs.push(eff);
        }                    
    }

    fn add_rect_to_actor(&mut self, a : &mut super::actors::Actor, size: Size, color : Color){
        a.drawable = super::render::Renderable::DynamicRect {
            color   : color,
            size    : size,
        };            
        a.collision = super::actors::Collision::RectCollision { width: size.x, height: size.y };
    }

    fn add_rect_type(&mut self, mut a :  super::actors::Actor, max_size : f32) -> Id {
        let (x, y, size) = random_rect(max_size, &self.w.size);

        a.transform = Position{ x:x, y:y};
        self.add_rect_to_actor(&mut a, size, super::color::random_foreground_color());
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
        return self.add_rect_type(a, max_size);
    }

    fn add_background(&mut self, max_size : f32) -> Id {
        let mut a = super::actors::Actor::new(super::actors::ActorType::Background, get_id());     
        return self.add_rect_type(a, max_size);
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

struct Level{
    id   : Id,
    name : String,
    transitions : HashMap::<String, Id>
}

impl Level{
    pub fn new(name : String) -> Level{
        Level{
            id : get_id(),
            name : name,
            transitions : HashMap::<String, Id>::new()
        }
    }
    fn load(&self) -> World {
        let wb = WorldBuilder::new(self.name.clone());
        wb.build()
    } 
}