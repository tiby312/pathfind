



use crate::pathfind::*;
use crate::grid::*;
use crate::short_path::*;

use crate::axgeom::*;
use duckduckgeo::bot::*;


#[derive(Eq,PartialEq,Debug)]
enum GridBotState{
	DoingNothing,
	Thinking,
	Moving(Vec2<GridNum>,ShortPathIter)
}

struct GridBot{
	bot:Bot,
	pos:Vec2<GridNum>,
	state:GridBotState
}

fn move_to_point(bot:&mut Bot,target:Vec2<WorldNum>) -> bool{
	unimplemented!()
}



pub struct Game{
	grid:GridDim2D,
	bots:Vec<GridBot>,
	pathfinder:PathFinder
}

fn pick_empty_spot(grid:&GridDim2D)->Vec2<GridNum>{
	unimplemented!();
}
impl Game{

	fn step(&mut self){

		let mut path_requests=Vec::new();
		for (i,b) in self.bots.iter_mut().enumerate(){
			if b.state == GridBotState::DoingNothing{
				let req = PathFindInfo{start:b.pos,end:pick_empty_spot(&self.grid),bot_index:i};
				b.state = GridBotState::Thinking;
				path_requests.push(req);
			}
		}

		let mut results = self.pathfinder.handle_par(&self.grid.inner,path_requests);

		for res in results.drain(..){
			let b=&mut self.bots[res.info.bot_index];
			assert_eq!(b.state,GridBotState::Thinking);
			match res.path{
				Some(path)=>{
					b.state=GridBotState::Moving(b.pos,path.iter());		
				},
				None=>{

				}
			}
		}

		
		for b in self.bots.iter_mut(){
			if let GridBotState::Moving(ref mut curr_target,ref mut path)=&mut b.state{
				if move_to_point(&mut b.bot,self.grid.convert_to_world(*curr_target)){
					match path.next(){
						Some(target)=>{
							*curr_target+=target.into_vec();
						},
						None=>{
							b.state=GridBotState::DoingNothing;
						}
					}
				}
			}
		}
		


	}
}