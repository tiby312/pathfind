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

#[derive(Copy,Clone,Debug)]
pub enum GridRayCastResult{
    Found{t:WorldNum,cell:Vec2<GridNum>,dirhit:CardDir},
    NotFound
}


pub mod raycast{
    use core::iter::*;
    use crate::grid::*;
    use duckduckgeo::Ray;

    pub struct CollideCellEvent{
        //Cell colliding with
        pub cell:Vec2<GridNum>,
        //Direction in which we are colliding with it.
        pub dir_hit:CardDir,
    }
    pub struct RayCaster<'a>{
        grid:&'a GridViewPort,
        ray:Ray<WorldNum>,
        dir_sign:Vec2<GridNum>,
        next_dir_sign:Vec2<GridNum>,
        current_grid:Vec2<GridNum>,
        tval:WorldNum
    }
    impl<'a> RayCaster<'a>{
        pub fn new(grid:&'a GridViewPort,ray:Ray<WorldNum>)->Option<RayCaster>{
            let dir_sign=vec2(if ray.dir.x>0.0{1}else{0},if ray.dir.y>0.0{1}else{0});
            let next_dir_sign=vec2(if ray.dir.x>0.0{1}else{-1},if ray.dir.y>0.0{1}else{-1});
            
            let current_grid=grid.to_grid(ray.point);
            if ray.dir.x*ray.dir.x+ray.dir.y*ray.dir.y>0.0{
                Some(RayCaster{grid,ray,dir_sign,next_dir_sign,current_grid,tval:0.0})
            }else{
                None
            }
        }
    }
    impl FusedIterator for RayCaster<'_>{}
    impl Iterator for RayCaster<'_>{
        type Item=CollideCellEvent;
        fn next(&mut self)->Option<Self::Item>{
            let grid=&self.grid;
            let ray=&self.ray;
            let dir_sign=self.dir_sign;

            let next_grid=self.current_grid+dir_sign;
            let next_grid_pos=grid.to_world_topleft(next_grid);


            //A ray can be described as follows:
            //rx(t)=ray.dir.x*tval+ray.point.x
            //ry(t)=ray.dir.y*tval+ray.point.y
            //
            //The ray itself is all the points that satify those two equations,
            //where tval>0.
            //
            //As tval increases, so does the ray length.
            //
            //
            //We want to find out when a ray intersects
            //th next cell. A ray are intersect the cell either on
            //a x axis or in a y axis.
            //so we have two equations.
            //
            //Equation for when it hits the xaxis  
            //next_grid_pos.x=ray.dir.x*tval+ray.point.x
            //
            //Equation for when it hits the yaxis
            //next_grid_pos.y=ray.dir.y*tval+ray.point.y
            //
            //In both cases, lets solve for tval.
            //The equation with the smaller tval is the one
            //the ray will hit first.
            //
            //If the tval for the x equation is smaller, the ray
            //will intersect the Y axis first.
            //

            let tvalx=(next_grid_pos.x-ray.point.x)/ray.dir.x;
            let tvaly=(next_grid_pos.y-ray.point.y)/ray.dir.y;

            
            if tvalx.is_finite(){
                assert_gt!(tvalx,0.0,"{:?}",(ray,self.current_grid,next_grid));
            }
            if tvaly.is_finite(){
                assert_gt!(tvaly,0.0,"{:?}",(ray,self.current_grid,next_grid));
            }

            let mut dir_hit;
            if tvalx<=tvaly || tvaly.is_infinite() || tvaly.is_nan(){
                if dir_sign.x==1{
                    //hit left side
                    dir_hit=CardDir::L;
                    //1
                }else{
                    dir_hit=CardDir::R;
                    //hit right side
                }
                self.tval=tvalx;
                self.current_grid.x+=self.next_dir_sign.x;
            }else if tvaly<tvalx  || tvalx.is_infinite() || tvalx.is_nan(){
                if dir_sign.y==1{
                    //hit top side
                    dir_hit=CardDir::U;
                }else{
                    //hit bottom side
                    dir_hit=CardDir::L;
                }
                self.tval=tvaly;
                self.current_grid.y+=self.next_dir_sign.y;
            }else{
                unreachable!("{:?}, {:?}",(tvalx,tvaly),ray);
            }
            Some(CollideCellEvent{cell:next_grid,dir_hit})
        }
    }
}



/*
pub fn ray_compute_intersection_tvalue(grid:&GridDim2D,ray:&duckduckgeo::Ray<WorldNum>)->Vec2<WorldNum>{
    //r(t)=ray.dir*t+ray.point
    //r(t)-ray.point=ray.dir*t
    //(r(t)-ray.point)/ray.dir=t


    if ray.dir.x*ray.dir.x+ray.dir.y*ray.dir.y{
        let mut t=0;
        
        let dir_sign_x=ray.dir.x > 0 {1}else{0};
        let dir_sign_y=ray.dir.y > 0 {1}else{0};
        
           
        if dx<dy{
            t=dt;
            
            let dd=if ray.dir.x<0.0{
                CardDir::R
            }else{
                CardDir::L
            };
            GridRayCastResult::Found{t,cell,dirhit:dd}
        }else{
            t=dy;

            let dd=if ray.dir.y<0.0{
                CardDir::D
            }else{
                CardDir::U
            };
            GridRayCastResult::Found{t,cell,dirhit:dd}
        }
    }else{
        NotFound
    }

}
*/

/*




pub fn ray_cast(grid:&GridDim2D,ray:duckduckgeo::Ray<WorldNum>,max:Vec2<WorldNum>)->GridRayCastResult{
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
        let l = duckduckgeo::ray_compute_intersection_tvalue(&ray,XAXISS, grid.convert_to_world_topleft(cursor).x);
        let u = duckduckgeo::ray_compute_intersection_tvalue(&ray,YAXISS, grid.convert_to_world_topleft(cursor).y);
        let r = duckduckgeo::ray_compute_intersection_tvalue(&ray,XAXISS, grid.convert_to_world_topleft(cursor+vec2(1,0)).x);
        let d = duckduckgeo::ray_compute_intersection_tvalue(&ray,XAXISS, grid.convert_to_world_topleft(cursor+vec2(0,1)).y);


        let vals=[
            (l,vec2(-1, 0),CardDir::R),
            (u,vec2(0,  -1),CardDir::D),
            (r,vec2(1,0),CardDir::L),
            (d,vec2(0,1),CardDir::U)
        ];
        /*
        dbg!(vals);

        let ts:Vec<_>=vals.iter().map(|a|{
            //r(t)=ray.dir*t+ray.point
            //r(t)-ray.point=ray.dir*t
            //(r(t)-ray.point)/ray.dir=t

            let dx=(a.x-ray.point.x)/ray.dir.x;
            let dy=(a.y-ray.point.y)/ray.dir.y;
            vec2(dx,dy)
        }).collect();

        
        let a=[
            (ts[0],,CardDir::R),
            (ts[1],vec2( 1, 0),CardDir::L),
            (ts[2],vec2( 0,-1),CardDir::D),
            (ts[3],vec2( 0, 1),CardDir::U)
        ];
        */
        //dbg!(cursor,vals);

        
        let ans=vals.iter().filter(|a|a.0.is_some());
        let ans=ans.filter(|a|a.0.unwrap()>currentT); //strictly greater so that we actually make progress
        let ans=ans.min_by(|a,b|a.0.unwrap().partial_cmp(&b.0.unwrap()).unwrap() );

        let ans=match ans{
            Some(ans)=>{
                ans
            },
            None=>{
                return GridRayCastResult::NotFound;
            }
        };


        let next_cell=cursor+ans.1;
        

        match grid.inner.get_option(next_cell){
            Some(hit)=>{
                if hit{
                    return GridRayCastResult::Found{t:ans.0.unwrap(),cell:next_cell,dirhit:ans.2};
                }
            },
            None=>{
                return GridRayCastResult::NotFound
            }
        }

        currentT=ans.0.unwrap();
        cursor=next_cell;
    }


}
*/


/*
#[derive(Debug)]
pub struct ToGridError{
    dim:Rect<WorldNum>,
    pos:Vec2<WorldNum>
}

#[derive(Debug)]
pub struct ToWorldError{
    dim:Vec2<GridNum>,
    pos:Vec2<GridNum>
}
*/

pub struct GridViewPort{
    pub spacing:Vec2<WorldNum>,
    pub origin:Vec2<WorldNum>
}
impl GridViewPort{

    pub fn to_world_topleft(&self,pos:Vec2<GridNum>)->Vec2<WorldNum>{
        pos.inner_as().scale(self.spacing)+self.origin
    }

    pub fn to_world_center(&self,pos:Vec2<GridNum>)->Vec2<WorldNum>{
        pos.inner_as().scale(self.spacing)+self.origin+self.spacing/2.0
    }
    
    pub fn to_grid(&self,pos:Vec2<WorldNum>)->Vec2<GridNum>{
    
        let result = (pos-self.origin).inv_scale(self.spacing);

        result.inner_as()


        /*
        let xdim = self.grid_dim.x;
        let ydim = self.grid_dim.y;

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
        */
    }


    pub fn cell_radius(&self)->Vec2<WorldNum>{
        self.spacing
        /*
        let spacingx=(self.dim.x.right-self.dim.x.left)/self.grid_dim.x as f32;
        let spacingy=(self.dim.y.right-self.dim.y.left)/self.grid_dim.y as f32;
        vec2(spacingx,spacingy)
        */
    }
    /*
    pub fn convert_to_world_topleft(&self,val:Vec2<GridNum>)->Result<Vec2<WorldNum>,ToWorldError>{
        if val.x>=self.grid_dim.x || val.y>=self.grid_dim.y{
            Err(ToWorldError{dim:self.grid_dim,pos:val})
        }else{
            let top_left=vec2(self.dim.x.left,self.dim.y.left);

            let spacingx=(self.dim.x.right-self.dim.x.left)/self.grid_dim.x as f32;
            let spacingy=(self.dim.y.right-self.dim.y.left)/self.grid_dim.y as f32;
            

            let val=vec2(spacingx * val.x as f32,spacingy*val.y as f32);
            //let half=vec2(spacingx,spacingy)/2.0;
            Ok(top_left+val)
        }
    }
    pub fn convert_to_world_center(&self,val:Vec2<GridNum>)->Result<Vec2<WorldNum>,ToWorldError>{
        if val.x>=self.grid_dim.x || val.y>=self.grid_dim.y{
            Err(ToWorldError{dim:self.grid_dim,pos:val})
        }else{
            let top_left=vec2(self.dim.x.left,self.dim.y.left);

            let spacingx=(self.dim.x.right-self.dim.x.left)/self.grid_dim.x as f32;
            let spacingy=(self.dim.y.right-self.dim.y.left)/self.grid_dim.y as f32;
            

            let val=vec2(spacingx * val.x as f32,spacingy*val.y as f32);
            let half=vec2(spacingx,spacingy)/2.0;
            Ok(top_left+val+half)
        }
    }
    */
}

/*
pub struct GridDim2D{
    pub dim:Rect<WorldNum>,
    pub inner:Grid2D
}


use crate::short_path::ShortPath;
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
*/