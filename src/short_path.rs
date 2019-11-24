
use crate::axgeom::*;

use duckduckgeo::grid::*;



#[test]
fn test_short(){
    use crate::short_path::*;
    use CardDir::*;

    let test_path=[U,D,D,L,R,U,L,R,U,R,R,D,D,D,U,R,D,R,U,D,D,D,D,D,D,D,U,D,D,U,U];
    let s=ShortPath::new(test_path.iter().map(|a|*a));
    let v:Vec<_>=s.iter().collect();
    assert_eq!(&v as &[_],&test_path);
}


pub mod shortpath2{
    use super::*;

    pub const MAX_PATH_LENGTH:usize=42;
    const SENTINAL_VAL:u128=0b11;
    #[derive(Copy,Clone,Eq,PartialEq,Debug)]
    pub struct ShortPath2{
        value:u128
    }
    impl ShortPath2{
        pub fn new<I:IntoIterator<Item=CardDir2>+ExactSizeIterator>(it:I)->ShortPath2{
            assert!(it.len()<=MAX_PATH_LENGTH,"You can only store a path of up to length 31 != 32.");

            let mut value = 0;
            let mut bit_index=0;
            for a in it{
                value |= (a as u128) << bit_index;
                bit_index+=3;
            }
            value |= SENTINAL_VAL<<bit_index;

            ShortPath2{value}
        }

        pub fn len(&self)->usize{
            let l=self.value.leading_zeros() as usize;
            //println!("l={:?}",l);
            MAX_PATH_LENGTH-(l/2)
        }
        pub fn iter(&self)->ShortPath2Iter{
            ShortPath2Iter{path:*self}
        }
    }


    #[derive(Copy,Clone,Eq,PartialEq)]
    pub struct ShortPath2Iter{
        path:ShortPath2
    }

    impl fmt::Debug for ShortPath2Iter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let path:Vec<_> = self.collect();

            write!(f, "Path:{:?}", path)
        }
    }

    impl ShortPath2Iter{
        pub fn peek(&self)->Option<CardDir2>{
            if self.path.value ==SENTINAL_VAL{
                return None
            }

            //make a copy
            let cc=self.path.value;
            cc>>3;

            if cc == SENTINAL_VAL{
                return None
            }else{
                
                let k=self.path.value & 0b111;
                //dbg!(k);
                Some(CardDir2::from_u8(k as u8))
            }
        }
    }

    impl core::iter::FusedIterator for ShortPath2Iter{}
    impl ExactSizeIterator for ShortPath2Iter{}
    impl Iterator for ShortPath2Iter{
        type Item=CardDir2;
        
        fn size_hint(&self)->(usize,Option<usize>){
            let l = self.path.len();
            (l,Some(l))
        }
        
        fn next(&mut self)->Option<Self::Item>{
            //If the path has nothing left in it except for the sentinal val
            if self.path.value ==SENTINAL_VAL{
                return None
            }

            let dir = CardDir2::from_u8((self.path.value & 0b111) as u8);

            self.path.value=self.path.value >> 3;

            Some(dir)
        }
    }

}


const SENTINAL_VAL:u64=0b11;

pub const MAX_PATH_LENGTH:usize=31;
//CardDir only takes up 2 bit. So inside of a 64 bit integer,
//we can store a path of length 32.
//at that point just make it have to re-compute.


//
//   110000000000000000
//   xx8877665544332211
#[derive(Copy,Clone,Eq,PartialEq)]
pub struct ShortPath{
    value:u64
}


use core::fmt;
impl fmt::Debug for ShortPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path:Vec<_> = self.iter().collect();

        write!(f, "Path:{:?}", path)
    }
}

impl ShortPath{
    pub fn new<I:IntoIterator<Item=CardDir>+ExactSizeIterator>(it:I)->ShortPath{
        assert!(it.len()<=MAX_PATH_LENGTH,"You can only store a path of up to length 31 != 32.");

        let mut value = 0;
        let mut bit_index=0;
        for a in it{
            value |= (a.into_two_bits() as u64) << bit_index;
            bit_index+=2;
        }
        value |= SENTINAL_VAL<<bit_index;

        ShortPath{value}
    }

    pub fn len(&self)->usize{
        let l=self.value.leading_zeros() as usize;
        //println!("l={:?}",l);
        MAX_PATH_LENGTH  -(l/2)
    }
    pub fn iter(&self)->ShortPathIter{
        ShortPathIter{path:*self}
    }

    /*
    pub fn draw_map_and_path(&self,path:PathPointIter){
        use std::collections::HashMap;

        let mut res=String::new();
        
        
        
        let mut vv:Vec<_>=path.collect();
        vv.push(path.pos());

        println!("");
        for i in 0..self.dim().y{
            for j in 0..self.dim().x{
                
                let cc=if vv.iter().any(|a|*a==vec2(j,i)){
                    if self.get(vec2(j,i)){
                        "x "
                        //panic!("path goes through wall");
                    }else{
                        "* "
                    }
                }else{

                    if self.get(vec2(j,i)){
                        "1 "
                    }else{
                        "0 "
                    }
                };
                print!("{}",cc);
                //res.push_str(cc);
            }
            println!("{}",res);
            //fa=fa.and( writeln!(f,"{}",res));
            //res.clear();
        }
    }
    */
}

/*
TODO its absolutely amazing how it is better to store just 4 cardinal directions instead of 8,
since you can 'look ahead' and optimize cases where you have a down and then a right provided there
isnt a grid wall in a certain spot.
*/





#[derive(Eq,PartialEq,Copy,Clone,Debug)]
pub struct PathDiagAdapter{
    pub inner:PathPointIter
}
impl PathDiagAdapter{
    pub fn new(path:PathPointIter)->PathDiagAdapter{
        PathDiagAdapter{inner:path}
    }
}


fn intersects_walls(grid:&Grid2D,start:Vec2<GridNum>,end:Vec2<GridNum>)->bool{
    use line_drawing::Supercover;

    for (x, y) in Supercover::new((start.x,start.y),(end.x,end.y)) {
        if let Some(x)=grid.get_option(vec2(x,y)){
            if x{
                return true;
            }
        }
    }
    return false;
}

/*
128 bits
32 (8 directions)
16*2
2^5
*/


//DOES NOT IMPLEMENT EXACT SIZE
impl PathDiagAdapter{

    pub fn next(&mut self,_radius:WorldNum,_grid:&GridViewPort,_walls:&Grid2D)->Option<Vec2<GridNum>>{
        self.inner.next()
        /*
        let current_grid=self.inner.pos();

        let current=grid.to_world_center(self.inner.pos());

        match self.inner.next(){
            Some(a)=>{
                //If we arn't able to find any short cuts, we should just return a, at it is right now.
                let mut test_current_grid=a;
                let mut test_current=grid.to_world_center(a);

                let mut count=0;
                while let Some(b)=self.inner.peek(){
                    if count>=10{
                        break;
                    }
                    let tt=b;
                    let tt_world=grid.to_world_center(tt);
                    if crate::game::ray_hits_point(radius,current,tt_world,grid,walls){
                        let j=self.inner.next().unwrap();
                        assert_eq!(j,b);
                        test_current_grid=b;
                        test_current=grid.to_world_center(b);
       
                    }else{
                        break;
                    }
                    count+=1;
                }
                
                Some(test_current_grid)
            },
            None=>{
                None
            }
        }
        */
    }
}


 use crate::short_path::shortpath2::ShortPath2Iter;


#[derive(Eq,PartialEq,Copy,Clone,Debug)]
pub struct PathPointIter{
    cursor:Vec2<GridNum>,
    path:ShortPath2Iter
}
impl PathPointIter{
    pub fn new(start:Vec2<GridNum>,path:ShortPath2Iter)->PathPointIter{
        PathPointIter{cursor:start,path}
    }
    pub fn pos(&self)->Vec2<GridNum>{
        self.cursor
    }
    pub fn peek(&self)->Option<Vec2<GridNum>>{
        self.path.peek().map(|p|self.cursor+p.into_offset().0)
    }
}

impl core::iter::FusedIterator for PathPointIter{}
impl ExactSizeIterator for PathPointIter{}
impl Iterator for PathPointIter{
    type Item=Vec2<GridNum>;
    fn size_hint(&self)->(usize,Option<usize>){
        let l = self.path.len();
        (l,Some(l))
    }
    fn next(&mut self)->Option<Self::Item>{
        //let dis=curr_target.manhattan_dis(pointiter.pos());
        //assert_le!(dis,2);

        match self.path.next(){
            Some(p)=>{
                self.cursor+=p.into_offset().0;
                Some(self.cursor)      
            },
            None=>{
                None
            }
        }

    }
}


#[derive(Copy,Clone,Eq,PartialEq)]
pub struct ShortPathIter{
    path:ShortPath
}

impl fmt::Debug for ShortPathIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path:Vec<_> = self.collect();

        write!(f, "Path:{:?}", path)
    }
}

impl ShortPathIter{
    pub fn peek(&self)->Option<CardDir>{
        if self.path.value ==SENTINAL_VAL{
            return None
        }

        let cc=self.path.value;
        cc>>2;

        if cc == SENTINAL_VAL{
            return None
        }else{
            use CardDir::*;
            let val = match self.path.value & 0b11 {
                0b00=>{
                    U
                },
                0b01=>{
                    D
                },
                0b10=>{
                    L
                },
                0b11=>{
                    R
                },
                _=>{
                    unreachable!()
                }
            };
            Some(val)
        }
    }
}

impl core::iter::FusedIterator for ShortPathIter{}
impl ExactSizeIterator for ShortPathIter{}
impl Iterator for ShortPathIter{
    type Item=CardDir;
    
    fn size_hint(&self)->(usize,Option<usize>){
        let l = self.path.len();
        (l,Some(l))
    }
    
    fn next(&mut self)->Option<Self::Item>{
        //If the path has nothing left in it except for the sentinal val
        if self.path.value ==SENTINAL_VAL{
            return None
        }

        use CardDir::*;
        let val = match self.path.value & 0b11 {
            0b00=>{
                U
            },
            0b01=>{
                D
            },
            0b10=>{
                L
            },
            0b11=>{
                R
            },
            _=>{
                unreachable!()
            }
        };

        self.path.value=self.path.value >> 2;

        Some(val)
    }
}
