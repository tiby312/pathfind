use crate::axgeom::*;
use bit_vec::*;




pub type GridNum=isize;

#[derive(Debug)]
pub struct Grid2D {
    xs: usize,
    ys: usize,
    inner: BitVec,
}

impl Grid2D {
    pub fn new(xs: usize, ys: usize) -> Grid2D {
        let inner = BitVec::from_elem(xs * ys, false);

        Grid2D { xs, ys, inner }
    }
    pub fn xdim(&self) -> usize {
        self.xs
    }
    pub fn ydim(&self) -> usize {
        self.ys
    }
    pub fn get(&self, x: GridNum, y: GridNum) -> bool {
        self.inner[x as usize * self.ys + y as usize]
    }
    pub fn set(&mut self, x: GridNum, y: GridNum,val:bool) {
        self.inner.set(x as usize * self.ys + y as usize, val)
    }

}


struct GridDim2D{
    dim:Rect<f32>,
    inner:Grid2D
}
impl GridDim2D{
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