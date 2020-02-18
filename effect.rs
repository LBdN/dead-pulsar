use crate::App;
use crate::unit;
use crate::render;
use crate::player_handle_input;
use crate::InputState;
use crate::color;
use crate::actors;
use crate::level;

use ggez::audio::{SoundSource};
use ggez::{Context};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Effect{
    PlaceActor{actor_idx: usize, position: unit::Position},    
    MoveActor{actor_idx: usize, vector: unit::Position},
    UpdateScore{actor_idx: usize},
    SetScore{new_value : i32},
    ProcessInput,
    KillActor{actor_idx: usize},
    ResetActor{actor_idx: usize},
    // NextScene{cur_scene_idx : usize, next_scene_idx : usize},
    AutoNextScene{ duration : f32, cur_scene_idx : usize, next_scene_idx : usize},
    PlaySound{sound_index : usize},
}

impl Effect{
    pub fn apply(&self, app : &mut App, t : f32) -> bool{
        match self{
            Effect::MoveActor{actor_idx, vector} => {
                let a = &mut app.actors[*actor_idx];
                a.transform.x += vector.x;
                a.transform.y += vector.y;
                return false;
            },
            Effect::UpdateScore{actor_idx} => {
                if let Some(pa) = app.player.as_mut(){                    
                    if let Some(label_actor) = app.actors.get_mut(*actor_idx){ 
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
            Effect::PlaceActor{actor_idx, position} => {
                let player_actor = &mut app.actors[*actor_idx];
                player_actor.transform = *position;
                true
            },
            Effect::ProcessInput => {
                if let Some(pa) = &app.player{            
                    if let Some(player_actor) = app.actors.get_mut(pa.actor_idx){                        
                        //processing input
                        player_handle_input(&pa, player_actor);
                    }
                }
                return false;
            },
            Effect::KillActor{actor_idx} => {
                let a = &mut app.actors[*actor_idx];
                if let render::Renderable::DynamicRect{ref mut color, ..} = a.drawable {
                    *color = color::GREEN;
                }                 
                a.ticking = false;                
                return true;
            },
            Effect::ResetActor{actor_idx} => {
                let a = &mut app.actors[*actor_idx];
                if let render::Renderable::DynamicRect{ref mut color, ..} = a.drawable {
                    *color = color::random_foreground_color();
                } 
                
                a.ticking = true;                
                false
            }
            Effect::AutoNextScene{duration, cur_scene_idx, next_scene_idx} => {
                if *duration < t {
                    let current_scene = & app.scenes[*cur_scene_idx];                                     
                    let next_scene    = & app.scenes[*next_scene_idx];
                    if current_scene.active == false && next_scene.active == true{
                        return false;
                    }
                    // for (idx, mut scene) in &mut app.scenes.iter().enumerate(){
                    //     scene.active = idx == *next_scene_idx;
                    // }
                    let current_scene = &mut app.scenes[*cur_scene_idx];    
                    current_scene.active = false;
                    current_scene.clone().stop(app);
                    let next_scene    = &mut app.scenes[*next_scene_idx];
                    next_scene.active = true;
                    next_scene.clone().start(app);
                    app.current_scene = *next_scene_idx;
                    return false;                
                }
                return false;                
            },
            Effect::PlaySound{sound_index} => {
                let s = &mut app.sounds[*sound_index];
                let _ = s.play();
                return true;
            }

        }        
    }

    pub fn on_actor(&self, _actor : &mut actors::Actor, _ctx: &Context, _input : &InputState) -> Option::<level::WorldChange>{
        // level::WorldChange {
        //     score: 0,
        //     level: None
        // }
        None
    }
}

#[derive(Default)]
pub struct EffectResult{
    pub scene_changed : bool,
    pub dead_effects  : Vec::<usize>
}