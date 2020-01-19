extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use crate::rand::Rng;
use piston::PressEvent;

use graphics::math::{Vec2d, Matrix2d, sub, square_len};
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::input::{Button, Key};
use piston::window::WindowSettings;


pub struct World {
    size: Vec2d<f64>
}

pub struct App {
    gl           : GlGraphics,   // OpenGL drawing backend.
    rotation     : f64,          // Rotation for the square.
    background   : Vec<Rect>,
    foreground   : Vec<Rect>,
    cam_transform: Vec2d<f64>,
    world        : World,
    player       : Option<Rect>,
    dead         : bool
}

pub struct Rect {
    rotation   : f64,
    translation: Vec2d<f64>,
    scale      : f64,
    color      : [f32; 4],
    size       : Vec2d<f64>,
}

impl Rect {

    fn _move(&mut self, v : &Vec2d){
        self.translation[0] += v[0];
        self.translation[1] += v[1];
    }

    fn render(&mut self, gl : &mut GlGraphics, world_transform: Matrix2d) {
        use graphics::*;

        let shape = [ 0.0,
                      0.0,
                      self.size[0],
                      self.size[1]];
        
        let transform = world_transform
                    .trans( self.translation[0], self.translation[1] )
                    .rot_rad(self.rotation)
                    .scale(self.scale, self.scale);
        
        rectangle(self.color, shape, transform , gl);        
        // line(self.color, 1.0, shape, transform, gl);
    }
}

impl App {
    pub const fn new(gl : GlGraphics) -> App {
        App{
            gl:gl,
            rotation: 0.0,
            background: Vec::<Rect>::new(),
            foreground: Vec::<Rect>::new(),
            cam_transform: [0.0, 0.0],
            world: World{ size: [3000.0, 640.0]},
            player: None,
            dead: false
        }
    }


    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        // const GREEN : [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const GREY  : [f32; 4] = [0.1; 4];
        // const RED   : [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        // let square   = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y)   = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        let tr = self.cam_transform;
        let bg_rects = &mut self.background;
        let fg_rects = &mut self.foreground;
        let player_r = &mut self.player;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            // clear(GREEN, gl);
            clear(GREY, gl);

            let world_transform = c
                .transform
                .trans(tr[0], tr[1]);
            
            let camera_transform = c.transform;

            for r in bg_rects {
                r.render(gl, world_transform);
            }
            for r in fg_rects {
                r.render(gl, world_transform);
            }
            if let Some(r) = player_r.as_mut(){
                r.render(gl, world_transform);
            }              
            


            let transform = camera_transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            // rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        // Rotate 2 radians per second.
        // self.rotation += 2.0 * args.dt;
        self.cam_transform[0] -= 1.0;
        //
        if let Some(r2) = self.player.as_mut(){
            r2._move( &[1.0, 0.0]);

            for r in &mut self.foreground {
                let square_dist =(r.size[0]+r2.size[0]) * (r.size[0]+r2.size[0]);
                if square_len(sub(r.translation, r2.translation)) < square_dist{
                    r.color = [1.0, 0.0, 0.0, 1.0];
                    self.dead = true;
                }            
            }
        }              

    }


    fn add_background_rect(&mut self){        
        const MAX_SIZE : f64 = 50.0;
        let mut rng = rand::thread_rng();
        let x    = rng.gen_range(0.0, self.world.size[0]);
        let y    = rng.gen_range(0.0, self.world.size[1]);
        let size = rng.gen_range(0.0, MAX_SIZE);
        let r    = rng.gen_range(0.1, 0.5);
        
        let r = Rect { 
            rotation   : 0.0,
            translation: [x, y],
            scale      : 1.0,
            color      : [r, r, r, 1.0],
            size       : [size, size]
         };
            
        self.background.push(r);    
    }

    fn add_foreground_rect(&mut self){        
        const MAX_SIZE : f64 = 50.0;
        let mut rng = rand::thread_rng();
        let x    = rng.gen_range(0.0, self.world.size[0]);
        let y    = rng.gen_range(0.0, self.world.size[1]);
        let size = rng.gen_range(0.0, MAX_SIZE);
        let b    = rng.gen_range(0.0, 1.0);
        
        let r = Rect { 
            rotation   : 0.0,
            translation: [x, y],
            scale      : 1.0,
            color      : [1.0, 1.0, b, 1.0],
            size       : [size, size]
         };
            
        self.foreground.push(r);    
    }

    fn add_player_rect(&mut self){
        

        self.player = Some(Rect { 
            rotation   : 0.0,
            translation: [10.0, self.world.size[1]/2.0],
            scale      : 1.0,
            color      : [1.0, 0.0, 0.0, 1.0],
            size       : [25.0, 25.0]
         });
    }

}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [640, 480])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let gl = GlGraphics::new(opengl);
    let mut app = App::new(gl) ;

    for _i in 1..10000 {
        app.add_background_rect();
    }
    for _i in 1..100 {
        app.add_foreground_rect();
    }
    app.add_player_rect();


    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            const LEFT  : [f64; 2] = [-10.0,   0.0];
            const RIGHT : [f64; 2] = [ 10.0,   0.0];
            const UP    : [f64; 2] = [  0.0, -10.0];
            const DOWN  : [f64; 2] = [  0.0,  10.0];
            if let Some(r) = app.player.as_mut(){
                if key == Key::Left {
                    r._move(&LEFT);
                }                                            
                if key == Key::Right {                
                    r._move(&RIGHT);
                }                
                if key == Key::Up {                    
                    r._move(&UP);
                }                            
                if key == Key::Down {
                    r._move(&DOWN);
                }                              
            }            
        };
    }
}