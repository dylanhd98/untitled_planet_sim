use std::collections::HashSet;

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

//handles perlin octive noise for generating more detailed features
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

//takes cartesian point on unit sphere, returns it as stereographic 2d projection 
pub fn stereographic(point: &glm::Vec3)->glm::Vec2{    
    glm::vec2(point.x/(1.0-point.z), point.y/(1.0-point.z))
}

//takes a set of points, returns their steriographic projection, a normalized "pole" must be defined as a focus point of the projection
pub fn stereographic_project(points: &Vec<glm::Vec3>, pole:glm::Vec3)->Vec<glm::Vec2>{
    //create rotation matrix used to center pole in projection
    let cross = glm::cross(&pole,&glm::Vec3::y()).normalize();
    let angle = glm::Vec3::y().angle(&pole);
    let rotation_mat = glm::rotation(angle, &cross);

    //rotate and apply projection to all points
    points.iter()
        .map(|p| {
            let rotated= (rotation_mat*glm::vec3_to_vec4(p)).xyz();
            stereographic(&rotated)
        })
        .collect()
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

//gets all edges in the mesh
pub fn indices_to_edges(indices: &Vec<u32>)->Vec<(usize,usize)>{
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

//does the same as the above, but return directed edges, as a result returns double the amount, each edge is ordered the same way as its triangle
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
pub fn do_edges_intersect(a:(glm::Vec2,glm::Vec2),b:(glm::Vec2,glm::Vec2))->bool{
    (tri_orientation(a.0, a.1, b.1) != tri_orientation(a.0, a.1, b.0))&&
    (tri_orientation(b.0, b.1, a.1) != tri_orientation(b.0, b.1, a.0))
}

//takes triangles, turns them into a polygon describing the external edges of the triangles
//pub fn triangles_to_polygon(tris:Vec<u32>)->Vec<usize>{

//}

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
pub fn tris_at_threshold(points:&Vec<glm::Vec2>,polygon: Vec<usize>, threshold: f32)->Vec<u32>{
    //go through every pair of connected edges in polygon, if angle between them inside polygon is less than angle given, add tri
    //do not add triangle if contains any other point in triangle
    let mut triangles:Vec<u32> = Vec::new();
    //record middle indices of tri when added, as to avoid future tris from containing them
    let mut avoid_points:HashSet<usize> = HashSet::new();
    //loop through every two edges/potential triangle
    for i in 0..polygon.len(){
        //indices of points in tri
        let tri:Vec<usize> = (0..=2).into_iter()
            .map(|x| (i+x)%polygon.len())
            .collect();
        //if tri contains point marked avoid, or if is not counterclockwise (either colinear or clockwise), skip
        if avoid_points.contains(&tri[0])|| avoid_points.contains(&tri[2])||
            tri_orientation(points[tri[0]].xy(), points[tri[1]].xy(), points[tri[2]].xy()) != Orientation::CounterClockwise{
            continue;
        }
        //get vectors for calculating angle, b as origin
        let btoa = points[tri[0]]-points[tri[1]];
        let btoc = points[tri[2]]-points[tri[1]];
        //if angle greater than specifed, continue
        if btoa.angle(&btoc)>threshold{
            continue;
        }
        //if new edge doesnt intersect with any other edge in polygon and added tris, add tri
        if !polygon.iter().zip(polygon.iter().skip(1))
            .filter_map(|(a,b)| {
                //filter out edges that share points with tri
                if !tri.contains(a) && !tri.contains(b){
                    Some((points[*a].xy(),points[*b].xy()))
                }else {
                    None
                }
            })
            .any(|edge| 
                do_edges_intersect(edge, (points[tri[0]].xy(),points[tri[2]].xy()))){
            //record middle index of tri as now "covered" by an edge, so that no future tri can connect to it
            avoid_points.insert(tri[1]);
            //add triangle to resultant triangulation
            triangles.append(&mut tri.into_iter().map(|i| polygon[i] as u32).collect());
        }
    }

    triangles
}

