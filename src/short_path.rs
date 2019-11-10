

#[test]
fn test_short(){
    use crate::shortpath::*;
    use CardDir::*;

    let test_path=[U,D,D,L,R,U,L,R,U,R,R,D,D,D,U,R,D,R,U,D,D,D,D,D,D,D,U,D,D,D,D,D];
    let s=ShortPath::new(test_path.iter().map(|a|*a));
    assert_eq!(s.len(),32);
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


const MAX_PATH_LENGTH:usize=32;
//CardDir only takes up 2 bit. So inside of a 64 bit integer,
//we can store a path of length 32.
//at that point just make it have to re-compute.


//
//   00000000000000000000
//   99887766554433221100
#[derive(Copy,Clone)]
pub struct ShortPath{
    value:u64
}
impl ShortPath{
    pub fn new<I:IntoIterator<Item=CardDir>+ExactSizeIterator>(it:I)->ShortPath{
        assert!(it.len()<=32);

        let mut value = 0;
        let mut bit_index=0;
        for a in it{
            value |= ((a.into_two_bits() as u64) << bit_index);
            bit_index+=2;
        }
        ShortPath{value}
    }

    pub fn len(&self)->usize{
        let l=self.value.leading_zeros() as usize;
        32-(l/2)
    }
    pub fn iter(&self)->ShortPathIter{
        ShortPathIter{path:*self}
    }
}

pub struct ShortPathIter{
    path:ShortPath
}
impl ExactSizeIterator for ShortPathIter{}
impl Iterator for ShortPathIter{
    type Item=CardDir;
    
    fn size_hint(&self)->(usize,Option<usize>){
        let l = self.path.len();
        (l,Some(l))
    }
    fn next(&mut self)->Option<Self::Item>{
        if self.path.value==0{
            return None
        }

        use CardDir::*;
        let val = match (self.path.value & 0b11) {
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
