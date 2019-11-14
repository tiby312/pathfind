use crate::axgeom::*;
use bit_vec::*;



#[derive(Copy,Clone)]
pub struct Iterator2D{
    counter:Vec2<GridNum>,
    dim:Vec2<GridNum>
}
impl Iterator2D{
    pub fn new(dim:Vec2<GridNum>)->Iterator2D{
        Iterator2D{counter:vec2same(0),dim}
    }
}
impl Iterator for Iterator2D{
    type Item=Vec2<GridNum>;
    fn next(&mut self)->Option<Self::Item>{

        if self.counter.y==self.dim.y{
            return None
        }

        let k=self.counter;

        self.counter.x+=1;
        if self.counter.x==self.dim.y{
            self.counter.x=0;
            self.counter.y+=1;
        }
        Some(k)
    }
}

#[test]
fn test_iterator2d(){
    let i=Iterator2D::new(vec2(10,10));

    for a in i{
        print!("{:?} ",a);
    }
    assert_eq!(i.count(),100);

}



pub type GridNum=isize;

#[derive(Debug)]
pub struct Grid2D {
    xs: GridNum,
    ys: GridNum,
    inner: BitVec,
}

impl Grid2D {
    pub fn new(xs: GridNum, ys: GridNum) -> Grid2D {
        let inner = BitVec::from_elem((xs * ys) as usize, false);

        Grid2D { xs, ys, inner }
    }
    pub fn xdim(&self) -> GridNum {
        self.xs
    }
    pub fn ydim(&self) -> GridNum {
        self.ys
    }
    pub fn get(&self, x: GridNum, y: GridNum) -> bool {
        self.inner[(x * self.ys + y) as usize]
    }
    pub fn set(&mut self, x: GridNum, y: GridNum,val:bool) {
        self.inner.set( (x * self.ys + y) as usize, val)
    }
    pub fn len(&self)->usize{
        (self.xs*self.ys) as usize
    }

}


pub type WorldNum=f32;
pub struct GridDim2D{
    pub dim:Rect<WorldNum>,
    pub inner:Grid2D
}
impl GridDim2D{
    
    pub fn convert_to_grid(&self,pos:Vec2<WorldNum>)->Vec2<GridNum>{
        
        let xdim = self.inner.xs;
        let ydim = self.inner.ys;

        let dim: &Rect<f32> = &self.dim;

        //https://math.stackexchange.com/questions/528501/how-to-determine-which-cell-in-a-grid-a-point-belongs-to
        
        let x = pos.x;
        let y = pos.y;
        let width = dim.x.right - dim.x.left;
        let height = dim.y.right - dim.y.left;

        let i = (x * (xdim as f32 / width))
            .floor()
            .max(0.0)
            .min((xdim - 1) as f32);
        let j = (y * (ydim as f32 / height))
            .floor()
            .max(0.0)
            .min((ydim - 1) as f32);
        let i = i as isize;
        let j = j as isize;

        /*
        if self.get(i, j) {
            //This bot is inside of this thing yo
            Some(self.get_rect(i, j))
        } else {
            None
        }
        */
        vec2(i,j)
    

    }

    pub fn convert_to_world(&self,val:Vec2<GridNum>)->Vec2<WorldNum>{
        let top_left=vec2(self.dim.x.left,self.dim.y.left);

        let spacingx=(self.dim.x.right-self.dim.x.left)/self.inner.xdim() as f32;
        let spacingy=(self.dim.y.right-self.dim.y.left)/self.inner.ydim() as f32;
        

        let val=vec2(spacingx * val.x as f32,spacingy*val.y as f32);
        top_left+val
    }
/*
    pub fn get_rect(&self, i: usize, j: usize) -> Rect<f32> {
        let dim = self.dim;
        let xdim = self.xs;
        let ydim = self.ys;
        let xratio = i as f32 / xdim as f32;
        let yratio = j as f32 / ydim as f32;
        let width = dim.x.right - dim.x.left;
        let height = dim.y.right - dim.y.left;

        let xratio2 = (i as f32 + 1.0) / xdim as f32;
        let yratio2 = (j as f32 + 1.0) / ydim as f32;

        Rect::new(
            dim.x.left + xratio * width,
            dim.x.left + xratio2 * width,
            dim.y.left + yratio * height,
            dim.y.left + yratio2 * height,
        )
    }
    
    fn detect_collision(&self, bot: &Bot, radius: f32) -> Option<Rect<f32>> {
        if bot.vel.magnitude2() < 0.01 * 0.01 {
            return None;
        }

        let xdim = self.xs;
        let ydim = self.ys;

        let dim: &Rect<f32> = self.dim.as_ref();

        //https://math.stackexchange.com/questions/528501/how-to-determine-which-cell-in-a-grid-a-point-belongs-to
        let jj = bot.vel.normalize_to(radius);

        let x = bot.pos.x + jj.x;
        let y = bot.pos.y + jj.y;
        let width = dim.x.right - dim.x.left;
        let height = dim.y.right - dim.y.left;

        let i = (x * (xdim as f32 / width))
            .floor()
            .max(0.0)
            .min((xdim - 1) as f32);
        let j = (y * (ydim as f32 / height))
            .floor()
            .max(0.0)
            .min((ydim - 1) as f32);
        let i = i as usize;
        let j = j as usize;

        if self.get(i, j) {
            //This bot is inside of this thing yo
            Some(self.get_rect(i, j))
        } else {
            None
        }
    }
    */
}