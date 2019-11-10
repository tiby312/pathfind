
use crate::*;
use crate::short_path::*;

use dists::grid::Grid;


#[derive(Copy,Clone)]
pub struct PathFindInfo{
    start:Vec2<GridNum>,
    end:Vec2<GridNum>,
    bot_index:BotIndex
}

pub struct PathFindResult{
    info:PathFindInfo,
    path:ShortPath
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



fn perform_astar(grid:&Grid,req:PathFindInfo)->ShortPath{
    unimplemented!()
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
    fn handle_par(&mut self,grid:&Grid,mut new_requests:Vec<PathFindInfo>)->Vec<PathFindResult>{
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