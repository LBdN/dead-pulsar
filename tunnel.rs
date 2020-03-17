use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::unit::*;
use crate::terrain::*;

#[derive(Copy, Clone)]
enum SizeChange{
    Same,
    Smaller,
    Bigger
}

const SIZES   : [SizeChange;3]   = [SizeChange::Same, SizeChange::Bigger, SizeChange::Smaller];

impl SizeChange{
    fn get(rng :&mut ThreadRng) -> SizeChange {
        *SIZES.choose(rng).unwrap()
    }

    fn get_slope(&self, rng :&mut ThreadRng) -> SlopeDirection{
        match self {
            SizeChange::Same => {
                let possibilities = [SlopeDirection::Down, SlopeDirection::Up, SlopeDirection::Flat];
                *possibilities.choose(rng).unwrap()
            },
            SizeChange::Bigger => {
                let possibilities = [SlopeDirection::Down, SlopeDirection::Flat];
                *possibilities.choose(rng).unwrap()
            },
            SizeChange::Smaller => {
                let possibilities = [SlopeDirection::Down, SlopeDirection::Up];
                *possibilities.choose(rng).unwrap()
            }
        }


    }
}



fn build_tunnel_section(world_top : f32, world_bottom : f32, max_length : f32, top_pt: &Position, bottom_pt: &Position, rng :&mut ThreadRng) -> (Position, Position) {
    let tan45 : f32 = (45.0 as f32).to_radians().tan();
    let sizechange = SizeChange::get(rng);
    let direction= sizechange.get_slope(rng);
    let space_above = world_top    - top_pt.y;
    let space_below = world_bottom - bottom_pt.y;
    let mut top_vec = top_pt.clone();
    let mut bot_vec = bottom_pt.clone();
    match direction {
        SlopeDirection::Flat => {        
            let vertical_move = 0.0f32;
            let horiz_move = vertical_move / tan45;
            if let SizeChange::Same = sizechange{                
                let horiz_move     = rng.gen_range(0.0, max_length);
                top_vec.x += horiz_move;
                bot_vec.x += horiz_move;                
            }
            else if let SizeChange::Bigger = sizechange{
                let max_vert_move  = space_above.min(max_length * tan45);
                let min_move = 0.0f32;
                if max_vert_move <= min_move{
                    println!("{} < {}", max_vert_move, min_move);
                }
                
                let vertical_move = if max_vert_move == 0.0 { 0.0 } else {rng.gen_range(min_move, max_vert_move)};
                let horiz_move     = vertical_move / tan45;                
                top_vec.x += horiz_move;
                top_vec.y += vertical_move;
                bot_vec.x += horiz_move;                                
            }      
            if let SizeChange::Smaller = sizechange{                
                let vertical_move  = rng.gen_range(0.0, max_length * tan45);
                let horiz_move     = vertical_move / tan45;
                top_vec.x += horiz_move;
                top_vec.y += -vertical_move;
                bot_vec.x += horiz_move;                                                
            }        
        }
        SlopeDirection::Down => {        
            let vertical_move = rng.gen_range(0.0,  space_below.min(max_length * tan45));
            let horiz_move = vertical_move / tan45;
            if let SizeChange::Same = sizechange{
                top_vec.x += horiz_move;
                top_vec.y += -vertical_move;
                bot_vec.x += horiz_move;
                bot_vec.y += -vertical_move;                
            }
            if let SizeChange::Bigger = sizechange{                
                top_vec.x += horiz_move;
                bot_vec.x += horiz_move;
                bot_vec.y += -vertical_move;                
            }        
        }
        SlopeDirection::Up => {        
            if let SizeChange::Same = sizechange{
                let max_vert_move  = space_above.min(max_length * tan45);                 
                let vertical_move = if max_vert_move == 0.0 { 0.0 } else {rng.gen_range(0.0, max_vert_move)};
                let horiz_move    =  vertical_move / tan45;
                top_vec.x += horiz_move;
                top_vec.y += vertical_move;
                bot_vec.x += horiz_move;
                bot_vec.y += vertical_move;                
            }
            if let SizeChange::Smaller = sizechange{                
                let horiz_move    = rng.gen_range(0.0, max_length);
                let vertical_move = horiz_move * tan45;
                top_vec.x += horiz_move;
                bot_vec.x += horiz_move;
                bot_vec.y += vertical_move;
            }        
        }
    }
    return (top_vec, bot_vec);
}

pub fn build_tunnel(world_size : &Size, max_length : f32) -> (Vec::<Position>, Vec::<Position>){    
    
    let mut length = 0.0;

    let mut top_pts    = Vec::<Position>::new();    
    let mut bottom_pts = Vec::<Position>::new();
    let tunnel_height = 10.0f32;
    let tunnel_bottom = world_size.y/3.0;
    top_pts.push( Position{ x:0.0, y:tunnel_bottom+tunnel_height}  );
    bottom_pts.push( Position{ x:0.0, y:tunnel_bottom}  );

    let mut rng = rand::thread_rng();  

    let world_top = world_size.y;
    let world_bottom = 0.0f32;
    // top_pt: &Position, bottom_pt: &Position, rng :&mut ThreadRng) -> (Position, Position) {

    while length < world_size.x{
        let segment_lenth = rng.gen_range(0.0f32, ( world_size.x - length).min(max_length));
        if segment_lenth == 0.0f32{
            break;
        }
        length += segment_lenth;
        let top_pt = top_pts.last().unwrap();
        let bottom_pt = bottom_pts.last().unwrap();
        let (top_pt, bot_pt) = build_tunnel_section(world_top, world_bottom, segment_lenth, top_pt, bottom_pt, &mut rng);
        top_pts.push(top_pt);
        bottom_pts.push(bot_pt);
    }
        
    (top_pts, bottom_pts)
}
