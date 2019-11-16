
use crate::axgeom::*;
use crate::grid::*;

#[test]
fn test_short(){
    use crate::short_path::*;
    use CardDir::*;

    let test_path=[U,D,D,L,R,U,L,R,U,R,R,D,D,D,U,R,D,R,U,D,D,D,D,D,D,D,U,D,D,U,U];
    let s=ShortPath::new(test_path.iter().map(|a|*a));
    let v:Vec<_>=s.iter().collect();
    assert_eq!(&v as &[_],&test_path);
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum CardDir{
    U,
    D,
    L,
    R
}
impl CardDir{
    pub fn into_char(self)->char{
        use CardDir::*;
        match self{
            U=>{
                '↑'
            },
            D=>{
                '↓'
            },
            L=>{
                '←'
            },
            R=>{
                '→'
            }
        }
    }
    pub fn into_vec(self)->Vec2<GridNum>{
        use CardDir::*;
        match self{
            U=>{
                vec2(0,-1)
            },
            D=>{
                vec2(0,1)
            },
            L=>{
                vec2(-1,0)
            },
            R=>{
                vec2(1,0)
            }
        }
    }
    fn into_two_bits(self)->u8{
        use CardDir::*;
        match self{
            U=>{
                0b00
            },
            D=>{
                0b01
            },
            L=>{
                0b10
            },
            R=>{
                0b11
            }

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
}

/*
TODO its absolutely amazing how it is better to store just 4 cardinal directions instead of 8,
since you can 'look ahead' and optimize cases where you have a down and then a right provided there
isnt a grid wall in a certain spot.
*/


use core::iter::Peekable;


#[derive(Eq,PartialEq,Copy,Clone,Debug)]
pub struct PathDiagAdapter{
    pub inner:PathPointIter
}
impl PathDiagAdapter{
    pub fn new(path:PathPointIter)->PathDiagAdapter{
        PathDiagAdapter{inner:path}
    }
}
//DOES NOT IMPLEMENT EXACT SIZE
impl PathDiagAdapter{

    pub fn next(&mut self,grid:&Grid2D)->Option<Vec2<GridNum>>{
        self.inner.next()
        /*
        match self.inner.next(){
            Some(a)=>{
                match self.inner.peek(){
                    Some(b)=>{
                        //if a and b are diagonal from each other and there is no wall
                        let diff=b-a;

                        //if its a diagonal
                        if diff.abs() == vec2(1,1){
                            println!("diagonal!");
                            let [rleft,rright]=diff.split_into_components();

                            if !grid.get(a+rleft) && !grid.get(a+rright){
                                //safe to skip!
                                self.inner.next() //which is b
                            }else{
                                Some(a)
                            }
                        }else{
                            Some(a)
                        }
                    },
                    None=>{
                        Some(a)
                    }
                }
            },
            None=>{
                None
            }
        }
        */
    }
}


#[derive(Eq,PartialEq,Copy,Clone,Debug)]
pub struct PathPointIter{
    cursor:Vec2<GridNum>,
    path:ShortPathIter
}
impl PathPointIter{
    pub fn new(start:Vec2<GridNum>,path:ShortPathIter)->PathPointIter{
        PathPointIter{cursor:start,path}
    }
    pub fn pos(&self)->Vec2<GridNum>{
        self.cursor
    }
    pub fn peek(&self)->Option<Vec2<GridNum>>{
        self.path.peek().map(|p|self.cursor+p.into_vec())
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
                self.cursor+=p.into_vec();
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
