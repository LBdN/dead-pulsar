
extern crate find_folder;
// use cgmath;
use std::env;
use std::path;
use rand;
use rand::Rng;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler, Axis};
use ggez::input::gamepad::GamepadId;
use ggez::input::keyboard::KeyCode;
use ggez::event::KeyMods;
use ggez::graphics;
use ggez::conf;
use ggez::graphics::{DrawParam, Color, Rect, Drawable, DrawMode, Mesh};
use ggez::timer;
use std::time;
use ggez::audio;
use ggez::audio::{SoundSource};

use cgmath::{Point2};
use cgmath::prelude::*;

type Position = mint::Point2::<f32>;
type Size     = mint::Point2::<f32>;

const GREY : Color = Color{ r: 0.5, g:0.5, b:0.5, a:1.0};
const RED  : Color = Color{ r: 1.0, g:0.0, b:0.0, a:1.0};


enum RectDraw{
    NoDraw,
    StaticRect(usize),
    DynamicRect{ color : Color, size : mint::Point2::<f32>},
    StaticText{text: graphics::Text },
    DynamicTextDraw { string: String, font : graphics::Font, fontsize : f32, color: graphics::Color},
    
}

impl RectDraw {
    fn draw(&self, transform : Position, mb : &mut graphics::MeshBuilder, meshes : &mut Vec::<Mesh>, ctx : &mut Context){
        match self {
            RectDraw::NoDraw => (),
            RectDraw::StaticRect(idx) => {
                let _ = meshes[*idx].draw(ctx, DrawParam::default().dest(transform));    
            },
            RectDraw::DynamicRect{color, size} => {
                mb.rectangle(
                    DrawMode::fill(),
                    Rect {
                        x:transform.x,
                        y:transform.y,
                        w:size.x,
                        h:size.y
                    },
                    *color,
                );
            },
            RectDraw::StaticText{text} =>{
                let _ = text.draw(ctx, DrawParam::default().dest(transform));
            },                        
            RectDraw::DynamicTextDraw{string, font, fontsize, color} => {
                let text = graphics::Text::new( (string.clone() , *font, *fontsize) );                
                let _ = text.draw(ctx, DrawParam::default().dest(transform).color(*color));
            },
            _ => ()
        } 
    }
}

#[derive(Copy, Clone, PartialEq)]
enum DrawContext{
    WorldSpace,
    ScreenSpace
}

#[derive(Debug, Copy, Clone)]
enum Collision{
    NoCollision,
    RectCollision{ width : f32, height : f32},
    DiscCollision( f32)
}

impl Collision {
    fn get_size(&self) -> Size {
        match self {
            Collision::RectCollision{width, height} => Size{x:*width, y:*height},
            Collision::DiscCollision(radius) => Size{x:*radius, y:*radius},
            Collision::NoCollision => Size{x:0.0, y:0.0}
        }
    }
}

fn collides( pos1 : &Position, col1 : &Collision, pos2 : &Position, col2 : &Collision) -> bool {
    let v1 = Point2::<f32>{x : pos1.x ,y : pos1.y };
    let v2 = Point2::<f32>{x : pos2.x ,y : pos2.y };
    let delta = v2-v1;

    match (col1, col2) {
        ( Collision::RectCollision{width : width1, height:height1}, Collision::RectCollision{width : width2, height:height2}) => {            
            if delta.x.abs() > ((width1 + width2)/2.0)  {
                return false
            }
            if delta.y.abs() > ((height1 + height2)/2.0) {
                return false
            }    
            return true
        },
        _ => {
            return false
        }
    }        
}

#[derive(PartialEq)]
pub enum ActorType {
    Background,
    Foreground,
    Player,
    EndGame,
    UI,
    Camera
}

pub struct Actor {
    transform  : Position,    
    drawable   : RectDraw,
    drawctx    : DrawContext,
    collision  : Collision,
    col_resp   : Vec::<Effect>,
    dead       : bool,
    visible    : bool,
    atype      : ActorType
}



impl Actor {
    pub fn new(atype : ActorType) -> Actor {
        Actor {
            transform: Position{ x:0.0, y:0.0},
            drawable : RectDraw::StaticRect(0),
            drawctx  : DrawContext::WorldSpace,
            collision: Collision::DiscCollision(0.0),
            col_resp : Vec::<Effect>::new(),
            dead     : true,
            visible  : false,
            atype    : atype
        }
    }
}

/// **********************************************************************
/// The `InputState` is exactly what it sounds like, it just keeps track of
/// the user's input state so that we turn keyboard events into something
/// state-based and device-independent.
/// **********************************************************************
#[derive(Debug)]
struct InputState {
    xaxis: f32,
    yaxis: f32,
    fire: bool,
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
    actor_idx : usize,
    input     : InputState
}

pub struct Camera{
    actor_idx : usize
}


pub struct World {
    size: [f64; 2]
}

#[derive(Debug, Copy, Clone)]
enum Effect{
    MoveActor{actor_idx: usize, vector: Position},
    UpdateScore{actor_idx: usize},
    ProcessInput,
    KillActor{actor_idx: usize},
    NextScene{cur_scene_idx : usize, next_scene_idx : usize},
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
                        if let RectDraw::DynamicTextDraw{string, ..} = &mut label_actor.drawable{
                            *string = format!( "Score: {}", pa.score);
                        }
                    }
                }
                return false;
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
                if let RectDraw::DynamicRect{ref mut color, ..} = a.drawable {
                    *color = RED;
                }                                        
                a.dead = true;
                return true;
            },
            Effect::NextScene{cur_scene_idx, next_scene_idx} => {
                let current_scene = &mut app.scenes[*cur_scene_idx];                
                current_scene.active = false;
                current_scene.clone().stop(app);
                let next_scene = &mut app.scenes[*next_scene_idx];
                next_scene.active = true;
                next_scene.clone().start(app);
                app.current_scene = *next_scene_idx;
                return true;
            },
            Effect::AutoNextScene{duration, cur_scene_idx, next_scene_idx} => {
                if *duration < t {
                    let current_scene = &mut app.scenes[*cur_scene_idx];                                     
                    current_scene.active = false;
                    current_scene.clone().stop(app);
                    let next_scene = &mut app.scenes[*next_scene_idx];
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
}

#[derive(Default)]
struct EffectResult{
    scene_changed : bool,
    dead_effects  : Vec::<usize>
}

#[derive(Debug, Clone)]
struct Scene {
    effects : Vec::<Effect>,
    actors: Vec::<usize>,    
    active: bool,
    name: String
}


impl Scene {
    pub fn new(name: String) -> Scene {
        Scene {
            effects : Vec::<Effect>::new(),
            actors : Vec::<usize>::new(),            
            active : false,
            name : name
        }
    }

    pub fn start(&self, app : &mut App ){
        println!("stopping {}", self.name);        
        for i in &self.actors{            
            let a = &mut app.actors[*i];
            a.visible = true;
            a.dead    = false;
        }
    }

    pub fn stop(&self, app : &mut App ){        
        println!("starting {}", self.name);
        for i in &self.actors{            
            let a = &mut app.actors[*i];
            a.visible = false;
            a.dead    = true;
        }
    }

    pub fn apply_effects(&self, app : &mut App, t : f32 ) -> EffectResult{
        println!("apply_effects {}", self.name);
        let before_effect_scene = app.current_scene;
        let mut eff_result = EffectResult::default();
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

struct App {
    font  : graphics::Font,    
    scenes: Vec::<Scene>, 
    actors: Vec::<Actor>,
    meshes: Vec::<Mesh>,
    sounds: Vec::<audio::Source>,
    player: Option<Player>,
    camera: Camera,
    world : World,    
    started: bool,
    current_scene : usize ,  
    last_scene_change: f32
}

impl App {
    pub fn new(ctx: &mut Context) -> App {
        // Load/create resources here: images, fonts, sounds, etc.
        // let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("res").unwrap();
        // let fp     = assets.join("FiraMono-Bold.ttf");

        
        let font = graphics::Font::new(ctx, "/font/edundot.ttf").unwrap();

        App {
            font   : font,            
            scenes : Vec::<Scene>::new(), 
            actors : Vec::<Actor>::new(),
            meshes : Vec::<Mesh>::new(),
            sounds : Vec::<audio::Source>::new(),
            player : None,
            camera : Camera{ actor_idx : 0 },
            world  : World{ size: [3000.0, 640.0]},            
            started : false,
            current_scene : 0,
            last_scene_change : 0.0
        }


        
    }

    fn create_scene(&mut self, name : String) -> usize {
        self.scenes.push(Scene::new(name));
        return self.scenes.len() -1;
    }

    fn add_sound(&mut self, rel_path : String,  ctx : &mut Context) -> usize{        
        let sound = audio::Source::new(ctx, rel_path).unwrap();
        self.sounds.push(sound);
        return self.sounds.len() - 1;
    }

    fn add_camera(&mut self, scene_idx: usize) ->usize {
        let mut a = Actor::new(ActorType::Camera);
        a.drawable  = RectDraw::NoDraw;
        a.collision = Collision::NoCollision;
        a.transform = Position{ x:0 as f32, y:0 as f32};
        self.actors.push(a);

        let s = &mut self.scenes[scene_idx];
        let actor_idx = self.actors.len() -1;
        s.actors.push(actor_idx);   

        self.camera.actor_idx = actor_idx;
        return actor_idx;
    }

    fn add_player(&mut self, scene_idx: usize) ->usize {
        
        
        let mut a = Actor::new(ActorType::Player);

        let size : f32 = 10.0;

        a.drawable = RectDraw::DynamicRect {
            color   : RED,
            size    : Size{ x:size, y:size},
        };

        a.collision = Collision::RectCollision { width: size, height: size };

        a.transform = Position{ x:320 as f32, y:240 as f32};
        self.actors.push(a);

        self.player = Some( Player{
            score    : 0,
            actor_idx: self.actors.len() - 1,
            input    : InputState:: default()
        });
        
        let s = &mut self.scenes[scene_idx];
        let actor_idx = self.actors.len() -1;
        s.actors.push(actor_idx);   
        return actor_idx;
                
    }

    fn add_foreground_rects(&mut self, scene_idx: usize, eff : Effect){
        
        const MAX_SIZE : f64 = 50.0;
        let actor_len = self.actors.len();
        for i in 1..100 {
            let mut a = Actor::new(ActorType::Foreground);

            let mut rng = rand::thread_rng();
            let x    = rng.gen_range(0.0, self.world.size[0]) as f32;
            let y    = rng.gen_range(0.0, self.world.size[1]) as f32;
            let size = rng.gen_range(0.0, MAX_SIZE) as f32;
            let r    = 1.0;
            let b    = rng.gen_range(0.0, 1.0);

            a.transform = Position{ x:x, y:y};

            a.drawable = RectDraw::DynamicRect {
                color   : Color{r:r, g:r, b:b, a:1.0},
                size    : Size{ x:size, y:size},
            };            

            a.collision = Collision::RectCollision { width: size, height: size };
            a.col_resp.push( Effect::KillActor{actor_idx:i+actor_len-1});
            a.col_resp.push( eff);            

            self.actors.push(a);            
            let actor_idx = self.actors.len() -1;
            let s = &mut self.scenes[scene_idx];
            s.actors.push(actor_idx);   

            
        }
        
    }

    fn add_end_rects(&mut self, exit_size : f64, scene_idx: usize) -> [usize; 3]{
        
        let lose_rect_height = (self.world.size[1] / 2.0) - exit_size;

        let mut res : [usize; 3] = [0,0,0];

        let mut yy = 0.0;
        for (i, rect_height) in [lose_rect_height, exit_size, lose_rect_height].iter().enumerate() {
            let mut a = Actor::new(ActorType::EndGame);
            a.transform = Position{ x:self.world.size[0] as f32, y:yy as f32};
            let size = Size{ x:50 as f32, y:*rect_height as f32 };
            a.collision = Collision::RectCollision { width: size.x, height: size.y };
            a.drawable = RectDraw::DynamicRect {
                color   : RED,
                size    : size,
            };

            self.actors.push(a);
            let s = &mut self.scenes[scene_idx];
            let end_idx = self.actors.len() -1;
            s.actors.push(end_idx);   
            yy += rect_height;
            res[i] = end_idx;
        }
        
        res 
    }

    fn add_background_rects(&mut self, ctx : &mut Context, scene_idx: usize){
        
        const MAX_SIZE : f64 = 50.0;
        let mut mb = graphics::MeshBuilder::new();

        let mut a = Actor::new(ActorType::Background);

        for _i in 1..10000 {
            let mut rng = rand::thread_rng();
            let x    = rng.gen_range(0.0, self.world.size[0]);
            let y    = rng.gen_range(0.0, self.world.size[1]);
            let size = rng.gen_range(0.0, MAX_SIZE);
            let r    = rng.gen_range(0.1, 0.5);

            mb.rectangle(
                DrawMode::fill(),
                Rect {x:x as f32, y:y as f32, w:size as f32, h:size as f32},
                Color{r:r, g:r, b:r, a:1.0}
            );

        }

        let mesh = mb.build(ctx).unwrap();
        self.meshes.push(mesh);
        a.drawable = RectDraw::StaticRect( self.meshes.len() - 1);
        self.actors.push(a);
        let s = &mut self.scenes[scene_idx];
        s.actors.push(self.actors.len() -1);
    }

    fn add_text(&mut self, text: String, fontsize: f32, pos: Position, static_:bool, scene_idx: usize) -> usize{
        let mut a   = Actor::new(ActorType::UI);
        a.drawctx   = DrawContext::ScreenSpace;
        if static_{
            let gtext   = graphics::Text::new((text, self.font, fontsize));
            a.drawable  = RectDraw::StaticText{ text: gtext };            
        } else {
            a.drawable = RectDraw::DynamicTextDraw{ string: text, font : self.font, fontsize : fontsize, color: graphics::WHITE};
        }
        
        a.transform = pos;
        self.actors.push(a);
        let s = &mut self.scenes[scene_idx];
        let actor_idx = self.actors.len() -1;
        s.actors.push(actor_idx);
        return actor_idx;
    }


}

fn player_handle_input(p: &Player, pa : &mut Actor) {

    const MOVE_STEP : f32 = -10.0;    
    
    let movex = p.input.xaxis * -MOVE_STEP;
    let movey = p.input.yaxis * MOVE_STEP;
        
    pa.transform.x += movex;
    pa.transform.y += movey;
    
}

impl EventHandler for App {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if !self.started{
            self.current_scene = 0;
            let s = self.scenes[self.current_scene].clone();
            s.start(self);                 
            let s = &mut self.scenes[self.current_scene];
            s.active = true;
            self.started = true;  
            self.last_scene_change = 0.0;     
        }


        let original_scene = self.current_scene;
        let t = timer::time_since_start(_ctx).as_secs_f32() - self.last_scene_change;
        let s = self.scenes[original_scene].clone();
        let eff_result = s.apply_effects(self, t);

        let s = &mut self.scenes[original_scene];
        for i in eff_result.dead_effects.iter().rev(){
            s.effects.remove(*i);
        }
        if eff_result.scene_changed{
            self.last_scene_change += t;
            return Ok(());
        }


        // Update code here...
        
        
        if let Some(pa) = self.player.as_mut(){                        
            let mut pos1       = Position{x : 0.0, y: 0.0};
            let mut collision1 = Collision::DiscCollision(0.0);
            if let Some(player_actor) = self.actors.get(pa.actor_idx){                
                let size1  = player_actor.collision.get_size();
                pos1       = Position{x : player_actor.transform.x + size1.x/2.0,y : player_actor.transform.y + size1.y/2.0};
                collision1 = player_actor.collision;
            }
                         
            for a in &mut self.actors {
                if let ActorType::Background = a.atype {
                    continue;
                }
                if let ActorType::Player = a.atype {
                    continue;
                }
                if a.dead {
                    continue;
                }                    

                let size2 = a.collision.get_size();                    
                let pos2 = Position{x : a.transform.x + size2.x/2.0,y : a.transform.y + size2.y/2.0};
                
                if collides(&pos1, &collision1, &pos2, &a.collision){
                    // if let RectDraw::DynamicRect{ref mut color, ..} = a.drawable {
                    //     *color = RED;
                    // }                        
                    pa.score += (1000.0 / (size2.x*size2.y)) as i32;
                    // a.dead = true;
                    let s = &mut self.scenes[self.current_scene];
                    s.effects.append(& mut a.col_resp.clone());
                }

            }
                
            
        }
        
        // println!("FPS: {}", ggez::timer::fps(_ctx));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        

        let mut mb = graphics::MeshBuilder::new();
        
        // Draw code here...        
        let mut smtg_drawn = false;

        let mut draw_ctx = DrawContext::WorldSpace;        
        let t = self.actors[self.camera.actor_idx].transform;
        let cam_transform = DrawParam::default().dest(t).to_matrix();
        graphics::push_transform(ctx, Some(cam_transform));
        graphics::apply_transformations(ctx).unwrap();

        for a in &self.actors {
            if !a.visible {
                continue;
            }
            if a.atype == ActorType::UI{
                continue;
            }

            if draw_ctx != a.drawctx {
                if let DrawContext::ScreenSpace = a.drawctx {
                    graphics::pop_transform(ctx);
                    graphics::apply_transformations(ctx).unwrap();            
                }
                if let DrawContext::WorldSpace = a.drawctx{                    
                    graphics::push_transform(ctx, Some(cam_transform));
                    graphics::apply_transformations(ctx).unwrap();
                }
                draw_ctx = a.drawctx;
            }

            a.drawable.draw(a.transform, &mut mb, &mut self.meshes, ctx);
            if let RectDraw::DynamicRect{color:_, size:_} = a.drawable{
                smtg_drawn = true;
            }
            
        }    
        if smtg_drawn == true{
            // let transform = DrawParam::default().dest(self.cam_transform).to_matrix();
            graphics::push_transform(ctx, Some(cam_transform));
            graphics::apply_transformations(ctx).unwrap();
            let mesh = mb.build(ctx).unwrap();
            mesh.draw(ctx, DrawParam::default().dest([0.0,0.0])).unwrap();
        }
        

        let mut draw_ctx = DrawContext::WorldSpace;      
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx).unwrap();           
        for a in &self.actors {
            if !a.visible {
                continue;
            }
            if a.atype != ActorType::UI{
                continue;
            }

            if draw_ctx != a.drawctx {
                if let DrawContext::ScreenSpace = a.drawctx {
                    graphics::pop_transform(ctx);
                    graphics::apply_transformations(ctx).unwrap();            
                }
                if let DrawContext::WorldSpace = a.drawctx{                    
                    graphics::push_transform(ctx, Some(cam_transform));
                    graphics::apply_transformations(ctx).unwrap();
                }
                draw_ctx = a.drawctx;
            }

            a.drawable.draw(a.transform, &mut mb, &mut self.meshes, ctx);                        
        }    

        graphics::present(ctx)
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

    let center = Position{x: 320.0, y: 240.0};

    let intro_scene_idx = app.create_scene("intro".to_string());    
    app.add_text("Pulsar 3".to_string(), 56.0, center, true, intro_scene_idx);

    let play_scene_idx = app.create_scene("play".to_string());    
    {
        app.add_background_rects(& mut ctx, play_scene_idx);
        let sound_idx = app.add_sound("/Randomize6.wav".to_string(), &mut ctx);
        let eff = Effect::PlaySound{sound_index:sound_idx};
        app.add_foreground_rects(play_scene_idx, eff);    
        let player_actor_idx = app.add_player(play_scene_idx);
        let camera_idx       = app.add_camera(play_scene_idx);
        let text_idx         = app.add_text("Pulsar 3".to_string(), 28.0, Position{x: 10.0, y: 10.0}, true, play_scene_idx);
        
        let a = &app.actors[text_idx];        
        let margin = 10.0;
        let mut p = Position{x:0.0+margin, y: 10.0};
        if let RectDraw::StaticText{text} = &a.drawable{            
            p = Position{x:a.transform.x+text.width(&mut ctx) as f32 +margin, y: 10.0};        
        } 
        let text_idx = app.add_text("Score: 0".to_string(), 28.0, p, false, play_scene_idx);
                
        let s = &mut app.scenes[play_scene_idx];
        s.effects.push( Effect::MoveActor{actor_idx:camera_idx,       vector:Position{x:-1.0, y:0.0}} );
        s.effects.push( Effect::MoveActor{actor_idx:player_actor_idx, vector:Position{x :1.0, y:0.0}} );    
        s.effects.push( Effect::ProcessInput );     
        s.effects.push( Effect::UpdateScore{actor_idx:text_idx});
    }
    let [lose_rect1_idx, win_rect_idx, lose_rect2_idx]  = app.add_end_rects(50.0, play_scene_idx);
    {
        let a = &mut app.actors[win_rect_idx];   
        if let RectDraw::DynamicRect{ref mut color, ..} = a.drawable {
            *color = GREY;
        }         
    }

    let lose_scene_idx = app.create_scene("Game Over".to_string());
    {        
        app.add_text("Game Over".to_string(), 28.0, center, true, lose_scene_idx);
        let s = &mut app.scenes[lose_scene_idx];
        let auto_transition = Effect::AutoNextScene{duration:3.0, cur_scene_idx : lose_scene_idx, next_scene_idx : intro_scene_idx};        
        s.effects.push( auto_transition );
    }
    let win_scene_idx = app.create_scene("Victory".to_string());
    {        
        app.add_text("Victory".to_string(), 28.0, center, true, win_scene_idx);
        let s = &mut app.scenes[win_scene_idx];
        let auto_transition = Effect::AutoNextScene{duration:3.0, cur_scene_idx : win_scene_idx, next_scene_idx : intro_scene_idx};        
        s.effects.push( auto_transition );
    }

    {
        let s = &mut app.scenes[intro_scene_idx];
        let auto_transition = Effect::AutoNextScene{duration:3.0, cur_scene_idx : intro_scene_idx, next_scene_idx : play_scene_idx};        
        s.effects.push( auto_transition );
    }
    {
        let lose_game_transition = Effect::NextScene{cur_scene_idx : play_scene_idx, next_scene_idx : lose_scene_idx};
        let a = &mut app.actors[lose_rect1_idx];                
        a.col_resp.push(lose_game_transition);    
        let a2 = &mut app.actors[lose_rect2_idx];                
        a2.col_resp.push(lose_game_transition);        

        let win_game_transition = Effect::NextScene{cur_scene_idx : play_scene_idx, next_scene_idx : win_scene_idx};
        let a2 = &mut app.actors[win_rect_idx];                
        a2.col_resp.push(win_game_transition);        
    }
          

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}