
extern crate find_folder;
use cgmath;
use std::env;
use std::path;
use rand;
use rand::Rng;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::conf;
use ggez::graphics::{DrawParam, Color, Rect, Drawable, DrawMode, Mesh};

use mint::{Point2};

const GREY : Color = Color{ r: 0.5, g:0.5, b:0.5, a:1.0};


    // color      : Color,
    // size       : [f32; 2],

pub struct Actor {
    transform  : mint::Point2::<f32>,        
    mesh_idx   : usize
}

impl Actor {
    pub fn default() -> Actor {
        Actor {
            transform: mint::Point2::<f32>{ x:0.0, y:0.0},
            mesh_idx: 0
        }
    }
}


pub struct World {
    size: [f64; 2]
}


struct App {
    text  : graphics::Text,
    actors: Vec::<Actor>,
    meshes: Vec::<Mesh>,
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
            text   : text,
            actors : Vec::<Actor>::new(),
            meshes : Vec::<Mesh>::new(),
            world  : World{ size: [3000.0, 640.0]},
            cam_transform: [0.0, 0.0],
        }
    }

    fn add_foreground_rects(&mut self, ctx : &mut Context){

        const MAX_SIZE : f64 = 50.0;
        for _i in 1..100 {
            let mut a = Actor::default();

            let mut rng = rand::thread_rng();
            let x    = rng.gen_range(0.0, self.world.size[0]);
            let y    = rng.gen_range(0.0, self.world.size[1]);
            let size = rng.gen_range(0.0, MAX_SIZE);
            let r    = 1.0;
            let b    = rng.gen_range(0.0, 1.0);

            let m = graphics::Mesh::new_rectangle(
                ctx, 
                DrawMode::fill(),
                Rect {x:x as f32, y:y as f32, w:size as f32, h:size as f32}, 
                Color{r:r, g:r, b:b, a:1.0}
            ).unwrap();

            self.meshes.push(m);    
            a.mesh_idx = self.meshes.len() -1;
            self.actors.push(a);
        }
    }


    fn add_background_rects(&mut self, ctx : &mut Context){        
        const MAX_SIZE : f64 = 50.0;
        let mut mb = graphics::MeshBuilder::new();

        let mut a = Actor::default();

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
        a.mesh_idx = self.meshes.len() -1;
        self.actors.push(a);
    }
}

impl EventHandler for App {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Update code here...
        self.cam_transform[0] -= 1.0;
        println!("FPS: {}", ggez::timer::fps(_ctx));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        // apply canera transform
        let transform = DrawParam::default().dest(self.cam_transform).to_matrix();
        graphics::push_transform(ctx, Some(transform));
        graphics::apply_transformations(ctx).unwrap();

        // Draw code here...        
        for a in &self.actors {                        
            self.meshes[a.mesh_idx].draw(ctx, DrawParam::default().dest(a.transform)).unwrap();
        }        

        // remove canera transform
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx).unwrap();
        let offset = 10.0;
        let dest_point = mint::Point2::<f32>{ x:offset, y:offset};
        graphics::draw(ctx, &self.text, DrawParam::default().dest(dest_point).color(graphics::WHITE))?;

        graphics::present(ctx)
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
    app.add_foreground_rects(& mut ctx);
    


    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}