use std::ops::BitOrAssign;

//external crates
use nalgebra_glm as glm;
use noise::{NoiseFn, Perlin, Seedable};

//use internal crates
use crate::planet::surface::Cell;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Orientation{
    Clockwise,
    CounterClockwise,
    Collinear
}

//handles perlin noise for generating base
pub fn octive_noise(perlin: Perlin, pos:&glm::Vec3, scale:f32, octives:u8, persistance:f32, lacunarity:f32)->f32{
    let mut noise_value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;

    for _o in 0..octives{
        let perlin_value = perlin.get([
            (pos[0]/scale * frequency) as f64,
            (pos[1]/scale *frequency) as f64,
            (pos[2]/scale *frequency) as f64
        ]) as f32;

        noise_value += perlin_value*amplitude;
        amplitude *= persistance;
        frequency *= lacunarity;
    }
    noise_value
}

//get connections of every cell
pub fn indices_to_connections(indices: &Vec<u32>)->Vec<Vec<usize>>{
    //iterate through indices, for every index, store other two in triangle
    let mut connections:Vec::<Vec<usize>> = vec![Vec::with_capacity(6);indices.len()/3];
    //for each triangle
    indices.chunks(3)
        .for_each(|x|{
                //adds connections of each vert in triangle
                for i in 0..3{
                    connections[x[i] as usize].push(x[(i+1)%3] as usize);
                }
            }
        );
    connections
}

//gets all edges, like the above but can be used more efficiently i think
pub fn indices_to_edges(indices: &Vec<u32>)->Vec<(usize,usize)>{
    //TODO: FIND MORE EFFICIENT WAY TO DO THIS, IM SURE THERE IS ONE
    //iterate through indices, for every index, store other two in triangle
    let mut edges:Vec::<(usize,usize)> = Vec::with_capacity(indices.len());
    //for each triangle
    indices.chunks(3)
        .for_each(|x|{
                //adds connections of each vert in triangle
                for i in 0..3{
                    let edge = (x[i] as usize,x[(i+1)%3] as usize);
                    //since a cell can never connect to itself and out of the two duplicates of an edge only one will ever be ordered, 
                    //checking if ordered before doing anything will ensure the edge is unique
                    if edge.0<edge.1 {
                        edges.push(edge);
                    }
                }
            }
        );
    edges
}

//does the same as the above, just return directed edges, as a result returns double the amount, each edge is ordered the same way as its triangle
pub fn indices_to_directed_edges(indices: &Vec<u32>)->Vec<(usize,usize)>{
    //iterate through indices, for every index, store other two in triangle
    let mut edges:Vec::<(usize,usize)> = Vec::with_capacity(indices.len());
    indices.chunks(3)//for each triangle
        .for_each(|x|{
                //adds connections of each vert in triangle
                for i in 0..3{
                    edges.push((x[i] as usize,x[(i+1)%3] as usize));
                }
            }
        );
    edges
}

//gets circumcenter of triangle
pub fn circumcenter(points: &Vec<glm::Vec3>, tri: Vec<u32>)->glm::Vec3{
    //vectors pointing along triangle edges, and their cross product, for calculation
    let atoc = points[tri[2] as usize]-points[tri[0] as usize];
    let atob = points[tri[1] as usize]-points[tri[0] as usize];
    let cross = glm::cross(&atoc, &atob);

    //vector pointing to circumcenter
    let to_circumcenter = 
        (glm::cross(&cross, &atob)*atoc.magnitude_squared() + 
        glm::cross(&atoc, &cross)*atob.magnitude_squared())
        /(2.0*cross.magnitude_squared());

    //actual location in space 
    points[tri[0] as usize]-to_circumcenter
}

//gets orientation of 3 2d vectors
pub fn tri_orientation(a:glm::Vec2,b:glm::Vec2,c:glm::Vec2)->Orientation{
    //determinant of this matrix used to find rotational direction of tri
    //if greater than zero, is counter clockwise
    //if less than, is clockwise
    //if zero, is collinear
    match glm::Mat3::new(
        a.x,a.y,1.0,
        b.x,b.y, 1.0,
        c.x,c.y, 1.0
    ).determinant().total_cmp(&0.0){
        std::cmp::Ordering::Greater=>Orientation::CounterClockwise,
        std::cmp::Ordering::Less=>Orientation::Clockwise,
        std::cmp::Ordering::Equal=>Orientation::Collinear,
    }
}

//takes two edges, returns if they intersect
pub fn do_edges_intersect(){

}


//takes surrounding triangles and a target point, returns new traingles all connecting surrounding edges to target
pub fn connect_point(tris:Vec<u32>, target: u32)->Vec<u32>{
    //divide tris into edges
    //filter out shared edges, by filtering out any with a flipped variaent also contained in edges
    let mut edges = indices_to_directed_edges(&tris);
    edges = edges.iter()
        .filter(|edge| {
            //check if any member of edges isnt the flipped varient of current edge
            !edges.iter()
                .any(|e| edge == &&(e.1,e.0))
        })
        .map(|e| *e)
        .collect();
    
    //return, connecting each valid edge to target
    edges.iter()
        .map(|edge|  vec![edge.0 as u32,edge.1 as u32,target])
        .flatten()
        .collect()
}

//takes a polygon, adds triangles between edges at or less than a specified threshold angle in radians
pub fn tris_at_threshold(points:&Vec<glm::Vec3>, mut polygon: Vec<usize>, threshold: f32)->Vec<u32>{
    //go through every pair of connected edges in polygon, if angle between them inside polygon is less than angle given, add tri
    //do not add triangle if contains any other point in triangle
    let mut triangles:Vec<u32> = Vec::new();
    //loop through every two edges
    for i in 0..polygon.len(){
        //indices of points in tri
        let tri:Vec<usize> = (0..=2).into_iter()
            .map(|x| (i+x)%polygon.len())
            .collect();

        //if potential triangle is not counterclockwise (either colinear or clockwise), skip
        if tri_orientation(points[tri[0]].xy(), points[tri[1]].xy(), points[tri[2]].xy()) != Orientation::CounterClockwise{
            continue;
        }
        //get vectors for calculating angle, b as origin
        let btoa = points[tri[0]]-points[tri[1]];
        let btoc = points[tri[2]]-points[tri[1]];
        //if angle greater than specifed, continue
        if btoa.angle(&btoc)>threshold{
            continue;
        }
        //if new edge doesnt intersect with anything, add tri
    }

    triangles
}

//takes cartesian point on unit sphere, returns it as stereographic, a pole must be specified
pub fn stereographic(point: glm::Vec3,pole: &glm::Vec3)->glm::Vec3{    
    glm::vec3(point.x/(1.0-point.z), point.y/(1.0-point.z), 0.0)
}