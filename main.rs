extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use crate::rand::Rng;
use piston::PressEvent;

use graphics::math::{Vec2d, Matrix2d};
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
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.
    renderables: Vec<Rect>,
    cam_transform : Vec2d<f64>,
    world: World
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
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN : [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED   : [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square   = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y)   = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        let tr = self.cam_transform;
        let rs = &mut self.renderables;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            // clear(GREEN, gl);
            clear(GREEN, gl);

            let world_transform = c
                .transform
                .trans(tr[0], tr[1]);
            
            let camera_transform = c.transform;

            for r in rs {
                r.render(gl, world_transform);
            }

            let transform = camera_transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
        self.cam_transform[0] -= 1.0;
    }


    fn add_rect(&mut self){
        const MAX_X    : f64 = 1640.0;
        const MAX_Y    : f64 = 480.0;
        const MAX_SIZE : f64 = 50.0;
        let mut rng = rand::thread_rng();
        let x    = rng.gen_range(0.0, MAX_X);
        let y    = rng.gen_range(0.0, MAX_Y);
        let size = rng.gen_range(0.0, MAX_SIZE);
        let r    = rng.gen_range(0.0, 1.0);
        
        let r = Rect { 
            rotation   : 0.0,
            translation: [x, y],
            scale      : 1.0,
            color      : [r, 1.0, 0.0, 1.0],
            size       : [size, size]
         };
    
        self.renderables.push(r);
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
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
        renderables: Vec::<Rect>::new(),
        cam_transform: [0.0, 0.0],
        world: World{ size: [3000.0, 640.0]}
    };

    for _i in 1..10000 {
        app.add_rect();
    }


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
            if key == Key::Left {
                // app.renderables[0]._move(&LEFT);
                app.cam_transform[0] -= LEFT[0];
            }
            if key == Key::Right {
                // app.renderables[0]._move(&RIGHT);
                app.cam_transform[0] -= RIGHT[0];
            }
            if key == Key::Up {
                // app.renderables[0]._move(&UP);
                app.cam_transform[1] -= UP[1];
            }
            if key == Key::Down {
                // app.renderables[0]._move(&DOWN);
                app.cam_transform[1] -= DOWN[1];
            }            
        };
    }
}