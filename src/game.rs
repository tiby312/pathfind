



use duckduckgeo::bot::BotProp;
use crate::pathfind::*;
use crate::short_path::*;

use crate::axgeom::*;
use duckduckgeo::grid::*;
use duckduckgeo::grid::raycast::*;
//use duckduckgeo::grid::CardDir;
use duckduckgeo::bot::Dist;

#[derive(Eq,PartialEq,Debug,Copy,Clone)]
pub enum GridBotState{
	DoingNothing,
	Thinking,
	Moving(PathPointIter,usize) //Time since it last hit something.
}

#[derive(Copy,Clone,Debug)]
pub struct Bot{
	pub pos:Vec2<f32>,
	pub vel:Vec2<f32>,
	pub steering:Vec2<f32>,
	pub counter:usize
}

/*
#[test]
fn testy(){
	let b1=Bot{pos:vec2(10.0,10.0),vel:vec2(-1.0,-0.5)};
	let b2=Bot{pos:vec2(0.0,0.0),vel:vec2(0.0,0.0)};

	dbg!(b1,b2);
	b1.predict_collision(&b2,5.0);
	panic!("fail")
}
*/

impl Bot{

	fn predict_collision(&self,other:&Bot,radius:f32,max_tval:f32)->Option<(f32,f32,Vec2<f32>)>{
		let a=self;
		let b=other;

		let vel=b.vel-a.vel;
		if vel.magnitude()<0.01{
			return None
		}

		let pos=b.pos-a.pos;

		let vel_normal=vel.normalize_to(1.0);
		//let tval=cross(&pos,&vel_normal).abs();//.dot(pos);
		
		//let tval=pos.normalize_to(1.0).rotate_90deg_left()*tval
		let tval=-vel_normal.dot(pos);//pos.dot(vel_normal);
		if tval>0.0 && tval < max_tval{
			
			
			let closest_pos=pos+vel_normal*tval;

			let distance=closest_pos.magnitude();

			if distance<radius*2.0{
				//assert!(!tval.is_nan());
				
				return Some((tval,distance,-closest_pos.normalize_to(1.0)))
			}
		}

		None
	}
}


#[derive(Copy,Clone,Debug)]
pub struct GridBot{
	pub bot:Bot,
	pub state:GridBotState
}



impl seq_impulse::VelocitySolvable for GridBot{
	fn pos(&self)->&Vec2<f32>{
		&self.bot.pos
	}
	fn vel_mut(&mut self)->&mut Vec2<f32>{
		&mut self.bot.vel
	}
}





pub struct Game{
    bot_prop:BotProp,
	grid:GridViewPort,
	walls:Grid2D,
	bots:Vec<GridBot>,
	pathfinder:PathFinder,
	velocity_solver:seq_impulse::CollisionVelocitySolver
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
█   ██  ██    ██                  ██                           █
█   ██  ██ ██ ██ ██   ██          ██          ██         ██    █
█       ██ ██    ██  ███  ██  ██  ██   ██     ██         ██    █
█    █████    ██ ██  ██   ██  ██  ██   ██                      █
█    ███████████ ██████   ██  ██              ████████         █
█     ██████████ ██████   ██  ██              ████████    ██   █
█            ███ ██           ██   ██         ████████    ██   █
████████   █████ ████  █████  ██   ██    ███████               █
████████   █████ ████  █████  ███  ██    ███████         ██    █
█                              ██  ██                    ██    █
█  ███████████    ██████████   ██                    ██        █
█  ███████████    ██████████   ██                    ██        █
█       █████     ██  ██  ██   ██     ██        ██             █
█       ████              ██   ██     ██        ██             █
█      ████        ██  ██      ██                              █
█████████     ███████████████  ██              █████████████████
█████████     ███████████████  ████████  ███   █████████████████
█                          ██   ███████  ███                   █
█     █████████            ██    ███      ██                   █
█     █████████                           ██████               █
█        ██████████            ██         ██████               █
█            ████████          ████       ██     ██   ██       █
█                ████      ███ ████       ██    ███  ███       █
█                         ████  ███       ██   ███  ███        █
██████████████    ██████████         ████████████  █████████████
██████████████    █████████          ███████████  ██████████████
█           ██                       ███    ███  ███           █
█           ██                             ███  ███            █
█           ██                            ███  ███             █
█           ██                           ███  ███              █
█           ██    █████████████         ███  ███               █
█           ██    █████████████        ███  ███                █
█        ██ ██    █████████████        ███  ███                █
█        ██       █████████████                                █
████████████████████████████████████████████████████████████████
"};
}


fn create_bbox_wall(bot:&Bot,bot_prop:&BotProp)->Rect<WorldNum>{
	let radius=bot_prop.radius.dis()*0.5;
	Rect::from_point(bot.pos,vec2same(radius))
}

impl Game{
	pub fn new()->(Game,Vec2<f32>){
		let pathfinder=PathFinder::new();
		let area=vec2(1920.,1080.);

		let dim=Rect::new(0.0,1920.,0.0,1080.);
		let map=maps::GRID_STR3;
		let grid_dim=map.dim;

		assert_eq!(1920./grid_dim.x as f32,1080./grid_dim.y as f32);

		let grid=GridViewPort{origin:vec2(0.0,0.0),spacing:1920./grid_dim.x as f32};

		let walls=Grid2D::from_str(map);

		let bot_prop=BotProp{
            radius:Dist::new(6.0),
            collision_drag:0.001,
            collision_push:0.01,
            minimum_dis_sqr:0.0001,
            viscousity_coeff:0.003
        };


        let num_bot=300;
        let s=dists::grid::Grid::new(*dim.clone().grow(-0.1),num_bot);
    	let mut bots:Vec<GridBot>=s.take(num_bot).map(|pos|{
    		let bot=Bot{pos:pos.inner_as(),vel:vec2same(0.0),steering:vec2same(0.0),counter:0};
    		GridBot{bot,state:GridBotState::DoingNothing}
    	}).collect();


    	for b in bots.iter_mut(){
    		let bot=&mut b.bot;
    		let prop=&bot_prop;

    		if rect_is_touching_wall( &Rect::from_point(bot.pos, vec2same(prop.radius.dis())),&grid,&walls){
				bot.pos=grid.to_world_center(walls.find_closest_empty(grid.to_grid(bot.pos)).unwrap());
				//assert!(!rect_is_touching_wall(&bot.create_bbox(prop),&grid,&walls));
			}	
    	}
    	

		(Game{grid,walls,bots,pathfinder,bot_prop,velocity_solver:seq_impulse::CollisionVelocitySolver::new()},area)
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
	pub fn step(&mut self,canvas:&mut egaku2d::SimpleCanvas){
				
		handle_path_assignment(self);		
		
		
		for b in self.bots.iter_mut(){
			handle_bot_steering(b,&self.pathfinder,&self.grid,&self.walls);
		}
		


		let avoid_radius=self.bot_prop.radius.dis()*20.0;
		//TODO calculate bbox to grow in direction of velocity
		let mut tree=dinotree_alg::collectable::CollectableDinoTree::new(&mut self.bots,|bot|{
			Rect::from_point(bot.bot.pos,vec2same(avoid_radius)).inner_try_into::<NotNan<_>>().unwrap()
		});

	    let radius=self.bot_prop.radius.dis();
	    
	    
	    //let mut lines =canvas.lines(2.0);
	    
	    let mut pairs=tree.collect_collisions_list_par(|a,b|{
	    	let a=&mut a.bot;
	    	let b=&mut b.bot;


	    	let offset=b.pos-a.pos;
	    	let distance=offset.magnitude();
	    	if distance>0.01 && distance<avoid_radius*2.0{
	    		Some((offset,distance))
	    	}else{
	    		None
	    	}
	    });

	    
	    for b in tree.get_bots_mut().iter_mut(){
	    	b.bot.steering=b.bot.vel;
	    	b.bot.counter=1;
	    }

	    pairs.for_every_pair_mut_par(&mut tree,|a,b,&mut (offset,distance)|{

	    	let a=&mut a.bot;
	    	let b=&mut b.bot;

    		//alignment
    		a.counter+=1;
    		b.counter+=1;
    		a.steering+=b.vel;
    		b.steering+=a.vel;
    	
	    	//seperation	
	    	let sep_coeff=0.00005;
	    	let dis_mag=(avoid_radius*2.0)/distance;
	    	let offset_norm=offset.normalize_to(1.0);
	    	assert!(!dis_mag.is_nan());
	    	a.vel-=offset_norm*dis_mag*sep_coeff;
	    	b.vel+=offset_norm*dis_mag*sep_coeff;
		

	    	let avoid_coeff=0.1;
	    	if let Some((tval,distance,aa)) = a.predict_collision(&b,radius,80.0){
	    		//both between 0..1
	    		let hit_mag=(radius*2.0-distance)/radius*2.0;
	    		let tval_mag=(80.0-tval)/80.0;

	    		let mag=avoid_coeff * (tval_mag+hit_mag)*0.5;
	    		if mag>0.01{
		    		let kk=aa*mag;
		    		a.vel+=kk;
		    		b.vel-=kk;
	    		}
	    	}

	    });
	    
	    //lines.send_and_uniforms(canvas).with_color([1.0,1.0,0.2,0.3]).draw();

	    let alignment_coeff=0.01;
	    for a in tree.get_bots_mut().iter_mut(){
	    	let a=&mut a.bot;

	    	//apply alignment
	    	let k=a.steering/a.counter as f32;
	    	a.vel+=k*alignment_coeff;
	    }
	    
 

	    for b in tree.get_bots_mut().iter_mut(){
	    	b.bot.steering=b.bot.pos;
	    	b.bot.counter=1;
	    }

	    pairs.for_every_pair_mut_par(&mut tree,|a,b,&mut (offset,distance)|{
	    	if distance>0.01 && distance<avoid_radius*2.0{
	    		let a=&mut a.bot;
	    		let b=&mut b.bot;

	    		a.steering+=a.pos;
	    		b.steering+=b.pos;
	    		a.counter+=1;
	    		b.counter+=1;
			}
	    });

	    let cohesion_coeff=0.1;
	    for a in tree.get_bots_mut().iter_mut(){
	    	let a=&mut a.bot;

	    	//apply cohesion
	    	let avg_pos=a.steering/a.counter as f32;
	    	let dir=avg_pos-a.pos;
	    	a.vel+=dir*cohesion_coeff;
	    
	    	//a.vel+=vec2(0.0,0.1);
	    }



		use ordered_float::*;
		let bot_prop=&self.bot_prop;
	    let mut tree=dinotree_alg::collectable::CollectableDinoTree::new(&mut self.bots,|bot|{
	    	Rect::from_point(bot.bot.pos,vec2same(bot_prop.radius.dis())).inner_try_into::<NotNan<_>>().unwrap()
	    });




	    self.velocity_solver.solve(self.bot_prop.radius.dis(),&self.grid,&self.walls,&mut tree);

		
		for b in self.bots.iter_mut(){
			b.bot.pos+=b.bot.vel;
		}
	}
}



fn rect_is_touching_wall(rect:&Rect<WorldNum>,grid:&GridViewPort,walls:&Grid2D)->bool{
	let corners=[
		vec2(rect.x.start,rect.y.start),
		vec2(rect.x.start,rect.y.end),
		vec2(rect.x.end,rect.y.start),
		vec2(rect.x.end,rect.y.end)
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
			vec2(rect.x.start,rect.y.start),
			vec2(rect.x.start,rect.y.end),
			vec2(rect.x.end,rect.y.start),
			vec2(rect.x.end,rect.y.end)
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

	let ray=axgeom::Ray{point,dir};
	
	let caster= RayCaster::new(grid,ray);
	

	if let Some(wall)=walls.get_option(grid.to_grid(point)){
		let grid_mod=grid.to_grid_mod(point);
		if wall{
			return None
		}
		//assert!(!wall,"We are starting the raycast inside a wall! point:{:?} grid mod:{:?}",point,grid_mod);
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


fn handle_path_assignment(game:&mut Game){
	let mut path_requests=Vec::new();
	for (i,b) in game.bots.iter_mut().enumerate(){
		if b.state ==GridBotState::DoingNothing{
			let start =game.grid.to_grid(b.bot.pos);

			let start =match game.walls.get_option(start){
				None=>{
					game.walls.find_closest_empty(start).unwrap()
				},
				Some(walls)=>{
					if walls{
						game.walls.find_closest_empty(start).unwrap()
					}else{
						start
					}
				}
			};

			let end = game.walls.pick_empty_spot().unwrap();
				
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
				let k=PathPointIter::new(res.info.start,path.iter());

				b.state=GridBotState::Moving(k,game.pathfinder.get_time());		
			},
			None=>{

	
			}
		}
	}
}


fn handle_bot_steering(b:&mut GridBot,pathfinder:&PathFinder,grid:&GridViewPort,walls:&Grid2D){
	let _grid_bot_save=*b;
	let state=&mut b.state;
	let bot=&mut b.bot;

	
	let target_radius=grid.cell_radius()*0.5;


	struct RayCastToSquare<'a>{
		bot:&'a Bot,
		grid:&'a GridViewPort,
		walls:&'a Grid2D,
	}
	impl<'a> RayCastToSquare<'a>{
		fn cast(&self,end:Vec2<GridNum>)->bool{
			let end=self.grid.to_world_center(end);
			let start=self.bot.pos;
			let offset=end-start;
			let max_tval=offset.magnitude();
			cast_ray(self.grid,self.walls,start,offset.normalize_to(1.0),max_tval).is_some()
		}
	}



	match state{
		GridBotState::Moving(ref mut pointiter,time)=>{

			
			let r=RayCastToSquare{bot,grid,walls};

			match pointiter.peek(){
				Some((_carddir,next))=>{

					//If we can't see our target
					let k=if r.cast(next){
						//If we can't see our previous target
						if r.cast(pointiter.pos()){	
							
							if let Some((_carddir,ppos)) =pointiter.double_peek(){
								//If we can't see our next target.
								if r.cast(ppos){	
									
									//Just try and go to the center of the cell we're in. 
									//maybe that will help us.
									let gp=grid.to_grid(bot.pos);

									//first check that we're not in a wall
									if let Some(wall)=walls.get_option(gp){
										assert!(!wall);
									}

									(gp,false)
								}else{
									(ppos,false)
								}
							
							}else{
								//just try and go to the center of the cell we're in. 
								//maybe that will help us.
								let gp=grid.to_grid(bot.pos);

								//first check that we're not in a wall
								if let Some(wall)=walls.get_option(gp){
									assert!(!wall);
								}
								(gp,false)	
							}

						}else{
							//We can't see our target, but we can see our last target.
							//try going to our last target.
							(pointiter.pos(),false)	
						}
						
					}else{
						(next,true)
					};


					let offset={
						let target=grid.to_world_center(k.0);
						//let grid_offset=k.0-grid.to_grid(bot.pos);
						//let k=grid_offset.rotate_90deg_left().inner_as();
						//let new_target=target+k*16.0;
						//new_target-bot.pos
						target-bot.pos
					};

					let steer=(offset-bot.vel*(30.0))*0.0006;
					//let steer=steer.truncate_at(0.2);
					assert!(!steer.x.is_nan()&&!steer.y.is_nan());
					bot.vel+=steer;
					//We have clear line of sight to our target.
					//Lets go to the target.
					if k.1{
						if offset.magnitude()<10.0{
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
					
				},
				None=>{
					*state=GridBotState::DoingNothing;
				}
			
			}
		},
		GridBotState::Thinking |
		GridBotState::DoingNothing=>{
			bot.vel=-bot.vel.truncate_at(0.03);
			
		}
	}
	
	
	//Get square to display the 4 ray casts.
	//Confirm you get different values of tval for each.
	
	//bot.vel+=bot.acc;
	//bot.acc=vec2same(0.0);
}
/*
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
*/
