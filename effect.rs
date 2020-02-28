use crate::App;
use crate::unit::*;
use crate::render;
use crate::player_handle_input;
use crate::InputState;
use crate::color;
use crate::actors;
use crate::level;
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
    PlaySound{sound_index : Id},
}

impl Effect{
    pub fn apply(&self, app : &mut App, t : f32) -> bool{
        match self{
            Effect::MoveActor{actor_id, vector} => {
                if let Some(a) = app.actors.get_mut(actor_id){
                    a.transform.x += vector.x;
                    a.transform.y += vector.y;                    
                }
                return false;
            },
            Effect::UpdateScore{actor_id} => {
                if let Some(pa) = app.player.as_mut(){                    
                    if let Some(label_actor) = app.actors.get_mut(actor_id){ 
                        if let render::Renderable::DynamicTextDraw{string, ..} = &mut label_actor.drawable{
                            *string = format!( "Score: {}", pa.score);
                        }
                    }
                }
                return false;
            },
            Effect::SetScore{new_value} => {
                if let Some(pa) = app.player.as_mut(){                    
                        pa.score = *new_value;
                }
                return false;
            },
            Effect::PlaceActor{actor_id, position} => {
                let player_actor = app.actors.get_mut(actor_id).unwrap();
                player_actor.transform = *position;
                true
            },
            Effect::ProcessInput => {
                if let Some(pa) = app.player.as_mut(){            
                    if let Some(player_actor) = app.actors.get_mut(&pa.actor_id){                        
                        //processing input
                        player_handle_input(&pa.input, player_actor);
                    }
                }
                return false;
            },
            Effect::KillActor{actor_id} => {
                let a = app.actors.get_mut(actor_id).unwrap();
                if let render::Renderable::DynamicRect{ref mut color, ..} = a.drawable {
                    *color = color::GREEN;
                }                 
                a.ticking = false;                
                return true;
            },
            Effect::ResetActor{actor_id} => {
                let a = app.actors.get_mut(actor_id).unwrap();
                if let render::Renderable::DynamicRect{ref mut color, ..} = a.drawable {
                    *color = color::random_foreground_color();
                } 
                
                a.ticking = true;                
                false
            }
            Effect::AutoNextScene{duration, cur_scene_idx, next_scene_idx} => {
                if *duration < t {
                    let current_scene = & app.scenes[cur_scene_idx];                                     
                    let next_scene    = & app.scenes[next_scene_idx];
                    if current_scene.active == false && next_scene.active == true{
                        return false;
                    }
                    // for (idx, mut scene) in &mut app.scenes.iter().enumerate(){
                    //     scene.active = idx == *next_scene_idx;
                    // }
                    let current_scene = app.get_mut_scene(cur_scene_idx);    
                    current_scene.active = false;
                    current_scene.clone().stop(app);
                    let next_scene    = app.get_mut_scene(next_scene_idx);
                    next_scene.active = true;
                    next_scene.clone().start(app);
                    app.current_scene = *next_scene_idx;
                    return false;                
                }
                return false;                
            },
            Effect::PlaySound{sound_index} => {
                let s = app.sounds.get_mut(sound_index).unwrap();
                let _ = s.play();
                return true;
            }

        }        
    } 

    pub fn on_actor(&mut self, _actor : &mut actors::Actor, _ctx: &Context, _input : &InputState) -> Option::<level::WorldChange>{        
        match self {
            Effect::AutoNextScene{duration, cur_scene_idx, next_scene_idx} => {
                *duration -= timer::delta(_ctx).as_secs_f32();
                if *duration < 0.0 {
                    let levelchange = level::WorldChange {
                        score: 0,
                        level: Some(next_scene_idx.clone())
                    };
                    return Some(levelchange);                
                }
                None                
            },
            Effect::KillActor{actor_id} => {                
                if let render::Renderable::DynamicRect{ref mut color, ..} = _actor.drawable {
                    *color = color::GREEN;
                }                 
                _actor.ticking = false;                
                None
            },
            Effect::ProcessInput => {                
                player_handle_input(_input, _actor);
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
            _ => None
        }
        
    }
}

#[derive(Default)]
pub struct EffectResult{
    pub scene_changed : bool,
    pub dead_effects  : Vec::<usize>
}