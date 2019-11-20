



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



#[derive(Copy,Clone,Debug)]
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
	pathfinder:PathFinder,
	bots_debug:Vec<GridBot>
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
const GRID_STR2:&str= "\
████████████████████████████████
█    █   █     █               █
█    █   █  █  █ █    █        █ 
█        █  █    █   █     █   █
█    █████  █  █ █    █    █   █
█      █       █ █   ██    █   █
█      ███████ █ ██████    █   █
█            █ █ █             █
████████     █ █ █ █           █
█          █████ ███   █████   █
█                              █
███████████████   ███████████  █
█            █    █   █    █   █
█       █   █    ██ █ █ █  █   █
█       █  █   ██   █ █ █  █   █
█       █ █         █   █      █
█       █     ██████████████████
████████████████████████████████
";
impl Game{
	pub fn new()->Game{
		let pathfinder=PathFinder::new();
		let dim=Rect::new(0.0,1920.,0.0,1080.);
		let grid_dim=vec2(16,9)*2;
		let mut grid=GridViewPort{origin:vec2(0.0,0.0),spacing:vec2(1920./grid_dim.x as f32,1080./grid_dim.y as f32)};

		let walls=Grid2D::from_str(grid_dim,GRID_STR2);

		let bot_prop=BotProp{
            radius:Dist::new(12.0),
            collision_drag:0.003,
            collision_push:0.02,
            minimum_dis_sqr:0.0001,
            viscousity_coeff:0.03
        };

        let num_bot=5000;
        let s=dists::grid::Grid::new(*dim.clone().grow(-0.1),num_bot);
    	let mut bots:Vec<GridBot>=s.take(num_bot).map(|pos|{
    		let mut bot=Bot::new(vec2(pos.x as f32,pos.y as f32));
    		//bot.vel.y=1.0;
    		//bot.vel.x=1.0;
    		GridBot{bot,state:GridBotState::DoingNothing}
    	}).collect();


    	for b in bots.iter_mut(){
    		let bot=&mut b.bot;
    		let prop=&bot_prop;

    		if !assert_bot_is_not_touching_wall(bot,prop,&grid,&walls){
				bot.pos=grid.to_world_center(find_closest_empty(&walls,grid.to_grid(bot.pos)).unwrap());
			}	
    	}
    	

		Game{grid,walls,bots,pathfinder,bot_prop,bots_debug:Vec::new()}
	}

	
	pub fn get_wall_grid(&self)->(&GridViewPort,&Grid2D){
		(&self.grid,&self.walls)
	}

	pub fn bot_len(&self)->usize{
		self.bots.len()
	}


	pub fn get_bots(&self)->(&BotProp,&[GridBot]){
		//(&self.bot_prop,&self.bots_debug)
		(&self.bot_prop,&self.bots)
	}

	pub fn step(&mut self){
		/*
		for a in self.bots.iter(){
        	assert!(assert_bot_is_not_touching_wall(&a.bot,&self.bot_prop,&self.grid,&self.walls));
        }*/

        
		let mut path_requests=Vec::new();
		for (i,b) in self.bots.iter_mut().enumerate(){
			if b.state ==GridBotState::DoingNothing{
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
				b.state = GridBotState::Thinking;
				path_requests.push(req);

				
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
        
		/*
        for a in self.bots.iter(){
        	assert!(assert_bot_is_not_touching_wall(&a.bot,&self.bot_prop,&self.grid,&self.walls));
        }*/
        

        let mut bots:Vec<BBoxMut<NotNan<f32>,GridBot>>=create_bbox_mut(&mut self.bots,|bot|{
            bot.bot.create_bbox(bot_prop).inner_try_into().unwrap()
        });

        let mut tree=DinoTreeBuilder::new(axgeom::YAXISS,&mut bots).build_par();

        
        dinotree_alg::colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b|{
            bot_prop.collide(&mut a.inner_mut().bot,&mut b.inner_mut().bot);
        });
		
       


		for b in self.bots.iter_mut(){
			let grid_bot_save=*b;
			let state=&mut b.state;
			let bot=&mut b.bot;

			
			let target_radius=self.grid.cell_radius().x;
			//assert!(assert_bot_is_not_touching_wall(&bot,&self.bot_prop,&self.grid,&self.walls));

			match state{
				GridBotState::Moving(ref mut pointiter)=>{

					
					if bot.move_to_point(self.grid.to_world_center(pointiter.inner.pos()),target_radius){
						
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
								*state=GridBotState::DoingNothing;
							}
						}
					}
				},
				GridBotState::Thinking |
				GridBotState::DoingNothing=>{
					bot.move_to_point(bot.pos,target_radius);
				}
			}
			
			//assert!(assert_bot_is_not_touching_wall(&bot,&self.bot_prop,&self.grid,&self.walls));

			
			if bot.acc.magnitude()>20.0{
				bot.acc.normalize_to(20.0);
			}

			//Get square to display the 4 ray casts.
			//Confirm you get different values of tval for each.
			
			bot.vel+=bot.acc;
			bot.acc=vec2same(0.0);

			//truncate speed
			/*
			if b.bot.vel.magnitude()>1.0{
				b.bot.vel=b.bot.vel.normalize_to(1.0);
			}
			*/

			


			//This is first set equal to the velocity.
			//as we collide with rectangles, we subtract from this vector.
			//then we should skip the step where we apply velocity to the position.
			//since we have been doing that.
			let mut amount_left_to_move=bot.vel.magnitude();


			//let mut amount_left_to_move_backup=bot.vel.magnitude();
			let vel_backup=bot.vel;
			//let mut results_backup=Vec::new();

			//assert!(assert_bot_is_not_touching_wall(&bot,&self.bot_prop,&self.grid,&self.walls));

			let mut final_dir=None;
			//Do twice incase we bounce off of one wall into another.
			for _ in 0..2{ //TODO how to fix this break out of loop if amount_left_to_move is almost zero.
				if !assert_bot_is_not_touching_wall_center(&bot,&self.bot_prop,&self.grid,&self.walls){
					bot.pos=self.grid.to_world_center(find_closest_empty(&self.walls,self.grid.to_grid(bot.pos)).unwrap());
				}
				else
				{
					if bot.vel.magnitude2()>0.0{ //don't need to check anything if we are not moving.
						
						match RayStorm::new(bot,&self.bot_prop).find_nearest_collision(&self.grid,&self.walls,amount_left_to_move)
						{
							Some((corner,a))=>{
								let corner_diff=corner-bot.pos;
								
								let tval=bounce_with_wall(&self.grid,&self.bot_prop,bot,corner_diff,&a);
								
								assert!(tval.is_finite());
								assert_le!(tval,amount_left_to_move);
				
								amount_left_to_move-=tval;
								//amount_left_to_move=amount_left_to_move.max(0.0);
								
								final_dir=Some(a.dir_hit);
							},
							None=>{

							}
						}
					}
				}
			}
			//assert!(assert_bot_is_not_touching_wall(&bot,&self.bot_prop,&self.grid,&self.walls));


			
			let bot_save=*bot;
			if bot.vel.magnitude2()>0.0{		
				assert_ge!(amount_left_to_move,0.0);
				assert!(amount_left_to_move.is_finite());

				let nv=bot.vel.normalize_to(1.0)*amount_left_to_move;
				
				if let Some(dir)=final_dir{
					use CardDir::*;
					match dir{
						U=>{
							assert!(nv.y<=0.0);
						},
						D=>{
							assert!(nv.y>=0.0);
						},
						L=>{
							assert!(nv.x<=0.0);
						},
						R=>{
							assert!(nv.x>=0.0);
						}

					}
				}
				
				//assert!(assert_bot_is_not_touching_wall(&bot,&self.bot_prop,&self.grid,&self.walls));

				bot.pos+=nv;
				
				//assert!(assert_bot_is_not_touching_wall(&bot,&self.bot_prop,&self.grid,&self.walls));


				
				if !assert_bot_is_not_touching_wall(&bot,&self.bot_prop,&self.grid,&self.walls)
				{
					dbg!("GRIDDDDD",self.grid.to_grid(bot.pos));
					let mut prop=self.bot_prop;
					prop.radius=Dist::new(prop.radius.dis());
					let ray=RayStorm::new(&bot_save,&self.bot_prop);
					let a=ray.find_nearest_collision(&self.grid,&self.walls,100.0);
							
					//let a=cast_ray(&self.grid,&self.walls,bot.pos,bot.vel.normalize_to(1.0),100.0);	
					dbg!((self.grid.to_grid_mod(grid_bot_save.bot.pos),self.grid.to_grid_mod(bot.pos),nv,vel_backup,&bot,amount_left_to_move,a));

					//panic!("fail");
					let mut p=vec2(0.0,0.0);
					for a in 0..50{
						
						let mut k=grid_bot_save;
						k.bot.pos+=nv*(a as f32);
						self.bots_debug.push(k);
						
					}
					//bot.pos=bot_save.pos;
				}

			}else{
				
			}



							
								
			//assert!(assert_bot_is_not_touching_wall(&bot,&self.bot_prop,&self.grid,&self.walls));

			assert!(b.bot.pos.x.is_finite() |b.bot.pos.y.is_finite() );

			
			

			

		}
	}
}


fn assert_bot_is_not_touching_wall_center(bot:&Bot,bot_prop:&BotProp,grid:&GridViewPort,walls:&Grid2D)->bool{
	let mut rect= bot.create_bbox(bot_prop);
	let corners=[
		vec2(bot.pos.x,bot.pos.y),
	];



	for &a in corners.iter(){
		let mut a=a;
		let k = grid.to_grid_mod(a);

		
		let mut l=grid.to_grid(a);
		
		/*
		if k.x==0.0{
			
			//println!("XXXX");
			l.x-=1;
		}
		if k.y==0.0{
			//println!("YYYY");
			l.y-=1;
		}
		*/
		
		//dbg!("GRIDDDDD",k);
		match walls.get_option(l){
			Some(walls)=>{
				if walls{
					return false;
				}
			},
			None=>{

			}
		}
	}
	return true;
}
fn assert_bot_is_not_touching_wall(bot:&Bot,bot_prop:&BotProp,grid:&GridViewPort,walls:&Grid2D)->bool{
	let mut rect= bot.create_bbox(bot_prop);
	let corners=[
		//vec2(bot.pos.x,bot.pos.y),
		vec2(rect.x.left,rect.y.left),
		vec2(rect.x.left,rect.y.right),
		vec2(rect.x.right,rect.y.left),
		vec2(rect.x.right,rect.y.right)
		
	];

	for &a in corners.iter(){
		match walls.get_option(grid.to_grid(a)){
			Some(walls)=>{
				if walls{
					return false;
				}
			},
			None=>{

			}
		}
	}
	return true;
}


#[derive(Copy,Clone,Debug)]
struct RayStorm<'a>{
	inner:[Vec2<WorldNum>;4],
	bot:&'a Bot
}
impl<'a> RayStorm<'a>{
	fn new(bot:&'a Bot,prop:&BotProp)->RayStorm<'a>{
		let rect= *bot.create_bbox(prop).grow(0.1);
		let inner=[
			vec2(rect.x.left,rect.y.left),
			vec2(rect.x.left,rect.y.right),
			vec2(rect.x.right,rect.y.left),
			vec2(rect.x.right,rect.y.right)
		];
		RayStorm{inner,bot}
	}

	fn find_nearest_collision(&self,grid:&GridViewPort,walls:&Grid2D,amount_left_to_move:WorldNum)->Option<(Vec2<WorldNum>,CollideCellEvent)>{
		let mut results=Vec::new();
		for &corner in self.inner.iter(){
			let a=cast_ray(grid,walls,corner,self.bot.vel.normalize_to(1.0),amount_left_to_move);	
			if let Some(a)=a{
				//assert!(a.tval.is_finite());
				assert_le!(a.tval,amount_left_to_move);
				//dbg!(a.tval);
				results.push((corner,a));
			}
		}

		match results.iter().min_by(|a,b|a.1.tval.partial_cmp(&b.1.tval).unwrap()){
			Some(&(corner,a))=>{
				//let corner_diff=corner-bot.pos;
				
				Some((corner,a))
			},
			None=>{
				None
			}
		}
	}
}


#[derive(Copy,Clone,Debug)]
struct RayStorm2<'a>{
	inner:[Vec2<WorldNum>;1],
	bot:&'a Bot
}
impl<'a> RayStorm2<'a>{
	fn new(bot:&'a Bot,prop:&BotProp)->RayStorm2<'a>{
		let rect= *bot.create_bbox(prop).grow(0.1);
		let inner=[
			vec2(bot.pos.x,bot.pos.y),
		];
		RayStorm2{inner,bot}
	}

	fn find_nearest_collision(&self,grid:&GridViewPort,walls:&Grid2D,amount_left_to_move:WorldNum)->Option<(Vec2<WorldNum>,CollideCellEvent)>{
		let mut results=Vec::new();
		for &corner in self.inner.iter(){
			let a=cast_ray(grid,walls,corner,self.bot.vel.normalize_to(1.0),amount_left_to_move);	
			if let Some(a)=a{
				//assert!(a.tval.is_finite());
				assert_le!(a.tval,amount_left_to_move);
				//dbg!(a.tval);
				results.push((corner,a));
			}
		}

		match results.iter().min_by(|a,b|a.1.tval.partial_cmp(&b.1.tval).unwrap()){
			Some(&(corner,a))=>{
				//let corner_diff=corner-bot.pos;
				
				Some((corner,a))
			},
			None=>{
				None
			}
		}
	}
}



fn cast_ray(grid:&GridViewPort,walls:&Grid2D,point:Vec2<WorldNum>,dir:Vec2<WorldNum>,max_tval:WorldNum)->Option<CollideCellEvent>{

	let ray=duckduckgeo::Ray{point,dir};
	
	match RayCaster::new(grid,ray){
		Some(caster)=>{
			for mut a in caster{
				if a.tval<=max_tval{				
					match walls.get_option(a.cell){
						Some(wall)=>{
							if wall{
								/*
								{//check that we are not repeling into another wall. (corner case)
									use CardDir::*;
									let k=match a.dir_hit{
										L=>{
											vec2(-1,0)
										},
										R=>{
											vec2(1,0)
										},
										U=>{
											vec2(0,-1)
										},
										D=>{
											vec2(0,1)
										}
									};

									//TODO use this
									//k.cross_product(ray.dir)

									if let Some(wall) = walls.get_option(a.cell+k){
										if wall{
											dbg!("YOOOO");
											use CardDir::*;
											let k=match a.dir_hit{
												L=>{
													if dir.y>0.0{
														U
													}else{
														D
													}
												},
												R=>{
													if dir.y>0.0{
														U
													}else{
														D
													}	
												},
												U=>{
													if dir.x>0.0{
														L
													}else{
														R
													}
												},
												D=>{
													if dir.x>0.0{
														L
													}else{
														R
													}
												}
											};
											a.dir_hit=k;
											return Some(a)
										}
									}
								}
								*/
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
	unreachable!()
}

fn bounce_with_wall(grid_dim:&GridViewPort,bot_prop:&BotProp,bot:&mut Bot,corner_diff:Vec2<WorldNum>,collide:&CollideCellEvent)->WorldNum{


	use CardDir::*;

	let current_cell=collide.cell;
	let bottom_right=collide.cell+vec2(1,1);

	//let e=0.1;

	let shift=0.001;
	//TODO important to add a diff here!!!!
	let current_cell_pos=grid_dim.to_world_topleft(current_cell)-vec2same(shift);
	let bottom_right_pos=grid_dim.to_world_topleft(bottom_right)+vec2same(shift);

	let slow=0.2;
	
	let va=bot.vel.normalize_to(1.0);
	let mut tval;
	match collide.dir_hit{
		L=>{
				let position_at_contact_x=current_cell_pos.x-corner_diff.x;
				tval=(position_at_contact_x-bot.pos.x)/va.x;

				bot.pos.x=position_at_contact_x;
				bot.vel.x=-bot.vel.x;
				//bot.vel.x*=slow;
		},
		R=>{
				let position_at_contact_x=bottom_right_pos.x-corner_diff.x;
				tval=(position_at_contact_x-bot.pos.x)/va.x;

				bot.pos.x=position_at_contact_x;
				bot.vel.x=-bot.vel.x;
				//bot.vel.x*=slow;
		},
		U=>{
				let position_at_contact_y=current_cell_pos.y-corner_diff.y;
				tval=(position_at_contact_y-bot.pos.y)/va.y;

				bot.pos.y=position_at_contact_y;
				bot.vel.y=-bot.vel.y;
				//bot.vel.y*=slow;
		},
		D=>{
				let position_at_contact_y=bottom_right_pos.y-corner_diff.y;
				tval=(position_at_contact_y-bot.pos.y)/va.y;

				bot.pos.y=position_at_contact_y;
				bot.vel.y=-bot.vel.y;
				//bot.vel.y*=slow;
		}
	}
	tval
}