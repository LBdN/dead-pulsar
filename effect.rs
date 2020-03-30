use crate::App;
use crate::unit::*;
use crate::render;
use crate::player_handle_input;
use crate::GameState;
use crate::color;
use crate::actors;
use crate::level;
use crate::{Systems};
use ggez::timer;

use ggez::audio::{SoundSource};
use ggez::{Context};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Effect{
    PlaceActor{actor_id: Id, position: Position},    
    MoveActor{actor_id: Id, vector: Position},
    UpdateScore{actor_id: Id},
    SetScore{new_value : i32},
    ProcessInput,
    KillActor{actor_id: Id},
    ResetActor{actor_id: Id},
    // NextScene{cur_scene_idx : usize, next_scene_idx : usize},
    AutoNextScene{ duration : f32, cur_scene_idx : Id, next_scene_idx : Id},
    PlaySound(usize),
}

impl Effect{
    pub fn apply(&self, app : &mut App, t : f32) -> bool{
        match self{
            _ => {
                true
            }

        }        
    } 

    pub fn on_actor(&mut self, _actor : &mut actors::Actor, _ctx: &Context, state : &GameState, worldbounds : &level::WorldBounds, systems : &mut Systems) -> Option::<level::WorldChange>{        
        match self {
            Effect::AutoNextScene{duration, cur_scene_idx, next_scene_idx} => {
                *duration -= timer::delta(_ctx).as_secs_f32();
                if *duration < 0.0 {
                    let levelchange = level::WorldChange {
                        score: 0,
                        level: Some(next_scene_idx.clone()),
                        dead_effect: false
                    };
                    return Some(levelchange);                
                }
                None                
            },
            Effect::KillActor{actor_id} => {                
                if let render::Renderable::DynamicRect{ref mut color, ..} = _actor.drawable {
                    *color = color::GREEN;
                }      
                if let render::Renderable::DynamicPoly{poly_idx, mesh_oidx, ref mut dirty} = _actor.drawable {
                    systems.renderer.polygons.get_mut(poly_idx).map(|poly| {
                        poly.color = color::GREEN;
                        poly
                    });
                    *dirty = true;
                }                 
                _actor.ticking = false;       
                _actor.collision = actors::mk_nocol();         
                Some(level::WorldChange {
                    score: 1,
                    level: None,
                    dead_effect: true
                })
            },
            Effect::ProcessInput => {         
                player_handle_input(&state.input, _actor, &worldbounds, timer::delta(_ctx).as_millis());
                None
            },
            Effect::MoveActor{actor_id, vector} => {                
                _actor.transform.x += vector.x;
                _actor.transform.y += vector.y;                                    
                None
            },
            Effect::PlaceActor{actor_id, position} => {                
                _actor.transform = *position;
                None
            },
            Effect::UpdateScore{actor_id} => {                                    
                if let render::Renderable::DynamicTextDraw{string, ..} = &mut _actor.drawable{
                    *string = format!( "Score: {}", state.score);
                }
                None
            },
            Effect::PlaySound(sound_index) => {
                let s = systems.sounds.get_mut(*sound_index).unwrap();
                let _ = s.play();      
                Some(level::WorldChange {
                    score: 0,
                    level: None,
                    dead_effect: true
                })          
            },
            _ => None
        }
        
    }
}

#[derive(Default)]
pub struct EffectResult{
    pub scene_changed : bool,
    pub dead_effects  : Vec::<usize>
}