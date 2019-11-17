



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
            collision_push:0.5,
            minimum_dis_sqr:0.0001,
            viscousity_coeff:0.03
        };

        let num_bot=1000;
        let s=dists::grid::Grid::new(*dim.clone().grow(-0.1),num_bot);
    	let bots:Vec<GridBot>=s.take(num_bot).map(|pos|{
    		let bot=Bot::new(vec2(pos.x as f32,pos.y as f32));
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

			//use crate::grid::raycast::RayCaster;
			//use crate::grid::raycast::CollideCellEvent;

			
			
			let bot=&mut b.bot;
			let ray=duckduckgeo::Ray{point:bot.pos,dir:bot.vel};
			

			if let Some(caster)=RayCaster::new(&self.grid,ray){
				for a in caster{
						match self.walls.get_option(a.cell){
							Some(wall)=>{
								if a.tval<ray.dir.magnitude(){
					
									if wall{
										//bounce_with_wall(&self.grid,&self.bot_prop,bot,&a);
										break;
									}
								}
							},
							None=>{
								break;
							}
						
					}
				}
			}
			
			fn bounce_with_wall(grid_dim:&GridViewPort,bot_prop:&BotProp,bot:&mut Bot,collide:&CollideCellEvent){
				use CardDir::*;

				let current_cell=collide.cell;
				let bottom_right=collide.cell+vec2(1,1);

				let e=bot_prop.radius.dis()+0.001;
	    		let current_cell_pos=grid_dim.to_world_topleft(current_cell);
	    		let bottom_right_pos=grid_dim.to_world_topleft(bottom_right);

	    		let slow=0.5;
	    		
				match collide.dir_hit{
					L=>{
						//hitting left
 						bot.pos.x=current_cell_pos.x-e;
 						bot.vel.x=-bot.vel.x;
 						bot.vel.x*=slow;
					},
					R=>{
 						//hitting right
 						bot.pos.x=bottom_right_pos.x+e;
 						bot.vel.x=-bot.vel.x;
 						bot.vel.x*=slow;
					},
					U=>{
 						//hitting top
 						bot.pos.y=current_cell_pos.y-e;
 						bot.vel.y=-bot.vel.y;
 						bot.vel.y*=slow;
					},
					D=>{
						//hitting bottom
 						bot.pos.y=bottom_right_pos.y+e;
 						bot.vel.y=-bot.vel.y;
 						bot.vel.y*=slow;
					}
				}
			}
			/*
			fn collide_grid(bot_prop:&BotProp,bot:&mut Bot,grid_dim:&GridDim2D,walls:&Grid2D){
			    
			    use line_drawing::Bresenham;
			    let start=grid_dim.convert_to_grid(bot.pos);
			    let end=grid_dim.convert_to_grid(bot.pos+bot.vel);

			    
			    if walls.get(start){
			    	//If we are in a wall already, just quit.
			    	return;
			    }


			    let mut last=start;
			    for (x, y) in Bresenham::new((start.x, start.y), (end.x, end.y)) {
			    	let curr=vec2(x,y);
			    	if walls.get(curr){
			    		//We are hitting this bot, from this angle:
			    		let angle=curr-last;
			    		let e=bot_prop.radius.dis()+0.001;
			    		let curr_cell_pos=grid_dim.convert_to_world_topleft(curr);
			    		let last_cell_pos=grid_dim.convert_to_world_topleft(last);
			    		let slow=0.5;
			    		match angle{
			    			Vec2{x:1,y:1} |
			    			Vec2{x:1,y:0}=>{
				    			//hitting left
		 						bot.pos.x=curr_cell_pos.x-e;
		 						bot.vel.x=-bot.vel.x;
		 						bot.vel.x*=slow;
			    			},
			    			Vec2{x:-1,y:-1}|
			    			Vec2{x:-1,y:0}=>{
		 						//hitting right
		 						bot.pos.x=last_cell_pos.x+e;
		 						bot.vel.x=-bot.vel.x;
		 						bot.vel.x*=slow;
			    			},
			    			Vec2{x:-1,y:1}|
			    			Vec2{x:0,y:1}=>{
		 						//hitting top
		 						bot.pos.y=curr_cell_pos.y+e;
		 						bot.vel.y=-bot.vel.y;
		 						bot.vel.y*=slow;
			    			},
			    			Vec2{x:1,y:-1}|
			    			Vec2{x:0,y:-1}=>{
								//hitting bottom
		 						bot.pos.y=last_cell_pos.y-e;
		 						bot.vel.y=-bot.vel.y;
		 						bot.vel.y*=slow;
			    			},
			    			_=>{
			    				unreachable!()
			    			}

			    		}
			    		//We hit a wall, return.
			    		return;
			    	}
			    	last=curr;
			    }
			}


			collide_grid(&self.bot_prop,&mut b.bot,&self.grid,&self.walls);
			*/

			//let bot=&mut b.bot;
			//let ray=duckduckgeo::Ray{point:bot.pos,dir:bot.vel};
			//ray_compute_intersection_tvalue(&self.grid,ray);
			/*
			//assumes that bot is smaller than a grid cell!!!!!!
			fn handle(prop:&BotProp,bot:&mut Bot,grid:&GridDim2D){
				let cell_curr = grid.convert_to_grid(bot.pos);
 				//let cell_next = grid.convert_to_grid(bot.pos+bot.vel);
 				let cell_curr_pos = grid.convert_to_world_topleft(cell_curr);
 				//let cell_next_pos = grid.convert_to_world_topleft(cell_next);

 				let top=bot.create_bbox(prop).y.left;
				let bottom=bot.create_bbox(prop).y.right;
				let left=bot.create_bbox(prop).x.left;
				let right=bot.create_bbox(prop).x.right;
				let radius=prop.radius.dis();
				let e=radius+0.001;
				
				{//handle y axis
					let vely=bot.vel.y;
					let pp=if vely>0.0{
						bot.pos.y+vely+radius
					}else{
						bot.pos.y+vely-radius
					};
					
					let nexta=grid.inner.get(grid.convert_to_grid(vec2(left,pp)));
					let nextb=grid.inner.get(grid.convert_to_grid(vec2(right,pp)));
					
					let nn=grid.convert_to_world_topleft(grid.convert_to_grid(vec2(left,pp)));
										

					if nexta|nextb{
						if vely>=0.0{	
							bot.pos.y=nn.y-e;
						}else{					
							bot.pos.y=cell_curr_pos.y+e;
						}
						bot.vel.y=-bot.vel.y;
						bot.vel.y*=0.5;
					}
				}

				{//handle x axis
					let velx=bot.vel.x;
					let pp=if velx>0.0{
						bot.pos.x+velx+radius
					}else{
						bot.pos.x+velx-radius
					};
					
					let nexta=grid.inner.get(grid.convert_to_grid(vec2(pp,top)));
					let nextb=grid.inner.get(grid.convert_to_grid(vec2(pp,bottom)));
					
					let nn=grid.convert_to_world_topleft(grid.convert_to_grid(vec2(pp,top)));
										

					if nexta|nextb{
						if velx>=0.0{	
							bot.pos.x=nn.x-e;
						}else{					
							bot.pos.x=cell_curr_pos.x+e;
						}
						bot.vel.x=-bot.vel.x;
						bot.vel.x*=0.5;
					}
				}
			}
			*/

			//handle(&self.bot_prop,&mut b.bot,&self.grid);
			
			/*
			let curr_in_wall=self.grid.inner.get(self.grid.convert_to_grid(b.bot.pos));
			let next_in_wall=self.grid.inner.get(self.grid.convert_to_grid(b.bot.pos+b.bot.vel));
			if !curr_in_wall && next_in_wall{
				//we are about to move into a wall.
				let cell_curr = self.grid.convert_to_grid(b.bot.pos);
 				let cell_next = self.grid.convert_to_grid(b.bot.pos+b.bot.vel);

 				let cell_curr_pos = self.grid.convert_to_world_topleft(cell_curr);
 				let cell_next_pos = self.grid.convert_to_world_topleft(cell_next);


 				let diff=(cell_next-cell_curr);

 				
 				let arr = if diff.abs().x+diff.abs().y==2{
 					let k=diff.split_into_components();
 					vec!(k[0],k[1])
 				}else{
 					vec!(diff)
 				};

 				let e=0.001;
 				for a in arr.iter(){
	 				match a{
	 					Vec2{x:-1,y:0}=>{
	 						//hitting left
	 						b.bot.pos.x=cell_curr_pos.x+e;
	 						b.bot.vel.x=-b.bot.vel.x;
	 					},
	 					Vec2{x:1,y:0}=>{
	 						//hitting right
	 						b.bot.pos.x=cell_next_pos.x-e;
	 						b.bot.vel.x=-b.bot.vel.x;
	 					},
	 					Vec2{x:0,y:-1}=>{
	 						//hitting top
	 						b.bot.pos.y=cell_curr_pos.y+e;
	 						b.bot.vel.y=-b.bot.vel.y;
	 					},
	 					Vec2{x:0,y:1}=>{
	 						//hitting bottom
	 						b.bot.pos.y=cell_next_pos.y-e;
	 						b.bot.vel.y=-b.bot.vel.y;
	 					},
	 					_=>{
	 						unreachable!("{:?}",a);
	 					}
	 				}
 				}

 				let curr_in_wall=self.grid.inner.get(self.grid.convert_to_grid(b.bot.pos));
				let next_in_wall=self.grid.inner.get(self.grid.convert_to_grid(b.bot.pos+b.bot.vel));
				assert!(!curr_in_wall && !next_in_wall);
			}
			*/

			b.bot.update();


		}
	}
}