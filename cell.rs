use std::f64::consts::{PI, FRAC_PI_2};

use rand::Rng;
use rand::rngs::ThreadRng;
use nalgebra as nal;
use nal::{Vector3, Rotation3};

use crate::unit::*;







fn minimum_distance(v : Point2, w : Point2, p : Point2) -> f32 {
    // Return minimum distance between line segment vw and point p
    let length_sq = na::distance_squared(&v, &w);
    if length_sq == 0.0 { // v == w case
        return na::distance(&v, &p);
    }   
    // Consider the line extending the segment, parameterized as v + t (w - v).
    // We find projection of point p onto the line. 
    // It falls where t = [(p-v) . (w-v)] / |w-v|^2
    // We clamp t from [0,1] to handle points outside the segment vw.
    let t = (na::dot(&(p -v),&( w-v )) / length_sq).min(1.0).max(0.0);    
    let projection = v + (w - v)*t;  // Projection falls on the segment
    return na::distance(&p, &projection);
  }

  #[derive(Copy, Clone)]
 pub struct Cell{
    pub x00 : Point2,
    pub x01 : Point2,
    pub x10 : Point2,
    pub x11 : Point2,

    pub vy : Vector2,
    pub vx1 : Vector2,
    pub vx2 : Vector2,    
}

pub fn create_cells(top_pts : &Vec<Position>, bot_pts : &Vec<Position>) -> Vec::<Cell> {
    let mut result = Vec::<Cell>::new();
    let mut last_top : Option<Position> = None;
    let mut last_bot : Option<Position> = None;
    for (top, bottom) in top_pts.iter().zip(bot_pts.iter()) {

        if let None = last_top {
            last_top = Some(top.clone());
            last_bot = Some(bottom.clone());
            continue;
        }


        let x00 = last_bot.unwrap();
        let x01 = last_top.unwrap();
        let x10 = bottom.clone();
        let x11 = top.clone();

        let c = Cell::new_from_pos(&x00, &x01, &x11, &x10);

        panic!(c.is_valid());
        
        result.push(c);
        
        last_top = Some(top.clone());
        last_bot = Some(bottom.clone());

    }
    result
}

impl Cell {

    pub fn new_from_pos(bl: &Position, tl: &Position, tr : &Position, br: &Position) -> Cell {
        let x00 = Point2::from(bl.clone());
        let x01 = Point2::from(tl.clone());
        let x11 = Point2::from(tr.clone());
        let x10 = Point2::from(br.clone());
        Cell::new(&x00, &x01, &x11, &x10)
    }

    pub fn new(x00: &Point2, x01: &Point2, x11 : &Point2, x10: &Point2) -> Cell{
        
        let vy  = x01 - x00;
        let vx1 = x10 - x00;
        let vx2 = x11 - x01;

        let c = Cell{
            x00 : x00.clone(),        
            x01 : x01.clone(),
            x10 : x10.clone(),
            x11 : x11.clone(),
            vy  : x01 - x00,
            vx1 : x10 - x00,
            vx2 : x11 - x01,
        };
        c
    }

    pub fn get_points(&self) -> Vec::<Position> {
        let mut v : Vec::<Position> =vec! [ self.x00.into(), self.x01.into(), self.x11.into(), self.x10.into()];
        v.reverse();
        v
    }

    pub fn get_center(&self) -> Position{
        self.get_point(0.5f32, 0.5f32)
    }

    pub fn get_point(&self, x :f32, y:f32) -> Position {        
        let vx : Vector2 = self.vx1 + ((self.vx2-self.vx1)*y);
        let pt : Point2  = self.x00 + ((self.vy * y) + (vx * x));        
        pt.into()
    }

    pub fn get_relative_point(&self, x :f32, y:f32) -> Position {
        let vx : Vector2 = self.vx1 + ((self.vx2-self.vx1)*y);
        let pt : Point2  = ((self.vy * y) + (vx * x)).into();        
        pt.into()
    }

    pub fn get_pos_and_normal(&self, x: f32, y: f32) -> (Position, Vector2) {
        let p  = self.get_point(x, y);
        let p2 = self.get_point(1.0, y);
        let v = Point2::from(p2) - Point2::from(p);
        let v2 = Vector2::new(-v.y,  v.x);
        (p, v2.normalize())
    }

    pub fn get_normal_bottom(&self) -> Vector2 {
        Vector2::new(-self.vx1.y,  self.vx1.x).normalize()
    }

    pub fn get_normal_top(&self) -> Vector2 {
        (Vector2::new(-self.vx2.y,  self.vx2.x)* -1.0).normalize() 
    }

    pub fn get_shrinked(&self, dist: f32) -> Cell {        
        let x00 = on_bisector_at(&self.x00, &self.vy, &self.vx1, dist);
        let x01 = on_bisector_at(&self.x01, &-self.vy, &self.vx2, dist);
        
        let vy = self.x11 - self.x10;        
        let x11 = on_bisector_at(&self.x11, &-vy, &-self.vx2, dist);        
        let x10 = on_bisector_at(&self.x10, &vy, &-self.vx1, dist);
        Cell::new(&x00, &x01, &x11, &x10)
    }

    pub fn get_shrinked_y(&self, dist: f32) -> Cell{        
        let vy2 = self.x11- self.x10;
        let x00 = self.x00 + self.vy.normalize() * dist;
        let x01 = self.x01 + self.vy.normalize() * -dist;
        let x10 = self.x10 + vy2.normalize() * dist;
        let x11 = self.x11 + vy2.normalize() * -dist;
        Cell::new( &x00, &x01, &x11, &x10)
    }

    pub fn get_bottom_slice(&self, dist: f32) -> Cell{        
        let vyn = self.vy.normalize();
        let vxn = self.vx1.normalize();
        let cos_alpha = vxn.dot(&vyn);
        let dist1 = dist / cos_alpha.acos().sin();
        let x01 = self.x00 + vyn * dist1;        

        let vy2 = self.x11 - self.x10;
        let vy2n = vy2.normalize();
        let cos_alpha = (-vxn).dot(&vy2n);
        let dist2 = dist / cos_alpha.acos().sin();
        let x11 = Point2::from(self.x10) + vy2n * dist2;        
        
        Cell::new( &self.x00, &x01, &x11, &self.x10)
    }

    pub fn get_top_slice(&self, dist: f32) -> Cell{        
        let vyn = self.vy.normalize() * -1.0; 
        let vxn = self.vx2.normalize();   
        // if vyn.cross(&vxn) > 1{

        // }
        let cos_alpha = vxn.dot(&vyn);
        let dist1 = dist / cos_alpha.acos().sin();
        let x00 = self.x01 + vyn * dist1;        

        let vy2 = self.x11 - self.x10;
        let vy2n = vy2.normalize() * -1.0;
        let cos_alpha = (-vxn).dot(&vy2n);
        let dist2 = dist / cos_alpha.acos().sin();
        let x10 = self.x11 + vy2n * dist2;        
        
        Cell::new( &x00, &self.x01, &self.x11, &x10)
    }

    pub fn split(&self, number: i32) -> Vec::<Cell> {
        let mut result = Vec::<Cell>::new();

        let mut positions = Vec::<Position>::new();
        for i in 0..=number{
            let y = i as f32 / number as f32;
            for j in 0..=number{
                let x = j as f32 / number as f32;
                let p = self.get_point(x, y);
                positions.push(p);
            }
        }

        let col_number = number + 1;
        for i in 0..number*number{
            let col = i % number;
            let row = i / number;
            let bottom_left : usize = (row*col_number + col) as usize;
            let top_left : usize    = ((row+1)*col_number + col) as usize;
            let c = Cell::new_from_pos(
                 &positions[bottom_left].clone(),
                 &positions[top_left].clone(),
                 &positions[top_left+1].clone(),
                 &positions[bottom_left+1].clone(),                 
            );
            result.push(c);
        }
        result
    }

    pub fn split_xy(&self, number_col: i32, number_row: i32) -> Vec::<Cell> {
        let mut result = Vec::<Cell>::new();

        let mut positions = Vec::<Position>::new();
        for i in 0..=number_row{
            let y = i as f32 / number_row as f32;
            for j in 0..=number_col{
                let x = j as f32 / number_col as f32;
                let p = self.get_point(x, y);
                positions.push(p);
            }
        }

        let col_number = number_col + 1;
        for i in 0..number_row*number_col{
            let col = i % number_col;
            let row = i / number_col;
            let bottom_left : usize = (row*col_number + col) as usize;
            let top_left : usize    = ((row+1)*col_number + col) as usize;
            let c = Cell::new_from_pos(
                &positions[bottom_left].clone(),
                &positions[top_left].clone(),
                &positions[top_left+1].clone(),
                &positions[bottom_left+1].clone(),                 
           );
            result.push(c);
        }
        result
    }

    pub fn is_valid(&self) -> bool{
        // TODO : change the coordinate system
        // which is 0,0 at topleft and y grow down. 
        // the sign on y is strange because of it.
        let a = self.x10.x > self.x00.x;
        let b = self.x01.y > self.x00.y;
        let c = self.x11.x > self.x01.x;
        let d = self.x11.y > self.x10.y;
        return a && b && c && d;
    }

    pub fn place_at_bottom(&self, rng :&mut ThreadRng) -> Position  {
        let x = rng.gen_range(0.0, 1.0);
        let y = 0.0;        
        self.get_point(x, y)    
      }

    pub fn can_contains(&self, radius : f32) -> bool{
        self.get_shrinked(radius).is_valid()                    
    }
}

fn on_bisector_at(p: &Point2, vy : &Vector2, vx: &Vector2, dist: f32) -> Point2{    
    let alpha = vy.angle(&vx);
    let angle = alpha / 2.0;
    let length = dist / angle.sin();
    let v : Vector2 = ((vy.norm() * vx) + (vx.norm()* vy)).normalize() * length;
    p + v
}



pub fn place_disc_in_cell(cell : &Cell, rng :&mut ThreadRng) -> Position  {
    let x = rng.gen_range(0.0, 1.0);
    let y = rng.gen_range(0.0, 1.0);
    cell.get_point(x, y)    
  }