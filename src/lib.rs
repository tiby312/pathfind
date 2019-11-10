pub use dinotree;
pub use duckduckgeo;
use crate::bot::*;
use dinotree::axgeom::ordered_float::*;
use dinotree::axgeom::*;
use dinotree::axgeom;

use dinotree::prelude::*;
use duckduckgeo::*;
use dinotree_alg::rect::*;



/*
type GridNum=isize;


mod bot{
    type BotIndex=usize;
}
mod card{

}
*/


/*
mod grid{
    pub struct Grid{

    }
}

mod pathfind{
    struct PathFindInfo{
        start:Vec2<GridNum>,
        end:Vec2<GridNum>,
        bot_index:BotIndex
    }

    struct PathFindTimer{
        req:PathFindInfo,
        //The number of ticks left before this request must be fulfilled.
        time_put_in:usize
    }


    struct PathFindResult{
        info:PathFindInfo,
        path:PathfindSolution
    }

    const DELAY=60;
    struct PathFinder{
        requests:Queue<PathFindTimer>,
        timer:usize //TODO what to do on overflow
    }

    impl PathFinder{
        //add some new requests and also
        //process some request
        //
        //all requests will be returned by this function after DELAY calls to this function.
        //no sooner, no later.
        fn handle_par(&mut self,grid:&Grid,new_requests:Vec<PathFindInfo>)->Vec<PathFindResult>{
            
            //add new requests to priority queue.

            //pull a decent amount from the priority queue.
            //at the very least pull out everything from the priotiy queue that needs to be computed in this tick.
            //
            //
            //handle them in parallel.
            self.timer+=1;
        }

    }
}

pub enum BotState{
    DoingNothing,
    Thinking,
    Moving(ShortPathIter)
}

struct GridBot{
    grid_position:Vec2<GridNum>,
    state:BotState
}


pub struct MainLogic{
    bots:Vec<Bot>
    grid_bots:Vec<GridBot>
}

impl MainLogic{

    fn step(&mut self){

        let mut requests=Vec::new();
        for b in bots.iter(){
            if b.wants_to_go_to_store(){
                b.state=Thinking.
                requests.push(send_bot_to_store)
            }
        }

        let results=self.pathfinder.handle_par(grid,requests);

        for res in results{
            let bot=&mut self.bots[res.bot_index];
            assert!(bot.state!=Moving);
            bot.state=Moving(res);
        }


        //actually move the bots now.
        for b in bots.iter(){
            b.move_to(self.grid_pos+b.moveing);

            let new_grid_pos = self.grid.lookup_pos(b.pos);

            if b.moving_up && new_grid_pos=b.grid_pos+vec2(0,1){
                //move to new grid position
            }else{
                //the real life bot got pushed around to the worng grid.
                //continue trying to get to the right grid.
            }
        }


        let tree=DinoTreeBuilder::new(&mut self.bots);
        colfind::query_par(&mut tree,|a,b|a.collide(b));


    }
}


//input:
//a minimum rectangle that must be visible in the game world
//the window dimensions.
//output:
//the game world dimensions
pub fn compute_border(rect:Rect<f32>,window:[f32;2])->Rect<f32>{
    
    println!("game word minimum={:?}",rect);
    println!("window={:?}",window);
    let target_aspect_ratio=window[0]/window[1];


    let ((x1,x2),(y1,y2))=rect.get();
    let w=x2-x1;
    let h=y2-y1;

    let current_aspect_ratio=w/h;

    let [xx,yy]=if target_aspect_ratio<current_aspect_ratio{
        //target is thinner
        [0.0,-h+(window[1]*w)/window[0]]

    }else{
        //target is wider
        [window[0]*h/window[1]-w,0.0]
    };

    let xx_half=xx/2.0;
    let yy_half=yy/2.0;

    let xx1=x1-xx_half;
    let xx2=x2+xx_half;

    let yy1=y1-yy_half;
    let yy2=y2+yy_half;

    let r=Rect::new(xx1,xx2,yy1,yy2);
    println!("game world target={:?}",r);
    r
}

pub struct BotSystem {
    mouse_prop:MouseProp,
    bots: Vec<Bot>,
    bot_prop:BotProp
}


impl BotSystem{

    pub fn new(aspect_ratio:f64,num_bots:usize) -> (BotSystem,Rect<f32>,f32) {
        
        let bot_prop=BotProp{
            radius:Dist::new(12.0),
            //radius:Dist::new(20.0),
            collision_drag:0.003,
            collision_push:1.3,
            minimum_dis_sqr:0.0001,
            viscousity_coeff:0.03
        };

        let (bots,mut container_rect) = create_bots(aspect_ratio,num_bots,&bot_prop).unwrap();
        container_rect.grow(10.0);
        //let session=Session::new();
        //let session=DinoTreeCache::new(axgeom::YAXISS);

        let mouse_prop=MouseProp{
            radius:Dist::new(200.0),
            force:20.0//1.0
        };
        let b=BotSystem {
            mouse_prop,
            bots,
            bot_prop,
        };
        (b,container_rect,bot_prop.radius.dis()*0.7)
    }

    
    pub fn get_bots(&self)->&[Bot]{
        &self.bots
    }

    pub fn get_bots_mut(&mut self)->&mut [Bot]{
        &mut self.bots
    }


    pub fn step(&mut self, poses: &[Vec2<f32>],border:&Rect<f32>) {
        
        let border=border.inner_try_into().unwrap();

        {                
            let bot_prop=&self.bot_prop;
            

            let mut bots=create_bbox_mut(&mut self.bots,|bot|{
                bot.create_bbox(bot_prop).inner_try_into().unwrap()
            });

            let mut tree=DinoTreeBuilder::new(axgeom::YAXISS,&mut bots).build_par();

            
            dinotree_alg::colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b|{
                bot_prop.collide(a.inner_mut(),b.inner_mut());
            });
            

            for k in poses{
                let mouse=Mouse::new(*k,self.mouse_prop);
                let mouserect=mouse.get_rect().inner_try_into().unwrap();
                 
                for_all_in_rect_mut(&mut tree,&mouserect,|mut a|{
                    bot_prop.collide_mouse(a.inner_mut(),&mouse);
                });
            }
            
            
            for_all_not_in_rect_mut(&mut tree,&border,|mut a|{
                duckduckgeo::collide_with_border(a.inner_mut(),border.as_ref(),0.5);
            });

        }



        //update bots
        for bot in self.bots.iter_mut() {
            bot.vel+=bot.acc;    
            bot.pos+=bot.vel;
            bot.acc=vec2(0.0,0.0);
        }        
    }

}


#[derive(Copy,Clone,Debug)]
pub struct NoBots;
pub fn create_bots(aspect_ratio:f64,num_bot:usize,bot_prop: &BotProp)->Result<(Vec<Bot>,axgeom::Rect<f32>),NoBots>{
    
    //let s=dists::spiral::Spiral::new([0.0,0.0],12.0,1.0);

    
    let mut bots=Vec::with_capacity(num_bot);
    dists::grid::from_center(vec2(0.0,0.0),aspect_ratio as f32,10.0,num_bot,|v|{
        bots.push(Bot::new(v))
    });
    
    
    /*
    let s=dists::grid::Grid::new(axgeom::Rect::new(-2000.,2000.,-1300.,1300.),num_bot);
    let bots:Vec<Bot>=s.take(num_bot).map(|pos|Bot::new(vec2(pos.x as f32,pos.y as f32))).collect();
    */
    assert_eq!(bots.len(),num_bot);

    let rect=bots.iter().fold(None,|rect:Option<Rect<NotNan<f32>>>,bot|{
        match rect{
            Some(mut rect)=>{
                rect.grow_to_fit(&bot.create_bbox(bot_prop).inner_try_into().unwrap());
                Some(rect)
            },
            None=>{
                Some(bot.create_bbox(bot_prop).inner_try_into().unwrap())
            }
        }
    });


    

    let rect=match rect{
        Some(x)=>{
            x
        },
        None=>{
            return Err(NoBots)
        }
    };
    
    let rect=rect.inner_into();
    
    /*
    let midpoint=vec2(rect.x.right-rect.x.left,rect.y.right-rect.y.left);
    for b in bots.iter_mut(){
        b.pos-=midpoint;
    }
    */

    Ok((bots,rect))
}




*/