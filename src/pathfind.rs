
use crate::*;
use crate::short_path::*;

use crate::grid::*;


#[derive(Copy,Clone)]
pub struct PathFindInfo{
    start:Vec2<GridNum>,
    end:Vec2<GridNum>,
    bot_index:BotIndex
}

pub struct PathFindResult{
    info:PathFindInfo,
    path:Option<ShortPath>
}



struct PathFindTimer{
    info:PathFindInfo,
    time_put_in:usize
}

use std::collections::VecDeque;


const DELAY:usize=60;
pub struct PathFinder{
    requests:VecDeque<PathFindTimer>,
    finished:VecDeque<(usize,PathFindResult)>,
    timer:usize //TODO what to do on overflow
}



fn perform_astar(grid:&Grid2D,req:PathFindInfo)->Option<ShortPath>{
    use pathfinding::prelude::*;

    #[derive(Copy,Clone,Eq,PartialEq,Hash)]
    struct Pos{
        x:isize,
        y:isize
    };

    fn pos(x:isize,y:isize)->Pos{
        Pos{x,y}
    }

    impl Pos {
        fn distance(&self, other: &Pos) -> u32 {
            //manhatan distance
            (absdiff(self.x, other.x) + absdiff(self.y, other.y)) as u32
        }

        fn successors(&self) -> Vec<(Pos, u32)> {
            let &Pos{x, y} = self;
             vec![pos(x+1,y), pos(x-1,y), pos(x,y+1), pos(x,y-1)]
             .into_iter().map(|p| (p, 1)).collect()
        }
    }

    let start=Pos{x:req.start.x,y:req.start.y};
    let end  =Pos{x:req.end.x,y:req.end.y};

    let result = pathfinding::directed::astar::astar(&start,|p|p.successors(),|p|p.distance(&end),|p|p==&end);

    match result{
        Some((mut a,_))=>{
            let mut cursor=vec2(start.x,start.y);

            let mut dirs=Vec::new();
            
            for curr in a.drain(..).take(31){
                let curr=vec2(curr.x,curr.y);
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
                    _=>{
                        unreachable!()
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


impl PathFinder{
    pub fn new()->PathFinder{
        PathFinder{requests:VecDeque::new(),finished:VecDeque::new(),timer:0}
    }
    //add some new requests and also
    //process some request
    //
    //all requests will be returned by this function after DELAY calls to this function.
    //no sooner, no later.
    fn handle_par(&mut self,grid:&Grid2D,mut new_requests:Vec<PathFindInfo>)->Vec<PathFindResult>{
        for a in new_requests.drain(..){
            self.requests.push_back(PathFindTimer{info:a,time_put_in:self.timer})    
        }



        use rayon;

        let (aa,bb) = self.requests.as_slices();



        let infos_that_must_be_processed=self.requests.iter().enumerate().find(|a| (self.timer - a.1.time_put_in) < DELAY ).map(|a|a.0);

        
        let num_to_process = match infos_that_must_be_processed{
            Some(a)=>{
                a.max(1000)
            },
            None=>{
                1000
            }
        };


        let problem_vec:Vec<_>=self.requests.drain(0..num_to_process).collect();
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