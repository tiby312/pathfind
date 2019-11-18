



use crate::pathfind::*;
use crate::short_path::*;

use crate::axgeom::*;
use duckduckgeo::bot::*;
use duckduckgeo::grid::*;
use duckduckgeo::grid::raycast::*;
//use duckduckgeo::grid::CardDir;

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
	grid:GridViewPort,
	walls:Grid2D,
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
		let grid_dim=vec2(16,9);
		let mut grid=GridViewPort{origin:vec2(0.0,0.0),spacing:vec2(1920./grid_dim.x as f32,1080./grid_dim.y as f32)};

		let walls=Grid2D::from_str(grid_dim,GRID_STR);

		let bot_prop=BotProp{
            radius:Dist::new(12.0),
            collision_drag:0.003,
            collision_push:0.1,
            minimum_dis_sqr:0.0001,
            viscousity_coeff:0.03
        };

        let num_bot=1000;
        let s=dists::grid::Grid::new(*dim.clone().grow(-0.1),num_bot);
    	let bots:Vec<GridBot>=s.take(num_bot).map(|pos|{
    		let mut bot=Bot::new(vec2(pos.x as f32,pos.y as f32));
    		//bot.vel.y=-1.0;
    		//bot.vel.x=1.0;
    		GridBot{bot,state:GridBotState::DoingNothing}
    	}).collect();

		Game{grid,walls,bots,pathfinder,bot_prop}
	}

	
	pub fn get_wall_grid(&self)->(&GridViewPort,&Grid2D){
		(&self.grid,&self.walls)
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
				//dbg!(b.bot.pos);
				let start =self.grid.to_grid(b.bot.pos);

				let start =match self.walls.get_option(start){
					None=>{
						find_closest_empty(&self.walls,start).unwrap()
					},
					Some(walls)=>{
						if walls{
							find_closest_empty(&self.walls,start).unwrap()
						}else{
							start
						}
					}
				};

				let end = pick_empty_spot(&self.walls).unwrap();
					
				let req = PathFindInfo{start,end,bot_index:i};
				//dbg!(req.end);
				b.state = GridBotState::Thinking;
				path_requests.push(req);

				//println!("queueing {:?}",(start,end));
				//assert!(!self.grid.inner.get(start));
				
			}
		}

		let mut results = self.pathfinder.handle_par(&self.walls,path_requests);

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
			

			let target_radius=self.grid.cell_radius().x;
			
			match &mut b.state{
				GridBotState::Moving(ref mut pointiter)=>{

					
					if b.bot.move_to_point(self.grid.to_world_center(pointiter.inner.pos()),target_radius){
						
						match pointiter.next(&self.walls){
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
			


			//Get square to display the 4 ray casts.
			//Confirm you get different values of tval for each.
			//




			let mut skip_update=false;
			let bot=&mut b.bot;
			match self.walls.get_option(self.grid.to_grid(bot.pos)){
				Some(walls)=>{
					if !walls{
						if bot.vel.magnitude2()>0.0{
							
							
							let rect= bot.create_bbox(&self.bot_prop);
							let corners=[
								vec2(rect.x.left,rect.y.left),
								vec2(rect.x.left,rect.y.right),
								vec2(rect.x.right,rect.y.left),
								vec2(rect.x.right,rect.y.right)
							];

							let mut results=Vec::new();
							for &corner in corners.iter(){
							
								let a=cast_ray(&self.grid,&self.walls,corner,bot.vel.normalize_to(1.0),bot.vel.magnitude());	
								if let Some(a)=a{
									//bounce_with_wall(&self.grid,&self.bot_prop,bot,&a);
									results.push((corner,a));
								}
							}

							/*
							if !results.is_empty(){
								dbg!(&results);
							}
							*/

							match results.iter().min_by(|a,b|a.1.tval.partial_cmp(&b.1.tval).unwrap()){
								Some(&(corner,a))=>{
									let corner_diff=corner-bot.pos;
									skip_update=true;
									bounce_with_wall(&self.grid,&self.bot_prop,bot,corner_diff,&a);
								},
								None=>{

								}
							}
							
							
							
							
						}
					}
				},
				None=>{

				}
			}
			
			//b.bot.pos+=b.bot.vel;
			//b.bot.vel+=b.bot.acc;
			//b.bot.acc=vec2same(0.);
			if b.bot.vel.magnitude()>1.0{
				b.bot.vel=b.bot.vel.normalize_to(1.0);
			}
			b.bot.update();
			

			

		}
	}
}





fn cast_ray(grid:&GridViewPort,walls:&Grid2D,point:Vec2<WorldNum>,dir:Vec2<WorldNum>,max_tval:WorldNum)->Option<CollideCellEvent>{

	let ray=duckduckgeo::Ray{point,dir};
	
	match RayCaster::new(grid,ray){
		Some(caster)=>{
			for a in caster{
				if a.tval<=max_tval{				
					match walls.get_option(a.cell){
						Some(wall)=>{
							if wall{
								return Some(a);
							}		
						},
						None=>{
							return None; //We've ray casted off the wall grid.
						}		
					}
				}else{
					return None;
				}
			}
		},
		None=>{
			panic!("failed to make ray caster {:?}",ray);
		}
	}
	None
}

fn bounce_with_wall(grid_dim:&GridViewPort,bot_prop:&BotProp,bot:&mut Bot,corner_diff:Vec2<WorldNum>,collide:&CollideCellEvent){


	use CardDir::*;

	let current_cell=collide.cell;
	let bottom_right=collide.cell+vec2(1,1);

	let e=bot_prop.radius.dis()+0.001;
	let current_cell_pos=grid_dim.to_world_topleft(current_cell);
	let bottom_right_pos=grid_dim.to_world_topleft(bottom_right);

	let slow=0.2;
	
	match collide.dir_hit{
		L=>{
				bot.pos.x=current_cell_pos.x-corner_diff.x;
				bot.vel.x=-bot.vel.x;
				bot.vel.x*=slow;
		},
		R=>{
				
				bot.pos.x=bottom_right_pos.x-corner_diff.x;
				bot.vel.x=-bot.vel.x;
				bot.vel.x*=slow;
		},
		U=>{
				bot.pos.y=current_cell_pos.y-corner_diff.y;
				bot.vel.y=-bot.vel.y;
				bot.vel.y*=slow;
		},
		D=>{
				bot.pos.y=bottom_right_pos.y-corner_diff.y;
				bot.vel.y=-bot.vel.y;
				bot.vel.y*=slow;
		}
	}
}