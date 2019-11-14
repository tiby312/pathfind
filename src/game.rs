



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
	pos:Vec2<GridNum>, //needed to infer the next step
	state:GridBotState
}


fn update_bot(_bot:&mut Bot){
	unimplemented!()
}
fn move_to_point(_bot:&mut Bot,_target:Vec2<WorldNum>) -> bool{
	unimplemented!()
}



pub struct Game{
    bot_prop:BotProp,
	grid:GridDim2D,
	bots:Vec<GridBot>,
	pathfinder:PathFinder
}

fn pick_empty_spot(grid:&GridDim2D)->Vec2<GridNum>{
	let gg=&grid.inner;
	let k:Vec<_>=Iterator2D::new(gg.get_dim()).filter(|a|!gg.get(*a)).collect();

	unimplemented!();
}

impl Game{
	pub fn new()->Game{
		let pathfinder=PathFinder::new();
		let dim=Rect::new(-100.,100.,-100.,100.);
		let mut grid=GridDim2D{dim,inner:Grid2D::new(10,10)};

		grid.inner.set(vec2(0,0),true);
		grid.inner.set(vec2(0,9),true);
		grid.inner.set(vec2(9,0),true);
		grid.inner.set(vec2(9,9),true);


		grid.inner.set(vec2(3,0),true);
		grid.inner.set(vec2(3,1),true);
		grid.inner.set(vec2(3,2),true);
		grid.inner.set(vec2(3,3),true);
		grid.inner.set(vec2(3,4),true);


		grid.inner.set(vec2(7,5),true);
		grid.inner.set(vec2(7,6),true);
		grid.inner.set(vec2(7,7),true);
		grid.inner.set(vec2(7,8),true);
		grid.inner.set(vec2(7,9),true);


		let bot_prop=BotProp{
            radius:Dist::new(12.0),
            collision_drag:0.003,
            collision_push:1.3,
            minimum_dis_sqr:0.0001,
            viscousity_coeff:0.03
        };

        let num_bot=1000;
        let s=dists::grid::Grid::new(dim,num_bot);
    	let bots:Vec<GridBot>=s.take(num_bot).map(|pos|{
    		let bot=Bot::new(vec2(pos.x as f32,pos.y as f32));
    		GridBot{bot,pos:grid.convert_to_grid(pos),state:GridBotState::DoingNothing}
    	}).collect();

		Game{grid,bots,pathfinder,bot_prop}
	}

	pub fn wall_len(&self)->usize{
		self.grid.inner.len()
	}
	pub fn wall_iter<'a>(&'a self)->impl Iterator<Item=Vec2<WorldNum>> + 'a{
		let a = Iterator2D::new(vec2(self.grid.inner.xdim(),self.grid.inner.ydim()));
		a.filter(move |a|self.grid.inner.get(*a)).map(move |a|self.grid.convert_to_world(a))
	}

	pub fn bot_len(&self)->usize{
		self.bots.len()
	}
	pub fn bots_iter(&self)->impl Iterator<Item=&Bot>{
		self.bots.iter().map(|a|&a.bot)
	}

	pub fn step(&mut self){

		let mut path_requests=Vec::new();
		for (i,b) in self.bots.iter_mut().enumerate(){
			if b.state == GridBotState::DoingNothing{
				let req = PathFindInfo{start:self.grid.convert_to_grid(b.bot.pos),end:pick_empty_spot(&self.grid),bot_index:i};
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
			update_bot(&mut b.bot);

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