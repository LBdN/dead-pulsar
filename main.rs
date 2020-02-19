
extern crate find_folder;
// use cgmath;
use std::env;
use std::path;
use std::collections::HashMap;
use rand;
use rand::Rng;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler, Axis};
use ggez::input::gamepad::GamepadId;
use ggez::input::keyboard::KeyCode;
use ggez::event::KeyMods;
use ggez::graphics;
use ggez::conf;
use ggez::graphics::{Rect, DrawMode};
use ggez::graphics::{Color};
use ggez::timer;
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

/// **********************************************************************
/// The `InputState` is exactly what it sounds like, it just keeps track of
/// the user's input state so that we turn keyboard events into something
/// state-based and device-independent.
/// **********************************************************************
#[derive(Debug)]
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

pub struct Player{
    score     : i32,
    actor_id : Id,
    input     : InputState
}

pub struct Camera{
    actor_id : Id
}


pub struct World {
    size: [f64; 2]
}



#[derive(Debug, Clone)]
struct Scene {
    start_effects : Vec::<effect::Effect>,
    effects : Vec::<effect::Effect>,
    actors: Vec::<Id>,    
    active: bool,
    name: String
}


impl Scene {
    pub fn new(name: String) -> Scene {
        Scene {
            start_effects : Vec::<effect::Effect>::new(),
            effects : Vec::<effect::Effect>::new(),
            actors : Vec::<Id>::new(),            
            active : false,
            name : name
        }
    }

    pub fn start(&self, app : &mut App ){
        println!("start {}", self.name);        
        for eff in &self.start_effects{
            let _ = eff.apply(app, 0.0);
        }
        for i in &self.actors{            
            if let Some(a) = &mut app.actors.get_mut(&i) {
                a.visible = true;
                a.ticking = true;
            }            
        }        
    }

    pub fn stop(&self, app : &mut App ){        
        println!("stop {}", self.name);
        for i in &self.actors{            
            if let Some(a) = &mut app.actors.get_mut(&i) {
                a.visible = false;
                a.ticking = false;
            }            
        }
    }

    pub fn apply_effects(&self, app : &mut App, t : f32 ) -> effect::EffectResult{
        println!("apply_effects {}", self.name);
        let before_effect_scene = app.current_scene;
        let mut eff_result = effect::EffectResult::default();
        for (i, eff) in self.effects.iter().enumerate(){            
            let to_remove = eff.apply(app, t);
            if to_remove {
                eff_result.dead_effects.push(i);
            }
        }
        eff_result.scene_changed = app.current_scene != before_effect_scene ;
        eff_result
    }
}

pub struct App {    
    renderer : render::Renderer,
    scenes: Vec::<Scene>, 
    actors: KeyedResource::<actors::Actor>,    
    sounds: KeyedResource::<audio::Source>,
    player: Option<Player>,
    camera: Camera,
    world : World,    
    started: bool,
    current_scene : usize ,  
    last_scene_change: f32,
    levels : Vec::<level::Level>
}


fn connect_levels(app : &mut App){
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
}

impl App {
    pub fn new(ctx: &mut Context) -> App {
        // Load/create resources here: images, fonts, sounds, etc.
        // let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("res").unwrap();
        // let fp     = assets.join("FiraMono-Bold.ttf");

        let mut fonts = HashMap::<String, graphics::Font>::new();
        fonts.insert("edundot".to_string(), graphics::Font::new(ctx, "/font/edundot.ttf").unwrap());
        fonts.insert("Pixeled".to_string(), graphics::Font::new(ctx, "/font/Pixeled.ttf").unwrap());        
        fonts.insert("FORCED SQUARE".to_string(), graphics::Font::new(ctx, "/font/FORCED SQUARE.ttf").unwrap());        
        fonts.insert("V5PRD___".to_string(), graphics::Font::new(ctx, "/font/V5PRD___.TTF").unwrap());        

        
        

        let mut a = App {
            renderer : render::Renderer::new(),
            scenes : Vec::<Scene>::new(), 
            actors : KeyedResource::<actors::Actor>::new(),            
            sounds : KeyedResource::<audio::Source>::new(),
            player : None,
            camera : Camera{ actor_id :  no_id()},
            world  : World{ size: [1000.0, 640.0]},            
            started : false,
            current_scene : 0,
            last_scene_change : 0.0,
            levels : Vec::<level::Level>::new()
        };

        a.renderer.fonts = fonts;
        a
    }

    fn start(&mut self){
        self.current_scene = 0;        
        self.start_scene();
        self.started = true;          
    }

    fn start_scene(&mut self){
        let s = self.scenes[self.current_scene].clone();
        s.start(self);                 
        let s = &mut self.scenes[self.current_scene];
        self.last_scene_change = 0.0;     
        s.active = true;
    }


    fn display_status(&self){
        println!("--> Result");
        for (i, s) in self.scenes.iter().enumerate() {
            println!("{} {}", s.name,  s.active);
            if s.active && self.current_scene != i{
                println!(" ↳ shouldn't be active");
            }
            if !s.active && self.current_scene == i{
                println!(" ↳ shouldn't active");
            }
            for actor_id in &s.actors{
                let a = self.actors.get(actor_id).unwrap();
                if a.ticking != s.active {
                    println!(" ↳ discrepencies between scene and actor status");
                }                                
            }
        }
    }

    fn apply_effect(&mut self, _ctx: &mut Context) -> bool {
        println!("---");
        let original_scene = self.current_scene;
        let t = timer::time_since_start(_ctx).as_secs_f32() - self.last_scene_change;
        let s = self.scenes[original_scene].clone();
        let eff_result = s.apply_effects(self, t);

        let s = &mut self.scenes[original_scene];
        for i in eff_result.dead_effects.iter().rev(){
            s.effects.remove(*i);
        }
        if eff_result.scene_changed{
            self.last_scene_change = timer::time_since_start(_ctx).as_secs_f32();       
            // cleanup effect dynamically placed.     
            let mut eff_to_remove = Vec::<usize>::new();
            for actor_id in &s.actors{
                if let Some(a) = &self.actors.get_mut(actor_id){
                    for eff in &a.on_collision{
                        for (i, eff2) in s.effects.iter().enumerate(){
                            if *eff == *eff2 && !eff_to_remove.contains(&i) {
                                 eff_to_remove.push(i);
                            }
                        }
                    }   
                }
                             
            }
            for i in eff_to_remove{
                s.effects.remove(i);
            }            
        }
        self.display_status();
        println!("---");
        eff_result.scene_changed
        
    }

    
    fn create_scene(&mut self, name : String) -> usize {
        self.scenes.push(Scene::new(name));
        return self.scenes.len() -1;
    }

    fn add_sound(&mut self, rel_path : String,  ctx : &mut Context) -> Id{        
        let sound = audio::Source::new(ctx, rel_path).unwrap();
        let id = get_id();
        self.sounds.insert(id, sound);
        id
    }

    fn add_camera(&mut self, scene_idx: usize) -> Id {
        let mut a = actors::Actor::new(actors::ActorType::Camera, unit::get_id());
        a.drawable  = render::Renderable::NoDraw;
        a.collision = actors::Collision::NoCollision;
        a.transform = unit::Position{ x:0 as f32, y:0 as f32};
        
        self.camera.actor_id = self.add_actor(a, scene_idx);
        self.camera.actor_id
    }

    fn add_player(&mut self, scene_idx: usize) ->Id {
        
        let size  = unit::Size{x:10.0, y:10.0};
        
        let mut a = actors::Actor::new(actors::ActorType::Player, unit::get_id());
        self.rect_behavior(&mut a, size, color::RED);

        let actor_id = self.add_actor(a, scene_idx);
        
        self.player = Some( Player{
            score    : 0,
            actor_id : actor_id,
            input    : InputState:: default()
        });

        actor_id
    }

    fn random_rect(&self, maxsize : f32) -> (f32, f32, unit::Size) {
        let mut rng = rand::thread_rng();
        let x    = rng.gen_range(0.0, self.world.size[0]) as f32;
        let y    = rng.gen_range(0.0, self.world.size[1]) as f32;
        let size = rng.gen_range(0.0, maxsize) as f32;            
        (x, y, unit::Size{x:size, y:size})
    }

    fn rect_behavior(&mut self, a : &mut actors::Actor, size: unit::Size, color : Color){
        a.drawable = render::Renderable::DynamicRect {
            color   : color,
            size    : size,
        };            
        a.collision = actors::Collision::RectCollision { width: size.x, height: size.y };
    }

    fn add_foreground_rects(&mut self, scene_idx: usize, eff : effect::Effect) -> [Id; 100]{
        
        const MAX_SIZE : f32 = 50.0;        
        let mut indices : [Id; 100] = [no_id();100];
        for i in 0..100 {
            let mut a = actors::Actor::new(actors::ActorType::Foreground, unit::get_id());
  
            let (x, y, size) = self.random_rect(MAX_SIZE);

            a.transform = unit::Position{ x:x, y:y};
            self.rect_behavior(&mut a, size, color::random_foreground_color());

            a.on_collision.push(effect::Effect::KillActor{actor_id:a.id.clone()});
            a.on_collision.push(eff);            

            indices[i] = self.add_actor(a, scene_idx);            
        }
        indices
    }

    fn add_end_rects(&mut self, exit_size : f64, scene_idx: usize) -> [Id; 3]{
        
        let lose_rect_height = (self.world.size[1]- exit_size) / 2.0;

        let mut res : [Id; 3] = [no_id(); 3];

        let mut yy = 0.0;
        for (i, rect_height) in [lose_rect_height, exit_size, lose_rect_height].iter().enumerate() {
            let mut a = actors::Actor::new(actors::ActorType::EndGame, unit::get_id());
            a.transform = unit::Position{ x:self.world.size[0] as f32, y:yy as f32};
            let size = unit::Size{ x:50 as f32, y:*rect_height as f32 };

            self.rect_behavior(&mut a, size, color::RED);            

            yy += rect_height;
            res[i] = self.add_actor(a, scene_idx);
        }
        
        res 
    }

    fn add_background_rects(&mut self, ctx : &mut Context, scene_idx: usize) -> Id{
        
        const MAX_SIZE : f32 = 50.0;
        

        let mut a = actors::Actor::new(actors::ActorType::Background, unit::get_id());

        let mut mb = graphics::MeshBuilder::new();
        for _i in 1..10000 {
            let (x, y, size) = self.random_rect(MAX_SIZE);            

            mb.rectangle(
                DrawMode::fill(),
                Rect {x:x, y:y, w:size.x, h:size.y},
                color::random_grey_color()
            );
        }
        let mesh = mb.build(ctx).unwrap();

        self.renderer.meshes.push(mesh);
        a.drawable = render::Renderable::StaticRect( self.renderer.meshes.len() - 1);

        self.add_actor(a, scene_idx)
    }

    fn add_text(&mut self, text: String, fontstyle : text::FontStyle, static_:bool, scene_idx: usize, centered: bool) -> Id{
        let mut a   = actors::Actor::new(actors::ActorType::UI, unit::get_id());
        a.drawctx   = actors::DrawContext::ScreenSpace;
        let font    = self.renderer.fonts[&fontstyle.name];
        let gtext   = graphics::Text::new((text.clone(), font, fontstyle.size));        
        if static_{                        
            let text_anchor = if centered  {render::TextAnchor::Center} else {render::TextAnchor::TopLeft};
            a.drawable  = render::Renderable::StaticText{ text: gtext, text_anchor : text_anchor };            
        } else {
            a.drawable = render::Renderable::DynamicTextDraw{ string: text, fontstyle : fontstyle};
        }        
        self.add_actor(a, scene_idx)
    }

    fn add_actor(&mut self, a: actors::Actor, scene_idx: usize) -> Id {
        let id = a.id.clone();
        let s = &mut self.scenes[scene_idx];        
        s.actors.push(id);
        self.actors.insert(id, a);                
        id
    }


}

fn player_handle_input(p: &Player, pa : &mut actors::Actor) {

    const MOVE_STEP : f32 = -10.0;    
    
    let movex = p.input.xaxis * -MOVE_STEP;
    let movey = p.input.yaxis * MOVE_STEP;
        
    pa.transform.x += movex;
    pa.transform.y += movey;
    
}

impl EventHandler for App {


    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if !self.started{
            self.start();   
        }

        let scene_changed = self.apply_effect(_ctx);
        if scene_changed {
            return Ok(());
        }



        // Update code here...
        
        
        if let Some(pa) = self.player.as_mut(){                        
            let mut pos1       = unit::Position{x : 0.0, y: 0.0};
            let mut collision1 = actors::Collision::DiscCollision(0.0);
            
            if let Some(player_actor) = &self.actors.get_mut(&pa.actor_id){            
                let size1  = player_actor.collision.get_size();
                pos1       = unit::Position{x : player_actor.transform.x + size1.x/2.0,y : player_actor.transform.y + size1.y/2.0};
                collision1 = player_actor.collision;
            }
                         
            for a in &mut self.actors.values() {
                if !a.ticking{
                    continue;
                }
                if let actors::ActorType::Background = a.atype {
                    continue;
                }
                if let actors::ActorType::Player = a.atype {
                    continue;
                }
                if let actors::Collision::NoCollision = a.collision {
                    continue;
                }                    

                let size2 = a.collision.get_size();                    
                let pos2 = unit::Position{x : a.transform.x + size2.x/2.0,y : a.transform.y + size2.y/2.0};
                
                if actors::collides(&pos1, &collision1, &pos2, &a.collision){      
                    pa.score += (1000.0 / (size2.x*size2.y)) as i32;
                    let s = &mut self.scenes[self.current_scene];
                    s.effects.append(& mut a.on_collision.clone());
                }

            }
                
            
        }
        
        // println!("FPS: {}", ggez::timer::fps(_ctx));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {

        let t = self.actors[&self.camera.actor_id].transform;
        self.renderer.start_frame(ctx, t);
            
        self.renderer.start_batch();
            
        // Draw code here...        
        let mut smtg_drawn = false;
        let mut draw_ctx = actors::DrawContext::WorldSpace;        
        self.renderer.push_cam_transform(ctx);

        for a in self.actors.values() {
            if !a.visible {
                continue;
            }
            if a.atype == actors::ActorType::UI{
                continue;
            }

            if draw_ctx != a.drawctx {
                if let actors::DrawContext::ScreenSpace = a.drawctx {                    
                    self.renderer.pop_cam_transform(ctx);
                }
                if let actors::DrawContext::WorldSpace = a.drawctx{                                        
                    self.renderer.push_cam_transform(ctx);
                }
                draw_ctx = a.drawctx;
            }
            a.drawable.draw(a.transform, &mut self.renderer, ctx);
            if let render::Renderable::DynamicRect{color:_, size:_} = a.drawable{
                smtg_drawn = true;
            }
            
        }    
        if smtg_drawn == true{
            self.renderer.push_cam_transform(ctx);            
            self.renderer.end_batch(ctx);            
        }
        

        let mut draw_ctx = actors::DrawContext::WorldSpace;              
        self.renderer.pop_cam_transform(ctx);
        for a in self.actors.values() {
            if !a.visible {
                continue;
            }
            if a.atype != actors::ActorType::UI{
                continue;
            }

            if draw_ctx != a.drawctx {
                if let actors::DrawContext::ScreenSpace = a.drawctx {                    
                    self.renderer.pop_cam_transform(ctx);                                
                }
                if let actors::DrawContext::WorldSpace = a.drawctx{                                        
                    self.renderer.push_cam_transform(ctx);
                }
                draw_ctx = a.drawctx;
            }
            a.drawable.draw(a.transform, &mut self.renderer, ctx);                        
        }    

        self.renderer.end_frame(ctx)
    }


    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods,  _repeat: bool) {
        if let Some(p) = self.player.as_mut(){

            match keycode {
                KeyCode::Up => {
                    p.input.yaxis = 1.0;
                }
                KeyCode::Down => {
                    p.input.yaxis = -1.0;
                }
                KeyCode::Left => {
                    p.input.xaxis = -1.0;
                }
                KeyCode::Right => {
                    p.input.xaxis = 1.0;
                }
                KeyCode::Space => {
                    p.input.fire = true;
                }
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
        
        if let Some(p) = self.player.as_mut(){            
            if axis == Axis::LeftStickX {
                p.input.xaxis = _value;
            }
            if axis == Axis::LeftStickY {
                p.input.yaxis = _value;                
            }

        }
    }

}





pub const title_style : text::FontStyle= text::FontStyle{
    size: 56.0,
    name: "edundot".to_string(),
    weight: text::FontWeight::Normal,
    color: ggez::graphics::WHITE,
};

pub const tuto_style: text::FontStyle= text::FontStyle{
    size: 30.0,
    name: "V5PRD___".to_string(),
    weight: text::FontWeight::Normal,
    color: ggez::graphics::WHITE,
};

pub const ui_style: text::FontStyle = text::FontStyle{
    size: 28.0,
    name: "edundot".to_string(),
    weight: text::FontWeight::Normal,
    color: ggez::graphics::WHITE,
};

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

    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) = ContextBuilder::new("dead pulsar", "LBdN")
           .add_resource_path(resource_dir)
           .window_setup(conf::WindowSetup::default().title("Dead Pulsar"))
           .build()
           .unwrap();



    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut app = App::new(&mut ctx);

    connect_levels(&mut app);

    let center = unit::Position{x: 400.0, y: 300.0};


    let intro_scene_idx = app.create_scene("intro".to_string());    
    let text_id = app.add_text("Pulsar 3".to_string(), title_style.clone(), true, intro_scene_idx, true);
    let a = app.actors.get_mut(&text_id).unwrap();
    a.transform = center;

    let tutorial_idx = app.create_scene("intro".to_string());    {
        let tuto_text = "Catch the yellow blocks and\n exit with the green one.".to_string();
        let text_id = app.add_text(tuto_text, tuto_style.clone(), true, tutorial_idx, true);
        let a = app.actors.get_mut(&text_id).unwrap();
        a.transform = center;
    }
    

    let play_scene_idx = app.create_scene("play".to_string());    
    {
        app.add_background_rects(& mut ctx, play_scene_idx);
        let sound_idx = app.add_sound("/Randomize6.wav".to_string(), &mut ctx);
        let eff = effect::Effect::PlaySound{sound_index:sound_idx};
        let actor_ids       = app.add_foreground_rects(play_scene_idx, eff);    
        let player_actor_id = app.add_player(play_scene_idx);
        let camera_idx       = app.add_camera(play_scene_idx);
        let text_id         = app.add_text("Pulsar 3".to_string(), ui_style.clone(), true, play_scene_idx, false);
        let a = app.actors.get_mut(&text_id).unwrap();
        a.transform = unit::Position{x: 10.0, y: 10.0};
        
        let a = &app.actors[&text_id];        
        let margin = 10.0;
        let mut p = unit::Position{x:0.0+margin, y: 10.0};
        if let render::Renderable::StaticText{text, ..} = &a.drawable{            
            p = unit::Position{x:a.transform.x+text.width(&mut ctx) as f32 +margin, y: 10.0};        
        } 
        let text_id = app.add_text("Score: 0".to_string(), ui_style.clone(), false, play_scene_idx, false);
        let a = app.actors.get_mut(&text_id).unwrap();
        a.transform = p;
                
        let s = &mut app.scenes[play_scene_idx];
        s.effects.push( effect::Effect::MoveActor{actor_id:camera_idx,       vector:unit::Position{x:-1.0, y:0.0}} );
        s.effects.push( effect::Effect::MoveActor{actor_id:player_actor_id, vector:unit::Position{x :1.0, y:0.0}} );    
        s.effects.push( effect::Effect::ProcessInput );     
        s.effects.push( effect::Effect::UpdateScore{actor_id:text_id});

        let mut p_pos = center.clone();
        p_pos.x = 10.0;
        s.start_effects.push( effect::Effect::PlaceActor{actor_id:player_actor_id, position: p_pos} );
        s.start_effects.push( effect::Effect::PlaceActor{actor_id:camera_idx, position:  unit::Position{ x:0 as f32, y:0 as f32}} );
        s.start_effects.push( effect::Effect::SetScore{new_value : 0} );
        for i in actor_ids.iter() {
            s.start_effects.push( effect::Effect::ResetActor{actor_id : *i} );
        }
        
        
    }
    let [lose_rect1_idx, win_rect_idx, lose_rect2_idx]  = app.add_end_rects(50.0, play_scene_idx);
    {
        let a = app.actors.get_mut(&win_rect_idx).unwrap();   
        if let render::Renderable::DynamicRect{ref mut color, ..} = a.drawable {
            *color = color::GREY;
        }         
    }

    let lose_scene_idx = app.create_scene("Game Over".to_string());
    {        
        let text_idx = app.add_text("Game Over".to_string(), title_style.clone(), true, lose_scene_idx, true);
        let a = app.actors.get_mut(&text_idx).unwrap();
        a.transform = center;
        let s = &mut app.scenes[lose_scene_idx];
        let auto_transition = effect::Effect::AutoNextScene{duration:3.0, cur_scene_idx : lose_scene_idx, next_scene_idx : intro_scene_idx};        
        s.effects.push( auto_transition );
    }
    let win_scene_idx = app.create_scene("Victory".to_string());
    {        
        let text_idx = app.add_text("Victory".to_string(), title_style.clone(), true, win_scene_idx, true);
        let a = app.actors.get_mut(&text_idx).unwrap();
        a.transform = center;
        let s = &mut app.scenes[win_scene_idx];
        let auto_transition = effect::Effect::AutoNextScene{duration:3.0, cur_scene_idx : win_scene_idx, next_scene_idx : intro_scene_idx};        
        s.effects.push( auto_transition );
    }

    {
        let s = &mut app.scenes[intro_scene_idx];
        let auto_transition = effect::Effect::AutoNextScene{duration:3.0, cur_scene_idx : intro_scene_idx, next_scene_idx : tutorial_idx};        
        s.effects.push( auto_transition );
    }
    {
        let s = &mut app.scenes[tutorial_idx];
        let auto_transition = effect::Effect::AutoNextScene{duration:3.0, cur_scene_idx : tutorial_idx, next_scene_idx : play_scene_idx};        
        s.effects.push( auto_transition );
    }
    {
        let lose_game_transition = effect::Effect::AutoNextScene{duration:0.0, cur_scene_idx : play_scene_idx, next_scene_idx : lose_scene_idx};
        let a = app.actors.get_mut(&lose_rect1_idx).unwrap();
        a.on_collision.push(lose_game_transition);    
        let a2 = app.actors.get_mut(&lose_rect2_idx).unwrap();
        a2.on_collision.push(lose_game_transition);        

        let win_game_transition = effect::Effect::AutoNextScene{duration:0.0, cur_scene_idx : play_scene_idx, next_scene_idx : win_scene_idx};
        let a2 = app.actors.get_mut(&win_rect_idx).unwrap();                
        a2.on_collision.push(win_game_transition);        
    }
          

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}