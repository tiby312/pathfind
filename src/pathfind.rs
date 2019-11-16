
use crate::*;
use crate::short_path::*;

use crate::grid::*;


#[derive(Copy,Clone,Debug)]
pub struct PathFindInfo{
    pub start:Vec2<GridNum>,
    pub end:Vec2<GridNum>,
    pub bot_index:BotIndex
}


#[derive(Copy,Clone,Debug)]
pub struct PathFindResult{
    pub info:PathFindInfo,
    pub path:Option<ShortPath>
}



struct PathFindTimer{
    info:PathFindInfo,
    time_put_in:usize
}

use std::collections::VecDeque;


const DELAY:usize=60;



mod test{
    use super::*;
    
    #[test]
    fn test(){
        let mut grid=Grid2D::new(vec2(10,10));

        for i in 0..10{
            for j in 0..10{
                grid.set(vec2(i,j),false);
            }
        }


        let mut k=PathFinder::new();

        let start=vec2(0,0);
        let end=vec2(9,9);
        
        let _ =k.handle_par(&grid,vec!(PathFindInfo{start,end,bot_index:0}));
            
        for _ in 0..59{
            let k =k.handle_par(&grid,vec!());
            
            //dbg!(k);
        }

        let k =k.handle_par(&grid,vec!());    
        dbg!(&k);

        use CardDir::*;
        assert_eq!(k[0].path.unwrap(),ShortPath::new([R,R,R,R,R,R,R,R,R,D,D,D,D,D,D,D,D,D].iter().map(|a|*a)));
    }
}


fn perform_astar(grid:&Grid2D,req:PathFindInfo)->Option<ShortPath>{
    //TODO this function does a bunch of dynamic allocation. how to avoid?
    use pathfinding::prelude::*;
    fn successors(a:&Vec2<GridNum>,grid:&Grid2D) -> Vec<(Vec2<GridNum>, u32)> {
        


        let mut v=Vec::new();
        if a.x<grid.dim().x-1{
            let k=*a+vec2(1,0);
            if !grid.get(k){
                v.push(k)
            }
        }
        if a.x>0{
            let k=*a+vec2(-1,0);
            if !grid.get(k){
                v.push(k)   
            }
        }

        if a.y>0{
            let k=*a+vec2(0,-1);
            if !grid.get(k){
                v.push(k)
            }
        }

        if a.y<grid.dim().y-1{
            let k=*a+vec2(0,1);
            if !grid.get(k){
                v.push(k)   
            }
        }

        v.into_iter().map(|p| (p, 1)).collect()
    }


    let start=req.start;
    let end  =req.end;

    let result = pathfinding::directed::astar::astar(&start,|p|successors(p,grid),|p|p.manhattan_dis(end) as u32,|p|p==&end);

    match result{
        Some((mut a,_))=>{
            let mut cursor=start;

            let mut dirs=Vec::new();
            
            for curr in a.drain(..).skip(1).take(31){
            
                use CardDir::*;

                let dir=match curr-cursor{
                    Vec2{x:1,y:0}=>{
                        R
                    },
                    Vec2{x:-1,y:0}=>{
                        L
                    },
                    Vec2{x:0,y:-1}=>{
                        U
                    },
                    Vec2{x:0,y:1}=>{
                        D
                    },
                    uhoh=>{
                        unreachable!("{:?}",uhoh);
                    }
                };
                dirs.push(dir);
                cursor=curr;
            }

            Some(ShortPath::new(dirs.drain(..)))
        },
        None=>{
            None
        }
    }
}

pub struct PathFinder{
    requests:VecDeque<PathFindTimer>,
    finished:VecDeque<(usize,PathFindResult)>,
    timer:usize //TODO what to do on overflow
}
impl PathFinder{
    pub fn new()->PathFinder{
        PathFinder{requests:VecDeque::new(),finished:VecDeque::new(),timer:0}
    }
    //add some new requests and also
    //process some request
    //
    //all requests will be returned by this function after DELAY calls to this function.
    //no sooner, no later.
    pub fn handle_par(&mut self,grid:&Grid2D,mut new_requests:Vec<PathFindInfo>)->Vec<PathFindResult>{
        for a in new_requests.drain(..){
            self.requests.push_back(PathFindTimer{info:a,time_put_in:self.timer})    
        }




        


        let infos_that_must_be_processed=self.requests.iter().enumerate().find(|a| (self.timer - a.1.time_put_in) < DELAY ).map(|a|a.0);

        
        let num_to_process = match infos_that_must_be_processed{
            Some(a)=>{
                a.max(1000)
            },
            None=>{
                1000
            }
        };


        let problem_vec:Vec<_>=self.requests.drain(0..num_to_process.min(self.requests.len())).collect();
        use rayon::prelude::*;

        let mut newv:Vec<_> = problem_vec.into_par_iter().map(|a|{
            (a.time_put_in,PathFindResult{info:a.info,path:perform_astar(grid,a.info)})
        }).collect();

        for a in newv.drain(..){
            self.finished.push_back(a);
        }



        let mut newv = Vec::new();

        loop{
            match self.finished.front(){
                Some(front)=>{
                    if self.timer - front.0 != DELAY{
                        break;
                    }
                    newv.push(self.finished.pop_front().unwrap().1);
                },
                None=>{
                    break;
                }
            }
        }
        
        //pull a decent amount from the priority queue.
        //at the very least pull out everything from the priotiy queue that needs to be computed in this tick.
        //
        //
        //handle them in parallel.
        self.timer+=1;

        newv
    }

}