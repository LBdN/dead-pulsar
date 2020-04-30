use crate::actors;
use crate::cell;
use crate::color;
use crate::effect;
use crate::mesh_gen;
use crate::render;
use crate::terrain;
use crate::text;
use crate::unit::*;
use crate::GameState;
use crate::Systems;
use ggez::graphics::Color;
use ggez::Context;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use std::mem;

fn random_rect(maxsize: f32, world_size: &Size) -> (Position, Size) {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(0.0, world_size.x) as f32;
    let y = rng.gen_range(0.0, world_size.y) as f32;
    let size = rng.gen_range(0.0, maxsize) as f32;
    (Position { x: x, y: y }, Size { x: size, y: size })
}

pub struct WorldBounds {
    pub min: Size,
    pub max: Size,
}

pub struct WorldChange {
    pub score: u32,
    pub level: Option<Id>,
    pub dead_effect: bool,
}

impl WorldChange {
    pub fn default() -> WorldChange {
        WorldChange {
            score: 0,
            level: None,
            dead_effect: false,
        }
    }
}

type KeyedEffects = KeyedGroup<effect::Effect>;

pub struct World {
    start_effects: KeyedEffects,
    tick_effects: KeyedEffects,
    pub actors: Vec<super::actors::Actor>,
    player_atr_id: Id,
    camera_atr_id: Id,
    //
    active: bool,
    pub name: String,
    pub size: Size,
}

impl World {
    pub fn empty() -> World {
        World::new("".to_string())
    }

    fn new(name: String) -> World {
        World {
            start_effects: KeyedEffects::new(),
            tick_effects: KeyedEffects::new(),
            actors: Vec::<super::actors::Actor>::new(),
            player_atr_id: no_id(),
            camera_atr_id: no_id(),
            active: false,
            name: name,
            size: Size { x: 0.0, y: 0.0 },
        }
    }

    // pub fn start(&mut self, ctx: &Context, input : &super::InputState){
    pub fn start(&mut self, ctx: &Context, state: &GameState, systems: &mut Systems) {
        self.active = true;
        //
        let wb = WorldBounds {
            min: self.get_camera_actor().transform,
            max: self.size,
        };
        for a in &mut self.actors {
            a.start();
        }
        //..
        for a in &mut self.actors {
            for effs in self.start_effects.get_mut(&a.id) {
                for e in effs {
                    e.on_actor(a, ctx, state, &wb, systems);
                }
            }
        }
        self.start_effects.clear();
    }

    pub fn stop(&mut self) {
        self.active = false;
        self.actors.clear();
        self.tick_effects.clear();
    }

    pub fn get_camera_actor(&self) -> &actors::Actor {
        self.get_actor(&self.camera_atr_id).unwrap()
    }

    fn get_player_actor(&self) -> &actors::Actor {
        self.get_actor(&self.player_atr_id).unwrap()
    }

    pub fn get_actor(&self, id: &Id) -> Option<&actors::Actor> {
        for a in &self.actors {
            if a.id == *id {
                return Some(a);
            }
        }
        None
    }

    fn get_mut_actor(&mut self, id: &Id) -> Option<&mut actors::Actor> {
        for a in &mut self.actors {
            if a.id == *id {
                return Some(a);
            }
        }
        None
    }

    fn process_collisions(&mut self) {
        if self.player_atr_id == no_id() {
            return;
        }

        let player_actor = self.get_player_actor();
        let size1 = player_actor.collision.get_size();
        let pos1 = Position {
            x: player_actor.transform.x + size1.x / 2.0,
            y: player_actor.transform.y + size1.y / 2.0,
        };
        let collision1 = player_actor.collision.clone();

        for a in &mut self.actors {
            if !a.has_collision() {
                continue;
            }

            let size2 = a.collision.get_size();
            let pos2 = Position {
                x: a.transform.x + size2.x / 2.0,
                y: a.transform.y + size2.y / 2.0,
            };

            if super::actors::collides(&pos1, &collision1, &pos2, &a.collision) {
                self.tick_effects.insert(a.id, a.on_collision.clone());
            }
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        state: &GameState,
        systems: &mut Systems,
    ) -> WorldChange {
        let mut default_wc = WorldChange::default();

        if state.paused {
            return default_wc;
        }

        self.process_collisions();

        let wb = WorldBounds {
            min: opposite_pos(&self.get_camera_actor().transform),
            max: self.size,
        };
        for a in &mut self.actors {
            let mut eff_to_remove = Vec::<usize>::new();
            for effs in self.tick_effects.get_mut(&a.id) {
                for (i, e) in effs.iter_mut().enumerate() {
                    if let Some(wc) = e.on_actor(a, ctx, state, &wb, systems) {
                        if let Some(_) = wc.level {
                            return wc;
                        } else {
                            default_wc.score += wc.score;
                        }
                        if wc.dead_effect {
                            eff_to_remove.push(i);
                        }
                    }
                }

                for i in eff_to_remove.iter().rev() {
                    effs.remove(*i);
                }
            }
        }
        default_wc
    }
}

struct WorldBuilder {
    w: World,
    debug_mm: render::MeshModel,
}

impl WorldBuilder {
    fn new(name: String) -> WorldBuilder {
        WorldBuilder {
            w: World::new(name),
            debug_mm: render::MeshModel::new(),
        }
    }

    fn set_size(&mut self, size: Size) {
        self.w.size = size;
    }

    fn get_mut_actor(&mut self, id: &Id) -> Option<&mut actors::Actor> {
        self.w.get_mut_actor(id)
    }

    fn get_actor(&mut self, id: &Id) -> Option<&actors::Actor> {
        self.w.get_actor(id)
    }

    fn add_effect_to_actor(&mut self, actor_id: &Id, eff: effect::Effect, start: bool) {
        let opt_effs = if start {
            self.w.start_effects.entry(*actor_id)
        } else {
            self.w.tick_effects.entry(*actor_id)
        };
        let effs = opt_effs.or_insert(Vec::<effect::Effect>::new());
        effs.push(eff);
    }

    // fn add_rect_to_actor(&mut self, a: &mut actors::Actor, size: Size, color: Color) {
    //     a.add_drawable(render::Renderable::DynamicRect {
    //         color: color,
    //         size: size,
    //     });
    //     let ncol = actors::rect_col_polygon(size.x, size.y);
    //     a.collision = super::actors::Collision::RectCollision {
    //         width: size.x,
    //         height: size.y,
    //         ncol: ncol,
    //     };
    // }

    fn add_to_world(&mut self, a: actors::Actor) -> Id {
        let atr_id = a.id.clone();
        self.w.actors.push(a);
        atr_id
    }

    fn add_rect_type(&mut self, mut a: super::actors::Actor, max_size: f32, color: Color) -> Id {
        let (pos, size) = random_rect(max_size, &self.w.size);

        a.transform = pos;
        // self.add_rect_to_actor(&mut a, size, color);
        let atr_id = a.id.clone();
        self.w.actors.push(a);
        atr_id
    }

    //

    fn add_player(&mut self) -> Id {
        let size = Size { x: 10.0, y: 10.0 };
        let mut a = actors::ActorType::Player.make();
        // self.add_rect_to_actor(&mut a, size, super::color::RED);
        self.w.player_atr_id = a.id.clone();
        self.w.actors.push(a);
        self.w.player_atr_id.clone()
    }

    fn add_camera(&mut self) -> Id {
        let mut a = actors::ActorType::Camera.make();        
        a.transform = Position {
            x: 0 as f32,
            y: 0 as f32,
        };

        self.w.camera_atr_id = a.id.clone();
        self.w.actors.push(a);
        self.w.camera_atr_id.clone()
    }

    fn add_default_camera(&mut self) {
        if self.w.camera_atr_id == no_id() {
            let camera_start = Position {
                x: 0 as f32,
                y: 0 as f32,
            };
            let camera_id = self.add_camera();
            let eff = effect::Effect::PlaceActor {
                actor_id: camera_id,
                position: camera_start,
            };
            self.add_effect_to_actor(&camera_id, eff, true);
            self.w.camera_atr_id = camera_id;
        }
    }

    fn add_antagonist(&mut self, max_size: f32) -> Id {
        let a = actors::ActorType::Foreground.make();
        return self.add_rect_type(a, max_size, color::random_foreground_color());
    }

    fn add_text(&mut self, text: String, fontstyle: super::text::FontStyle, position: &Position, centered: bool, systems : &mut Systems) -> Id {
        let mut a = super::actors::ActorType::UI.make();
        a.drawctx = super::actors::DrawContext::ScreenSpace;
        let text_anchor = if centered { render::TextAnchor::Center } else { render::TextAnchor::TopLeft };
        let id = systems.renderer_source.add_text_model(render::TextModel::new(
            text,
            fontstyle,
            text_anchor
            ));

        a.add_drawable(id);        
        a.transform = position.clone();
        self.add_to_world(a)
    }

    // fn add_end_rects(&mut self, exit_size: f32) -> [Id; 3] {
    //     let lose_rect_height = (self.w.size.y - exit_size) / 2.0;

    //     let mut res: [Id; 3] = [no_id(); 3];

    //     let mut yy = 0.0;
    //     for (i, rect_height) in [lose_rect_height, exit_size, lose_rect_height]
    //         .iter()
    //         .enumerate()
    //     {
    //         let mut a = actors::ActorType::Foreground.make();
    //         a.transform = Position {
    //             x: self.w.size.x as f32,
    //             y: yy as f32,
    //         };
    //         let size = Size {
    //             x: 50 as f32,
    //             y: *rect_height as f32,
    //         };

    //         self.add_rect_to_actor(&mut a, size, color::RED);
    //         res[i] = self.add_to_world(a);
    //         yy += rect_height;
    //     }

    //     res
    // }

    // BackgroundPts

    // pub fn add_background_actor(&self, pts , pos: Position, effect_on_col) -> Id{
    //     let mut a = actors::ActorType::Background.make();
    //     a.drawable = systems.renderer.add_dynamic_poly(&pts11.clone(), color::BLACK);
    //     a.collision = actors::mk_polycol(&pts);
    //     a.on_collision.push( eff_on_col );
    //     a.transform = c2.get_point(0.0, 0.0);
    //     wb.debug_pos(&a.transform);
    //     wb.add_to_world(a);
    // }

    // DEBUG

    pub fn debug_pos(&mut self, pos: &Position) {
        let mut dbg_pts = mesh_gen::regular_polygon(1.0, 4);
        for pt in &mut dbg_pts {
            pt.x += pos.x;
            pt.y += pos.y;
        }
        self.debug_mm.add_poly(&dbg_pts, &color::RED);
    }

    pub fn debug_polyline(&mut self, pts: &Vec<Position>, origin: &Position) {
        let mut dbg_pts = pts.clone();
        for pt in &mut dbg_pts {
            pt.x += origin.x;
            pt.y += origin.y;
        }
        self.debug_mm.add_polyline(&dbg_pts, &color::RED, 1.0f32);
    }

    pub fn debug_ray(&mut self, origin: &Point2, vector: &Vector2) {
        let p: Position = (*origin).into();
        self.debug_pos(&p);
        // let end_point =  origin+vector;
        let normal = Vector2::new(-vector.y, vector.x).normalize();
        let mut pts = Vec::<Position>::new();
        pts.push(Origin);
        pts.push(vec_to_pos(&normal));
        pts.push(vec_to_pos(&-normal));
        pts.push(vec_to_pos(&vector));
        // let pts = vec![origin, &(origin+normal), &end_point, &(origin-normal)];
        // let pts2 : Vec::<Position> =pts.iter().map(|v| {*v.into()});
        self.debug_polyline(&pts, &pt_to_pos(&origin));
    }

    pub fn add_debug_actor(&mut self, systems: &mut Systems, ctx: &mut Context) -> Id {
        if self.debug_mm.polygons.len() > 0 {
            let mut a = actors::ActorType::Background.make();
            let debug_mm = mem::replace(&mut self.debug_mm, render::MeshModel::new());            
            a.add_drawable(systems.renderer_source.add_mesh_model(debug_mm));
            a.visible = true;
            a.ticking = false;
            self.add_to_world(a)
        } else {
            no_id()
        }
    }

    // BUILD

    fn build(mut self, systems: &mut Systems, ctx: &mut Context) -> World {
        self.add_debug_actor(systems, ctx);
        self.w
    }
}

type LevelLoader = fn(&Level, &mut GameState, &mut Systems, &mut Context) -> World;

#[derive(Clone)]
pub struct Level {
    pub id: Id,
    name: String,
    transitions: HashMap<String, Id>,
    pub loader: LevelLoader,
}

impl Level {
    pub fn new(name: String) -> Level {
        Level {
            id: get_id(),
            name: name,
            transitions: HashMap::<String, Id>::new(),
            loader: emptyload,
        }
    }

    pub fn add_transition(&mut self, transition_name: &String, level: &Level) {
        self.transitions
            .insert(transition_name.clone(), level.id.clone());
    }

    pub fn load(&self, state: &mut GameState, systems: &mut Systems, ctx: &mut Context) -> World {
        return (self.loader)(self, state, systems, ctx);
    }

    pub fn get_transition_effect(&self, transition_name: String, duration: f32) -> effect::Effect {
        let next_id = self.transitions.get(&transition_name).unwrap();
        effect::Effect::AutoNextScene {
            duration: duration,
            cur_scene_idx: self.id.clone(),
            next_scene_idx: next_id.clone(),
        }
    }
}

fn emptyload(
    level: &Level,
    state: &mut GameState,
    systems: &mut Systems,
    ctx: &mut Context,
) -> World {
    let wb = WorldBuilder::new(level.name.clone());
    wb.build(systems, ctx)
}

pub fn introload(
    level: &Level,
    state: &mut GameState,
    systems: &mut Systems,
    ctx: &mut Context,
) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let center = Position {
        x: state.screen.x / 2.0,
        y: state.screen.y / 2.0,
    };

    let id = wb.add_text("Pulsar 3".to_string(), text::title_style(), &center, false, systems );
    wb.add_effect_to_actor(
        &id,
        level.get_transition_effect("next".to_string(), 0.0),
        false,
    );    
    wb.add_default_camera();
    wb.build(systems, ctx)
}

pub fn tutoload(
    level: &Level,
    state: &mut GameState,
    systems: &mut Systems,
    ctx: &mut Context,
) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let y_step = state.screen.y / 5.0;
    let center = Position {
        x: state.screen.x / 2.0,
        y: y_step * 2.0,
    };

    let tuto_text = format!("Level {}", state.level);
    let id = wb.add_text(tuto_text, text::title_style(), &center, true, systems);    

    let center = Position {
        x: state.screen.x / 2.0,
        y: y_step * 3.0,
    };
    let tuto_text = "Don't crash on the cavern walls...".to_string();
    let id = wb.add_text(tuto_text, text::tuto_style(), &center, true, systems);
    wb.add_effect_to_actor(
        &id,
        level.get_transition_effect("next".to_string(), 3.0),
        false,
    );
    

    let center = Position {
        x: state.screen.x / 2.0,
        y: y_step * 4.0,
    };
    let tuto_text = "and catch the yellow blocks.".to_string();
    let id = wb.add_text(tuto_text, text::tuto_style(), &center, true, systems);
    wb.add_effect_to_actor(
        &id,
        level.get_transition_effect("next".to_string(), 3.0),
        false,
    );    
    wb.add_default_camera();
    wb.build(systems, ctx)
}

pub fn gameoverload(
    level: &Level,
    state: &mut GameState,
    systems: &mut Systems,
    ctx: &mut Context,
) -> World {
    state.level = 0;
    state.score = 0;

    let mut wb = WorldBuilder::new(level.name.clone());

    let center = Position {
        x: state.screen.x / 2.0,
        y: state.screen.y / 2.0,
    };

    let id = wb.add_text("Game Over".to_string(), text::title_style(), &center, true, systems);
    wb.add_effect_to_actor(
        &id,
        level.get_transition_effect("next".to_string(), 3.0),
        false,
    );
    

    wb.add_default_camera();
    wb.build(systems, ctx)
}

pub fn victoryload(
    level: &Level,
    state: &mut GameState,
    systems: &mut Systems,
    ctx: &mut Context,
) -> World {
    state.level += 1;

    let mut wb = WorldBuilder::new(level.name.clone());

    let center = Position {
        x: state.screen.x / 2.0,
        y: state.screen.y / 2.0,
    };

    let id = wb.add_text("Victory".to_string(), text::title_style(), &center, true, systems);
    wb.add_effect_to_actor(
        &id,
        level.get_transition_effect("next".to_string(), 3.0),
        false,
    );    

    wb.add_default_camera();
    wb.build(systems, ctx)
}

pub fn playload(
    level: &Level,
    state: &mut GameState,
    systems: &mut Systems,
    ctx: &mut Context,
) -> World {
    let mut wb = WorldBuilder::new(level.name.clone());

    let mut rng = rand::thread_rng();
    let state_level = state.level + 3;
    let mut debug_mb = render::MeshBuilderOps::new();

    wb.set_size(Size {
        x: ((state_level + 3) as f32) * 1000.0,
        y: 720.0,
    });

    // PLAYER part 1
    let ship_size   = 5.0f32;
    let ship_pts    = mesh_gen::base_ship(ship_size);
    let ship_radius = Bounds2D::from_positions(&ship_pts).get_radius() * 1.2;

    let first_section_length = ship_radius * 15.0;

    let absolute_min = ship_radius * 3.0;
    let decrease_ratio = 0.65f32;
    let min_height = (50.0 * 3.0 * decrease_ratio.powi(state_level)).max(absolute_min);

    let decrease_ratio = 0.75f32;
    let max_height = (wb.w.size.y * decrease_ratio.powi(state_level)).max(absolute_min);

    let height_bounds = Bounds1D {
        min: min_height,
        max: max_height,
    };

    let section_length = Bounds1D {
        min: min_height,
        max: min_height * 4.0,
    };
    let (height_ranges, xpositions) = terrain::build_tunnel2(
        &wb.w.size,
        &section_length,
        &height_bounds,
        first_section_length,
    );
    let (top, bottom) = terrain::convert_to_polygons(&height_ranges, &xpositions, &wb.w.size);    
    let cells = terrain::convert_to_cells(&height_ranges, &xpositions);    

    // BACKGROUND.
    {
        // CAVERN BG
        let mut a = actors::ActorType::Background.make();

        let b = Bounds2D {
            min: Size { x: 0.0, y: 0.0 },
            max: wb.w.size,
        };
        let pts = terrain::build_sky(&b);

        let mut mm = render::MeshModel::new();
        mm.add_poly(&pts, &color::DARKBLUE);        
        let id = systems.renderer_source.add_mesh_model(mm);        
        a.add_drawable(id);
        wb.add_to_world(a);

        //TUNNEL TOP
        let mut a = actors::ActorType::Background.make();
        let mut mm = render::MeshModel::new();
        mm.add_poly(&top, &color::BLACK);
        mm.add_polyline(&top, &color::DARKERBLUE, 2.0);        
        a.add_drawable(systems.renderer_source.add_mesh_model(mm));
        a.collision = actors::mk_polycol(&top);
        a.on_collision.push(level.get_transition_effect("lose".to_string(), 0.0));
        a.ticking = true;
        wb.add_to_world(a);

        //TUNNEL BOTTOM
        let mut a = actors::ActorType::Background.make();
        let mut mm = render::MeshModel::new();
        mm.add_poly(&bottom, &color::BLACK);
        mm.add_polyline(&bottom, &color::DARKERBLUE, 2.0);
        a.add_drawable(systems.renderer_source.add_mesh_model(mm));
        a.collision = actors::mk_polycol(&bottom);
        a.on_collision.push(level.get_transition_effect("lose".to_string(), 0.0));
        a.ticking = true;
        wb.add_to_world(a);

        let nbsteps   = 30;
        let mut bg_colors = color::fade_to(nbsteps, &color::MEDIUMBLUE, &color::DARKBLUE);
        // bg_colors.shuffle(&mut rng);
        let mut a = actors::ActorType::Background.make();
        let mut mm = render::MeshModel::new();
        for c in &cells{
            let split_cells = c.split_xy(1, nbsteps);
            for (i, split_cell) in split_cells.iter().enumerate() {
                let pts = split_cell.get_points();    
                let idx = i % nbsteps as usize;        
                let color = bg_colors[idx];
                mm.add_poly(&pts, &color);
                // wb.debug_polyline(&pts, &Origin);
            }            
        }
        a.ticking = false;        
        a.add_drawable(systems.renderer_source.add_mesh_model(mm));
        wb.add_to_world(a);
    }

    // PLAYER part 2
    {
        let c = &cells[0];
        let player_start = c.get_center();

        let mut a = actors::ActorType::Player.make();
        a.collision = actors::mk_polycol(&ship_pts);
        let mut mm = render::MeshModel::new();
        mm.add_poly(&mesh_gen::cockpit_ship(ship_size), &color::SKYBLUE);
        mm.add_poly(&ship_pts, &color::GREY);
        
        a.add_drawable(systems.renderer_source.add_mesh_model(mm));
        let player_actor_id = wb.add_to_world(a);
        wb.w.player_atr_id = player_actor_id.clone();

        let eff = effect::Effect::MoveActor {
            actor_id: player_actor_id,
            vector: Position { x: 2.0, y: 0.0 },
        };
        wb.add_effect_to_actor(&player_actor_id, eff, false);
        wb.add_effect_to_actor(&player_actor_id, effect::Effect::ProcessInput, false);
        let eff = effect::Effect::PlaceActor {
            actor_id: player_actor_id,
            position: player_start,
        };
        wb.add_effect_to_actor(&player_actor_id, eff, true);
    }

    // CRYSTALS

    
    let mut cells2 = cells.clone();
    {
        let max_size = 17.0;
        let min_size = 5.0;

        let before_last = cells2.len() - 2;
        for c in cells2.iter_mut().skip(1).take(before_last) {
            let dist = rng.gen_range(min_size, max_size) as f32;

            // DECORATIONS.
            let decoration_height = 10.0f32;
            let side_bounds = Bounds1D::<i32>::new(7, 30);
            let dist_bounds = Bounds1D::<f32>::new(3.0, decoration_height);
            let eff_on_col = level.get_transition_effect("lose".to_string(), 0.0);
            if c.get_shrinked_y(decoration_height)
                .can_contains(ship_radius)
            {
                // BOTTOM SLICE
                let bottom_slice = c.get_bottom_slice(decoration_height);

                let n = bottom_slice.get_normal_bottom();
                // wb.debug_ray(&bottom_slice.x00, &(n*30.0));
                let nb_side = rng.gen_range(side_bounds.min, side_bounds.max);
                let (mut pts11, xpos) = mesh_gen::bump2(&n, nb_side, &dist_bounds, &mut rng);
                let y = 0.0f32;
                for (p, x) in pts11.iter_mut().zip(xpos) {
                    let pc = bottom_slice.get_relative_point(x, y);
                    p.x += pc.x;
                    p.y += pc.y;
                }

                let mut a = actors::ActorType::Background.make();
                let mut mm = render::MeshModel::new();
                mm.add_poly(&pts11.clone(), &color::BLACK);
                a.add_drawable(
                    systems.renderer_source.add_mesh_model(mm)                        
                );
                a.collision = actors::mk_polycol(&pts11);
                a.on_collision.push(eff_on_col);
                a.transform = bottom_slice.get_point(0.0, 0.0);
                // wb.debug_pos(&a.transform);
                wb.add_to_world(a);

                // TOP SLICE
                let top_slice = c.get_top_slice(decoration_height);
                // wb.debug_polyline(&top_slice.get_points(), &Origin);

                let n = top_slice.get_normal_top();
                // wb.debug_ray(&top_slice.x01, &(n*30.0));
                let nb_side = rng.gen_range(side_bounds.min, side_bounds.max);
                let (mut pts11, xpos) = mesh_gen::bump2(&n, nb_side, &dist_bounds, &mut rng);
                let y = 1.0f32;
                for (p, x) in pts11.iter_mut().zip(xpos) {
                    let pc = top_slice.get_relative_point(x, y);
                    p.x += pc.x;
                    p.y += pc.y;
                }

                let mut a = actors::ActorType::Background.make();
                let mut mm = render::MeshModel::new();
                mm.add_poly(&pts11.clone(), &color::BLACK);
                a.add_drawable(
                    systems.renderer_source.add_mesh_model(mm)           
                );
                a.collision = actors::mk_polycol(&pts11);
                a.on_collision.push(eff_on_col);
                a.transform = top_slice.get_point(0.0, 0.0);
                // wb.debug_pos(&a.transform);
                wb.add_to_world(a);

                *c = c.get_shrinked_y(decoration_height)
            } // else { c };

            let cs = c.split(2);

            let can_be_enemy = {
                let mut nb_invalid = 0;
                for c in cs.iter() {
                    let valid = c.get_shrinked(ship_radius).is_valid();
                    if !valid {
                        // let pts = c.get_points().clone();
                        // debug_mb = debug_mb.polyline(&pts, 1.0f32, color::RED);
                        nb_invalid += 1;
                    }
                }
                nb_invalid < 3
            };

            let mut p: Option<Position> = None;
            let is_enemy = can_be_enemy && rng.gen::<bool>();
            if is_enemy {
                let c2: &cell::Cell = cs.choose(&mut rng).unwrap();
                {
                    let cc2 = c2.get_shrinked(dist);
                    if cc2.is_valid() {
                        // let pts = cc2.get_points().clone();
                        // debug_mb = debug_mb.polyline(&pts, 1.0f32, color::GREY);
                        p = Some(cell::place_disc_in_cell(&cc2, &mut rng));
                    }
                }
            } else {
                // let c2 = c.get_shrinked(dist);
                p = Some(c.place_at_bottom(&mut rng));
                // p =Some( cell::place_disc_in_cell(&c2, &mut rng) );
            }

            // let is_enemy = false;
            let color = if is_enemy {
                color::BLACK
            } else {
                color::SKYBLUE
            };

            if let Some(pos) = p {
                if !is_enemy {
                    let mut dbg_pts = mesh_gen::regular_polygon(1.0, 4);
                    for pt in &mut dbg_pts {
                        pt.x += pos.x;
                        pt.y += pos.y;
                    }
                    debug_mb = debug_mb.polyline(&dbg_pts, 2.0, color::RED)
                };

                let id = wb.add_antagonist(max_size);
                wb.add_effect_to_actor(
                    &id,
                    effect::Effect::ResetActor {
                        actor_id: id.clone(),
                    },
                    true,
                );
                let a = wb.get_mut_actor(&id).unwrap();

                let pts = if is_enemy {
                    mesh_gen::irregular_polygon(
                        &Bounds1D {
                            min: dist,
                            max: 1.3 * dist,
                        },
                        7,
                        &mut rng,
                    )
                } else {
                    mesh_gen::crystal_polygon(
                        Bounds1D {
                            min: dist / 2.0,
                            max: 2.5 * dist,
                        },
                        8,
                        &mut rng,
                    )
                };

                a.collision = actors::mk_polycol(&pts);
                           
                // let draw_id = systems.renderer.add_dynamic_poly(&pts, color);
                let mut mm = render::MeshModel::new();
                mm.add_poly(&pts, &color);
                a.add_drawable(systems.renderer_source.add_mesh_model(mm));

                let eff_on_col = if is_enemy {
                    level.get_transition_effect("lose".to_string(), 0.0)
                } else {
                    effect::Effect::KillActor {
                        actor_id: a.id.clone(),
                    }
                };
                a.on_collision.push(eff_on_col);

                let sound_oidx         = systems.get_sound("/Randomize6.wav");
                if let Some(sound_idx) = sound_oidx {
                    a.on_collision.push(effect::Effect::PlaySound(*sound_idx));
                }

                a.transform = pos;
            }
        }
    }

    // END TRIGGER

    let mut a = actors::ActorType::Foreground.make();
    let c = cells.iter().nth(cells.len() - 1).unwrap();
    a.collision = actors::mk_polycol(&c.get_points());
    a.on_collision
        .push(level.get_transition_effect("win".to_string(), 0.0));
    wb.add_to_world(a);

    // CAMERA
    let camera_start = Position {
        x: 0 as f32,
        y: 0 as f32,
    };
    let camera_id = wb.add_camera();
    let eff = effect::Effect::MoveActor {
        actor_id: camera_id,
        vector: Position { x: -2.0, y: 0.0 },
    };
    wb.add_effect_to_actor(&camera_id, eff, false);
    let eff = effect::Effect::PlaceActor {
        actor_id: camera_id,
        position: camera_start,
    };
    wb.add_effect_to_actor(&camera_id, eff, true);
    wb.w.camera_atr_id = camera_id;

    // UI
    {
        let title_ui_pos = Position { x: 10.0, y: 10.0 };
        let text_id = wb.add_text("Pulsar 3".to_string(), text::ui_style(), &title_ui_pos, false, systems);

        let margin = 10.0;
        let mut score_ui_pos  = title_ui_pos.clone();
        if let Some(title_ui_actor) = wb.get_actor(&text_id) {
            if let Some(tm) = systems.renderer_source.get_text_model(&title_ui_actor.get_drawable()){
                let (w, h) = tm.get_screen_size(&systems.renderer, ctx);
                score_ui_pos.x = title_ui_actor.transform.x + (w as f32) + margin;            
            }
            
        }

        let text_id = wb.add_text("Score: 0".to_string(), text::ui_style(), &score_ui_pos, false, systems);
        wb.add_effect_to_actor(&text_id, effect::Effect::SetScore { new_value: 0 }, true);
        wb.add_effect_to_actor(
            &text_id,
            effect::Effect::UpdateScore { actor_id: text_id },
            false,
        );
    }

    wb.build(systems, ctx)
}
