use nalgebra as na;
use crate::unit::{Position};
use rand::Rng;
use rand::rngs::ThreadRng;
use mint::Point2 as pp2;

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
    x00 : Position,
    x01 : Position,
    x10 : Position,
    x11 : Position,
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
        result.push(Cell{
            x00 : last_bot.unwrap(),
            x01 : last_top.unwrap(),
            x10 : bottom.clone(),
            x11 : top.clone(),
        });
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
}

  pub fn place_disc_in_cell(cell : &Cell, rng :&mut ThreadRng) -> Position  {
    let x = rng.gen_range(0.0, 1.0);
    let y = rng.gen_range(0.0, 1.0);
    cell.get_point(x, y)    
  }