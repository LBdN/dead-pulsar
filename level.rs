use std::collections::HashMap;
use ggez::{Context};

pub struct WorldChange{
    pub score : u32,
    pub level : Option::<super::unit::Id>
}

type KeyedEffects = HashMap<super::unit::Id, Vec::<super::Effect>>;

struct World{
    start_effects : KeyedEffects,
    effects       : KeyedEffects,
    actors        : Vec::<super::actors::Actor>,    
    active        : bool,
    name          : String,
    player_atr_id : super::unit::Id,
    camera_atr_id : super::unit::Id
}

impl World{
    fn new(name : String) -> World {
        World{
            start_effects : KeyedEffects::new(),
            effects       : KeyedEffects::new(),
            actors        : Vec::<super::actors::Actor>::new(),    
            active        : false,
            name          : name,
            player_atr_id : super::unit::get_id(),
            camera_atr_id :super::unit::get_id()
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
        let pos1       = super::unit::Position{
            x : player_actor.transform.x + size1.x/2.0,
            y : player_actor.transform.y + size1.y/2.0
        };
        let collision1 = player_actor.collision;
                                
        for a in &mut self.actors {
            if !a.has_collision() {
                continue;
            }                    

            let size2 = a.collision.get_size();                    
            let pos2 =  super::unit::Position{
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

    fn add_background_rect(){

    }

    fn add_actor_rect(){

    }

    fn build(self) -> World{
        self.w
    }
}

struct Level{
    id   : super::unit::Id,
    name : String,
    transitions : HashMap::<String, super::unit::Id>
}

impl Level{
    pub fn new(name : String) -> Level{
        Level{
            id : super::unit::get_id(),
            name : name,
            transitions : HashMap::<String, super::unit::Id>::new()
        }
    }
    fn load(&self) -> World {
        let wb = WorldBuilder::new(self.name.clone());
        wb.build()
    }
}