use nalgebra as na;
use crate::unit::{Position};
use rand::Rng;
use rand::rngs::ThreadRng;

type Vector2 = na::Vector2::<f32>;
type Point2  = na::Point2::<f32>;

fn get_point_in_cell(vx1: Vector2, vx2: Vector2, vy: Vector2, x :f32, y:f32) -> Position {
    let vx = vx1 + (vx2-vx1)*y;
    let pt = (vy * y) + (vx * x);
    Position{x: pt.x, y: pt.y}
}



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
    pub x00 : Position,
    pub x01 : Position,
    pub x10 : Position,
    pub x11 : Position,
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

        let c = Cell{
            x00 : last_bot.unwrap(),
            x01 : last_top.unwrap(),
            x10 : bottom.clone(),
            x11 : top.clone(),
        };

        panic!(c.is_valid());
        
        result.push(c);
        
        last_top = Some(top.clone());
        last_bot = Some(bottom.clone());

    }
    result
}

impl Cell {
    fn get_vectors(&self) -> [Vector2; 3] {
        let mut result : [Vector2; 3] = [Vector2::identity(); 3];

        let ox = Point2::from(self.x00);
        let oy = Point2::from(self.x01);
        result[0] = oy - ox ;
        let ox = Point2::from(self.x00);
        let oy = Point2::from(self.x10);
        result[1] =  oy - ox;
        let ox = Point2::from(self.x01);
        let oy = Point2::from(self.x11);
        result[2] = oy - ox;

        result
    }

    pub fn get_points(&self) -> Vec::<Position> {
        let mut v =vec! [ self.x00, self.x01, self.x11, self.x10];
        v.reverse();
        v
    }

    pub fn get_center(&self) -> Position{
        self.get_point(0.5f32, 0.5f32)
    }

    pub fn get_point(&self, x :f32, y:f32) -> Position {
        let [vy, vx1, vx2] = self.get_vectors();
        let mut p = get_point_in_cell(vx1, vx2, vy, x, y);
        p.x += self.x00.x;
        p.y += self.x00.y;
        p
    }

    pub fn get_shrinked(&self, dist: f32) -> Cell {
        let [vy, vx1, vx2] = self.get_vectors();
        let x00 = on_bisector_at(&self.x00, &vy, &vx1, dist);
        let x01 = on_bisector_at(&self.x01, &-vy, &vx2, dist);

        let o11 = Point2::from(self.x11);
        let o10 = Point2::from(self.x10);
        let vy = o11 - o10;        
        let x11 = on_bisector_at(&self.x11, &-vy, &-vx2, dist);        
        let x10 = on_bisector_at(&self.x10, &vy, &-vx1, dist);
        Cell{
            x00 : x00.into(),
            x01 : x01.into(),
            x10 : x10.into(),
            x11 : x11.into(),
        }
    }

    pub fn get_shrinked_y(&self, dist: f32) -> Cell{
        let [vy, vx1, vx2] = self.get_vectors();
        let vy2 = Point2::from(self.x11) - Point2::from(self.x10);
        let x00 = Point2::from(self.x00) + vy.normalize() * -dist;
        let x01 = Point2::from(self.x01) + vy.normalize() * dist;
        let x10 = Point2::from(self.x10) + vy2.normalize() * -dist;
        let x11 = Point2::from(self.x10) + vy2.normalize() * dist;
        Cell{
            x00 : x00.into(),
            x01 : x01.into(),
            x10 : x10.into(),
            x11 : x11.into(),
        }
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
            let c = Cell{
                x00 : positions[bottom_left].clone(),
                x01 : positions[top_left].clone(),
                x10 : positions[bottom_left+1].clone(),
                x11 : positions[top_left+1].clone(),
            };
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
}

fn on_bisector_at(p: &Position, vy : &Vector2, vx: &Vector2, dist: f32) -> Point2{    
    let alpha = vy.angle(&vx);
    let angle = alpha / 2.0;
    let length = dist / angle.sin();
    let v = ((vy.norm() * vx) + (vx.norm()* vy)).normalize() * length;
    Point2::from(p.clone()) + v
}



pub fn place_disc_in_cell(cell : &Cell, rng :&mut ThreadRng) -> Position  {
    let x = rng.gen_range(0.0, 1.0);
    let y = rng.gen_range(0.0, 1.0);
    cell.get_point(x, y)    
  }