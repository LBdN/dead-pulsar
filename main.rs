
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

use cgmath::{Point2};
use cgmath::prelude::*;

type Position = mint::Point2::<f32>;
type Size     = mint::Point2::<f32>;

const GREY : Color = Color{ r: 0.5, g:0.5, b:0.5, a:1.0};
const RED  : Color = Color{ r: 1.0, g:0.0, b:0.0, a:1.0};


enum RectDraw{
    StaticRect(usize),
    DynamicRect{ color : Color, size : mint::Point2::<f32>}
}

impl RectDraw {
    fn draw(&self, transform : Position, mb : &mut graphics::MeshBuilder, meshes : &mut Vec::<Mesh>, ctx : &mut Context){
        match self {
            RectDraw::StaticRect(idx) => {
                meshes[*idx].draw(ctx, DrawParam::default().dest(transform));    
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
            _ => ()
        } 
    }
}

// impl RectDescriptor{
//     pub fn dynamic_draw(&self, mb : &mut graphics::MeshBuilder, transform: mint::Point2::<f32>){
//         mb.rectangle(
//             DrawMode::fill(),
//             Rect {
//                 x:transform.x,
//                 y:transform.y,
//                 w:self.size.x,
//                 h:self.size.y
//             },
//             self.color,
//         );   
//     }
// }

#[derive(Debug, Copy, Clone)]
enum Collision{
    RectCollision{ width : f32, height : f32},
    DiscCollision( f32)
}

impl Collision {
    fn get_size(&self) -> Size {
        match self {
            Collision::RectCollision{width, height} => Size{x:*width, y:*height},
            Collision::DiscCollision(radius) => Size{x:*radius, y:*radius},
        }
    }
}

fn collides( pos1 : &Position, col1 : &Collision, pos2 : &Position, col2 : &Collision) -> bool {
    let v1 = Point2::<f32>{x : pos1.x ,y : pos1.y };
    let v2 = Point2::<f32>{x : pos2.x ,y : pos2.y };
    let delta = v2-v1;

    match (col1, col2) {
        ( Collision::RectCollision{width : width1, height:height1}, Collision::RectCollision{width : width2, height:height2}) => {            
            if delta.x.abs() < width1 + width2 {
                return true
            }
            if delta.y.abs() < height1 + height2 {
                return true
            }    
            return false
        },
        _ => {
            return false
        }
    }        
}

enum ActorType {
    Background,
    Foreground,
    Player,
    EndGame
}

pub struct Actor {
    transform  : Position,    
    drawable   : RectDraw,
    collision  : Collision,
    dead       : bool,
    visible    : bool,
    atype      : ActorType
}



impl Actor {
    pub fn new(atype : ActorType) -> Actor {
        Actor {
            transform: Position{ x:0.0, y:0.0},
            drawable : RectDraw::StaticRect(0),
            collision: Collision::DiscCollision(0.0),
            dead     : false,
            visible  : true,
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


pub struct World {
    size: [f64; 2]
}

struct Scene {
    actors: Vec::<usize>,    
    active: bool
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            actors : Vec::<usize>::new(),            
            active : false
        }
    }
    // pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    //     Ok(())    
    // }

    // pub fn tick(&mut self, app: &mut App, ctx: &mut Context) -> GameResult<()> {
    //     for a in &mut self.actors {
    //         if a.dead {
    //             continue;
    //         }
    //         // a.tick(self, app, ctx);
    //     }    
    // Ok(())
    // }
}

struct App {
    font  : graphics::Font,
    text  : graphics::Text,
    scenes: Vec::<Scene>, 
    actors: Vec::<Actor>,
    meshes: Vec::<Mesh>,
    player: Option<Player>,
    world : World,
    cam_transform: [f32; 2],
    // Your state here...
}

impl App {
    pub fn new(ctx: &mut Context) -> App {
        // Load/create resources here: images, fonts, sounds, etc.
        // let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("res").unwrap();
        // let fp     = assets.join("FiraMono-Bold.ttf");
        let font = graphics::Font::new(ctx, "/FiraMono-Bold.ttf").unwrap();
        let text = graphics::Text::new(("Pulsar 3", font, 28.0));


        App {
            font   : font,
            text   : text,
            scenes : Vec::<Scene>::new(), 
            actors : Vec::<Actor>::new(),
            meshes : Vec::<Mesh>::new(),
            player : None,
            world  : World{ size: [3000.0, 640.0]},
            cam_transform: [0.0, 0.0],
        }
    }

    fn add_player(&mut self) {
        self.scenes.push(Scene::new());
        let s = self.scenes.last_mut().unwrap();
        
        let mut a = Actor::new(ActorType::Player);

        let size : f32 = 10.0;

        a.drawable = RectDraw::DynamicRect {
            color   : Color{r:0.1, g:0.0, b:1.0, a:1.0},
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

                
    }

    fn add_foreground_rects(&mut self){
        self.scenes.push(Scene::new());
        let s = self.scenes.last_mut().unwrap();
        const MAX_SIZE : f64 = 50.0;
        for _i in 1..100 {
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

            self.actors.push(a);
            s.actors.push( self.actors.len() - 1);
        }
        
    }

    fn add_end_rects(&mut self){
        self.scenes.push(Scene::new());
        let s = self.scenes.last_mut().unwrap();

        let mut a = Actor::new(ActorType::EndGame);
        a.transform = Position{ x:self.world.size[0] as f32, y:0 as f32};
        a.drawable = RectDraw::DynamicRect {

            color   : RED,
            size    : Size{ x:50 as f32, y:self.world.size[1] as f32},
        };

        self.actors.push(a);
        s.actors.push( self.actors.len() - 1);       
    }

    fn add_background_rects(&mut self, ctx : &mut Context){
        
        self.scenes.push(Scene::new());
        let s = self.scenes.last_mut().unwrap();

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
        // Update code here...
        self.cam_transform[0] -= 1.0;

        

        if let Some(pa) = &self.player{            
            if let Some(player_actor) = self.actors.get_mut(pa.actor_idx){
                // default moving forward.
                player_actor.transform.x += 1.0;                    
                //processing input
                player_handle_input(&pa, player_actor);
            }
        }
        
        if let Some(pa) = &self.player.as_mut(){            
            let mut size1      = Size{x : 0.0, y: 0.0};
            let mut pos1       = Position{x : 0.0, y: 0.0};
            let mut collision1 = Collision::DiscCollision(0.0);
            if let Some(player_actor) = self.actors.get(pa.actor_idx){
                // let r2 = player_actor.collision;
                size1      = player_actor.collision.get_size();
                pos1       = Position{x : player_actor.transform.x + size1.x/2.0,y : player_actor.transform.y + size1.y/2.0};
                collision1 = player_actor.collision;
            }
             
            let mut score = 0;
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
                    if let RectDraw::DynamicRect{ref mut color, ..} = a.drawable {
                        *color = RED;
                    }                        
                    score += 1;
                    a.dead = true;
                }

            }
                
            
        }
        
        println!("FPS: {}", ggez::timer::fps(_ctx));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        // apply canera transform
        let transform = DrawParam::default().dest(self.cam_transform).to_matrix();
        graphics::push_transform(ctx, Some(transform));
        graphics::apply_transformations(ctx).unwrap();

        let mut mb = graphics::MeshBuilder::new();
        
        // Draw code here...        
        for a in &self.actors {
            if !a.visible {
                continue;
            }
            a.drawable.draw(a.transform, &mut mb, &mut self.meshes, ctx);
        }    

        let mesh = mb.build(ctx).unwrap();
        mesh.draw(ctx, DrawParam::default().dest([0.0,0.0])).unwrap();

        // remove canera transform
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx).unwrap();
        let offset = 10.0;
        let mut dest_point = mint::Point2::<f32>{ x:offset, y:offset};
        graphics::draw(ctx, &self.text, DrawParam::default().dest(dest_point).color(graphics::WHITE))?;

        let text = graphics::Text::new( (format!("Score {}", self.player.as_ref().unwrap().score) , self.font, 28.0));
        dest_point.x += (self.text.width(ctx) as f32) + 10.0;
        graphics::draw(ctx, &text, DrawParam::default().dest(dest_point).color(graphics::WHITE))?;
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

    app.add_background_rects(& mut ctx);
    app.add_foreground_rects();
    app.add_end_rects();
    app.add_player();



    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}