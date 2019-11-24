
use crate::*;
use crate::short_path::*;
use duckduckgeo::grid::*;
use crate::short_path::shortpath2::ShortPath2;



#[derive(Copy,Clone,Debug)]
pub struct PathFindInfo{
    pub start:Vec2<GridNum>,
    pub end:Vec2<GridNum>,
    pub bot_index:BotIndex
}


#[derive(Eq,PartialEq,Copy,Clone,Debug)]
pub struct StartEnd{
    pub start:Vec2<GridNum>,
    pub end:Vec2<GridNum>,
}
impl StartEnd{
    //TODO use this to re-use paths in the cache that are CLOSE to what we want.
    //and then combine them.
    fn compare_difference(&self,other:&StartEnd)->GridNum{
        let dis1=(other.start-self.start).magnitude2();
        let dis2=(other.end-self.end).magnitude2();
        dis1.max(dis2)
    }
    fn into_num(&self)->u64{
        (self.start.x as u64) << 48  | (self.start.y as u64) << 32 | (self.end.x as u64) << 16 | (self.end.y as u64)
    }
}

impl Ord for StartEnd {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.into_num().cmp(&other.into_num())
    }
}

impl PartialOrd for StartEnd {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}



#[derive(Copy,Clone,Debug)]
pub struct PathFindCacheResult{
    pub path:Option<ShortPath2>
}


#[derive(Copy,Clone,Debug)]
pub struct PathFindResult{
    pub info:PathFindInfo,
    pub path:Option<ShortPath2>
}

fn test(){
    use std::collections::BTreeMap;
    let _map:BTreeMap<StartEnd,PathFindCacheResult> = BTreeMap::new();

}


struct PathFindTimer{
    info:PathFindInfo,
    time_put_in:usize
}

use std::collections::VecDeque;


const DELAY:usize=60*3;


//TODO add caching
mod test{
    
    
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


fn perform_astar(grid:&Grid2D,req:PathFindInfo)->Option<ShortPath2>{
    //TODO this function does a bunch of dynamic allocation. how to avoid?
    fn successors(a:&Vec2<GridNum>,grid:&Grid2D) -> Vec<(Vec2<GridNum>, u32)> {
        
        let a=*a;

        let offsets=CardDir2::all_offsets();


        offsets.iter().filter(|&(b,_)|{
            let c=a+*b;
            match grid.get_option(c){
                Some(wall)=>{
                    if wall{
                        false
                    }else{
                        //if is diagonal
                        if b.abs()==vec2(1,1){
                            let vs=b.split_into_components();

                            let mut ff=true;
                            for &v in vs.iter(){
                                if let Some(w) = grid.get_option(a+v){
                                    if w{
                                        ff=false;
                                        break;
                                    }
                                }
                            }
                            ff
                        }else{
                            true
                        }
                    }
                },
                None=>{
                    false
                }
            }
        }).map(|&(b,c)|(a+b,c as u32)).collect()

    }


    let start=req.start;
    let end  =req.end;

    let result = pathfinding::directed::astar::astar(&start,|p|successors(p,grid),|p|p.manhattan_dis(end) as u32,|p|p==&end);

    match result{
        Some((mut a,_))=>{
            let mut cursor=start;

            let mut dirs=Vec::new();
            
            for curr in a.drain(..).skip(1).take(shortpath2::MAX_PATH_LENGTH){
                let dir=CardDir2::from_offset(curr-cursor);
                dirs.push(dir);
                cursor=curr;
            }

            Some(ShortPath2::new(dirs.drain(..)))
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

    pub fn get_time(&self)->usize{
        self.timer
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

        let mut infos_that_must_be_processed=0;
        for (index,a) in self.requests.iter().enumerate().rev(){
            if self.timer - a.time_put_in >DELAY{
                infos_that_must_be_processed=index;
                break;
            }
        }
       
        let num_to_process=100.max(infos_that_must_be_processed);
        
        let problem_vec:Vec<_>=self.requests.drain(0..num_to_process.min(self.requests.len())).collect();
        //use rayon::prelude::*;

        let mut newv:Vec<_> = problem_vec.iter().map(|a|{
            (a.time_put_in,PathFindResult{info:a.info,path:perform_astar(grid,a.info)})
        }).collect();
        

        for a in newv.drain(..){
            self.finished.push_back(a);
        }



        let mut newv = Vec::new();

        loop{
            match self.finished.front(){
                Some(front)=>{
                    
                    if self.timer - front.0 != DELAY+1{
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