extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
use crate::piston::PressEvent;

use graphics::math::{Vec2d, Matrix2d, identity};
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::input::{Button, Key};
use piston::window::WindowSettings;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.
    renderables: Vec<Rect>,
    cam_transform : Vec2d<f64>
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

    fn render(&mut self, gl : &mut GlGraphics, args: &RenderArgs) {
        use graphics::*;

        let shape = [ 0.0,
                      0.0,
                      self.size[0],
                      self.size[1]];

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform
                        .trans( self.translation[0], self.translation[1] )
                        .rot_rad(self.rotation)
                        .scale(self.scale, self.scale);
            
            rectangle(self.color, shape, transform , gl);
        });
    }
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        let tr = self.cam_transform;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);
            
            
            //             self.camera_transform[0],
            //             self.camera_transform[1]);

            let transform = c
                .transform
                .trans(tr[0], tr[1])
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });

        for r in &mut self.renderables {
            r.render(&mut self.gl, args);
        }
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
        // self.cam_transform[0] += 1.0;
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
        cam_transform: [0.0, 0.0]
    };

    let r = Rect { 
        rotation   : 0.0,
        translation: [0.0, 0.0],
        scale      : 1.0,
        color      : [1.0, 1.0, 0.0, 1.0],
        size       : [30.0, 30.00]
     };

    app.renderables.push(r);

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
                app.cam_transform[0] += LEFT[0];
            }
            if key == Key::Right {
                // app.renderables[0]._move(&RIGHT);
                app.cam_transform[0] += RIGHT[0];
            }
            if key == Key::Up {
                // app.renderables[0]._move(&UP);
                app.cam_transform[1] += UP[1];
            }
            if key == Key::Down {
                // app.renderables[0]._move(&DOWN);
                app.cam_transform[1] += DOWN[1];
            }            
        };
    }
}