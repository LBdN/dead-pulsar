
extern crate find_folder;
// use cgmath;
use std::env;
use std::path;
use std::collections::HashMap;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler, Axis};
use ggez::input::gamepad::GamepadId;
use ggez::input::keyboard::KeyCode;
use ggez::event::KeyMods;
use ggez::graphics;
use ggez::conf;
use ggez::audio;

use crate::unit::*;


// use cgmath::{Point2};
// use cgmath::prelude::*;

mod unit;
mod text;
mod color;
mod render;
mod actors;
mod level; 
mod effect;
mod terrain;
mod tunnel;
mod cell;
mod mesh_gen;
/// **********************************************************************
/// The `InputState` is exactly what it sounds like, it just keeps track of
/// the user's input state so that we turn keyboard events into something
/// state-based and device-independent.
/// **********************************************************************
#[derive(Debug, Copy, Clone)]
pub struct InputState {
    pub xaxis: f32,
    pub yaxis: f32,
    pub fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
            fire: false,
        }
    }
}

pub struct GameState{
    score     : i32,    
    input     : InputState,
    level     : i32,
    screen : Size
}

pub struct Systems{
    pub renderer    : render::Renderer,
    pub sounds      : Vec::<audio::Source>,
    pub sound_names : HashMap::<String, usize>
}

impl Systems{
    fn add_sound(&mut self, rel_path : String,  ctx : &mut Context) {        
        let sound = audio::Source::new(ctx, rel_path.clone()).unwrap();
        self.sounds.push(sound);
        self.sound_names.insert(rel_path, self.sounds.len() - 1);        
    }
    fn get_sound(&self, sound_name: &str) -> Option<&usize>{
        self.sound_names.get(sound_name)
    }
}

pub struct App {    
    systems: Systems,
    state: Option<GameState>,
    levels : Vec::<level::Level>,
    world : level::World,    
}




impl App {
    pub fn new(ctx: &mut Context, screen : Size) -> App {

        let mut fonts = HashMap::<String, graphics::Font>::new();
        fonts.insert("edundot".to_string(), graphics::Font::new(ctx, "/font/edundot.ttf").unwrap());
        fonts.insert("Pixeled".to_string(), graphics::Font::new(ctx, "/font/Pixeled.ttf").unwrap());        
        fonts.insert("FORCED SQUARE".to_string(), graphics::Font::new(ctx, "/font/FORCED SQUARE.ttf").unwrap());        
        fonts.insert("V5PRD___".to_string(), graphics::Font::new(ctx, "/font/V5PRD___.TTF").unwrap());        

        let mut a = App {
            systems : Systems{
                renderer    : render::Renderer::new(),
                sounds      : Vec::<audio::Source>::new(),
                sound_names : HashMap::<String, usize>::new()
            },            
            state : Some( GameState{
                score : 0,
                input : InputState:: default(),
                level : 0,
                screen : screen
            }),           
            levels : Vec::<level::Level>::new(),
            world : level::World::empty()
        };

        a.systems.add_sound("/Randomize6.wav".to_string(), ctx);
        a.systems.renderer.fonts = fonts;
        a
    }

    fn find_level(&self, id : &Id) -> Option::<&level::Level> {
        for a in &self.levels{
            if a.id == *id {
                return Some(a);
            }
        }
        None
    }

}

fn player_handle_input(input : &InputState, pa : &mut actors::Actor, worldbounds : &level::WorldBounds, dt :u128) {

    const MOVE_STEP : f32 = -180.5;    
    
    let movex = input.xaxis * -MOVE_STEP * dt as f32 / 1000.0;
    let movey = input.yaxis * MOVE_STEP  * dt as f32 / 1000.0;
        
    pa.transform.x += movex;
    pa.transform.y += movey;

    let actor_size = pa.collision.get_size();
    pa.transform.x = pa.transform.x.min(worldbounds.max.x - actor_size.y);
    pa.transform.y = pa.transform.y.min(worldbounds.max.y - actor_size.x);
    pa.transform.x = pa.transform.x.max(worldbounds.min.x);
    pa.transform.y = pa.transform.y.max(worldbounds.min.y);

    // println!("{} {} {} {}", pa.transform.x, worldbounds.min.x, pa.transform.y, worldbounds.max.x);
    
}

impl EventHandler for App {


    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {

        let mut wc = level::WorldChange::default();
        if let Some(state) = self.state.as_ref(){            
            wc = self.world.update(_ctx, &state, &mut self.systems);
        }

        if let Some(state) = self.state.as_mut(){
            state.score += wc.score as i32;
        }
            
        if let Some(level_id) = wc.level{
            self.world.stop();            
            let level = (*self.find_level(&level_id).unwrap()).clone();
            let mut state = self.state.as_mut().unwrap();
            self.world = level.load(&mut state, &mut self.systems, _ctx);
            self.world.start(_ctx, state, &mut self.systems);
        }
           
        
        println!("FPS: {}", ggez::timer::fps(_ctx));
        return Ok(());
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {


        let t = self.world.get_camera_actor().transform;
        // let t = self.actors[&self.camera.actor_id].transform;
        self.systems.renderer.start_frame(ctx, t);
            
        self.systems.renderer.start_batch();
            
        // Draw code here...        
        let mut smtg_drawn = false;
        let mut draw_ctx = actors::DrawContext::WorldSpace;        
        self.systems.renderer.push_cam_transform(ctx);

        // for a in self.actors.values() {
        for a in &mut self.world.actors {
            if !a.visible {
                continue;
            }
            if a.atype == actors::ActorType::UI{
                continue;
            }

            if draw_ctx != a.drawctx {
                if let actors::DrawContext::ScreenSpace = a.drawctx {                    
                    self.systems.renderer.pop_cam_transform(ctx);
                }
                if let actors::DrawContext::WorldSpace = a.drawctx{                                        
                    self.systems.renderer.push_cam_transform(ctx);
                }
                draw_ctx = a.drawctx;
            }
            a.drawable.draw(a.transform, &mut self.systems.renderer, ctx);
            if let render::Renderable::DynamicRect{color:_, size:_} = a.drawable{
                smtg_drawn = true;
            }
            
        }    
        if smtg_drawn == true{
            self.systems.renderer.push_cam_transform(ctx);            
            self.systems.renderer.end_batch(ctx);            
        }
        

        let mut draw_ctx = actors::DrawContext::WorldSpace;              
        self.systems.renderer.pop_cam_transform(ctx);
        for a in &mut self.world.actors {
            if !a.visible {
                continue;
            }
            if a.atype != actors::ActorType::UI{
                continue;
            }

            if draw_ctx != a.drawctx {
                if let actors::DrawContext::ScreenSpace = a.drawctx {                    
                    self.systems.renderer.pop_cam_transform(ctx);                                
                }
                if let actors::DrawContext::WorldSpace = a.drawctx{                                        
                    self.systems.renderer.push_cam_transform(ctx);
                }
                draw_ctx = a.drawctx;
            }
            a.drawable.draw(a.transform, &mut self.systems.renderer, ctx);                        
        }    

        self.systems.renderer.end_frame(ctx)
    }


    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods,  _repeat: bool) {
        if let Some(p) = self.state.as_mut(){

            match keycode {
                KeyCode::Up    => { p.input.yaxis = 1.0;  }
                KeyCode::Down  => { p.input.yaxis = -1.0; }
                KeyCode::Left  => { p.input.xaxis = -1.0; }
                KeyCode::Right => { p.input.xaxis = 1.0;  }
                KeyCode::Space => { p.input.fire  = true; }
                KeyCode::P => {
                    let img = graphics::screenshot(ctx).expect("Could not take screenshot");
                    img.encode(ctx, graphics::ImageFormat::Png, "/screenshot.png")
                        .expect("Could not save screenshot");
                }
                KeyCode::Escape => event::quit(ctx),
                _ => (), // Do nothing
            }
        }
    }

    fn gamepad_axis_event(&mut self, _ctx: &mut Context, axis: Axis, _value: f32, _id: GamepadId ) {
        
        if let Some(p) = self.state.as_mut(){            
            if axis == Axis::LeftStickX {
                p.input.xaxis = _value;
            }
            if axis == Axis::LeftStickY {
                p.input.yaxis = _value;                
            }

        }
    }

}


fn connect_levels(app : &mut App, ctx: &mut Context){
    let mut intro    = level::Level::new("Intro".to_string());
    let mut tutorial = level::Level::new("tuto".to_string());
    let mut play     = level::Level::new("play".to_string());
    let mut gameover = level::Level::new("gameover".to_string());
    let mut victory  = level::Level::new("victory".to_string());
    let ref next_str = "next".to_string();
    intro.add_transition(next_str, &tutorial);
    tutorial.add_transition(next_str, &play);
    play.add_transition(&"win".to_string(), &victory);
    play.add_transition(&"lose".to_string(), &gameover);
    victory.add_transition(next_str, &intro);
    gameover.add_transition(next_str, &intro);
    //
    intro.loader    = level::introload;
    tutorial.loader = level::tutoload;
    gameover.loader = level::gameoverload;
    victory.loader  = level::victoryload;
    play.loader     = level::playload;
    app.levels.push(intro);
    app.levels.push(tutorial);
    app.levels.push(gameover);
    app.levels.push(victory);
    app.levels.push(play);
    
    let mut state = app.state.as_mut().unwrap();
    let mut w = app.levels[0].load(&mut state, &mut app.systems, ctx);

    if let Some(state) = app.state.as_ref(){
        // let input = &app.player.as_ref().unwrap().input.clone();

        w.start(ctx, state,  &mut app.systems);
        app.world = w;
    }

    
}



fn main() {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let window_setup = conf::WindowSetup::default().title("Dead Pulsar");
    let window_mode  = conf::WindowMode::default();

    let screen = Size{x: window_mode.width, y :window_mode.height};

    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) = ContextBuilder::new("dead pulsar", "LBdN")
           .add_resource_path(resource_dir)
           .window_setup(window_setup)
           .window_mode(window_mode)
           .build()
           .unwrap();

    let mut app = App::new(&mut ctx, screen);

    connect_levels(&mut app, &mut ctx);
          
    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}