use crate::axgeom::*;
use bit_vec::*;
use crate::short_path::*;


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

use core::iter::*;

impl FusedIterator for Iterator2D{}
impl ExactSizeIterator for Iterator2D{}
impl Iterator for Iterator2D{
    type Item=Vec2<GridNum>;
    fn size_hint(&self)->(usize,Option<usize>){
        let diff = vec2(self.dim.x,self.dim.y-1)-self.counter;
        //TODO test this
        let l=(self.dim.x*diff.y+diff.x) as usize;
        (l,Some(l))
    }
    fn next(&mut self)->Option<Self::Item>{

        if self.counter.y==self.dim.y{
            return None
        }

        let k=self.counter;

        self.counter.x+=1;
        if self.counter.x==self.dim.x{
            self.counter.x=0;
            self.counter.y+=1;
        }
        Some(k)
    }
}

#[test]
fn test_iterator2d(){
    let i=Iterator2D::new(vec2(10,20));
    assert_eq!(i.len(),200);
    assert_eq!(i.count(),200);



    let i=Iterator2D::new(vec2(20,10));
    assert_eq!(i.len(),200);
    assert_eq!(i.count(),200);

}


impl<'a> FusedIterator for CellIterator<'a>{}
impl<'a> ExactSizeIterator for CellIterator<'a>{}
pub struct CellIterator<'a>{
    grid:&'a Grid2D,
    inner:Iterator2D
}
impl<'a> Iterator for CellIterator<'a>{
    type Item=(Vec2<GridNum>,bool);

    fn size_hint(&self)->(usize,Option<usize>){
        self.inner.size_hint()
    }
    fn next(&mut self)->Option<Self::Item>{
        match self.inner.next(){
            Some(v)=>{
                Some((v,self.grid.get(v)))
            },
            None=>{
                None
            }
        }
    }

}



pub type GridNum=isize;

pub struct Grid2D {
    dim: Vec2<GridNum>,
    inner: BitVec,
}

impl Grid2D {
    pub fn from_str(dim:Vec2<GridNum>,p:&str)->Grid2D{
        let mut grid=Grid2D::new(dim);

        for (y,line) in p.lines().enumerate(){
            for (x,c) in line.chars().enumerate(){
                match c{
                    '█'=>{
                        grid.set(vec2(x,y).inner_as(),true);
                    },
                    ' '=>{

                    }
                    _=>{
                        panic!("unknown char");
                    }
                }
            }
        }
        grid

    }
    pub fn new(dim:Vec2<GridNum>) -> Grid2D {
        let inner = BitVec::from_elem((dim.x * dim.y) as usize, false);

        Grid2D { dim, inner }
    }
    pub fn dim(&self)->Vec2<GridNum>{
        self.dim
    }

    pub fn iter(&self)->CellIterator{

        let inner = Iterator2D::new(self.dim);
        CellIterator{grid:self,inner}
    }
    pub fn get(&self, p:Vec2<GridNum>) -> bool {
        self.inner[(p.x * self.dim.y + p.y) as usize]
    }
    pub fn get_option(&self, p:Vec2<GridNum>) -> Option<bool> {
        self.inner.get((p.x * self.dim.y + p.y) as usize)
    }
    pub fn set(&mut self, p:Vec2<GridNum>,val:bool) {
        self.inner.set( (p.x * self.dim.y + p.y) as usize, val)
    }
    pub fn len(&self)->usize{
        (self.dim.x*self.dim.y) as usize
    }

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
    pub fn draw_map(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        let mut res=String::new();
        
        let mut fa=Ok(());



        fa.and(writeln!(f,""));
        for i in 0..self.dim().y{
            for j in 0..self.dim().x{
                
                let cc=if self.get(vec2(j,i)){
                    "1 "
                }else{
                    "0 "
                };

                res.push_str(cc);
            }
            fa=fa.and( writeln!(f,"{}",res));
            res.clear();
        }
        fa
    }


}


pub type WorldNum=f32;
pub struct GridDim2D{
    pub dim:Rect<WorldNum>,
    pub inner:Grid2D
}


#[test]
fn testy(){
    let mut inner=Grid2D::new(vec2(20,20));
    let k =GridDim2D{dim:Rect::new(-100.0,100.0,-100.0,100.0),inner};

    let j = k.convert_to_grid(vec2(56.0,56.0));
    
    dbg!(j);
    
    let back=k.convert_to_world(j);
    assert_eq!(back,vec2(50.0,50.0));
    //dbg!(back);
    //panic!("yo");
}


use core::fmt;
impl fmt::Debug for Grid2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.draw_map(f)
    }
}


pub fn pick_empty_spot(grid:&Grid2D)->Option<Vec2<GridNum>>{
    //TODO inefficient
    use rand::prelude::*;

    let mut k:Vec<_>=Iterator2D::new(grid.dim()).filter(|a|!grid.get(*a)).collect();

    let mut rng = rand::thread_rng();
    k.shuffle(&mut rng);

    k.first().map(|a|*a)
}

pub fn find_closest_empty(grid:&Grid2D,start:Vec2<GridNum>)->Option<Vec2<GridNum>>{
    //TODO inefficient.

    let mut k:Vec<_>=Iterator2D::new(grid.dim()).filter(|a|!grid.get(*a)).map(|a|(a,(start-a).magnitude2() )).collect();

    k.sort_by(|a,b|a.1.cmp(&b.1));

    k.first().map(|a|a.0)
}


pub enum GridRayCastResult{
    InsideWall{closest_non_wall:Option<Vec2<GridNum>>},
    Found{t:Vec2<WorldNum>,cell:Vec2<GridNum>,dirhit:CardDir},
    NotFound
}

pub fn ray_cast(grid:&GridDim2D,ray:duckduckgeo::Ray<WorldNum>)->GridRayCastResult{
    let start=grid.convert_to_grid(ray.point);
    if grid.inner.get(start){
        let closest_non_wall =find_closest_empty(&grid.inner,start);
        return GridRayCastResult::InsideWall{closest_non_wall};
    }


    //T value squared
    let mut currentT:WorldNum=0.0;
    loop{
        let mut cursor=start;

        //TODO inefficient to calculate them all
        let mut l=grid.convert_to_world_topleft(cursor+vec2(-1,0) );
        let mut r=grid.convert_to_world_topleft(cursor+vec2(1,0) );
        let mut u=grid.convert_to_world_topleft(cursor+vec2(0,-1) );
        let mut d=grid.convert_to_world_topleft(cursor+vec2(0,1) );

        let vals=[l,r,u,d];


        let ts:Vec<_>=vals.iter().map(|a|{
            //r(t)=ray.dir*t+ray.point
            //r(t)-ray.point=ray.dir*t
            //(r(t)-ray.point)/ray.dir=t

            let dx=(a.x-ray.point.x)/ray.dir.x;
            let dy=(a.y-ray.point.y)/ray.dir.y;
            vec2(dx,dy)
        }).collect();



        let a=[
            (ts[0],vec2(-1, 0),CardDir::R),
            (ts[1],vec2( 1, 0),CardDir::L),
            (ts[2],vec2( 0,-1),CardDir::D),
            (ts[3],vec2( 0, 1),CardDir::U)
        ];

        
        let ans=a.iter().filter(|a|a.0.magnitude2()>currentT).min_by(|a,b|a.0.magnitude2().partial_cmp(&b.0.magnitude2()).unwrap());
        let ans=ans.unwrap();

        let next_cell=cursor+ans.1;
        
        match grid.inner.get_option(next_cell){
            Some(hit)=>{
                if hit{
                    return GridRayCastResult::Found{t:ans.0,cell:next_cell,dirhit:ans.2};
                }
            },
            None=>{
                return GridRayCastResult::NotFound
            }
        }

        currentT=ans.0.magnitude2();
        cursor=next_cell;
    }


}


use crate::short_path::ShortPath;
impl GridDim2D{
    

    pub fn convert_to_grid(&self,pos:Vec2<WorldNum>)->Vec2<GridNum>{
        
        let xdim = self.inner.dim().x;
        let ydim = self.inner.dim().y;

        let dim: &Rect<f32> = &self.dim;

        let pos=pos-dim.top_left();
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

       
        vec2(i,j)
    

    }


    pub fn cell_radius(&self)->Vec2<WorldNum>{
        let spacingx=(self.dim.x.right-self.dim.x.left)/self.inner.dim().x as f32;
        let spacingy=(self.dim.y.right-self.dim.y.left)/self.inner.dim().y as f32;
        vec2(spacingx,spacingy)
    }

    pub fn convert_to_world_topleft(&self,val:Vec2<GridNum>)->Vec2<WorldNum>{
        let top_left=vec2(self.dim.x.left,self.dim.y.left);

        let spacingx=(self.dim.x.right-self.dim.x.left)/self.inner.dim().x as f32;
        let spacingy=(self.dim.y.right-self.dim.y.left)/self.inner.dim().y as f32;
        

        let val=vec2(spacingx * val.x as f32,spacingy*val.y as f32);
        //let half=vec2(spacingx,spacingy)/2.0;
        top_left+val
    }
    pub fn convert_to_world_center(&self,val:Vec2<GridNum>)->Vec2<WorldNum>{
        let top_left=vec2(self.dim.x.left,self.dim.y.left);

        let spacingx=(self.dim.x.right-self.dim.x.left)/self.inner.dim().x as f32;
        let spacingy=(self.dim.y.right-self.dim.y.left)/self.inner.dim().y as f32;
        

        let val=vec2(spacingx * val.x as f32,spacingy*val.y as f32);
        let half=vec2(spacingx,spacingy)/2.0;
        top_left+val+half
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