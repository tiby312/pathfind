
use crate::axgeom::*;

use duckduckgeo::grid::*;


/*
#[test]
fn test_short(){
    use crate::short_path::*;
    use CardDir::*;

    let test_path=[U,D,D,L,R,U,L,R,U,R,R,D,D,D,U,R,D,R,U,D,D,D,D,D,D,D,U,D,D,U,U];
    let s=ShortPath::new(test_path.iter().map(|a|*a));
    let v:Vec<_>=s.iter().collect();
    assert_eq!(&v as &[_],&test_path);
}
*/

use core::fmt;
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
            //If the path has nothing left in it except for the sentinal val
            if self.path.value ==SENTINAL_VAL{
                return None
            }

            let dir = CardDir2::from_u8((self.path.value & 0b111) as u8);

            //self.path.value=self.path.value >> 3;

            Some(dir)
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
    pub fn peek(&self)->Option<(CardDir2,Vec2<GridNum>)>{
        self.path.peek().map(|a|(a,self.cursor+a.into_offset().0))
    }

    pub fn double_peek(&self)->Option<(CardDir2,Vec2<GridNum>)>{
        let mut k=*self;
        k.next();
        k.peek()
    }
}

impl core::iter::FusedIterator for PathPointIter{}
impl ExactSizeIterator for PathPointIter{}
impl Iterator for PathPointIter{
    type Item=(CardDir2,Vec2<GridNum>);
    fn size_hint(&self)->(usize,Option<usize>){
        let l = self.path.len();
        (l,Some(l))
    }
    fn next(&mut self)->Option<Self::Item>{
        
        match self.path.next(){
            Some(p)=>{
                self.cursor+=p.into_offset().0;
                Some((p,self.cursor))      
            },
            None=>{
                None
            }
        }

    }
}
