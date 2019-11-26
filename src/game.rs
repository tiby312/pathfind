



use crate::pathfind::*;
use crate::short_path::*;

use crate::axgeom::*;
use duckduckgeo::bot::*;
use duckduckgeo::grid::*;
use duckduckgeo::grid::raycast::*;
//use duckduckgeo::grid::CardDir;

#[derive(Eq,PartialEq,Debug,Copy,Clone)]
pub enum GridBotState{
	DoingNothing,
	Thinking,
	Moving(PathPointIter,usize) //Time since it last hit something.
}



#[derive(Copy,Clone,Debug)]
pub struct GridBot{
	pub bot:Bot,
	pub state:GridBotState
}





pub struct Game{
    bot_prop:BotProp,
	grid:GridViewPort,
	walls:Grid2D,
	bots:Vec<GridBot>,
	pathfinder:PathFinder,
}


mod maps{
	use super::*;
	

	pub const GRID_STR1:Map<'static>= Map{dim:vec2(16,9),str:"\
████████████████
█    █   █     █
█    █   █  █  █
█        █  █  █
█    █████  █  █
█      █       █
█      █       █
█              █
████████████████
"};

	pub const GRID_STR2:Map<'static>=Map{dim:vec2(16*2,9*2),str:"\
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
"};

	pub const GRID_STR3:Map<'static>= Map{dim:vec2(16*4,9*4),str:"\
████████████████████████████████████████████████████████████████
█    █   █     █                  █                            █
█    █   █  █  █ █    █           █            █               █
█        █  █    █   █     █   █  █    █                 █     █
█    █████  █  █ █    █    █   █  █                            █
█      █       █ █   ██    █   █              ████████         █
█      ███████ █ ██████    █   █              ████████         █
█            █ █ █             █    █         ████████     █   █
████████     █ █ █ █           █         ███████               █
█          █████ ███   █████   █         ███████         █     █
█                              █   █                           █
███████████████   ███████████  █                      █        █
█            █    █   █    █    █                              █
█       █   █    ██ █ █ █  █    █      █        █              █
█       █  █   ██   █ █ █  █    █                              █
█       █ █         █   █      █          █                    █
█       █     ████████████████ █                               █
█████████  ██████████████████  ████████  ███   █████████████████
█                           █   █          █                   █
█                            █   █         █         █         █
█     █████████        █       █           █                   █
█        ██████████            █           ██████         █    █
█            ████████                      █       █  █        █
█                            █             █      █  █         █
█                           █              █     █  █          █
██████████████    ██████████   ████  ████████████  █████████████
█            █                       ███       █  █            █
█    █       █     █                 ███      █  █       █     █
█            █          █     █              █  █              █
█        █   █                   █          █  █     █         █
█            █                             █  █                █
█            █    █████████████     █     █  █                 █
█          █ █    █████████████          █  █                  █
█         █  █    █████████████            █              █    █
█         █       █████████████                                █
████████████████████████████████████████████████████████████████
"};
}



fn create_bbox_wall(bot:&Bot,bot_prop:&BotProp)->Rect<WorldNum>{
	let radius=bot_prop.radius.dis()*0.5;
	Rect::from_point(bot.pos,vec2same(radius))
}

impl Game{
	pub fn new()->Game{
		let pathfinder=PathFinder::new();
		let dim=Rect::new(0.0,1920.,0.0,1080.);
		let map=maps::GRID_STR3;
		let grid_dim=map.dim;

		assert_eq!(1920./grid_dim.x as f32,1080./grid_dim.y as f32);

		let grid=GridViewPort{origin:vec2(0.0,0.0),spacing:1920./grid_dim.x as f32};

		let walls=Grid2D::from_str(map);

		let bot_prop=BotProp{
            radius:Dist::new(8.0),
            collision_drag:0.001,
            collision_push:0.01,
            minimum_dis_sqr:0.0001,
            viscousity_coeff:0.03
        };


        dbg!(&grid);

        let num_bot=5000;
        let s=dists::grid::Grid::new(*dim.clone().grow(-0.1),num_bot);
    	let mut bots:Vec<GridBot>=s.take(num_bot).map(|pos|{
    		let bot=Bot::new(vec2(pos.x as f32,pos.y as f32));
    		//bot.pos=vec2(86.70752,647.98);
    		//bot.vel=vec2(-0.03991765,0.22951305);
    		GridBot{bot,state:GridBotState::DoingNothing}
    	}).collect();


    	for b in bots.iter_mut(){
    		let bot=&mut b.bot;
    		let prop=&bot_prop;

    		if rect_is_touching_wall(&bot.create_bbox(prop),&grid,&walls){
				bot.pos=grid.to_world_center(find_closest_empty(&walls,grid.to_grid(bot.pos)).unwrap());
				assert!(!rect_is_touching_wall(&bot.create_bbox(prop),&grid,&walls));
			}	
    	}
    	

		Game{grid,walls,bots,pathfinder,bot_prop}
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

	//main loop!!!!
	pub fn step(&mut self){
				
		handle_path_assignment(self);		
		handle_bot_bot_collision(self);

		for b in self.bots.iter_mut(){
			handle_bot_steering(b,&self.pathfinder,&self.grid,&self.walls);
			handle_bot_moving(b,&self.bot_prop,&self.pathfinder,&self.grid,&self.walls);
		}
	}
}



fn rect_is_touching_wall(rect:&Rect<WorldNum>,grid:&GridViewPort,walls:&Grid2D)->bool{
	let corners=[
		vec2(rect.x.left,rect.y.left),
		vec2(rect.x.left,rect.y.right),
		vec2(rect.x.right,rect.y.left),
		vec2(rect.x.right,rect.y.right)
	];

	for (_i,&a) in corners.iter().enumerate(){
		let _k = grid.to_grid_mod(a);

		match walls.get_option(grid.to_grid(a)){
			Some(walls)=>{
				if walls{
					return true;
				}
			},
			None=>{

			}
		}
	}
	return false;
}



/*
pub fn ray_hits_point(radius:WorldNum,start:Vec2<WorldNum>,end:Vec2<WorldNum>,grid:&GridViewPort,walls:&Grid2D)->bool{
	let r=RayStorm::new(Rect::from_point(start,vec2same(radius)));

	let dir=(end-start);
	
	let length=dir.magnitude();
	match r.find_nearest_collision(grid,walls,dir.normalize_to(1.0),length){
		Some(k)=>{
			false
		},
		None=>{
			true
		}
	}
}
*/

#[derive(Copy,Clone,Debug)]
struct RayStorm{
	inner:[Vec2<WorldNum>;4]
}

#[derive(Debug)]
struct BBoxCollideCellEvent{
	inner:CollideCellEvent,
	corner:Vec2<WorldNum> //offset
}

impl RayStorm{
	fn new(rect:Rect<WorldNum>)->RayStorm{
	
		let inner=[
			vec2(rect.x.left,rect.y.left),
			vec2(rect.x.left,rect.y.right),
			vec2(rect.x.right,rect.y.left),
			vec2(rect.x.right,rect.y.right)
		];
		RayStorm{inner}
	}

	fn find_nearest_collision(&self,grid:&GridViewPort,walls:&Grid2D,dir:Vec2<WorldNum>,amount_left_to_move:WorldNum)->Option<BBoxCollideCellEvent>{
		let mut results=Vec::new();
		for &corner in self.inner.iter(){
			let a=cast_ray(grid,walls,corner,dir,amount_left_to_move);	
			if let Some(a)=a{
				assert_le!(a.tval,amount_left_to_move);
				results.push((corner,a));
			}
		}

		match results.iter().min_by(|a,b|a.1.tval.partial_cmp(&b.1.tval).unwrap()){
			Some(&(corner,a))=>{
				Some(BBoxCollideCellEvent{corner,inner:a})
			},
			None=>{
				None
			}
		}
	}
}





fn cast_ray(grid:&GridViewPort,walls:&Grid2D,point:Vec2<WorldNum>,dir:Vec2<WorldNum>,max_tval:WorldNum)->Option<CollideCellEvent>{

	let ray=duckduckgeo::Ray{point,dir};
	
	let caster= RayCaster::new(grid,ray);
	

	if let Some(wall)=walls.get_option(grid.to_grid(point)){
		let grid_mod=grid.to_grid_mod(point);
		assert!(!wall,"We are starting the raycast inside a wall! point:{:?} grid mod:{:?}",point,grid_mod);
	}


	for a in caster{
		if a.tval<=max_tval{				
			match walls.get_option(a.cell){
				Some(wall)=>{
					if wall{								
						
						if let Some(wall) = walls.get_option(a.cell+a.dir_hit.into_vec()){
							if wall{
								panic!("dont know how to handle this case")
							}
						}
					
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
	unreachable!()
}




fn handle_bot_bot_collision(game:&mut Game){
	use dinotree::prelude::*;
	use axgeom::ordered_float::*;
    
	let bot_prop=&game.bot_prop;
    let mut bots:Vec<BBoxMut<NotNan<f32>,GridBot>>=create_bbox_mut(&mut game.bots,|bot|{
        bot.bot.create_bbox(bot_prop).inner_try_into().unwrap()
    });

    let mut tree=DinoTreeBuilder::new(axgeom::YAXISS,&mut bots).build_par();

    
    dinotree_alg::colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b|{
        bot_prop.collide(&mut a.inner_mut().bot,&mut b.inner_mut().bot);
    });
}


fn handle_path_assignment(game:&mut Game){
	let mut path_requests=Vec::new();
	for (i,b) in game.bots.iter_mut().enumerate(){
		if b.state ==GridBotState::DoingNothing{
			let start =game.grid.to_grid(b.bot.pos);

			let start =match game.walls.get_option(start){
				None=>{
					find_closest_empty(&game.walls,start).unwrap()
				},
				Some(walls)=>{
					if walls{
						find_closest_empty(&game.walls,start).unwrap()
					}else{
						start
					}
				}
			};

			let end = pick_empty_spot(&game.walls).unwrap();
				
			let req = PathFindInfo{start,end,bot_index:i};
			b.state = GridBotState::Thinking;
			path_requests.push(req);

			
		}
	}

	let mut results = game.pathfinder.handle_par(&game.walls,path_requests);

	for res in results.drain(..){
		let b=&mut game.bots[res.info.bot_index];
		assert_eq!(b.state,GridBotState::Thinking);
		match res.path{
			Some(path)=>{
				//dbg!(b.pos,path);
				//println!("Attempting to go to {:?}",(b.pos,self.grid.convert_to_world(b.pos)));
				//println!("starting to new. current pos={:?}",(b.pos,b.bot.pos));
				let k=PathPointIter::new(res.info.start,path.iter());

				//let _ = self.grid.inner.draw_map_and_path(k.inner);
				//println!("starting new path path={:?}",k);
				b.state=GridBotState::Moving(k,game.pathfinder.get_time());		
			},
			None=>{

				//println!("failed for {:?}",res);
				//println!("grid looks like={:?}",&self.grid.inner);
	
			}
		}
	}
}


fn handle_bot_steering(b:&mut GridBot,pathfinder:&PathFinder,grid:&GridViewPort,walls:&Grid2D){
	let _grid_bot_save=*b;
	let state=&mut b.state;
	let bot=&mut b.bot;

	
	let target_radius=grid.cell_radius()*0.4;
	//assert!(assert_bot_is_not_touching_wall(&bot,&self.bot_prop,&self.grid,&self.walls));


	/*
	//if we get pushed too far away from our spot, just recalculate.
	match state{
		GridBotState::Moving(ref mut pointiter,time)=>{
			let a=grid.to_grid(bot.pos);
			if (a-pointiter.pos()).magnitude2()>2{
				*state=GridBotState::DoingNothing;
			}
		},
		_=>{}
	}
	*/

	match state{
		GridBotState::Moving(ref mut pointiter,time)=>{


			match pointiter.peek(){
				Some(next)=>{
					let offset=(grid.to_world_center(next)-bot.pos);
					let max_tval=offset.magnitude();
					//let max_tval=grid.spacing.x*0.7;//*(2.0_f32.sqrt()+0.01);
			
					match cast_ray(grid,walls,bot.pos,offset.normalize_to(1.0),max_tval){
						Some(hit)=>{

							let offset=(grid.to_world_center(pointiter.pos())-bot.pos);
							let max_tval=offset.magnitude();
							match cast_ray(grid,walls,bot.pos,offset.normalize_to(1.0),max_tval){
								Some(hit)=>{
									//We can't even see our last position.
									//just try and go to the center of the cell we're in. 
									//maybe that will help us.
									let gp=grid.to_grid(bot.pos);

									//first check that we're not in a wall
									if let Some(wall)=walls.get_option(gp){
										assert!(!wall);
									}

									let _ = bot.move_to_point(grid.to_world_center(gp),target_radius);
								},
								None=>{
									//We can't see our target, but we can see our last target.
									//try going to our last target.
									let _ = bot.move_to_point(grid.to_world_center(pointiter.pos()),target_radius);		
								}
							}
						},
						None=>{
							//We have clear line of sight to our target.
							//Lets go to the target.
							if bot.move_to_point(grid.to_world_center(next),target_radius){
								
								//dbg!("HIT TARGET!");
								match pointiter.next(){
									Some(_target)=>{
										*time=pathfinder.get_time();
									},
									None=>{
										
									}
								}
							}
						}
					}
				},
				None=>{
					*state=GridBotState::DoingNothing;
					//unreachable!("should be impossible?");
				}
			
			}
		},
		GridBotState::Thinking |
		GridBotState::DoingNothing=>{
			bot.acc=-bot.vel.truncate_at(0.03);
			//bot.move_to_point(bot.pos,target_radius);
		}
	}
	
	
	//Get square to display the 4 ray casts.
	//Confirm you get different values of tval for each.
	
	bot.vel+=bot.acc;
	bot.acc=vec2same(0.0);
}

fn handle_bot_moving(b:&mut GridBot,prop:&BotProp,_pathfinder:&PathFinder,grid:&GridViewPort,walls:&Grid2D){

	let _state=&mut b.state;
	let bot=&mut b.bot;

	//you have a bot
	//make aabb
	//grow aabb a bit
	//		if new aabb intersects a wall, reposition.
	//raycast out of the corners
	//find min tval
	//move bot as close as possible.

	//This is first set equal to the velocity.
	//as we collide with rectangles, we subtract from this vector.
	//then we should skip the step where we apply velocity to the position.
	//since we have been doing that.
	const EXTRA:WorldNum=0.01;
		
	if bot.vel.magnitude2()>0.0{	
		let mut amount_left_to_move=bot.vel.magnitude();
		let _last_speed=amount_left_to_move;

		while amount_left_to_move>0.0{
	
			let _bot_save=*bot;

			//we grow the aabb a bit to carch corner cases where we move diagonally
			let rect= *create_bbox_wall(bot,prop).grow(EXTRA); //TODO figure this out.
			assert!(!rect_is_touching_wall(&rect,grid,walls));
			
			match RayStorm::new(rect).find_nearest_collision(grid,walls,bot.vel.normalize_to(1.0),amount_left_to_move+EXTRA*2.)
			{
				Some(BBoxCollideCellEvent{corner,inner})=>{
					let _corner_diff=corner-bot.pos;
		
					let va=bot.vel.normalize_to(1.0);

					let mm=(inner.tval-EXTRA*4.0).max(0.0);
					bot.pos+=va*mm;	
					amount_left_to_move-=mm;

					let rect= *create_bbox_wall(bot,prop).grow(EXTRA); //TODO figure this out.
					assert!(!rect_is_touching_wall(&rect,grid,walls));
			
					//assert!(!rect_is_touching_wall(&create_bbox_wall(bot,prop),grid,walls));

					use CardDir::*;
					match inner.dir_hit{
						L=>{
							bot.vel.x=-bot.vel.x;
							assert!(bot.vel.x<0.0);
						},
						R=>{
							bot.vel.x=-bot.vel.x;
							assert!(bot.vel.x>0.0);
						},
						U=>{
							bot.vel.y=-bot.vel.y;
							assert!(bot.vel.y<0.0);
						},
						D=>{
							bot.vel.y=-bot.vel.y;
							assert!(bot.vel.y>0.0);
						}
					}
				},
				None=>{
					
					assert_ge!(amount_left_to_move,0.0);
					assert!(amount_left_to_move.is_finite());

					let nv=bot.vel.normalize_to(1.0)*amount_left_to_move;
					bot.pos+=nv;
					assert!(!rect_is_touching_wall(&create_bbox_wall(bot,prop),grid,walls));


					
					break;
				}
			}
				
		}
	}
	assert!(!rect_is_touching_wall(&create_bbox_wall(bot,prop),grid,walls));

}
