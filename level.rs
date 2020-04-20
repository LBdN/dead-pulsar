use std::collections::HashMap;
use ggez::{Context};
use ggez::graphics::{Color};
use rand::Rng;
use rand::seq::SliceRandom;
use crate::unit::*;
use crate::effect;
use crate::text;
use crate::color;
use crate::terrain;
use crate::render;
use crate::GameState;
use crate::actors;
use crate::cell;
use crate::mesh_gen;
use crate::{Systems};

fn random_rect(maxsize : f32, world_size : &Size) -> (Position, Size) {
    let mut rng = rand::thread_rng();
    let x    = rng.gen_range(0.0, world_size.x) as f32;
    let y    = rng.gen_range(0.0, world_size.y) as f32;
    let size = rng.gen_range(0.0, maxsize) as f32;
    (Position{x:x, y:y}, Size{x:size, y:size})
}

pub struct WorldBounds{
    pub min: Size,
    pub max: Size
}

pub struct WorldChange{
    pub score : u32,
    pub level : Option::<Id>,
    pub dead_effect : bool
}

impl WorldChange{
    pub fn default() -> WorldChange{
        WorldChange{
            score:0,
            level: None,
            dead_effect: false
        }
    }
}

type KeyedEffects = KeyedGroup::<effect::Effect>;

pub struct World{
    start_effects : KeyedEffects,
    effects       : KeyedEffects,
    pub actors        : Vec::<super::actors::Actor>,
    player_atr_id : Id,
    camera_atr_id : Id,
    //
    active        : bool,
    pub name          : String,
    pub size          : Size

}

impl World{
    pub fn empty() -> World{
        World::new("".to_string())
    }

    fn new(name : String) -> World {
        World{
            start_effects : KeyedEffects::new(),
            effects       : KeyedEffects::new(),
            actors        : Vec::<super::actors::Actor>::new(),
            player_atr_id : no_id(),
            camera_atr_id : no_id(),
            active        : false,
            name          : name,
            size          :Size{x:0.0, y:0.0}

        }
    }

    // pub fn start(&mut self, ctx: &Context, input : &super::InputState){
    pub fn start(&mut self, ctx: &Context, state : &GameState, systems : &mut Systems){
        self.active = true;
        //
        let wb = WorldBounds{min: self.get_camera_actor().transform, max:self.size};
        for a in &mut self.actors{
            a.start();
        }
        //..
        for a in &mut self.actors{
            for effs in self.start_effects.get_mut(&a.id){
                for e in effs{
                    e.on_actor(a, ctx, state, &wb, systems );
                }
            }
        }
        self.start_effects.clear();
    }

    pub fn stop(&mut self){
        self.active = false;
        self.actors.clear();
        self.effects.clear();
    }

    pub fn get_camera_actor(&self) -> &actors::Actor {
        self.get_actor(&self.camera_atr_id).unwrap()
    }

    fn get_player_actor(&self) -> &actors::Actor {
        self.get_actor(&self.player_atr_id).unwrap()
    }

    pub fn get_actor(&self, id : &Id) -> Option::<&actors::Actor> {
        for a in &self.actors{
            if a.id == *id {
                return Some(a);
            }
        }
        None
    }

    fn get_mut_actor(&mut self, id : &Id) -> Option::<&mut actors::Actor> {
        for a in &mut self.actors{
            if a.id == *id {
                return Some(a);
            }
        }
        None
    }

    fn process_collisions(&mut self){
        if self.player_atr_id == no_id(){
            return;
        }

        let player_actor = self.get_player_actor();
        let size1      = player_actor.collision.get_size();
        let pos1       = Position{
            x : player_actor.transform.x + size1.x/2.0,
            y : player_actor.transform.y + size1.y/2.0
        };
        let collision1 = player_actor.collision.clone();

        for a in &mut self.actors {
            if !a.has_collision() {
                continue;
            }

            let size2 = a.collision.get_size();
            let pos2 =  Position{
                x : a.transform.x + size2.x/2.0,
                y : a.transform.y + size2.y/2.0
            };

            if super::actors::collides(&pos1, &collision1, &pos2, &a.collision){
                self.effects.insert(a.id, a.on_collision.clone());
            }

        }

    }

    pub fn update(&mut self, ctx: &Context, state : &GameState, systems : &mut Systems ) -> WorldChange {
        let mut default_wc = WorldChange::default();

        if state.paused {
            return  default_wc;
        }

        self.process_collisions();


        let wb = WorldBounds{min: opposite_pos(&self.get_camera_actor().transform), max:self.size};
        for a in &mut self.actors{
            let mut eff_to_remove = Vec::<usize>::new();
            for effs in self.effects.get_mut(&a.id){

                for (i, e) in effs.iter_mut().enumerate(){
                    if let Some(wc) = e.on_actor(a, ctx, state, &wb, systems ){
                        if let Some(_) = wc.level{
                            return wc;
                        } else{
                            default_wc.score += wc.score;
                        }
                        if wc.dead_effect{
                            eff_to_remove.push(i);
                        }
                    }
                }

                for i in eff_to_remove.iter().rev(){
                    effs.remove(*i);
                }
            }
        }
        default_wc
    }
}


struct WorldBuilder{
    w : World
}

impl WorldBuilder{
    fn new(name : String) -> WorldBuilder{
        WorldBuilder { w : World::new(name)}
    }

    fn set_size(&mut self, size : Size){
        self.w.size = size;
    }

    fn get_mut_actor(&mut self, id : &Id) -> Option::<&mut actors::Actor>{
        self.w.get_mut_actor(id)
    }

    fn get_actor(&mut self, id : &Id) -> Option::<&actors::Actor>{
        self.w.get_actor(id)
    }

    fn add_effect_to_actor(&mut self, actor_id : &Id, eff : effect::Effect, start : bool ){
        let opt_effs = if start { self.w.start_effects.entry(*actor_id) }
                       else { self.w.effects.entry(*actor_id) };
        let effs = opt_effs.or_insert(Vec::<effect::Effect>::new());
        effs.push(eff);
    }

    fn add_rect_to_actor(&mut self, a : &mut super::actors::Actor, size: Size, color : Color){
        a.drawable = super::render::Renderable::DynamicRect {
            color   : color,
            size    : size,
        };
        let ncol = actors::rect_col_polygon(size.x, size.y);
        a.collision = super::actors::Collision::RectCollision { width: size.x, height: size.y, ncol : ncol };
    }

    fn add_to_world(&mut self, a : actors::Actor) -> Id{
        let atr_id = a.id.clone();
        self.w.actors.push(a);
        atr_id
    }

    fn add_rect_type(&mut self, mut a :  super::actors::Actor, max_size : f32, color : Color) -> Id {
        let (pos, size) = random_rect(max_size, &self.w.size);

        a.transform = pos;
        self.add_rect_to_actor(&mut a, size, color);
        let atr_id = a.id.clone();
        self.w.actors.push(a);
        atr_id
    }

    //

    fn add_player(&mut self) -> Id {
        let size  = Size{x:10.0, y:10.0};
        let mut a = actors::ActorType::Player.make();
        self.add_rect_to_actor(&mut a, size, super::color::RED);
        self.w.player_atr_id = a.id.clone();
        self.w.actors.push(a);
        self.w.player_atr_id.clone()
    }

    fn add_camera(&mut self) -> Id {
        let mut a = super::actors::Actor::new(super::actors::ActorType::Camera, get_id());
        a.drawable  = super::render::Renderable::NoDraw;
        a.collision = super::actors::mk_nocol();
        a.transform = Position{ x:0 as f32, y:0 as f32};

        self.w.camera_atr_id = a.id.clone();
        self.w.actors.push(a);
        self.w.camera_atr_id.clone()
    }

    fn add_default_camera(&mut self)  {
        if self.w.camera_atr_id == no_id(){
            let camera_start = Position{ x:0 as f32, y:0 as f32};
            let camera_id = self.add_camera();
            let eff = effect::Effect::PlaceActor{actor_id:camera_id, position: camera_start};
            self.add_effect_to_actor(&camera_id, eff, true);
            self.w.camera_atr_id =camera_id;
        }
    }



    fn add_antagonist(&mut self, max_size : f32) -> Id {
        let a = actors::ActorType::Foreground.make();
        return self.add_rect_type(a, max_size, color::random_foreground_color());
    }



    fn add_text(&mut self, text: String, fontstyle: super::text::FontStyle, centered: bool) -> Id{
        let mut a = super::actors::ActorType::UI.make();
        a.drawctx   = super::actors::DrawContext::ScreenSpace;
        a.drawable  = super::render::Renderable::DynamicTextDraw{
            string  : text,
            fontstyle : fontstyle,
            text_anchor : if centered { render::TextAnchor::Center} else {render::TextAnchor::TopLeft}
        };
        self.add_to_world(a)
    }


    fn add_end_rects(&mut self, exit_size : f32) -> [Id; 3]{

        let lose_rect_height = (self.w.size.y- exit_size) / 2.0;

        let mut res : [Id; 3] = [no_id(); 3];

        let mut yy = 0.0;
        for (i, rect_height) in [lose_rect_height, exit_size, lose_rect_height].iter().enumerate() {
            let mut a = actors::ActorType::Foreground.make();
            a.transform = Position{ x:self.w.size.x as f32, y:yy as f32};
            let size = Size{ x:50 as f32, y:*rect_height as f32 };

            self.add_rect_to_actor(&mut a, size, color::RED);
            res[i] = self.add_to_world(a);
            yy += rect_height;
        }

        res
    }

    pub fn add_debug_actor(&mut self,  mb : render::MeshBuilderOps, systems: &mut Systems, ctx: &mut Context) -> Id{
        let mut a = actors::ActorType::Background.make();
        let drawable = mb.build(&mut systems.renderer, ctx);
        a.drawable = drawable;
        a.visible = true;
        a.ticking = false;
        self.add_to_world(a)
    }

    fn build(self) -> World{
        self.w
    }

    pub fn debug_pos(&self, mut debug_mb : render::MeshBuilderOps, pos : &Position) -> render::MeshBuilderOps {
        let mut dbg_pts = mesh_gen::regular_polygon(1.0, 4);
        for pt in &mut dbg_pts{
            pt.x += pos.x;
            pt.y += pos.y;
        }
        debug_mb.polyline(&dbg_pts, 2.0, color::RED)        
    }
    
}

type LevelLoader = fn(&Level, &mut GameState, &mut Systems, &mut Context) -> World;


#[derive(Clone)]
pub struct Level{
    pub id         : Id,
    name       : String,
    transitions: HashMap::<String, Id>,
    pub loader     : LevelLoader,
}

impl Level{
    pub fn new(name : String) -> Level{
        Level{
            id         : get_id(),
            name       : name,
            transitions: HashMap::<String, Id>::new(),
            loader     : emptyload
        }
    }

    pub fn add_transition(&mut self, transition_name : &String, level : &Level){
        self.transitions.insert(transition_name.clone(), level.id.clone());
    }

    pub fn load(&self, state: &mut GameState, systems: &mut Systems, ctx: &mut Context) -> World {
        return (self.loader)(self, state, systems, ctx);
    }

    pub fn get_transition_effect(&self, transition_name : String, duration: f32) -> effect::Effect{
        let next_id = self.transitions.get(&transition_name).unwrap();
        effect::Effect::AutoNextScene{ duration:duration, cur_scene_idx : self.id.clone(), next_scene_idx : next_id.clone()}
    }

}

fn emptyload(level : &Level, state: &mut GameState, _systems: &mut Systems, _ctx: &mut Context) -> World {
    let wb = WorldBuilder::new(level.name.clone());
    wb.build()
}

pub fn introload(level : &Level, state: &mut GameState, _systems: &mut Systems, _ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let center = Position{x: state.screen.x /2.0, y: state.screen.y /2.0 };

    let id = wb.add_text("Pulsar 3".to_string(), text::title_style(),false);
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string(), 0.0 ), false );
    wb.get_mut_actor(&id).map( |a| {
        a.transform = center;
        a
    });

    wb.add_default_camera();
    wb.build()
}

pub fn tutoload(level : &Level, state: &mut GameState, _systems: &mut Systems, _ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let y_step = state.screen.y /5.0;
    let center = Position{x: state.screen.x /2.0, y: y_step*2.0};

    let tuto_text = format!( "Level {}", state.level);
    let id = wb.add_text(tuto_text, text::title_style(),true);
    wb.get_mut_actor(&id).unwrap().transform = center;

    let center = Position{x: state.screen.x /2.0, y: y_step*3.0};
    let tuto_text = "Don't crash on the cavern walls...".to_string();
    let id = wb.add_text(tuto_text, text::tuto_style(),true);
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string(), 3.0), false );
    wb.get_mut_actor(&id).unwrap().transform = center;

    let center = Position{x: state.screen.x /2.0, y: y_step*4.0};
    let tuto_text = "and catch the yellow blocks.".to_string();
    let id = wb.add_text(tuto_text, text::tuto_style(),true);
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string(), 3.0), false );
    wb.get_mut_actor(&id).unwrap().transform = center;

    wb.add_default_camera();
    wb.build()
}

pub fn gameoverload(level : &Level, state: &mut GameState, _systems: &mut Systems, _ctx: &mut Context) -> World {
    state.level =0;
    state.score =0;

    let mut wb = WorldBuilder::new(level.name.clone());

    let center = Position{x: state.screen.x /2.0, y: state.screen.y /2.0 };

    let id = wb.add_text("Game Over".to_string(), text::title_style(),true);
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string(), 3.0), false );
    wb.get_mut_actor(&id).unwrap().transform = center;

    wb.add_default_camera();
    wb.build()
}

pub fn victoryload(level : &Level, state: &mut GameState, _systems: &mut Systems, _ctx: &mut Context) -> World {
    state.level +=1;

    let mut wb = WorldBuilder::new(level.name.clone());

    let center = Position{x: state.screen.x /2.0, y: state.screen.y /2.0 };

    let id = wb.add_text("Victory".to_string(), text::title_style(),true);
    wb.add_effect_to_actor( &id, level.get_transition_effect("next".to_string(), 3.0), false );
    wb.get_mut_actor(&id).unwrap().transform = center;

    wb.add_default_camera();
    wb.build()
}

pub fn playload(level : &Level, state: &mut GameState, systems: &mut Systems, ctx: &mut Context) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let state_level = state.level+3;
    let mut debug_mb = render::MeshBuilderOps::new();


    wb.set_size(Size{x:((state_level+3) as f32)*1000.0, y:720.0});


    // PLAYER part 1
    let player_size = 5.0f32;
    let pts = mesh_gen::base_ship( player_size);
    let player_radius = Bounds2D::from_positions(&pts).get_radius() * 1.2;

    let first_section_length = player_radius * 15.0;


    let absolute_min  = player_radius * 3.0;
    let decrease_ratio = 0.65f32;
    let min_height = (50.0 * 3.0 * decrease_ratio.powi(state_level)).max(absolute_min);

    let decrease_ratio = 0.75f32;
    let max_height = (wb.w.size.y * decrease_ratio.powi(state_level)).max(absolute_min);

    let height_bounds = Bounds1D{min: min_height, max: max_height};

    let section_length = Bounds1D{min:min_height, max:min_height*2.0};
    let (height_ranges, xpositions) = terrain::build_tunnel2(&wb.w.size, &section_length, &height_bounds, first_section_length);
    let (top, bottom) = terrain::convert_to_polygons(&height_ranges, &xpositions, &wb.w.size);
    // terrain::invert_pos(&wb.w.size, &mut top, false);
    // terrain::invert_pos(&wb.w.size, &mut bottom, false);
    let cells = terrain::convert_to_cells(&height_ranges, &xpositions);
    // let cells = cell::create_cells(&top, &bottom);

    // BACKGROUND.
    {
        // SKY
        let mut a = actors::ActorType::Background.make();
        let mut mb = render::MeshBuilderOps::new();
        let b = Bounds2D{min: Size{x:0.0, y:0.0}, max: wb.w.size};
        let pts = terrain::build_sky(&b);
        mb = mb.polygon(&pts, color::DARKBLUE);
        let drawable = mb.build(&mut systems.renderer, ctx);
        a.drawable = drawable;
        wb.add_to_world(a);

        //TUNNEL TOP
        let mut a = actors::ActorType::Background.make();
        let mut mb = render::MeshBuilderOps::new();
        mb = mb.polygon(&top, color::BLACK);
        mb = mb.polyline(&top, 2.0, color::DARKERBLUE);
        let drawable = mb.build(&mut systems.renderer, ctx);
        a.drawable = drawable;
        a.collision = actors::mk_polycol(&top);
        a.on_collision.push(level.get_transition_effect("lose".to_string(), 0.0));
        a.ticking = true;
        wb.add_to_world(a);

        //TUNNEL BOTTOM
        let mut a = actors::ActorType::Background.make();
        let mut mb = render::MeshBuilderOps::new();
        mb = mb.polygon(&bottom, color::BLACK);
        mb = mb.polyline(&bottom, 2.0, color::DARKERBLUE);
        let drawable = mb.build(&mut systems.renderer, ctx);
        a.drawable = drawable;
        a.collision = actors::mk_polycol(&bottom);
        a.on_collision.push(level.get_transition_effect("lose".to_string(), 0.0));
        a.ticking = true;
        wb.add_to_world(a);


        // let nbsteps = 5;
        // for (i, c)  in color::fade_to(nbsteps, &color::RED, &color::GREEN).iter().enumerate(){
        //     let b1 = Bounds{min: Size{x:0.0, y:((nbsteps - i as i32) as f32)*10.0}, max: wb.w.size};
        //     let mut pts = terrain::build_terrain(&b1, wb.w.size.x / 10.0);
        //     terrain::invert_pos(&wb.w.size, &mut pts);
        //     mb = mb.polygon(pts.clone(), *c);
        //     if (i as i32) == nbsteps -1{
        //         a.collision = actors::mk_polycol(&pts);
        //         a.on_collision.push(level.get_transition_effect("lose".to_string(), 0.0));
        //         a.ticking = true;
        //     }
        // }
        // let (top, bottom) = tunnel::build_tunnel(&wb.w.size, wb.w.size.x / 10.0);

        // terrain::invert_pos(&wb.w.size, &mut pts);


        // mb = mb.polygon(bottom.clone(), color::RED);
        // mb = mb.polyline(cells.iter().nth(cells.len()-2).unwrap().get_points(), 2.0, color::GREY);

        // // for (i, c)  in color::fade_to(nbsteps, &color::RED, &color::GREEN).iter().enumerate(){
        // //     let b1 = Bounds{min: Size{x:0.0, y:((nbsteps - (i+1) as i32) as f32)*10.0}, max: wb.w.size};
        // //     let pts = terrain::build_terrain(&b1, wb.w.size.x / 10.0);
        // //     mb = mb.polygon(pts, *c);
        // // }

        // let drawable = mb.build(&systems.renderer, ctx);
        // a.drawable = drawable;
        // wb.add_to_world(a);


    }

    // PLAYER part 2
    {
        let c = &cells[0];
        let player_start = c.get_center();

        let mut a = actors::ActorType::Player.make();
        a.collision = actors::mk_polycol(&pts);
        a.drawable = systems.renderer.add_dynamic_poly(&pts, color::GREY);
        let player_actor_id = wb.add_to_world(a);
        wb.w.player_atr_id = player_actor_id.clone();

        let eff = effect::Effect::MoveActor{actor_id:player_actor_id, vector:Position{x :2.0, y:0.0}};
        wb.add_effect_to_actor(&player_actor_id, eff, false);
        wb.add_effect_to_actor(&player_actor_id, effect::Effect::ProcessInput, false);
        let eff = effect::Effect::PlaceActor{actor_id:player_actor_id, position: player_start};
        wb.add_effect_to_actor(&player_actor_id, eff, true);
    }


    // CRYSTALS



    let mut rng = rand::thread_rng();
    let mut cells2 = cells.clone();
    {
        let max_size = 17.0;
        let min_size = 5.0;

        let before_last = cells2.len()-2;
        for c in cells2.iter_mut().skip(1).take(before_last){

            let dist    = rng.gen_range(min_size, max_size) as f32;

            // decorations.
            let decoration_height = 10.0f32;
            let side_bounds = Bounds1D::<i32>::new(3, 7);
            let dist_bounds = Bounds1D::<f32>::new(3.0, 10.0);
            let eff_on_col = level.get_transition_effect("lose".to_string(), 0.0);
            if c.get_shrinked_y(decoration_height).can_contains(player_radius){
                let c2       = c.get_bottom_slice(decoration_height);


                // debug_mb = debug_mb.polyline(&c2.get_points(), 1.0f32, color::GREY);
                let x   = rng.gen_range(0.0f32, 1.0f32);
                let y   = 0.0f32;
                let nb_side  = rng.gen_range(side_bounds.min, side_bounds.max);
                let (_, n) = c2.get_pos_and_normal(x,y);
                // let pts = mesh_gen::bump(&n, &dist_bounds, nb_side, &mut rng );                
                let (mut pts11, xpos) = mesh_gen::bump2(&n, nb_side, &dist_bounds, &mut rng );                
                //
                for (p, x) in pts11.iter_mut().zip(xpos){
                    let pc = c2.get_relative_point(x, y);
                    p.x += pc.x;
                    p.y += pc.y;
                }
                let mut a = actors::ActorType::Background.make();
                // DEBUG
                // let mut dbg_pts =  pts11.clone();
                // for pt in &mut dbg_pts{
                //     pt.x += pos.x;
                //     pt.y += pos.y;
                // }
                // debug_mb = debug_mb.polyline(&dbg_pts, 1.0f32, color::RED);
                // END DEBUG
                a.drawable = systems.renderer.add_dynamic_poly(&pts11.clone(), color::BLACK);
                a.collision = actors::mk_polycol(&pts);                
                a.on_collision.push( eff_on_col );
                a.transform = c2.get_point(0.0, 0.0);
                // debug_mb = wb.debug_pos(debug_mb, &pos);
                wb.add_to_world(a);
                *c = c.get_shrinked_y(decoration_height)
            } // else { c };



            let cs = c.split(2);

            let can_be_enemy= {
                let mut nb_invalid = 0;
                for c in cs.iter(){
                    let valid = c.get_shrinked(player_radius).is_valid();
                    if !valid {
                        // let pts = c.get_points().clone();
                        // debug_mb = debug_mb.polyline(&pts, 1.0f32, color::RED);
                        nb_invalid += 1;
                    }
                }
                nb_invalid < 3
            };

            let mut p : Option<Position> = None;
            let is_enemy = can_be_enemy && rng.gen::<bool>();
            if is_enemy {
                let c2 : &cell::Cell = cs.choose(&mut rng).unwrap();
                {
                    let cc2 = c2.get_shrinked(dist);
                    if cc2.is_valid(){
                        // let pts = cc2.get_points().clone();
                        // debug_mb = debug_mb.polyline(&pts, 1.0f32, color::GREY);
                        p = Some( cell::place_disc_in_cell(&cc2, &mut rng) );
                    }
                }
            } else {
                // let c2 = c.get_shrinked(dist);
                p = Some(c.place_at_bottom(&mut rng));
                // p =Some( cell::place_disc_in_cell(&c2, &mut rng) );
            }

            // let is_enemy = false;
            let color = if is_enemy{
                color::BLACK
            } else {
                color::SKYBLUE
            };

            if let Some(pos) = p {

                if !is_enemy{
                    let mut dbg_pts = mesh_gen::regular_polygon(1.0, 4);
                    for pt in &mut dbg_pts{
                        pt.x += pos.x;
                        pt.y += pos.y;
                    }
                    debug_mb =debug_mb.polyline(&dbg_pts, 2.0, color::RED)
                };


                let id = wb.add_antagonist(max_size);
                wb.add_effect_to_actor(&id, effect::Effect::ResetActor{actor_id : id.clone()}, true);
                let a = wb.get_mut_actor(&id).unwrap();


                // let dist    = rng.gen_range(2.0, max_size) as f32;


                let pts = if is_enemy {mesh_gen::irregular_polygon(&Bounds1D{min:dist, max: 1.3*dist}, 7, &mut rng)}
                          else {mesh_gen::crystal_polygon(Bounds1D{min:dist/2.0, max: 2.5*dist} ,8, &mut rng)} ;

                a.collision = actors::mk_polycol(&pts);
                a.drawable = systems.renderer.add_dynamic_poly(&pts, color);

                let eff_on_col = if is_enemy{
                    level.get_transition_effect("lose".to_string(), 0.0)
                } else {
                    effect::Effect::KillActor{actor_id:a.id.clone()}
                };

                a.on_collision.push( eff_on_col );
                let sound_oidx = systems.get_sound("/Randomize6.wav");
                if let Some(sound_idx) = sound_oidx{
                    a.on_collision.push( effect::Effect::PlaySound(*sound_idx));
                }

                a.transform = pos;
            }
        }
    }

    // END TRIGGER

    let mut a = actors::ActorType::Foreground.make();
    let c = cells.iter().nth(cells.len()-1).unwrap();
    a.collision = actors::mk_polycol(&c.get_points());
    a.on_collision.push(level.get_transition_effect("win".to_string(), 0.0));
    wb.add_to_world(a);




    // CAMERA
    let camera_start = Position{ x:0 as f32, y:0 as f32};
    let camera_id = wb.add_camera();
    let eff = effect::Effect::MoveActor{actor_id:camera_id, vector:Position{x :-2.0, y:0.0}};
    wb.add_effect_to_actor(&camera_id, eff, false);
    let eff = effect::Effect::PlaceActor{actor_id:camera_id, position: camera_start};
    wb.add_effect_to_actor(&camera_id, eff, true);
    wb.w.camera_atr_id =camera_id;

    // UI
    {
        let text_id  = wb.add_text("Pulsar 3".to_string(), text::ui_style(), false);

        let ui_pos =Position{x: 10.0, y: 10.0};
        wb.get_mut_actor(&text_id).map(|a|{
            a.transform = ui_pos.clone();
            a.drawable = systems.renderer.convert_to_static_text(&a.drawable);
            a
        });

        let margin = 10.0;
        let mut p = ui_pos.clone();
        if let Some(a) = wb.get_actor(&text_id){
            if let render::Renderable::StaticText(i) = a.drawable{
                let (w, _) = systems.renderer.texts[i].text.dimensions(ctx);
                p.x = a.transform.x+(w as f32) +margin;
            }
        }

        let text_id = wb.add_text("Score: 0".to_string(), text::ui_style(), false);
        wb.get_mut_actor(&text_id).map(|a|{
            a.transform = p;
            a
        });
        wb.add_effect_to_actor(&text_id, effect::Effect::SetScore{new_value : 0}, true);
        wb.add_effect_to_actor(&text_id, effect::Effect::UpdateScore{actor_id:text_id}, false);
    }

    wb.add_debug_actor(debug_mb , systems, ctx);
    wb.build()
}