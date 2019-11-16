



use crate::pathfind::*;
use crate::grid::*;
use crate::short_path::*;

use crate::axgeom::*;
use duckduckgeo::bot::*;


#[derive(Eq,PartialEq,Debug,Copy,Clone)]
enum GridBotState{
	DoingNothing,
	Thinking,
	Moving(PathDiagAdapter)
}




pub struct GridBot{
	bot:Bot,
	state:GridBotState
}
impl GridBot{
	pub fn get(&self)->&Bot{
		&self.bot
	}
}





pub struct Game{
    bot_prop:BotProp,
	grid:GridDim2D,
	bots:Vec<GridBot>,
	pathfinder:PathFinder
}


const GRID_STR:&str= "\
████████████████
█    █   █     █
█    █   █  █  █
         █  █
     █████  █
       █
█      █
█   
████████
";

impl Game{
	pub fn new()->Game{
		let pathfinder=PathFinder::new();
		let dim=Rect::new(0.0,1920.,0.0,1080.);
		let mut grid=GridDim2D{dim,inner:Grid2D::from_str(vec2(16,9),GRID_STR)};

		let bot_prop=BotProp{
            radius:Dist::new(12.0),
            collision_drag:0.003,
            collision_push:0.5,
            minimum_dis_sqr:0.0001,
            viscousity_coeff:0.03
        };

        let num_bot=1000;
        let s=dists::grid::Grid::new(dim,num_bot);
    	let bots:Vec<GridBot>=s.take(num_bot).map(|pos|{
    		let bot=Bot::new(vec2(pos.x as f32,pos.y as f32));
    		GridBot{bot,state:GridBotState::DoingNothing}
    	}).collect();

		Game{grid,bots,pathfinder,bot_prop}
	}

	
	pub fn get_wall_grid(&self)->&GridDim2D{
		&self.grid
	}

	pub fn bot_len(&self)->usize{
		self.bots.len()
	}


	pub fn get_bots(&self)->(&BotProp,&[GridBot]){
		(&self.bot_prop,&self.bots)
	}

	pub fn step(&mut self){

		let mut path_requests=Vec::new();
		for (i,b) in self.bots.iter_mut().enumerate(){
			if b.state ==GridBotState::DoingNothing{
				let start =self.grid.convert_to_grid(b.bot.pos);

				let start=if self.grid.inner.get(start){
					find_closest_empty(&self.grid.inner,start).unwrap()
				}else{
					start
				};

				let end = pick_empty_spot(&self.grid.inner).unwrap();
					
				let req = PathFindInfo{start,end,bot_index:i};
				//dbg!(req.end);
				b.state = GridBotState::Thinking;
				path_requests.push(req);

				//println!("queueing {:?}",(start,end));
				//assert!(!self.grid.inner.get(start));
				
			}
		}

		let mut results = self.pathfinder.handle_par(&self.grid.inner,path_requests);

		for res in results.drain(..){
			let b=&mut self.bots[res.info.bot_index];
			assert_eq!(b.state,GridBotState::Thinking);
			match res.path{
				Some(path)=>{
					//dbg!(b.pos,path);
					//println!("Attempting to go to {:?}",(b.pos,self.grid.convert_to_world(b.pos)));
					//println!("starting to new. current pos={:?}",(b.pos,b.bot.pos));
					let k=PathDiagAdapter::new(PathPointIter::new(res.info.start,path.iter()));

					//let _ = self.grid.inner.draw_map_and_path(k.inner);
					//println!("starting new path path={:?}",k);
					b.state=GridBotState::Moving(k);		
				},
				None=>{

					//println!("failed for {:?}",res);
					//println!("grid looks like={:?}",&self.grid.inner);
		
				}
			}
		}


		use dinotree::prelude::*;
		use axgeom;
		use axgeom::ordered_float::*;
		let bot_prop=&self.bot_prop;
            
        let mut bots:Vec<BBoxMut<NotNan<f32>,GridBot>>=create_bbox_mut(&mut self.bots,|bot|{
            bot.bot.create_bbox(bot_prop).inner_try_into().unwrap()
        });

        let mut tree=DinoTreeBuilder::new(axgeom::YAXISS,&mut bots).build_par();

        
        dinotree_alg::colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b|{
            bot_prop.collide(&mut a.inner_mut().bot,&mut b.inner_mut().bot);
        });


		for b in self.bots.iter_mut(){
			
			let target_radius=self.grid.cell_radius().x/2.0;

			match &mut b.state{
				GridBotState::Moving(ref mut pointiter)=>{

					
					if b.bot.move_to_point(self.grid.convert_to_world_center(pointiter.inner.pos()),target_radius){
						
						match pointiter.next(&self.grid.inner){
							Some(target)=>{
								//dbg!(*curr_target,target,target.into_vec(),"hit waypoint");
								//b.pos=*curr_target;
								//println!(".Attempting to go to {:?}",target);
								//println!("fo={:?}",self.grid.convert_to_world(*curr_target));
							},
							None=>{
								//b.bot.vel=vec2same(0.0);
								//println!("reached target i guess");
								b.state=GridBotState::DoingNothing;
							}
						}
					}
				},
				GridBotState::Thinking |
				GridBotState::DoingNothing=>{
					b.bot.move_to_point(b.bot.pos,target_radius);
				}
			}

			b.bot.update();
		}
	}
}